//! Thread-safe wrapper around the VsaBackend for server use.

use engram_core::backend::{CpuBackend, Memory, VsaBackend};
use std::sync::{Arc, Mutex};

/// Thread-safe, shared backend handle used across MCP and REST handlers.
pub type SharedStore = Arc<Mutex<StoreHandle>>;

/// Wraps a VsaBackend with its configured store path.
pub struct StoreHandle {
    backend: CpuBackend,
    path: String,
}

impl StoreHandle {
    pub fn new(path: &str) -> Self {
        let expanded = shellexpand::tilde(path).into_owned();
        std::fs::create_dir_all(&expanded).ok();
        Self {
            backend: CpuBackend::new(&expanded),
            path: expanded,
        }
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
}

/// Create a new SharedStore from a path string.
pub fn open_store(path: &str) -> SharedStore {
    Arc::new(Mutex::new(StoreHandle::new(path)))
}
