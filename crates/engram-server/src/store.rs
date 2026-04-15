//! Thread-safe wrapper around the VsaBackend for server use.
//!
//! Detects `~/.engram/sheaf.toml` on boot. If present, opens a multi-manifold
//! `SheafBackend`. Otherwise falls back to a single `CpuBackend` for full
//! backwards compatibility.

use engram_core::backend::{CpuBackend, Memory, VsaBackend, SheafBackend};
use engram_core::types::Leg3Pointer;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Thread-safe, shared backend handle used across MCP and REST handlers.
pub type SharedStore = Arc<Mutex<StoreHandle>>;

/// Sheaf config — parses `~/.engram/sheaf.toml`.
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

/// Internal enum to hold either a single or multi-manifold backend.
enum Backend {
    Single(CpuBackend),
    Sheaf(SheafBackend),
}

impl Backend {
    fn remember(&self, concept: &str, text: &str) -> anyhow::Result<()> {
        match self {
            Backend::Single(b) => b.remember(concept, text),
            Backend::Sheaf(b)  => b.remember(concept, text),
        }
    }
    fn recall(&self, q: &str, k: usize) -> Vec<Memory> {
        match self {
            Backend::Single(b) => b.recall(q, k),
            Backend::Sheaf(b)  => b.recall(q, k),
        }
    }
    fn forget(&self, concept: &str) -> anyhow::Result<()> {
        match self {
            Backend::Single(b) => b.forget(concept),
            Backend::Sheaf(b)  => b.forget(concept),
        }
    }
    fn list(&self) -> Vec<String> {
        match self {
            Backend::Single(b) => b.list(),
            Backend::Sheaf(b)  => b.list(),
        }
    }
    fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer> {
        match self {
            Backend::Single(b) => b.fetch_block(concept),
            Backend::Sheaf(b)  => b.fetch_block(concept),
        }
    }
    fn fetch(&self, concept: &str) -> Option<Box<[engram_core::Complex32; 8192]>> {
        match self {
            Backend::Single(b) => b.fetch(concept),
            Backend::Sheaf(b)  => b.fetch(concept),
        }
    }
    fn encode(&self, text: &str) -> Leg3Pointer {
        match self {
            Backend::Single(b) => b.encode(text),
            Backend::Sheaf(b)  => b.encode(text),
        }
    }
    fn query(&self, q: &[engram_core::Complex32; 8192], k: usize) -> Vec<Memory> {
        match self {
            Backend::Single(b) => b.query(q, k),
            Backend::Sheaf(b)  => b.query(q, k),
        }
    }
    fn store(&self, concept: &str, block: Leg3Pointer) -> anyhow::Result<()> {
        match self {
            Backend::Single(b) => b.store(concept, block),
            Backend::Sheaf(b)  => b.store(concept, block),
        }
    }
    fn set_active_stalk(&self, name: &str) -> bool {
        match self {
            Backend::Single(_) => false,
            Backend::Sheaf(b)  => b.set_active_stalk(name),
        }
    }
    fn stalk_names(&self) -> Vec<String> {
        match self {
            Backend::Single(_) => vec!["default".to_string()],
            Backend::Sheaf(b)  => b.stalk_names().into_iter().map(|s| s.to_string()).collect(),
        }
    }
    fn active_stalk_name(&self) -> String {
        match self {
            Backend::Single(_) => "default".to_string(),
            Backend::Sheaf(b)  => b.active_stalk_name().to_string(),
        }
    }
    fn is_sheaf(&self) -> bool {
        matches!(self, Backend::Sheaf(_))
    }
}

/// Wraps a VsaBackend with its configured store path.
pub struct StoreHandle {
    backend: Backend,
    path: String,
    pub daemon: Option<Arc<crate::daemon::DaemonControl>>,
}

impl StoreHandle {
    pub fn new(path: &str) -> Self {
        let expanded = shellexpand::tilde(path).into_owned();
        std::fs::create_dir_all(&expanded).ok();

        // Detect sheaf.toml at the parent engram dir (~/.engram/sheaf.toml)
        let engram_root = PathBuf::from(shellexpand::tilde("~/.engram").into_owned());
        let sheaf_config_path = engram_root.join("sheaf.toml");

        let backend = if sheaf_config_path.exists() {
            match std::fs::read_to_string(&sheaf_config_path)
                .ok()
                .and_then(|s| toml::from_str::<SheafConfig>(&s).ok())
            {
                Some(config) => {
                    let stalks: Vec<(String, PathBuf)> = config.stalks.iter().map(|s| {
                        (s.name.clone(), PathBuf::from(shellexpand::tilde(&s.path).into_owned()))
                    }).collect();
                    let sheaf = SheafBackend::new(stalks);
                    if let Some(active) = &config.active_stalk {
                        sheaf.set_active_stalk(active);
                    }
                    tracing::info!("Engram Sheaf mode: {} stalks loaded from {:?}", config.stalks.len(), sheaf_config_path);
                    Backend::Sheaf(sheaf)
                }
                None => {
                    tracing::warn!("sheaf.toml found but could not be parsed, falling back to single-store mode");
                    Backend::Single(CpuBackend::new(&expanded))
                }
            }
        } else {
            Backend::Single(CpuBackend::new(&expanded))
        };

        Self { backend, path: expanded, daemon: None }
    }

    pub fn boot_daemon(store_arc: SharedStore) {
        let control = crate::daemon::spawn(store_arc.clone());
        let mut lock = store_arc.lock().unwrap();
        lock.daemon = Some(control);
    }

    pub fn store_path(&self) -> &str { &self.path }
    pub fn is_sheaf_mode(&self) -> bool { self.backend.is_sheaf() }
    pub fn stalk_names(&self) -> Vec<String> { self.backend.stalk_names() }
    pub fn active_stalk_name(&self) -> String { self.backend.active_stalk_name() }
    pub fn set_active_stalk(&self, name: &str) -> bool { self.backend.set_active_stalk(name) }

    pub fn remember(&self, concept: &str, text: &str) -> anyhow::Result<()> {
        self.backend.remember(concept, text)
    }
    pub fn recall(&self, query: &str, k: usize) -> Vec<Memory> {
        self.backend.recall(query, k)
    }
    pub fn forget(&self, concept: &str) -> anyhow::Result<()> {
        self.backend.forget(concept)
    }
    pub fn list(&self) -> Vec<String> {
        self.backend.list()
    }
    pub fn fetch(&self, concept: &str) -> Option<Box<[engram_core::Complex32; 8192]>> {
        self.backend.fetch(concept)
    }
    pub fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer> {
        self.backend.fetch_block(concept)
    }
    pub fn encode(&self, text: &str) -> Leg3Pointer {
        self.backend.encode(text)
    }
    pub fn query(&self, query_vec: &[engram_core::Complex32; 8192], k: usize) -> Vec<Memory> {
        self.backend.query(query_vec, k)
    }
    pub fn store(&self, concept: &str, block: Leg3Pointer) -> anyhow::Result<()> {
        self.backend.store(concept, block)
    }
}

/// Create a new SharedStore from a path string.
pub fn open_store(path: &str) -> SharedStore {
    Arc::new(Mutex::new(StoreHandle::new(path)))
}
