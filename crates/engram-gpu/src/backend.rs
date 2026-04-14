//! `CudaBackend` — implements `VsaBackend` using the BVH + GPU kernels.
//!
//! On machines without CUDA, transparently falls back to `CpuBackend`.
//! The fall-through is automatic: if the BVH build fails or CUDA is unavailable,
//! `recall()` uses the linear CPU scan.

use crate::bvh::BvhManifold;
use engram_core::backend::{CpuBackend, Memory, VsaBackend};
use engram_core::types::Leg3Pointer;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
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
    /// Whether a GPU was detected at startup
    gpu_available: bool,
}

impl CudaBackend {
    /// Create a new CUDA backend. BVH is built lazily on first query.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = shellexpand::tilde(
            path.as_ref().to_str().unwrap_or("~/.engram/manifold")
        ).into_owned();
        std::fs::create_dir_all(&path).ok();

        let gpu_available = Self::probe_cuda();
        if gpu_available {
            eprintln!("[engram-gpu] CUDA device detected.");
        } else {
            eprintln!("[engram-gpu] No CUDA device — using CPU BVH fallback.");
        }

        Self {
            cpu: CpuBackend::new(&path),
            store_path: PathBuf::from(path),
            bvh: RwLock::new(None),
            gpu_available,
        }
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
    /// Uses the CUDA runtime API via a safe probe.
    fn probe_cuda() -> bool {
        // Try to call cudaGetDeviceCount via libc dlsym
        // If libcuda.so isn't loaded or returns 0 devices → false
        #[cfg(target_os = "linux")]
        {
            // We check by probing the shared library — no compile-time CUDA required
            unsafe {
                let lib = libc::dlopen(
                    b"libcuda.so.1\0".as_ptr() as *const libc::c_char,
                    libc::RTLD_NOW | libc::RTLD_LOCAL,
                );
                if lib.is_null() { return false; }
                let sym = libc::dlsym(lib, b"cuDeviceGetCount\0".as_ptr() as *const libc::c_char);
                if sym.is_null() {
                    libc::dlclose(lib);
                    return false;
                }
                let get_count: extern "C" fn(*mut i32) -> i32 = std::mem::transmute(sym);
                let mut count: i32 = 0;
                let rc = get_count(&mut count);
                libc::dlclose(lib);
                rc == 0 && count > 0
            }
        }
        #[cfg(not(target_os = "linux"))]
        { false }
    }

    /// Ensure the BVH is populated, building it if needed.
    fn ensure_bvh(&self) {
        if let Ok(guard) = self.bvh.read() {
            if guard.is_some() { return; }
        }
        self.rebuild_bvh();
    }
}

impl VsaBackend for CudaBackend {
    fn encode(&self, text: &str) -> Leg3Pointer {
        self.cpu.encode(text)
    }

    fn query(&self, q: &[num_complex::Complex32; 8192], k: usize) -> Vec<Memory> {
        self.ensure_bvh();

        // Try BVH O(log N) path first
        if let Ok(guard) = self.bvh.read() {
            if let Some(bvh) = guard.as_ref() {
                if !bvh.is_empty() {
                    return bvh.query(q, k);
                }
            }
        }

        // Fallback: linear scan via CpuBackend
        self.cpu.query(q, k)
    }

    fn store(&self, concept: &str, block: Leg3Pointer) -> Result<()> {
        let result = self.cpu.store(concept, block);
        // Invalidate BVH — it will be rebuilt lazily on next query
        if result.is_ok() {
            if let Ok(mut guard) = self.bvh.write() {
                *guard = None;
            }
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
