use crate::store::SharedStore;
use notify_debouncer_full::{new_debouncer, notify::*};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Starts the global agentic background daemon attached to the MCP / REST Server.
///
/// Autophagy GC is DISABLED. Nothing is ever evicted automatically.
/// The daemon runs purely as a workspace file watcher — auto-ingesting saved files.
pub fn spawn(store: SharedStore) -> Arc<DaemonControl> {
    let (watch_tx, watch_rx) = flume::unbounded::<PathBuf>();

    let daemon = Arc::new(DaemonControl {
        active_watch: Arc::new(tokio::sync::RwLock::new(None)),
        shutdown: Arc::new(AtomicBool::new(false)),
        watch_tx,
    });

    let ctrl = daemon.clone();

    tokio::spawn(async move {
        let (tx, rx) = flume::unbounded();

        let mut debouncer = new_debouncer(Duration::from_millis(1500), None, move |res| {
            if let Ok(events) = res {
                for e in events {
                    let _ = tx.send(e);
                }
            }
        }).unwrap();

        info!("Agentic Daemon (Phase 7) online. Autophagy GC DISABLED — watcher only.");

        // Flush hot access timestamps to disk every 60 seconds (Mac hardening)
        let mut flush_interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            if ctrl.shutdown.load(Ordering::Relaxed) {
                break;
            }

            tokio::select! {
                new_watch = watch_rx.recv_async() => {
                    if let Ok(p) = new_watch {
                        if let Err(e) = debouncer.watch(&p, RecursiveMode::Recursive) {
                            error!("Daemon failed to bind OS watcher to {}: {}", p.display(), e);
                        } else {
                            info!("Daemon dynamically bound OS watcher to: {}", p.display());
                        }
                    }
                }

                _ = flush_interval.tick() => {
                    // Flush hot access timestamps to disk every 60 seconds
                    let mut lock = store.lock().unwrap();
                    lock.access_index.flush_if_dirty();
                }

                event = rx.recv_async() => {
                    if let Ok(ev) = event {
                        if ev.kind.is_modify() || ev.kind.is_create() {
                            for path in &ev.paths {
                                if !path.is_file() { continue; }
                                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                                let allowed_exts = [
                                    "rs", "md", "txt", "js", "ts", "json", "toml", "py",
                                    "c", "cpp", "h", "csv", "sh", "go", "java", "rb",
                                    "zig", "php", "html", "css", "yml", "yaml", "sql",
                                    "ex", "exs", "swift",
                                ];

                                if allowed_exts.contains(&ext)
                                    && !path.to_string_lossy().contains("/target/")
                                    && !path.to_string_lossy().contains("/.git/")
                                {
                                    if let Ok(content) = std::fs::read_to_string(path) {
                                        let file_name = path
                                            .file_name()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("unknown");
                                        let concept_name = format!(
                                            "{}_daemon",
                                            file_name.replace('.', "_")
                                        );

                                        let safe_end = content.len().min(8000);
                                        let mut end = safe_end;
                                        while end > 0 && !content.is_char_boundary(end) {
                                            end -= 1;
                                        }

                                        let mut lock = store.lock().unwrap();
                                        if let Err(e) = lock.remember(&concept_name, &content[..end]) {
                                            error!(
                                                "Daemon failed to auto-sync file {}: {}",
                                                path.display(),
                                                e
                                            );
                                        } else {
                                            debug!(
                                                "Daemon: Auto-synced {}",
                                                path.display()
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    daemon
}

pub struct DaemonControl {
    pub active_watch: Arc<tokio::sync::RwLock<Option<PathBuf>>>,
    shutdown: Arc<AtomicBool>,
    watch_tx: flume::Sender<PathBuf>,
}

impl DaemonControl {
    pub async fn set_watch_workspace(&self, path: impl AsRef<Path>) {
        let p = path.as_ref().to_path_buf();
        let mut lock = self.active_watch.write().await;
        *lock = Some(p.clone());
        let _ = self.watch_tx.send(p);
    }
}
