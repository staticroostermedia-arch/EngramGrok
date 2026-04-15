//! Backend abstraction layer — swappable compute backends for VSA operations.
//!
//! Implement this trait to add a new hardware backend (CPU, CUDA, ROCm, Vulkan).
//! The rest of the Engram stack (server, CLI, MCP) is backend-agnostic.

use crate::types::{HolographicBlock, Leg3Pointer};
use crate::ops::cosine_similarity;
use num_complex::Complex32;
use anyhow::Result;

/// A retrieved memory with its concept name and similarity score.
#[derive(Debug, Clone)]
pub struct Memory {
    /// The concept identifier (e.g. "krebs_cycle")
    pub concept: String,
    /// Cosine similarity to the query vector [−1.0, 1.0]
    pub score: f32,
    /// The CRS (Coherence-Reliability Score) of the stored block [0.0, 1.0]
    pub crs: f32,
    /// The ProvLog text stored in the block's payload field
    pub provlog: String,
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

/// Pure-CPU backend using Rayon parallel iterators.
///
/// Performs an exact linear scan over all blocks in the manifold directory.
/// At < 50,000 blocks this is imperceptibly fast. For larger manifolds,
/// prefer the CUDA backend which uses an O(log N) BVH index.
pub struct CpuBackend {
    /// Directory containing `.leg` block files.
    pub manifold_dir: std::path::PathBuf,
}

impl CpuBackend {
    pub fn new(manifold_dir: impl Into<std::path::PathBuf>) -> Self {
        Self { manifold_dir: manifold_dir.into() }
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
                let score = cosine_similarity(query, &block.q);
                let crs = block.crs_score;
                let provlog = String::from_utf8_lossy(&block.payload)
                    .trim_matches('\0')
                    .chars()
                    .take(512)
                    .collect();
                Some(Memory { concept, score, crs, provlog })
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
