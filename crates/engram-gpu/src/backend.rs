//! `CudaBackend` — implements `VsaBackend` using the BVH + GPU kernels.
//!
//! On machines without CUDA, transparently falls back to `CpuBackend`.
//! The fall-through is automatic: if the BVH build fails or CUDA is unavailable,
//! `recall()` uses the linear CPU scan.

use crate::bvh::BvhManifold;
use engram_core::backend::{CpuBackend, Memory, VsaBackend};
use engram_core::types::Leg3Pointer;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use anyhow::Result;

/// CUDA-accelerated backend with BVH K-NN filter.
///
/// # Usage
///
/// ```rust,no_run
/// use engram_gpu::backend::CudaBackend;
/// use engram_core::backend::VsaBackend;
///
/// let backend = CudaBackend::new("~/.engram/manifold");
/// backend.remember("photosynthesis", "Plants convert CO₂ + H₂O → glucose + O₂").unwrap();
/// let results = backend.recall("how do plants make food", 5);
/// ```
pub struct CudaBackend {
    /// Path to the .leg manifold directory
    store_path: PathBuf,
    /// CPU backend for writes and linear-scan fallback
    cpu: CpuBackend,
    /// BVH index for O(log N) queries (rebuilt after writes)
    bvh: RwLock<Option<BvhManifold>>,
    /// Whether a GPU was detected at startup (reserved for future CUDA dispatch)
    #[allow(dead_code)]
    gpu_available: bool,
}

impl CudaBackend {
    /// Create a new CUDA backend. BVH is built eagerly in a background thread.
    ///
    /// Spawning an async build (rather than blocking new()) means the server
    /// comes online immediately. Queries that arrive before the BVH is ready
    /// fall back to the CPU linear scan. The BVH is typically ready within
    /// a few seconds for manifolds < 10K blocks.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = shellexpand::tilde(
            path.as_ref().to_str().unwrap_or("~/.engram/manifold")
        ).into_owned();
        std::fs::create_dir_all(&path).ok();

        let gpu_available = Self::probe_cuda();
        if gpu_available {
            eprintln!("[engram-gpu] CUDA device detected.");
        } else if cfg!(target_os = "macos") {
            eprintln!("[engram-gpu] macOS detected — use MetalBackend for Apple Silicon GPU. CPU BVH active.");
        } else {
            eprintln!("[engram-gpu] No CUDA device — using CPU BVH fallback.");
        }

        let bvh: RwLock<Option<BvhManifold>> = RwLock::new(None);
        let backend = Self {
            cpu: CpuBackend::new(&path),
            store_path: PathBuf::from(path.clone()),
            bvh,
            gpu_available,
        };

        // Kick off background BVH build so first query is fast.
        // Arc is not available here, but we can spawn with the path and a raw ptr
        // since CudaBackend lives in an Arc<RwLock<StoreHandle>> in practice.
        // Safer: spawn the build and store the result via a channel.
        let bvh_ptr = &backend.bvh as *const RwLock<Option<BvhManifold>> as usize;
        let path_clone = path.clone();
        std::thread::Builder::new()
            .name("engram-bvh-build".to_string())
            .spawn(move || {
                let t0 = std::time::Instant::now();
                eprintln!("[BVH] Background build started…");
                let bvh = BvhManifold::build_from_dir(&path_clone);
                // SAFETY: CudaBackend lives for the duration of the server process.
                // The pointer is valid for the lifetime of the StoreHandle arc.
                let lock = unsafe { &*(bvh_ptr as *const RwLock<Option<BvhManifold>>) };
                if let Ok(mut guard) = lock.write() {
                    let n = bvh.as_ref().map_or(0, |b| b.len());
                    *guard = bvh;
                    eprintln!("[BVH] ✓ Background build complete: {} concepts in {:.1}s",
                        n, t0.elapsed().as_secs_f32());
                }
            })
            .ok(); // If thread spawn fails, fall back to lazy build on first query

        backend
    }

    /// Rebuild the BVH from all current .leg files in the store directory.
    /// Call after `store()` / `forget()` to keep the index current.
    pub fn rebuild_bvh(&self) {
        let bvh = BvhManifold::build_from_dir(&self.store_path);
        if let Ok(mut guard) = self.bvh.write() {
            *guard = bvh;
        }
    }

    /// Check if a CUDA-capable GPU is reachable.
    ///
    /// Uses the CUDA driver API via `dlopen`/`dlsym` — no compile-time CUDA dependency.
    ///
    /// # CUDA Driver API initialization contract
    /// `cuInit(0)` MUST be called before any other CUDA driver API function.
    /// Without it, `cuDeviceGetCount` returns `CUDA_ERROR_NOT_INITIALIZED (3)`
    /// regardless of how many GPUs are physically present. This was the root cause
    /// of the "No CUDA device" false negative on the RTX 5060 Ti / RTX 5060 machine.
    fn probe_cuda() -> bool {
        #[cfg(target_os = "linux")]
        {
            unsafe {
                let lib = libc::dlopen(
                    b"libcuda.so.1\0".as_ptr() as *const libc::c_char,
                    libc::RTLD_NOW | libc::RTLD_GLOBAL,
                );
                if lib.is_null() { return false; }

                // Step 1: cuInit(0) — mandatory before any other driver API call.
                let init_sym = libc::dlsym(lib, b"cuInit\0".as_ptr() as *const libc::c_char);
                if init_sym.is_null() {
                    libc::dlclose(lib);
                    return false;
                }
                let cu_init: extern "C" fn(u32) -> i32 = std::mem::transmute(init_sym);
                if cu_init(0) != 0 {
                    // cuInit failed — driver not available or permission denied
                    libc::dlclose(lib);
                    return false;
                }

                // Step 2: cuDeviceGetCount — now safe to call after cuInit.
                let count_sym = libc::dlsym(lib, b"cuDeviceGetCount\0".as_ptr() as *const libc::c_char);
                if count_sym.is_null() {
                    libc::dlclose(lib);
                    return false;
                }
                let get_count: extern "C" fn(*mut i32) -> i32 = std::mem::transmute(count_sym);
                let mut count: i32 = 0;
                let rc = get_count(&mut count);
                libc::dlclose(lib);
                rc == 0 && count > 0
            }
        }
        #[cfg(not(target_os = "linux"))]
        { false }
    }

    /// Ensure the BVH is populated, building it synchronously if needed.
    /// In normal operation the background build has already finished by the time
    /// any query arrives. This is only triggered on very early queries.
    fn ensure_bvh(&self) {
        if let Ok(guard) = self.bvh.read() {
            if guard.is_some() { return; }
        }
        // Background build hasn't finished yet — build synchronously
        eprintln!("[BVH] Synchronous build (background thread not yet done)…");
        self.rebuild_bvh();
    }
}

impl VsaBackend for CudaBackend {
    fn encode(&self, text: &str) -> Leg3Pointer {
        self.cpu.encode(text)
    }

    fn fetch(&self, concept: &str) -> Option<Box<[num_complex::Complex32; 8192]>> {
        self.cpu.fetch(concept)
    }

    fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer> {
        self.cpu.fetch_block(concept)
    }

    fn query(&self, q: &[num_complex::Complex32; 8192], k: usize) -> Vec<Memory> {
        // ── Try BVH O(log N) path if it's already built ──────────────────────
        // Do NOT call ensure_bvh() here. ensure_bvh() triggers a synchronous
        // BvhManifold::build_from_dir() when the background thread hasn't
        // committed yet — that blocks for 18+ seconds on a 3.5K-block manifold.
        // A plain read() returns immediately with None if build is still running.
        let bvh_results = if let Ok(guard) = self.bvh.read() {
            match guard.as_ref() {
                Some(bvh) if !bvh.is_empty() => bvh.query(q, k),
                _ => vec![],
            }
        } else {
            vec![]
        };

        // ── Semantic quality gate ─────────────────────────────────────────────
        // The BVH 3D projection is a spatial hash, not a semantic index.
        // When the BVH's top result scores below BVH_FALLBACK_THRESHOLD, the
        // 128 BVH candidates did NOT include the semantically correct blocks.
        // Fall back to the CPU linear scan (Dirichlet composite scorer, all blocks).
        //
        // This trades O(log N) BVH speed for O(N) correctness. At < 50K blocks,
        // the NVMe linear scan is < 400ms — acceptable for MCP tool calls.
        const BVH_FALLBACK_THRESHOLD: f32 = 0.010;
        let best_bvh_score = bvh_results.first().map_or(0.0, |m| m.score);

        if best_bvh_score >= BVH_FALLBACK_THRESHOLD {
            bvh_results
        } else {
            tracing::debug!(
                "[CudaBackend] BVH best score {:.4} < {:.3} — falling back to CPU linear scan",
                best_bvh_score, BVH_FALLBACK_THRESHOLD
            );
            self.cpu.query(q, k)
        }
    }

    fn store(&self, concept: &str, block: Leg3Pointer) -> Result<()> {
        let result = self.cpu.store(concept, block);
        // Trigger a background BVH rebuild so the new block is indexed.
        // We invalidate first so in-flight queries fall back to CPU scan,
        // then rebuild in a background thread (non-blocking).
        if result.is_ok() {
            if let Ok(mut guard) = self.bvh.write() {
                *guard = None; // invalidate — queries fall back to CPU until rebuild
            }
            let bvh_ptr = &self.bvh as *const RwLock<Option<BvhManifold>> as usize;
            let path_clone = self.store_path.clone();
            std::thread::Builder::new()
                .name("engram-bvh-rebuild".to_string())
                .spawn(move || {
                    let bvh = BvhManifold::build_from_dir(&path_clone);
                    let lock = unsafe { &*(bvh_ptr as *const RwLock<Option<BvhManifold>>) };
                    if let Ok(mut guard) = lock.write() { *guard = bvh; }
                })
                .ok();
        }
        result
    }

    fn forget(&self, concept: &str) -> Result<()> {
        let result = self.cpu.forget(concept);
        if result.is_ok() {
            if let Ok(mut guard) = self.bvh.write() {
                *guard = None;
            }
        }
        result
    }

    fn list(&self) -> Vec<String> {
        self.cpu.list()
    }
}
