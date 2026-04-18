//! Backend abstraction layer — swappable compute backends for VSA operations.
//!
//! Implement this trait to add a new hardware backend (CPU, CUDA, ROCm, Vulkan).
//! The rest of the Engram stack (server, CLI, MCP) is backend-agnostic.

use crate::types::Leg3Pointer;
use crate::ops::cosine_similarity;
use num_complex::Complex32;
use anyhow::Result;

/// A retrieved memory with its concept name and similarity score.
#[derive(Debug, Clone)]
pub struct Memory {
    /// The concept identifier (e.g. "krebs_cycle")
    pub concept: String,
    /// Composite weighted score (cosine × crs_weight × stability × depth_bonus)
    pub score: f32,
    /// The CRS (Coherence-Reliability Score) of the stored block [0.0, 1.0]
    pub crs: f32,
    /// The ProvLog text stored in the block's payload field
    pub provlog: String,
    // ── Physics fields (Phase 8 loop closure) ────────────────────────────────
    /// Lyapunov drift velocity from last update — 0.0=stable, 1.0=major shift
    pub drift_velocity: f32,
    /// How many times this concept has been reinforced via update()
    pub superposition_depth: u32,
    /// ZEDOS epistemic tag (0xD=declarative, 0xA=episodic, 0x50=praxis, etc.)
    pub zedos_tag: u8,
    /// Epistemic affirm weight from last session_end (0.0–1.0)
    pub alpha_a: f32,
    /// Epistemic deny weight from last session_end (0.0–1.0)
    pub alpha_d: f32,
    /// Spatial bounding box min [row, col, 0.0] — file coordinates of AST node
    pub aabb_min: [f32; 3],
    /// Spatial bounding box max [row, col, 0.0] — file coordinates of AST node
    pub aabb_max: [f32; 3],
}

/// Distance metric and quantization mode for nearest-neighbour search.
///
/// Pass to [`VsaBackend::query_with_mode`] to select the search strategy.
/// Backends that do not implement a given mode fall back to [`SearchMode::Cosine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SearchMode {
    /// Flat cosine similarity in full f32 precision (default, all backends).
    #[default]
    Cosine,
    /// Poincaré ball hyperbolic distance in f32 precision.
    /// More accurate than cosine for hierarchical / phylogenetic concept spaces.
    Poincare,
    /// INT8-quantised Poincaré distance via WebGPU compute shader.
    /// 170× fewer bytes per block; requires the `wgpu-backend` feature on `engram-gpu`.
    Int8Poincare,
}


/// Swappable VSA compute backend.
///
/// # Implementing a backend
///
/// ```rust,ignore
/// use engram_core::backend::VsaBackend;
///
/// pub struct MyBackend { /* ... */ }
///
/// impl VsaBackend for MyBackend {
///     fn encode(&self, text: &str) -> Leg3Pointer { /* ... */ }
///     fn query(&self, q: &[Complex32; 8192], k: usize) -> Vec<Memory> { /* ... */ }
/// }
/// ```
pub trait VsaBackend: Send + Sync {
    /// Encode free-form text into a HolographicBlock.
    fn encode(&self, text: &str) -> Leg3Pointer;

    /// Fetch the exact phase vector for a named concept, if it exists.
    fn fetch(&self, concept: &str) -> Option<Box<[Complex32; 8192]>>;

    /// Fetch the complete HolographicBlock for a named concept.
    fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer>;

    /// Find the k most similar memories to a query vector.
    fn query(&self, query: &[Complex32; 8192], k: usize) -> Vec<Memory>;

    /// Find the k most similar memories using an explicit search strategy.
    ///
    /// The default implementation ignores `mode` and calls [`Self::query`] (cosine).
    /// Override in backends that support Poincaré or INT8 modes.
    fn query_with_mode(
        &self,
        query: &[Complex32; 8192],
        k: usize,
        _mode: SearchMode,
    ) -> Vec<Memory> {
        self.query(query, k)
    }

    /// Store a block under a concept name.
    fn store(&self, concept: &str, block: Leg3Pointer) -> Result<()>;

    /// Delete a concept from the manifold.
    fn forget(&self, concept: &str) -> Result<()>;

    /// List all concept names in the manifold.
    fn list(&self) -> Vec<String>;

    /// High-level convenience: encode text then store it.
    fn remember(&self, concept: &str, text: &str) -> Result<()> {
        let block = self.encode(text);
        self.store(concept, block)
    }

    /// High-level convenience: encode a query then find the k nearest memories.
    fn recall(&self, query_text: &str, k: usize) -> Vec<Memory> {
        let block = self.encode(query_text);
        self.query(&block.q, k)
    }
}

// ── CPU Backend (always compiled) ────────────────────────────────────────────

/// Pure-CPU backend with optional LBVH index for O(log N) NVMe-efficient queries.
///
/// When an LBVH index is present (`engram build-index` has been run), query()
/// projects the query to 3-D, traverses the lightweight tree to find the top
/// `KNN_FILTER_CANDIDATES` (128) candidates in O(log N), then reads only those
/// `.leg` files from NVMe via O_DIRECT and applies the physics composite scorer.
///
/// When no index exists, falls back to the exact linear scan using Rayon
/// parallel iterators over all `.leg` files in the manifold directory.
/// At < 50,000 blocks the O_DIRECT linear scan is imperceptibly fast;
/// the NVMe bus saturates at ~7 GB/s so 10K blocks (2.5 GB) reads in < 0.4s.
pub struct CpuBackend {
    /// Directory containing `.leg` block files.
    pub manifold_dir: std::path::PathBuf,
    /// Optional LBVH index for O(log N) candidate pre-filtering.
    pub bvh: Option<crate::index::BvhIndex>,
}

impl CpuBackend {
    /// Create a backend without an LBVH index (linear scan mode).
    pub fn new(manifold_dir: impl Into<std::path::PathBuf>) -> Self {
        Self { manifold_dir: manifold_dir.into(), bvh: None }
    }

    /// Create a backend and attempt to load the LBVH index from the default
    /// path inside manifold_dir. Falls back to linear scan if no index exists.
    pub fn with_index(manifold_dir: impl Into<std::path::PathBuf>) -> Self {
        let dir: std::path::PathBuf = manifold_dir.into();
        let idx_path = crate::index::default_index_path(&dir);
        let bvh = crate::index::BvhIndex::load(&idx_path).ok();
        if bvh.is_some() {
            tracing::info!("[BVH] Loaded LBVH index — O(log N) candidate pre-filter active");
        } else {
            tracing::debug!("[BVH] No index found — using O_DIRECT linear scan");
        }
        Self { manifold_dir: dir, bvh }
    }
}

impl VsaBackend for CpuBackend {
    fn fetch(&self, concept: &str) -> Option<Box<[Complex32; 8192]>> {
        let path = self.manifold_dir.join(format!("{}.leg", concept));
        let block = crate::storage::read_block(&path).ok()?;
        Some(Box::new(block.q))
    }

    fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer> {
        let path = self.manifold_dir.join(format!("{}.leg", concept));
        let block = crate::storage::read_block(&path).ok()?;
        Some(Leg3Pointer::from_boxed(block))
    }

    fn encode(&self, text: &str) -> Leg3Pointer {
        crate::encode::from_text(text)
    }

    fn query(&self, query: &[Complex32; 8192], k: usize) -> Vec<Memory> {
        // ── BVH fast path: O(log N) 3-D pre-filter then targeted NVMe reads ────
        if let Some(ref bvh) = self.bvh {
            if bvh.is_ready() {
                // Get up to KNN_FILTER_CANDIDATES (128) concept names from the tree
                let candidates = bvh.search(query, crate::index::KNN_FILTER_CANDIDATES);
                let mut scored: Vec<Memory> = candidates
                    .iter()
                    .filter_map(|concept| {
                        let path = self.manifold_dir.join(format!("{}.leg", concept));
                        let block = crate::storage::read_block(&path).ok()?;
                        Some(score_block(concept.clone(), query, &*block))
                    })
                    .collect();
                scored.sort_by(|a, b|
                    b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                scored.truncate(k);
                return scored;
            }
        }

        // ── Linear scan fallback: exact O(N) search (used when no index) ──────
        use rayon::prelude::*;
        use std::fs;

        let entries: Vec<_> = match fs::read_dir(&self.manifold_dir) {
            Ok(e) => e.flatten().collect(),
            Err(_) => return Vec::new(),
        };

        let mut scored: Vec<Memory> = entries.par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("leg") { return None; }
                let concept = path.file_stem()?.to_str()?.to_string();
                let block = crate::storage::read_block(&path).ok()?;
                Some(score_block(concept, query, &*block))
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    fn store(&self, concept: &str, block: Leg3Pointer) -> Result<()> {
        std::fs::create_dir_all(&self.manifold_dir)?;
        let path = self.manifold_dir.join(format!("{}.leg", concept));
        crate::storage::write_block(&path, &block)?;
        Ok(())
    }

    fn forget(&self, concept: &str) -> Result<()> {
        let path = self.manifold_dir.join(format!("{}.leg", concept));
        if path.exists() { std::fs::remove_file(path)?; }
        Ok(())
    }

    fn list(&self) -> Vec<String> {
        std::fs::read_dir(&self.manifold_dir)
            .map(|entries| {
                entries.flatten()
                    .filter_map(|e| {
                        let p = e.path();
                        if p.extension().and_then(|x| x.to_str()) != Some("leg") { return None; }
                        p.file_stem()?.to_str().map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ── Sheaf Backend (multi-manifold) ───────────────────────────────────────────

/// A `VsaBackend` spanning multiple independent manifold directories (stalks).
///
/// Queries fan out across all stalks in parallel and results merge by cosine
/// similarity, giving a unified ranked view. Writes go to the active stalk only.
///
/// This implements the Sheaf topology already encoded in the LEG Merkle footer:
/// each stalk is a local section; `SheafBackend` is the global section.
pub struct SheafBackend {
    stalks: Vec<(String, Box<dyn VsaBackend + Send + Sync>)>,
    active: std::sync::atomic::AtomicUsize,
}

impl SheafBackend {
    /// Create from a list of `(name, path)` pairs using CpuBackend per stalk (default).
    pub fn new(stalks: Vec<(String, std::path::PathBuf)>) -> Self {
        let stalks: Vec<(String, Box<dyn VsaBackend + Send + Sync>)> = stalks
            .into_iter()
            .map(|(name, path)| {
                std::fs::create_dir_all(&path).ok();
                let b: Box<dyn VsaBackend + Send + Sync> = Box::new(CpuBackend::new(path));
                (name, b)
            })
            .collect();
        Self { stalks, active: std::sync::atomic::AtomicUsize::new(0) }
    }

    /// Create with pre-built backend instances per stalk.
    /// Use this to pass `CudaBackend` or any other `VsaBackend` implementor.
    pub fn new_boxed(stalks: Vec<(String, Box<dyn VsaBackend + Send + Sync>)>) -> Self {
        Self { stalks, active: std::sync::atomic::AtomicUsize::new(0) }
    }

    pub fn set_active_stalk(&self, name: &str) -> bool {
        if let Some(idx) = self.stalks.iter().position(|(n, _)| n == name) {
            self.active.store(idx, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn active_stalk_name(&self) -> &str {
        let idx = self.active.load(std::sync::atomic::Ordering::Relaxed);
        &self.stalks[idx].0
    }

    pub fn stalk_names(&self) -> Vec<&str> {
        self.stalks.iter().map(|(n, _)| n.as_str()).collect()
    }
}

impl VsaBackend for SheafBackend {
    fn encode(&self, text: &str) -> Leg3Pointer { crate::encode::from_text(text) }

    fn fetch(&self, concept: &str) -> Option<Box<[Complex32; 8192]>> {
        self.stalks.iter().find_map(|(_, s)| s.fetch(concept))
    }

    fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer> {
        self.stalks.iter().find_map(|(_, s)| s.fetch_block(concept))
    }

    fn query(&self, query: &[Complex32; 8192], k: usize) -> Vec<Memory> {
        use rayon::prelude::*;
        let mut all: Vec<Memory> = self.stalks
            .par_iter()
            .flat_map_iter(|(stalk_name, backend)| {
                let name = stalk_name.clone();
                backend.query(query, k).into_iter().map(move |mut m| {
                    m.concept = format!("{}::{}", name, m.concept);
                    m
                })
            })
            .collect();
        all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all.truncate(k);
        all
    }

    fn store(&self, concept: &str, block: Leg3Pointer) -> anyhow::Result<()> {
        let idx = self.active.load(std::sync::atomic::Ordering::Relaxed);
        self.stalks[idx].1.store(concept, block)
    }

    fn forget(&self, concept: &str) -> anyhow::Result<()> {
        for (_, stalk) in &self.stalks {
            if stalk.fetch(concept).is_some() { return stalk.forget(concept); }
        }
        Ok(())
    }

    fn list(&self) -> Vec<String> {
        self.stalks.iter()
            .flat_map(|(name, stalk)| {
                stalk.list().into_iter().map(move |c| format!("{}::{}", name, c))
            })
            .collect()
    }
}

// ── Shared scoring helper ─────────────────────────────────────────────────────

/// Compute the physics-weighted composite score for a single block and return
/// a fully populated `Memory`. Used by both the HNSW fast path and the linear scan.
fn score_block(
    concept: String,
    query: &[Complex32; 8192],
    block: &crate::types::HolographicBlock,
) -> Memory {
    let base_sim = cosine_similarity(query, &block.q);
    let crs_weight = 0.85 + (block.crs_score * 0.15);
    let stability   = 1.0 - (block.energetics.dv * 0.10);
    let depth_bonus = 1.0 + (block.superposition_count.min(10) as f32 * 0.005);
    let score = (base_sim * crs_weight * stability * depth_bonus).clamp(-1.0, 1.0);
    let provlog = String::from_utf8_lossy(&block.payload)
        .trim_matches('\0').chars().take(512).collect();
    Memory {
        concept, score, crs: block.crs_score, provlog,
        drift_velocity: block.energetics.dv,
        superposition_depth: block.superposition_count,
        zedos_tag: block.zedos_tag,
        alpha_a: block.energetics.alpha_a,
        alpha_d: block.energetics.alpha_d,
        aabb_min: block.aabb_min,
        aabb_max: block.aabb_max,
    }
}
