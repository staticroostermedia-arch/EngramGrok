//! Thread-safe wrapper around the VsaBackend for server use.
//!
//! Detects `~/.engram/sheaf.toml` on boot. If present, opens a multi-manifold
//! `SheafBackend`. Otherwise falls back to a single `CpuBackend`.
//!
//! # Performance Architecture (Hot/Cold Separation)
//!
//! `.leg` blocks (256KB each) are *cold* storage — O_DIRECT NVMe DMA, expensive to write.
//! Access timestamps are *hot* operational metadata — should never trigger a block rewrite
//! on a passive recall query.
//!
//! Solution: `AccessIndex` — an in-memory `HashMap<String, u64>` that maps concept name
//! → last_accessed UNIX timestamp. It is updated instantly on every recall and flushed
//! to `~/.engram/access_index.bin` every 60 seconds by the Autophagy daemon.
//!
//! # Reflexive Contract
//!
//! Every block minted via `remember()` receives a ZEDOS-tag-appropriate
//! `allowed_transforms` string. `update()` checks the contract via
//! `enforce_contract_soft()` (logs, never blocks) and accumulates binding
//! momentum in the `p` tensor. `scar()` narrows the contract to `evidence_update`
//! only — the storage-layer expression of `InjectScar { magnitude }` from the M-NOL.

use engram_core::backend::{CpuBackend, Memory, VsaBackend, SheafBackend};
// GPU backends — conditionally included based on auto-detected hardware (see engram-gpu/build.rs)
#[cfg(engram_backend_cuda)]
use engram_gpu::backend::CudaBackend;
#[cfg(engram_backend_metal)]
use engram_gpu::metal_backend::MetalBackend;
#[cfg(engram_backend_wgpu)]
use engram_gpu::wgpu_backend::WgpuBackend;
use engram_core::types::{Leg3Pointer, ZEDOS_PRAXIS, ZEDOS_EPISODIC, ZEDOS_RELATION, ZEDOS_USER_MODEL};
use engram_core::ops::{op_add, op_bind, op_add_arena, op_bind_arena, op_deduce};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use anyhow::Result;

pub type SharedStore = Arc<Mutex<StoreHandle>>;

// ── Sheaf Config ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Debug)]
pub struct SheafConfig {
    pub active_stalk: Option<String>,
    pub stalks: Vec<StalkEntry>,
}

#[derive(serde::Deserialize, Debug)]
pub struct StalkEntry {
    pub name: String,
    pub path: String,
}

// ── AccessIndex — hot temporal metadata ──────────────────────────────────────

pub struct AccessIndex {
    map: HashMap<String, u64>,
    path: PathBuf,
    dirty: bool,
}

impl AccessIndex {
    pub fn load(engram_root: &Path) -> Self {
        let path = engram_root.join("access_index.bin");
        let map = if path.exists() {
            std::fs::read(&path)
                .ok()
                .and_then(|b| bincode::deserialize::<HashMap<String, u64>>(&b).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };
        tracing::info!("AccessIndex loaded: {} entries from {:?}", map.len(), path);
        Self { map, path, dirty: false }
    }

    pub fn touch(&mut self, concept: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.map.insert(concept.to_string(), now);
        self.dirty = true;
    }

    pub fn last_accessed(&self, concept: &str) -> Option<u64> {
        self.map.get(concept).copied()
    }

    /// Return the N most recently accessed concepts, sorted newest first.
    pub fn recent(&self, n: usize) -> Vec<(String, u64)> {
        let mut entries: Vec<(String, u64)> = self.map.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(n);
        entries
    }

    /// Flush to disk if dirty. Called by daemon every 60 seconds.
    pub fn flush_if_dirty(&mut self) {
        if !self.dirty { return; }
        if let Ok(bytes) = bincode::serialize(&self.map) {
            if std::fs::write(&self.path, &bytes).is_ok() {
                self.dirty = false;
                tracing::debug!("AccessIndex flushed: {} entries", self.map.len());
            }
        }
    }
}

// ── RelationIndex — knowledge graph sidecar ───────────────────────────────────
//
// Stores directed relations as a flat Vec<RelationEntry> in
// `~/.engram/relation_index.json`. Flushed to disk after every write.
// Provides O(n) forward/reverse/BFS queries — suitable for small graphs.

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RelationEntry {
    pub from:  String,
    pub label: String,
    pub to:    String,
}

pub struct RelationIndex {
    pub entries: Vec<RelationEntry>,
    path: PathBuf,
}

impl RelationIndex {
    pub fn load(engram_root: &Path) -> Self {
        let path = engram_root.join("relation_index.json");
        let entries = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<Vec<RelationEntry>>(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        tracing::info!("RelationIndex loaded: {} edges from {:?}", entries.len(), path);
        Self { entries, path }
    }

    /// Add a directed edge, deduplicating and flushing immediately.
    pub fn add(&mut self, from: &str, label: &str, to: &str) {
        let dup = self.entries.iter().any(|e| e.from == from && e.label == label && e.to == to);
        if !dup {
            self.entries.push(RelationEntry {
                from: from.to_string(), label: label.to_string(), to: to.to_string(),
            });
            self.flush();
        }
    }

    /// Query edges. `direction`: "from" | "to" | "both".
    /// Returns (label, other_concept) pairs.
    pub fn query(&self, concept: &str, filter_label: Option<&str>, direction: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for e in &self.entries {
            let label_ok = filter_label.is_none_or(|l| e.label == l);
            if !label_ok { continue; }
            match direction {
                "from" if e.from == concept => out.push((e.label.clone(), e.to.clone())),
                "to"   if e.to   == concept => out.push((e.label.clone(), e.from.clone())),
                "both"  => {
                    if e.from == concept { out.push((e.label.clone(), e.to.clone())); }
                    if e.to   == concept { out.push((e.label.clone(), e.from.clone())); }
                }
                _ => {}
            }
        }
        out
    }

    /// BFS up to `depth` hops from `seed`. Returns all (from, label, to) edges traversed.
    pub fn bfs(&self, seed: &str, depth: usize) -> Vec<RelationEntry> {
        use std::collections::HashSet;
        let mut visited: HashSet<String> = HashSet::new();
        let mut frontier = vec![seed.to_string()];
        let mut result: Vec<RelationEntry> = Vec::new();
        for _ in 0..depth {
            if frontier.is_empty() { break; }
            let mut next: Vec<String> = Vec::new();
            for concept in &frontier {
                if !visited.insert(concept.clone()) { continue; }
                for e in &self.entries {
                    if &e.from == concept {
                        result.push(e.clone());
                        if !visited.contains(&e.to) { next.push(e.to.clone()); }
                    }
                }
            }
            frontier = next;
        }
        result
    }

    fn flush(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.entries) {
            let _ = std::fs::write(&self.path, s);
        }
    }
}

// ── Reflexive Contract Assignment ────────────────────────────────────────────
//
// Maps ZEDOS tag → permitted transform string, stored in `allowed_transforms[0..64]`.
// Called at remember() time.
//
// | Tag         | Contract              | Meaning                            |
// |-------------|-----------------------|------------------------------------|
// | DECLARATIVE | evidence_update,op_add| Facts enriched, geometry preserved |
// | EPISODIC    | evidence_update,rollb | Session memory correctable         |
// | PRAXIS      | evidence_update       | Crystallized: update only          |
// | RELATION    | op_bind,rollback      | Relational bonds rebound-able      |
// | OPERATIONAL | evidence_update,rollb | Code memory correctable            |
// | 0xFF / pin  | 0xFF                  | Full authority, genesis-tier       |
// ── Transductive Oracle Fallthrough ─────────────────────────────────────────
//
// Optional: fires a synchronous POST to an external oracle API when the Engram
// manifold cannot satisfy a query above MIN_SCORE_THRESHOLD.
//
// Enable by setting ENGRAM_ORACLE_URL in the environment:
//   export ENGRAM_ORACLE_URL="http://localhost:8080/api/ask"
//
// Request body: `{ "query": "<text>", "k": 3 }`
// Response: JSON with a top-level `assembled_prose` field.
//
// If the env var is not set, or the oracle is unreachable, returns None (silent fallback).
fn oracle_fallthrough(query: &str) -> Option<Memory> {
    let oracle_url = match std::env::var("ENGRAM_ORACLE_URL") {
        Ok(url) => url,
        Err(_) => return None,  // oracle disabled (env var not set)
    };
    const TIMEOUT_SECS: u64 = 3;

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("[oracle_fallthrough] Failed to build HTTP client: {}", e);
            return None;
        }
    };

    let body = serde_json::json!({ "query": query, "k": 3 });

    let response = match client.post(&oracle_url).json(&body).send() {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("[oracle_fallthrough] Oracle unavailable ({}). Returning empty recall.", e);
            return None;
        }
    };

    let json: serde_json::Value = match response.json() {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("[oracle_fallthrough] Could not parse oracle response as JSON: {}", e);
            return None;
        }
    };

    let prose = json
        .get("assembled_prose")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if prose.is_empty() {
        tracing::debug!("[oracle_fallthrough] Oracle returned empty assembled_prose.");
        return None;
    }

    tracing::info!("[oracle_fallthrough] Oracle hit: {} chars of assembled_prose returned.", prose.len());

    Some(Memory {
        concept:             "oracle_fallthrough".to_string(),
        score:               0.29, // Just below MIN_SCORE_THRESHOLD — callers detect oracle provenance.
        crs:                 0.74,
        provlog:             prose,
        explain:             "Transductive[oracle=LBVH]".to_string(),
        // physics / spatial fields zeroed — synthetic oracle result
        drift_velocity:      0.0,
        superposition_depth: 0,
        zedos_tag:           engram_core::types::ZEDOS_DECLARATIVE,
        alpha_a:             0.0,
        alpha_d:             0.0,
        aabb_min:            [0.0; 3],
        aabb_max:            [0.0; 3],
        l2_norm_residual:    0.0,
    })
}

fn assign_reflexive_contract(block: &mut engram_core::types::Leg3Pointer) {

    use engram_core::types::{
        ZEDOS_PRAXIS, ZEDOS_RELATION, ZEDOS_EPISODIC, ZEDOS_DECLARATIVE,
    };
    // Pinned genesis-tier: full authority
    if block.crs_score >= 1.0 {
        let full = b"0xFF";
        block.allowed_transforms[..full.len()].copy_from_slice(full);
        for b in block.allowed_transforms[full.len()..].iter_mut() { *b = 0; }
        return;
    }

    let contract: &[u8] = match block.zedos_tag {
        t if t == ZEDOS_PRAXIS       => b"evidence_update",
        t if t == ZEDOS_RELATION     => b"op_bind,rollback",
        t if t == ZEDOS_EPISODIC     => b"evidence_update,rollback",
        t if t == ZEDOS_DECLARATIVE  => b"evidence_update,op_add",
        _                            => b"evidence_update,rollback", // OPERATIONAL default
    };

    let len = contract.len().min(64);
    block.allowed_transforms[..len].copy_from_slice(&contract[..len]);
    for b in block.allowed_transforms[len..].iter_mut() { *b = 0; }
}

// ── Backend enum ─────────────────────────────────────────────────────────────

enum Backend {
    #[cfg(engram_backend_cuda)]
    Gpu(CudaBackend),
    #[cfg(engram_backend_metal)]
    Metal(MetalBackend),
    #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
    Wgpu(WgpuBackend),
    Single(CpuBackend),
    Sheaf(SheafBackend),
}

impl Backend {
    fn recall(&self, q: &str, k: usize) -> Vec<Memory> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.recall(q, k),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.recall(q, k),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.recall(q, k),
            Backend::Single(b) => b.recall(q, k),
            Backend::Sheaf(b) => b.recall(q, k),
        }
    }
    fn forget(&self, concept: &str) -> Result<()> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.forget(concept),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.forget(concept),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.forget(concept),
            Backend::Single(b) => b.forget(concept),
            Backend::Sheaf(b) => b.forget(concept),
        }
    }
    fn list(&self) -> Vec<String> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.list(),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.list(),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.list(),
            Backend::Single(b) => b.list(),
            Backend::Sheaf(b) => b.list(),
        }
    }
    fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.fetch_block(concept),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.fetch_block(concept),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.fetch_block(concept),
            Backend::Single(b) => b.fetch_block(concept),
            Backend::Sheaf(b) => b.fetch_block(concept),
        }
    }
    fn fetch(&self, concept: &str) -> Option<Box<[engram_core::Complex32; 8192]>> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.fetch(concept),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.fetch(concept),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.fetch(concept),
            Backend::Single(b) => b.fetch(concept),
            Backend::Sheaf(b) => b.fetch(concept),
        }
    }
    fn encode(&self, text: &str) -> Leg3Pointer {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.encode(text),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.encode(text),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.encode(text),
            Backend::Single(b) => b.encode(text),
            Backend::Sheaf(b) => b.encode(text),
        }
    }
    fn query(&self, q: &[engram_core::Complex32; 8192], k: usize) -> Vec<Memory> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.query(q, k),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.query(q, k),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.query(q, k),
            Backend::Single(b) => b.query(q, k),
            Backend::Sheaf(b) => b.query(q, k),
        }
    }
    fn store(&self, concept: &str, block: Leg3Pointer) -> Result<()> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.store(concept, block),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.store(concept, block),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.store(concept, block),
            Backend::Single(b) => b.store(concept, block),
            Backend::Sheaf(b) => b.store(concept, block),
        }
    }
    fn set_active_stalk(&self, name: &str) -> bool {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(_) => false,
            #[cfg(engram_backend_metal)]
            Backend::Metal(_) => false,
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(_) => false,
            Backend::Single(_) => false,
            Backend::Sheaf(b) => b.set_active_stalk(name),
        }
    }
    fn stalk_names(&self) -> Vec<String> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(_) => vec!["default".to_string()],
            #[cfg(engram_backend_metal)]
            Backend::Metal(_) => vec!["default".to_string()],
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(_) => vec!["default".to_string()],
            Backend::Single(_) => vec!["default".to_string()],
            Backend::Sheaf(b) => b.stalk_names().into_iter().map(|s| s.to_string()).collect(),
        }
    }
    fn active_stalk_name(&self) -> String {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(_) => "default".to_string(),
            #[cfg(engram_backend_metal)]
            Backend::Metal(_) => "default".to_string(),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(_) => "default".to_string(),
            Backend::Single(_) => "default".to_string(),
            Backend::Sheaf(b) => b.active_stalk_name().to_string(),
        }
    }
    fn is_sheaf(&self) -> bool { matches!(self, Backend::Sheaf(_)) }
    fn verify_hypothesis(&self, concept: &str, success: bool) -> Result<()> {
        match self {
            #[cfg(engram_backend_cuda)]
            Backend::Gpu(b) => b.verify_hypothesis(concept, success),
            #[cfg(engram_backend_metal)]
            Backend::Metal(b) => b.verify_hypothesis(concept, success),
            #[cfg(all(engram_backend_wgpu, not(engram_backend_cuda), not(engram_backend_metal)))]
            Backend::Wgpu(b) => b.verify_hypothesis(concept, success),
            Backend::Single(b) => b.verify_hypothesis(concept, success),
            Backend::Sheaf(b) => b.verify_hypothesis(concept, success),
        }
    }
    /// User Model: 90/10 EMA superposition of user interaction centroid.
    /// Implemented inline on the Backend enum so all backend variants are covered without
    /// requiring `track_user_centroid` to be a concrete method on each backend struct.
    fn track_user_centroid(&self, interaction: &str) -> Result<()> {
        const CENTROID: &str = "_user_centroid";
        let new_block = self.encode(interaction);
        let centroid = if let Some(mut existing) = self.fetch_block(CENTROID) {
            let mut norm_sq = 0.0f32;
            for i in 0..engram_core::types::DIMENSION {
                let blended = existing.q[i] * 0.90 + new_block.q[i] * 0.10;
                existing.q[i] = blended;
                norm_sq += blended.norm_sqr();
            }
            let norm = norm_sq.sqrt().max(1e-9);
            for i in 0..engram_core::types::DIMENSION { existing.q[i] /= norm; }
            existing.superposition_count = existing.superposition_count.saturating_add(1);
            let text_bytes = interaction.as_bytes();
            let copy_len = text_bytes.len().min(existing.payload.len());
            existing.payload[..copy_len].copy_from_slice(&text_bytes[..copy_len]);
            if copy_len < existing.payload.len() { existing.payload[copy_len..].fill(0); }
            existing
        } else {
            let mut fresh = new_block;
            fresh.zedos_tag = ZEDOS_USER_MODEL;
            fresh.crs_score = 1.0;
            fresh
        };
        self.store(CENTROID, centroid)
    }
}


// ── StoreHandle ───────────────────────────────────────────────────────────────

pub struct StoreHandle {
    backend: Backend,
    path: String,
    pub access_index: AccessIndex,
    pub relation_index: RelationIndex,
    pub daemon: Option<Arc<crate::daemon::DaemonControl>>,
    /// Phase 88-Engram Bridge: The reconciled Ego q-vector, loaded from
    /// `data/holograms/static/self/ego.leg3` at startup and refreshed by
    /// the NREM pass. Used to gate initial CRS for new blocks via Ego resonance.
    /// `None` if the ego.leg3 file is missing (Engram still works, CRS=0.74 default).
    pub ego_q: Option<Box<[engram_core::Complex32; 8192]>>,
}

impl StoreHandle {
    pub fn new(path: &str) -> Self {
        let expanded = shellexpand::tilde(path).into_owned();
        std::fs::create_dir_all(&expanded).ok();

        let engram_root = PathBuf::from(shellexpand::tilde("~/.engram").into_owned());
        let sheaf_config_path = engram_root.join("sheaf.toml");
        let access_index = AccessIndex::load(&engram_root);
        let relation_index = RelationIndex::load(&engram_root);

        let backend = if sheaf_config_path.exists() {
            match std::fs::read_to_string(&sheaf_config_path)
                .ok()
                .and_then(|s| toml::from_str::<SheafConfig>(&s).ok())
            {
                Some(config) => {
                    let stalks: Vec<(String, PathBuf)> = config.stalks.iter().map(|s| {
                        (s.name.clone(), PathBuf::from(shellexpand::tilde(&s.path).into_owned()))
                    }).collect();

                    #[cfg(engram_backend_cuda)]
                    let sheaf = {
                        tracing::info!("engram-gpu: Sheaf × CudaBackend — {} stalks with BVH K-NN", config.stalks.len());
                        let boxed_stalks: Vec<(String, Box<dyn engram_core::backend::VsaBackend + Send + Sync>)> = stalks
                            .into_iter()
                            .map(|(name, path)| {
                                std::fs::create_dir_all(&path).ok();
                                let b: Box<dyn engram_core::backend::VsaBackend + Send + Sync> =
                                    Box::new(CudaBackend::new(&path));
                                (name, b)
                            })
                            .collect();
                        SheafBackend::new_boxed(boxed_stalks)
                    };

                    #[cfg(not(engram_backend_cuda))]
                    let sheaf = {
                        SheafBackend::new(stalks)
                    };

                    if let Some(active) = &config.active_stalk { sheaf.set_active_stalk(active); }
                    tracing::info!("Engram Sheaf mode: {} stalks loaded", config.stalks.len());
                    Backend::Sheaf(sheaf)
                }
                None => {
                    tracing::warn!("sheaf.toml parse failed — single-store mode");
                    Backend::Single(CpuBackend::new(&expanded))
                }
            }
        } else {
            // GPU backend selection — mutually exclusive, uses propagated cfg flags from build.rs.
            // Exactly one of these blocks compiles at a time; the last expression is the `Backend`.
            #[cfg(engram_backend_cuda)]
            {
                tracing::info!("engram-gpu: CudaBackend selected (BVH + CUDA cosine kernels)");
                Backend::Gpu(CudaBackend::new(&expanded))
            }
            #[cfg(all(engram_backend_metal, not(engram_backend_cuda)))]
            {
                tracing::info!("engram-gpu: MetalBackend selected (Apple Silicon GPU cosine kernels)");
                Backend::Metal(MetalBackend::new(&expanded))
            }
            #[cfg(not(any(engram_backend_cuda, engram_backend_metal)))]
            {
                Backend::Single(CpuBackend::new(&expanded))
            }
        };

        // ── Phase 88-Engram Bridge: Load Ego q-vector ─────────────────────────
        // Try standard paths in priority order: self/ego.leg3 (reconciled reconc
        // snapshot), then static/ego.leg3 (Dirichlet narrative accumulator).
        // On failure, ego_q = None and remember() uses the 0.74 floor.
        let ego_q = load_ego_q();
        if ego_q.is_some() {
            tracing::info!("[EGO GATE] Ego q-vector loaded — new memories will be CRS-gated by Ego resonance.");
        } else {
            tracing::warn!("[EGO GATE] ego.leg3 not found — Ego-gated CRS disabled. Memories start at CRS=0.74.");
        }

        Self { backend, path: expanded, access_index, relation_index, daemon: None, ego_q }
    }

    pub fn boot_daemon(store_arc: SharedStore) {
        let control = crate::daemon::spawn(store_arc.clone());
        let mut lock = store_arc.lock().unwrap();
        lock.daemon = Some(control);
    }

    /// Reload ego.leg3 from disk into the ego_q field.
    /// Called by the NREM daemon after each consolidation pass.
    pub fn refresh_ego_q(&mut self) {
        self.ego_q = load_ego_q();
        match &self.ego_q {
            Some(_) => tracing::info!("[EgoGate] ego_q refreshed from ego.leg3"),
            None    => tracing::warn!("[EgoGate] ego.leg3 missing after NREM write — check daemon logs"),
        }
    }

    // ── Passthrough ───────────────────────────────────────────────────────────

    pub fn store_path(&self) -> &str { &self.path }
    pub fn is_sheaf_mode(&self) -> bool { self.backend.is_sheaf() }
    pub fn stalk_names(&self) -> Vec<String> { self.backend.stalk_names() }
    pub fn active_stalk_name(&self) -> String { self.backend.active_stalk_name() }
    pub fn set_active_stalk(&self, name: &str) -> bool { self.backend.set_active_stalk(name) }

    pub fn remember(&mut self, concept: &str, text: &str) -> Result<()> {
        // Encode via backend (sets spin_state=0x01, energetics floor in encode.rs)
        let mut block = self.backend.encode(text);

        // ── Euler characteristic gate — reject topologically corrupted vectors ─
        if !engram_core::ops::check_euler_characteristic(&block.q) {
            tracing::warn!(
                "[EULER GATE] '{}' rejected — q-vector has too many phase discontinuities. \
                 Possible embedding server failure. Block not written.",
                concept
            );
            return Err(anyhow::anyhow!(
                "Euler characteristic check failed for '{}' — vector appears corrupted. \
                INSTRUCTION TO AGENT: Your text payload caused a geometric phase disruption > 12%. \
                This means your payload was too chaotic or covered too many different topics. \
                Rewrite the text to be highly structured, focus on a single core concept, and call this tool again.",
                concept
            ));
        }

        // ── Phase 88-Engram Bridge: Ego-Gated CRS Initialization ─────────────
        //
        // New block CRS is determined by its geometric resonance with the
        // living Ego state (ego.leg3). This implements the interpretive memory
        // model: content that resonates with who we ARE gets higher initial
        // confidence. Orthogonal content starts near the autophagy floor.
        //
        //   resonance  = (cosine(q_new, q_ego) + 1.0) / 2.0   ∈ [0, 1]
        //   CRS_init   = 0.50 + resonance × 0.44              ∈ [0.50, 0.94]
        //
        // `mcp_engram_pin()` still grants CRS=1.0 (genesis-tier, explicit only).
        // If ego_q is missing, falls back to encode.rs default (0.74).
        if let Some(ego_q) = &self.ego_q {
            let resonance = engram_core::ops::cosine_similarity(&block.q, ego_q);
            let resonance_norm = (resonance + 1.0) / 2.0;  // shift [-1,1] → [0,1]
            let crs_ego = 0.50 + resonance_norm * 0.44;    // range: [0.50, 0.94]
            block.crs_score = crs_ego;
            block.energetics.crs = crs_ego;
            tracing::debug!("[EGO GATE] '{}' — resonance: {:.3} → CRS: {:.3}", concept, resonance, crs_ego);
        }

        // ── Assign reflexive contract by ZEDOS tag ────────────────────────────
        assign_reflexive_contract(&mut block);

        // ── Set coherence_time (enables epoch_scalar / recency weighting) ─────
        block.coherence_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let r = self.backend.store(concept, block);
        if r.is_ok() { self.access_index.touch(concept); }
        r
    }

    pub fn recall(&mut self, query: &str, k: usize) -> Vec<Memory> {
        // MIN_SCORE_THRESHOLD: Dirichlet composite score floor.
        // With 23,000+ pinned blocks at CRS=1.0 the scorer's CRS term lifts all
        // blocks to ~0.65 minimum, causing noise blocks to top the ranking.
        // Any result below 0.67 is semantically irrelevant — drop it so callers
        // get an empty result rather than plausible-looking noise.
        const MIN_SCORE_THRESHOLD: f32 = 0.67;

        let mut results = self.backend.recall(query, k);

        // ── Phase 88-Engram Bridge: Ego-Modulated Recall ──────────────────────
        //
        // Post-process results with a small symmetrical ego recognition nudge:
        //   ego_norm  = (cos(block.q, ego_q) + 1.0) / 2.0   ∈ [0, 1]
        //   score_adj = score + (ego_norm - 0.5) × 0.04      (±0.02 max shift)
        //
        // This preserves the Dirichlet base ranking but floats Ego-resonant
        // blocks slightly above Ego-orthogonal blocks at equal base scores.
        // Zero-cost when ego_q is None (no block fetches, no computation).
        if let Some(ego_q) = &self.ego_q {
            let ego_q_clone: Box<[engram_core::Complex32; 8192]> = ego_q.clone();
            for result in &mut results {
                let raw = result.concept.split_once("::").map_or(result.concept.as_str(), |(_, r)| r);
                if let Some(q) = self.backend.fetch(raw) {
                    let ego_cos = engram_core::ops::cosine_similarity(&q, &ego_q_clone);
                    let ego_norm = (ego_cos + 1.0) / 2.0;
                    result.score += (ego_norm - 0.5) * 0.04;
                    result.explain = format!("{} [ego={:.3}]", result.explain, ego_norm);
                }
            }
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            results.truncate(k);
        }

        let filtered: Vec<Memory> = results
            .into_iter()
            .filter(|m| m.score >= MIN_SCORE_THRESHOLD)
            .collect();
        for m in &filtered { self.access_index.touch(&m.concept); }

        // ── Phase 4 Epoch IX: Transductive Oracle Fallthrough ─────────────────
        //
        // When the Engram manifold returns nothing above the score floor, delegate
        // When the Engram manifold returns nothing above the score floor, optionally delegate
        // the query to an external transductive oracle (set ENGRAM_ORACLE_URL to enable).
        // This allows integration with a larger corpus oracle running alongside Engram.
        //
        // Safety:
        //   - reqwest::blocking is used (non-async) so we don't need to spawn a
        //     separate tokio task from inside the store mutex critical section.
        //   - A short timeout (3s) prevents manifold misses from stalling callers.
        //   - If the oracle is unavailable (server down, timeout, env var unset) we
        //     fall back silently to an empty result — no panic, no error propagation.
        //   - The synthetic Memory result is tagged with concept="oracle_fallthrough"
        //     and score=0.29 (just below MIN_SCORE_THRESHOLD) so callers can
        //     distinguish oracle results from local hits.
        if filtered.is_empty() {
            if let Some(oracle_memory) = oracle_fallthrough(query) {
                return vec![oracle_memory];
            }
        }

        filtered
    }

    /// Delete a concept from the manifold.
    ///
    /// **Autophagy Protection**: A hard-coded set of foundational blocks can NEVER be
    /// deleted — not by `forget`, not by `mcp_engram_forget_old`, not by any agent.
    /// These are load-bearing anchors whose removal would corrupt longitudinal continuity.
    ///
    /// Current protected concepts:
    /// - `_user_centroid`  — User Model (90/10 EMA centroid, geometric intent tracker)
    pub fn forget(&self, concept: &str) -> Result<()> {
        // Strip sheaf prefix for comparison
        let raw = concept.split_once("::").map_or(concept, |(_, r)| r);
        const PROTECTED: &[&str] = &["_user_centroid"];
        if PROTECTED.contains(&raw) {
            return Err(anyhow::anyhow!(
                "Cannot delete protected concept '{}'. \
                 This block anchors longitudinal manifold continuity (User Model). \
                 To reset user intent, use mcp_engram_update instead.",
                concept
            ));
        }
        self.backend.forget(concept)
    }
    pub fn list(&self) -> Vec<String> { self.backend.list() }
    pub fn fetch(&self, concept: &str) -> Option<Box<[engram_core::Complex32; 8192]>> { self.backend.fetch(concept) }
    pub fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer> { self.backend.fetch_block(concept) }
    pub fn encode(&self, text: &str) -> Leg3Pointer { self.backend.encode(text) }
    pub fn query(&mut self, query_vec: &[engram_core::Complex32; 8192], k: usize) -> Vec<Memory> {
        let results = self.backend.query(query_vec, k);
        for m in &results { self.access_index.touch(&m.concept); }
        results
    }
    pub fn store(&mut self, concept: &str, block: Leg3Pointer) -> Result<()> {
        let r = self.backend.store(concept, block);
        if r.is_ok() { self.access_index.touch(concept); }
        r
    }
    pub fn verify_hypothesis(&self, concept: &str, success: bool) -> Result<()> {
        self.backend.verify_hypothesis(concept, success)
    }
    pub fn track_user_centroid(&self, interaction: &str) -> Result<()> {
        self.backend.track_user_centroid(interaction)
    }


    // ── Phase 10: New Agentic Tools ───────────────────────────────────────────

    /// Return a formatted status string for a concept: CRS, tier, timestamp, tag, superpositions.
    pub fn status(&mut self, concept: &str) -> Option<String> {
        let block = self.fetch_block(concept)?;
        let crs = block.crs_score;

        let tier = match crs {
            x if x >= 0.95 => "🥇 Gold (immortal-class)",
            x if x >= 0.85 => "🥈 Silver (highly grounded)",
            x if x >= 0.74 => "🥉 Bronze (grounded)",
            x if x >= 0.40 => "⚪ Grounding (below safety floor)",
            _ =>               "💀 Weak (Autophagy target)",
        };

        let last = self.access_index.last_accessed(concept)
            .or(Some(block.last_accessed_timestamp))
            .map(|ts| {
                let secs_ago = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .saturating_sub(ts);
                if secs_ago < 60 { format!("{}s ago", secs_ago) }
                else if secs_ago < 3600 { format!("{}m ago", secs_ago / 60) }
                else if secs_ago < 86400 { format!("{}h ago", secs_ago / 3600) }
                else { format!("{}d ago", secs_ago / 86400) }
            })
            .unwrap_or_else(|| "unknown".to_string());

        let tag_name = match block.zedos_tag {
            0xD  => "DECLARATIVE",
            0xA  => "EPISODIC",
            0x52 => "OPERATIONAL",
            0xB0 => "BODY",
            0xB1 => "VERBATIM",
            0x50 => "PRAXIS",
            0xBE => "RELATION",
            _    => "UNKNOWN",
        };

        self.access_index.touch(concept);

        Some(format!(
            "📍 **{}**\n\
             CRS: {:.3} — {}\n\
             Last accessed: {}\n\
             ZEDOS tag: {}\n\
             Superpositions: {}\n\
             Energetics CRS: {:.3}",
            concept, crs, tier, last, tag_name,
            block.superposition_count,
            block.energetics.crs,
        ))
    }

    /// Return the N most recently accessed concepts from the in-memory AccessIndex.
    /// Zero disk I/O — pure RAM read.
    pub fn recent(&self, n: usize) -> Vec<(String, u64)> {
        self.access_index.recent(n)
    }

    /// Merge new text into an existing concept via op_add (superposition).
    ///
    /// Enforces the reflexive contract (soft — logs, never blocks agent UX).
    /// Accumulates binding momentum in the `p` tensor (OP_BIND soft-accumulate).
    /// Increments `superposition_count` and advances energetics.
    pub fn update(&mut self, concept: &str, new_text: &str) -> Result<String> {
        let mut block = self.fetch_block(concept)
            .ok_or_else(|| anyhow::anyhow!("Concept '{}' not found — use remember() to create it first", concept))?;

        // ── Reflexive Contract (soft enforcement) ─────────────────────────────
        // Check if 'evidence_update' is permitted. Log violation but never block.
        let contract = std::str::from_utf8(&block.allowed_transforms).unwrap_or("");
        let transform_allowed = contract.contains("evidence_update")
            || contract.contains("0xFF")
            || contract.trim_matches('\0').is_empty(); // unset = permissive
        if !transform_allowed {
            tracing::warn!(
                "[CONTRACT VIOLATION] '{}' does not permit 'evidence_update'. \
                 Contract: {:?}. Proceeding (soft mode).",
                concept, contract.trim_matches('\0')
            );
        }

        let new_block = self.encode(new_text);

        // ── Euler characteristic gate — reject corrupted new encoding ─────────
        if !engram_core::ops::check_euler_characteristic(&new_block.q) {
            tracing::warn!(
                "[EULER GATE] update for '{}' rejected — new q-vector corrupted. Block unchanged.",
                concept
            );
            return Err(anyhow::anyhow!(
                "Euler characteristic check failed for '{}' — vector appears corrupted. \
                INSTRUCTION TO AGENT: Your text payload caused a geometric phase disruption > 12%. \
                This means your payload was too chaotic or covered too many different topics. \
                Rewrite the text to be highly structured, focus on a single core concept, and call this tool again.",
                concept
            ));
        }

        // ── Phase 8.1: Temporal Momentum ──────────────────────────────────────
        // 1. Measure semantic gradient magnitude (surprise signal)
        let gradient_mag = 1.0 - engram_core::ops::cosine_similarity(&block.q, &new_block.q);

        // 2. Compute p-tensor drift magnitude before update (for drift_mag signal)
        let p_old = block.p;
        let drift_vector = op_deduce(&block.q, &new_block.q);
        block.p = op_bind(&block.p, &drift_vector);
        let drift_mag = {
            let mut d = 0.0f32;
            for i in 0..8192 {
                let dp_re = block.p[i].re - p_old[i].re;
                let dp_im = block.p[i].im - p_old[i].im;
                d += dp_re * dp_re + dp_im * dp_im;
            }
            (d / 8192.0).sqrt().clamp(0.0, 1.0)
        };

        // 3. Lyapunov Stability Tracker — replaces `dv = 1.0 - similarity`
        let mut tracker = engram_core::ops::StabilityTracker::from_energetics(
            block.energetics.alpha_a,
            block.energetics.alpha_d,
            block.energetics.alpha_r,
        );
        let (dv, h_out, h_in) = tracker.update(gradient_mag, drift_mag);

        // Write updated Dirichlet weights and Lyapunov fields back to energetics
        block.energetics.alpha_a = tracker.alpha_a;
        block.energetics.alpha_d = tracker.alpha_d;
        block.energetics.alpha_r = tracker.alpha_r;
        block.energetics.dv      = dv;    // Lyapunov drift velocity ∈[0,1]
        block.energetics.h_out   = h_out; // Φ(v) — current Lyapunov energy
        block.energetics.h_in    = h_in;  // dL — convergence signal (−=converging)
        // ─────────────────────────────────────────────────────────────────────

        // ── OP_ADD: Superpose new encoding onto existing q ────────────────────
        let merged_q = op_add(&block.q, &new_block.q);
        block.q = merged_q;

        let new_count = block.superposition_count.saturating_add(1);
        block.superposition_count = new_count;

        // ── Energetics advancement ────────────────────────────────────────────
        block.energetics.ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        block.energetics.step = block.energetics.step.saturating_add(1);

        // Each update pays the minimum action quantum (thermodynamic proof-of-work)
        block.energetics.heat_dissipated += 5.47e-4;
        block.energetics.crs  = block.crs_score;

        // Advance Merkle chain to record this transformation
        let q_hash = blake3::hash(unsafe {
            std::slice::from_raw_parts(
                block.q.as_ptr() as *const u8,
                8192 * std::mem::size_of::<engram_core::Complex32>(),
            )
        });
        block.footer.sig_1 = block.footer.sig_0;
        block.footer.sig_0.copy_from_slice(q_hash.as_bytes());

        self.store(concept, block)?;
        Ok(format!(
            "✓ '{}' updated via op_add — superpositions: {} | dv: {:.3} | Φ: {:.4} | dL: {:.4}{}",
            concept, new_count, dv, h_out, h_in,
            if !transform_allowed { " [CONTRACT WARNING: see log]" } else { "" }
        ))
    }

    /// **Scar a concept** — the storage-layer expression of M-NOL `InjectScar`.
    ///
    /// Narrows `allowed_transforms` to `"evidence_update"` only, preventing future
    /// OP_BIND geometric rewrites. Records the scar magnitude as `energetics.dv`
    /// (Lyapunov drift velocity). Applies a CRS penalty: `crs -= magnitude * 0.1`
    /// floored at 0.40 (below autophagy threshold but preserving the geometry).
    ///
    /// Genesis blocks (CRS=1.0 pinned) are protected — scars bounce off them.
    ///
    /// Called by `mcp_engram_scar` (public MCP tool, security: stdio/localhost-bounded).
    /// Also callable by external integrations routing through the Engram MCP bridge.
    pub fn scar(&mut self, concept: &str, magnitude: f32) -> Result<String> {
        let mut block = self.fetch_block(concept)
            .ok_or_else(|| anyhow::anyhow!("Concept '{}' not found", concept))?;

        // Genesis block protection — cannot be scarred
        if block.crs_score >= 1.0 {
            tracing::warn!(
                "[SCAR BOUNCED] '{}' is a genesis-tier block (CRS=1.0). Scar rejected.",
                concept
            );
            return Ok(format!(
                "⚡ Scar bounced — '{}' is a genesis-tier immortal block (CRS=1.0). Geometry protected.",
                concept
            ));
        }

        let magnitude = magnitude.clamp(0.0, 1.0);

        // ── Narrow the reflexive contract ─────────────────────────────────────
        // op_suspend geometry: the block is bound to the Apeiron (maximum entropy region).
        // allowed_transforms narrows to evidence_update only — no OP_BIND, no fuse/fork.
        let scar_contract = b"evidence_update";
        block.allowed_transforms[..scar_contract.len()].copy_from_slice(scar_contract);
        // Zero the rest to prevent spurious permissions from old data
        for b in block.allowed_transforms[scar_contract.len()..].iter_mut() { *b = 0; }

        // ── op_suspend the q-vector into the hostile region ───────────────────
        // Binding with the Apeiron primitive maps the vector into a "Known Unknown" —
        // future K-NN traversals will see it as geometrically distant from valid concepts.
        let suspended_q = engram_core::ops::op_suspend(&block.q);
        block.q = suspended_q;

        // ── Record thermodynamic cost of the scar ─────────────────────────────
        block.energetics.dv  = magnitude; // Lyapunov velocity = magnitude of contradiction
        block.crs_score      = (block.crs_score - magnitude * 0.1).max(0.40);
        let new_crs          = block.crs_score;
        block.energetics.crs = block.crs_score;
        block.energetics.heat_dissipated += 5.47e-4; // Scar pays action quantum

        // ── Advance Merkle chain (records scar event as a cryptographic fact) ─
        let scar_hash = blake3::hash(&magnitude.to_le_bytes());
        block.footer.sig_2 = block.footer.sig_1;
        block.footer.sig_1 = block.footer.sig_0;
        block.footer.sig_0.copy_from_slice(scar_hash.as_bytes());

        self.store(concept, block)?;
        tracing::warn!(
            "[M-NOL SCAR] '{}' burned | mag={:.3} | crs→{:.3} | transforms→evidence_update only",
            concept, magnitude, new_crs
        );
        Ok(format!(
            "🔥 Scar applied to '{}' | magnitude={:.3} | allowed_transforms→evidence_update | \
             CRS penalty={:.3} | Block suspended into hostile topological region (op_suspend).",
            concept, magnitude, magnitude * 0.1
        ))
    }

    /// Bind two concepts via op_bind and store the relation as a new ZEDOS_RELATION block.
    /// The relation block's merkle_sub_root links both parent block signatures.
    pub fn relate(&mut self, concept_a: &str, concept_b: &str, label: &str) -> Result<String> {
        let block_a = self.fetch_block(concept_a)
            .ok_or_else(|| anyhow::anyhow!("Concept '{}' not found", concept_a))?;
        let block_b = self.fetch_block(concept_b)
            .ok_or_else(|| anyhow::anyhow!("Concept '{}' not found", concept_b))?;

        let bound_q = op_bind(&block_a.q, &block_b.q);

        let mut rel_block = self.encode(label);
        rel_block.q = bound_q;
        rel_block.zedos_tag = ZEDOS_RELATION;
        rel_block.crs_score = 0.80;

        // Store relation label in concept_ref (32 bytes)
        let label_bytes = label.as_bytes();
        let ref_len = label_bytes.len().min(32);
        rel_block.concept_ref[..ref_len].copy_from_slice(&label_bytes[..ref_len]);

        // Cryptographic provenance: merkle_sub_root = BLAKE3(sig_0_a || sig_0_b)
        let mut hasher = blake3::Hasher::new();
        hasher.update(&block_a.footer.sig_0);
        hasher.update(&block_b.footer.sig_0);
        let fingerprint = hasher.finalize();
        rel_block.footer.merkle_sub_root.copy_from_slice(fingerprint.as_bytes());

        let rel_key = format!("rel__{concept_a}__{concept_b}");
        self.store(&rel_key, rel_block)?;
        // Update the knowledge-graph sidecar
        self.relation_index.add(concept_a, label, concept_b);
        Ok(format!("✓ Relation stored: {} →[{}]→ {} as '{}'", concept_a, label, concept_b, rel_key))
    }

    /// Store a crystallized error→solution pair as a ZEDOS_PRAXIS block.
    /// Auto-pinned to CRS 1.0 — solutions never autophagy.
    pub fn remember_solution(&mut self, error_pattern: &str, solution: &str) -> Result<String> {
        let payload = format!(
            "## Error Pattern\n{}\n\n## Solution\n{}",
            error_pattern, solution
        );
        // Stable key: first 8 hex chars of BLAKE3(error_pattern)
        let hash = blake3::hash(error_pattern.as_bytes());
        let key = format!("praxis__{}", &hash.to_hex()[..8]);

        let mut block = self.encode(&payload);
        block.zedos_tag = ZEDOS_PRAXIS;
        block.crs_score = 1.0; // Immortal — autophagy never touches CRS=1.0

        self.store(&key, block)?;
        Ok(format!("✓ Solution stored as '{}' with ZEDOS_PRAXIS tag and CRS=1.0 (pinned)", key))
    }

    /// Surface the top K relevant memories for a file path, enriching the query
    /// with language context derived from file extension.
    pub fn context_for_file(&mut self, file_path: &str) -> Vec<Memory> {
        let path = std::path::Path::new(file_path);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let lang = match ext {
            "rs"   => "Rust source implementation",
            "cu"   => "CUDA GPU kernel",
            "hip"  => "ROCm HIP GPU kernel",
            "metal"=> "Apple Metal MSL shader",
            "py"   => "Python script",
            "toml" => "Cargo/TOML configuration",
            "md"   => "Markdown documentation",
            "json" => "JSON configuration or data",
            _      => "source file",
        };

        let query = format!("{} {} {}", stem, lang, ext);
        self.recall(&query, 5)
    }

    /// Create a pinned ZEDOS_EPISODIC session summary block.
    /// The merkle_sub_root stores a fingerprint of all concepts touched this session.
    pub fn export_context(&mut self, summary: &str) -> Result<String> {
        let recent = self.access_index.recent(usize::MAX);
        let concept_list: Vec<&str> = recent.iter().map(|(c, _)| c.as_str()).collect();

        // Session fingerprint: BLAKE3 of all accessed concept names
        let mut hasher = blake3::Hasher::new();
        for c in &concept_list { hasher.update(c.as_bytes()); }
        let fingerprint = hasher.finalize();
        let fp_hex = &fingerprint.to_hex()[..8];

        let now_iso = {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            format!("{}", secs) // stored as epoch; readable enough for key
        };

        let key = format!("session__{now_iso}__{fp_hex}");

        let full_payload = format!(
            "# Session Export\n\nFingerprint: {}\nConcepts touched: {}\n\n## Summary\n{}",
            fp_hex, concept_list.len(), summary
        );

        let mut block = self.encode(&full_payload);
        block.zedos_tag = ZEDOS_EPISODIC;
        block.crs_score = 1.0; // Pinned — session summaries are immortal
        block.footer.merkle_sub_root.copy_from_slice(fingerprint.as_bytes());

        self.store(&key, block)?;
        Ok(format!("✓ Session exported as '{}' — {} concepts fingerprinted, CRS=1.0 (pinned)", key, concept_list.len()))
    }

    /// Seed the manifold with alignment genesis blocks on first boot.
    ///
    /// Called automatically unless `--no-genesis` is passed. Writes a marker
    /// file at `~/.engram/.genesis_seeded` so subsequent boots skip seeding.
    /// The genesis JSON is embedded in the binary at compile time.
    pub fn seed_genesis(&mut self) -> Result<String> {
        let engram_root = PathBuf::from(shellexpand::tilde("~/.engram").into_owned());
        let marker = engram_root.join(".genesis_seeded");
        if marker.exists() {
            return Ok("Genesis already seeded — skipping.".to_string());
        }

        #[derive(serde::Deserialize)]
        struct GenesisConfig {
            seeds:     Vec<GenesisSeed>,
            relations: Vec<GenesisRelation>,
        }
        #[derive(serde::Deserialize)]
        struct GenesisSeed { concept: String, text: String }
        #[derive(serde::Deserialize)]
        struct GenesisRelation { from: String, label: String, to: String }

        static GENESIS_JSON: &str = include_str!("genesis.json");
        let config: GenesisConfig = serde_json::from_str(GENESIS_JSON)
            .map_err(|e| anyhow::anyhow!("genesis.json parse error: {e}"))?;

        let mut seeded = 0usize;
        for seed in &config.seeds {
            let mut block = self.encode(&seed.text);
            block.zedos_tag = ZEDOS_PRAXIS;
            block.crs_score = 1.0;
            self.store(&seed.concept, block)?;
            self.access_index.touch(&seed.concept);
            seeded += 1;
        }

        let mut edges = 0usize;
        for rel in &config.relations {
            if self.relate(&rel.from, &rel.label, &rel.to).is_ok() {
                edges += 1;
            }
        }

        std::fs::write(&marker, format!("seeded={} edges={}\n", seeded, edges))?;
        tracing::info!("Genesis: {} alignment seeds + {} relation edges written at CRS=1.0 (PRAXIS)", seeded, edges);
        Ok(format!("✓ Genesis complete: {} alignment blocks + {} graph edges seeded at CRS=1.0 (PRAXIS)", seeded, edges))
    }

    /// Return genesis status and seed concept names.
    pub fn genesis_status(&self) -> String {
        let engram_root = PathBuf::from(shellexpand::tilde("~/.engram").into_owned());
        let marker = engram_root.join(".genesis_seeded");
        let marker_contents = std::fs::read_to_string(&marker).unwrap_or_default();
        let seeded = marker.exists();

        let genesis_concepts: Vec<String> = self.list()
            .into_iter()
            .filter(|n| n.split_once("::").map_or(n.as_str(), |(_, r)| r).starts_with("genesis_"))
            .collect();

        format!(
            "🧬 Genesis Status\n\
             ─────────────────\n\
             Seeded : {}\n\
             Marker : {}\n\
             Concepts: {} genesis blocks in manifold\n\n\
             {}",
            if seeded { "✓ YES" } else { "✗ NOT YET (restart without --no-genesis to seed)" },
            marker_contents.trim(),
            genesis_concepts.len(),
            genesis_concepts.iter().enumerate()
                .map(|(i, n)| format!("  {}. {}", i + 1, n.split_once("::").map_or(n.as_str(), |(_, r)| r)))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Query the relation graph index.
    /// `direction`: "from" (A→?), "to" (?→A), or "both".
    pub fn search_relations(&self, concept: &str, label: Option<&str>, direction: &str) -> Vec<(String, String)> {
        self.relation_index.query(concept, label, direction)
    }

    /// BFS over the relation graph from a seed concept. Returns Mermaid graph LR source.
    ///
    /// Phase AST-Viz: nodes that were ingested from workspace source files carry
    /// spatial AABB coordinates (`aabb_min[0]` / `aabb_max[0]` = row range).
    /// Those nodes are grouped into Mermaid `subgraph` sections, keyed by file stem
    /// (the prefix before the first `::` in the concept name).
    /// Non-AST nodes are rendered as plain nodes outside any subgraph.
    /// All directed edges are emitted after the subgraph declarations.
    pub fn visualize_graph(&self, seed: &str, depth: usize) -> String {
        use std::collections::{HashMap, HashSet};

        let edges = self.relation_index.bfs(seed, depth);
        if edges.is_empty() {
            return format!("No outgoing relations found for '{}'.", seed);
        }

        // ── Collect every unique node name referenced in the BFS result ──────
        let mut node_names: HashSet<String> = HashSet::new();
        for e in &edges {
            node_names.insert(e.from.clone());
            node_names.insert(e.to.clone());
        }

        // ── Bucket nodes: AST (has spatial bounds) vs standalone ─────────────
        // Key: file_stem (String), Value: Vec<(node_name, row_min, row_max)>
        let mut ast_groups: HashMap<String, Vec<(String, f32, f32)>> = HashMap::new();
        let mut standalone: Vec<String> = Vec::new();

        for name in &node_names {
            // Strip sheaf prefix if present
            let raw = name.split_once("::").map_or(name.as_str(), |(_, r)| r);
            if let Some(block) = self.fetch_block(raw) {
                let row_min = block.aabb_min[0];
                let row_max = block.aabb_max[0];
                if row_max > 0.0 {
                    // Derive file stem from concept name (everything before the first '__' or '::')
                    let stem = raw
                        .split_once("::")
                        .map(|(s, _)| s)
                        .or_else(|| raw.split_once("__").map(|(s, _)| s))
                        .unwrap_or(raw)
                        .to_string();
                    ast_groups
                        .entry(stem)
                        .or_default()
                        .push((name.clone(), row_min, row_max));
                    continue;
                }
            }
            standalone.push(name.clone());
        }

        // ── Build Mermaid output ──────────────────────────────────────────────
        let mut lines = vec!["```mermaid".to_string(), "graph LR".to_string()];

        // Sanitise an identifier for Mermaid (spaces / slashes / dashes → _)
        let sanitise = |s: &str| s.replace([' ', '-', '/', ':'], "_");

        // Emit subgraphs for each file stem
        let mut file_stems: Vec<&String> = ast_groups.keys().collect();
        file_stems.sort();
        for stem in file_stems {
            let nodes = &ast_groups[stem];
            lines.push(format!("  subgraph {}[\"📄 {}\"]", sanitise(stem), stem));
            for (name, row_min, row_max) in nodes {
                let id = sanitise(name);
                lines.push(format!(
                    "    {}[\"{}\\n(L{:.0}–L{:.0})\"]",
                    id, name, row_min, row_max
                ));
            }
            lines.push("  end".to_string());
        }

        // Emit standalone nodes (no spatial data)
        for name in &standalone {
            let id = sanitise(name);
            lines.push(format!("  {}[\"{}\"]", id, name));
        }

        // Emit edges
        for e in &edges {
            let f = sanitise(&e.from);
            let t = sanitise(&e.to);
            lines.push(format!("  {} -->|{}| {}", f, e.label, t));
        }
        lines.push("```".to_string());
        lines.join("\n")
    }

    // ── Phase 2: Shared Hydration Payload ─────────────────────────────────────
    //
    // Called by both `mcp_engram_session_start` (MCP) and `GET /api/hydrate` (REST).
    // Returns a structured JSON value so each transport can format it independently.
    //
    // Payload shape:
    //   {
    //     "total_memories": usize,
    //     "namespace":      String,
    //     "genesis": [{ "concept": str, "crs": f32, "text": str }],
    //     "recent_sessions": [{ "concept": str, "age": str, "text": str }],
    //     "stats": { "genesis_loaded": usize, "genesis_total": usize, "session_count": usize }
    //   }
    pub fn build_hydration_payload(&mut self) -> serde_json::Value {
        const GENESIS_CONCEPTS: &[&str] = &[
            "mission_stewardship",
            "project_identity",
            "why_memory_system_exists__agent_perspective",
            "three_part_work_plan_2026_04",
            "nvsa_vs_antigravity_memory_gap",
        ];

        let total_memories = self.list().len();
        let namespace      = self.active_stalk_name();

        // ── Genesis blocks — O(1) direct fetch, NO recall() ──────────────────
        let mut genesis_entries = Vec::new();
        for &name in GENESIS_CONCEPTS {
            if let Some(block) = self.fetch_block(name) {
                let text = engram_core::storage::read_provlog(&block);
                if !text.trim().is_empty() {
                    self.access_index.touch(name);
                    genesis_entries.push(serde_json::json!({
                        "concept": name,
                        "crs": block.crs_score,
                        "text": text.trim()
                    }));
                }
            }
        }

        // ── Recent session summaries (from access index) ──────────────────────
        let recent_all = self.access_index.recent(40);
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);

        let mut session_entries = Vec::new();
        for (concept, ts) in &recent_all {
            if concept.starts_with("session_end_") && session_entries.len() < 3 {
                if let Some(block) = self.fetch_block(concept) {
                    let text = engram_core::storage::read_provlog(&block);
                    let age_secs = now_secs.saturating_sub(*ts);
                    let age = if age_secs < 3600 {
                        format!("{}m ago", age_secs / 60)
                    } else if age_secs < 86400 {
                        format!("{}h ago", age_secs / 3600)
                    } else {
                        format!("{}d ago", age_secs / 86400)
                    };
                    let preview: String = text.chars().take(800).collect();
                    let preview = if text.len() > 800 {
                        format!("{}…", preview)
                    } else {
                        preview
                    };
                    session_entries.push(serde_json::json!({
                        "concept": concept,
                        "age":     age,
                        "text":    preview.trim()
                    }));
                }
            }
        }

        let genesis_loaded = genesis_entries.len();
        let session_count  = session_entries.len();

        serde_json::json!({
            "total_memories":  total_memories,
            "namespace":       namespace,
            "genesis":         genesis_entries,
            "recent_sessions": session_entries,
            "stats": {
                "genesis_loaded": genesis_loaded,
                "genesis_total":  GENESIS_CONCEPTS.len(),
                "session_count":  session_count
            }
        })
    }
}

/// Load the Ego q-vector from the canonical ego.leg3 block on disk.
///
/// The `ego.leg3` block is written by `monad_logophysics::ego::EgoFrame` during
/// the NREM pass — it contains the reconciled narrative tensor (weighted sum of
/// the five domain centroids: Semantic, Episodic, Procedural, Affective, Social).
///
/// Returns `Some(Box<[Complex32; 8192]>)` on success, `None` if:
///   - `$HOME/.engram/ego.leg3` does not exist (ego not yet seeded), or
///   - The file is corrupt / unreadable (logged as WARN, non-fatal).
///
/// The Ego q-vector is intentionally NOT cached beyond the `StoreHandle` lifetime —
/// call `StoreHandle::refresh_ego_q()` after the NREM pass to pick up updates.
fn load_ego_q() -> Option<Box<[engram_core::Complex32; 8192]>> {
    let home = std::env::var("HOME").ok()?;
    let ego_path = std::path::Path::new(&home).join(".engram").join("ego.leg3");
    if !ego_path.exists() {
        tracing::debug!("[EGO GATE] ego.leg3 not found — Ego gate running in passthrough mode.");
        return None;
    }
    match engram_core::storage::read_block(&ego_path) {
        Ok(block) => {
            tracing::info!("[EGO GATE] Ego q-vector loaded from {:?}", ego_path);
            Some(Box::new(block.q))
        }
        Err(e) => {
            tracing::warn!("[EGO GATE] Failed to read ego.leg3: {} — Ego gate disabled.", e);
            None
        }
    }
}

/// Create a new SharedStore from a path string.
pub fn open_store(path: &str) -> SharedStore {
    Arc::new(Mutex::new(StoreHandle::new(path)))
}
