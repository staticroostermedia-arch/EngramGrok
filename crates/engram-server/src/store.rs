//! Thread-safe wrapper around the VsaBackend for server use.

use engram_core::backend::{CpuBackend, Memory, VsaBackend};
use std::sync::{Arc, Mutex};

/// Thread-safe, shared backend handle used across MCP and REST handlers.
pub type SharedStore = Arc<Mutex<StoreHandle>>;

/// Wraps a VsaBackend with its configured store path.
pub struct StoreHandle {
    pub backend: CpuBackend,
    path: String,
    pub daemon: Option<Arc<crate::daemon::DaemonControl>>,
}

impl StoreHandle {
    pub fn new(path: &str) -> Self {
        let expanded = shellexpand::tilde(path).into_owned();
        std::fs::create_dir_all(&expanded).ok();
        Self {
            backend: CpuBackend::new(&expanded),
            path: expanded,
            daemon: None, // Initialized later via boot_daemon
        }
    }

    pub fn boot_daemon(store_arc: SharedStore) {
        let control = crate::daemon::spawn(store_arc.clone());
        let mut lock = store_arc.lock().unwrap();
        lock.daemon = Some(control);
    }

    pub fn store_path(&self) -> &str { &self.path }

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

    pub fn fetch_block(&self, concept: &str) -> Option<engram_core::types::Leg3Pointer> {
        self.backend.fetch_block(concept)
    }

    pub fn encode(&self, text: &str) -> engram_core::types::Leg3Pointer {
        self.backend.encode(text)
    }

    pub fn query(&self, query_vec: &[engram_core::Complex32; 8192], k: usize) -> Vec<Memory> {
        self.backend.query(query_vec, k)
    }

    pub fn store(&self, concept: &str, block: engram_core::types::Leg3Pointer) -> anyhow::Result<()> {
        self.backend.store(concept, block)
    }
}

/// Create a new SharedStore from a path string.
pub fn open_store(path: &str) -> SharedStore {
    Arc::new(Mutex::new(StoreHandle::new(path)))
}
