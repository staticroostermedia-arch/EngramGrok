use crate::store::SharedStore;
use notify_debouncer_full::{new_debouncer, notify::*};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

// ── Shadow Basis (Phase 1 — 768-dim genesis anchor) ────────────────────────
//
// On daemon startup, attempt to load the pre-generated 768-dim shadow vectors
// for the genesis pillars from ~/.engram/genesis_shadow/.
// If the directory doesn't exist yet, run gen_shadow_basis first.

/// Load a 768-dim shadow genesis vector from disk.
/// Path: ~/.engram/genesis_shadow/{concept}.bin  (768 × f32 LE bytes)
fn load_shadow_vector(concept: &str) -> Option<Vec<f32>> {
    let path = std::env::var("HOME").ok()
        .map(|h| PathBuf::from(h)
            .join(".engram")
            .join("genesis_shadow")
            .join(format!("{}.bin", concept)))?;
    let bytes = std::fs::read(&path).ok()?;
    if bytes.len() % 4 != 0 { return None; }
    let vec: Vec<f32> = bytes.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    if vec.is_empty() { None } else { Some(vec) }
}

/// OP_ADD(block.q, shadow) → L2 normalize — anchors an 8192-dim block to the genesis basis.
/// The shadow vector is 768-dim; it is applied to the first 768 real components of q.
/// This biases the block's semantic position toward the genesis pillar in the real subspace.
fn apply_shadow_anchor(block: &mut engram_core::types::HolographicBlock, shadow: &[f32]) {
    let len = shadow.len().min(block.q.len());
    for i in 0..len {
        block.q[i].re += shadow[i];
    }
    // L2 normalize the full 8192-dim q after anchoring
    let norm: f32 = block.q.iter().map(|c| c.re * c.re + c.im * c.im).sum::<f32>().sqrt();
    if norm > f32::EPSILON {
        block.q.iter_mut().for_each(|c| { c.re /= norm; c.im /= norm; });
    }
}

/// Read .engramignore files from the watched workspace root and ~/.engram/.
/// Returns a list of path patterns that the daemon should never watch or encode.
/// To customise: create a `.engramignore` in your project root or in `~/.engram/`.
fn load_engramignore() -> Vec<String> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    // ~/.engram/.engramignore — user-level global ignore list
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(std::path::PathBuf::from(&home).join(".engram").join(".engramignore"));
    }

    // $ENGRAM_LINKED_WORKSPACE/.engramignore — project-level ignore list
    if let Ok(ws) = std::env::var("ENGRAM_LINKED_WORKSPACE") {
        candidates.push(std::path::PathBuf::from(&ws).join(".engramignore"));
    }

    let mut ignored = Vec::new();
    for path in &candidates {
        if let Ok(text) = std::fs::read_to_string(path) {
            for line in text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    ignored.push(trimmed.to_string());
                }
            }
        }
    }
    ignored
}

/// Starts the global agentic background daemon attached to the MCP / REST Server.
///
/// Autophagy GC is DISABLED. Nothing is ever evicted automatically.
/// The daemon runs three autonomous loops:
///   1. File watcher  — inotify/fsevents → AST extraction → live project indexing
///   2. NREM cycle    — periodic ego narrative tensor consolidation (ego.leg3)
///   3. Health watchdog — config-driven process monitor (~/.engram/watchdog.toml)
pub fn spawn(store: SharedStore) -> Arc<DaemonControl> {
    // Load shadow basis vectors at spawn time (once — not per file event)
    let shadow_cybernetics: Option<Vec<f32>> = load_shadow_vector("cybernetics");
    match &shadow_cybernetics {
        Some(v) => info!("[ShadowBasis] Loaded cybernetics_768 ({} floats)", v.len()),
        None    => info!("[ShadowBasis] cybernetics_768 not found — run gen_shadow_basis to enable genesis anchoring"),
    }
    let engramignore = load_engramignore();
    if !engramignore.is_empty() {
        info!("[.engramignore] Excluding {} path patterns from watch", engramignore.len());
    }

    // ── Load health watchdog config (~/.engram/watchdog.toml) ────────────────
    // This is the ONLY place Engram learns about consumer-specific processes.
    // If the file doesn't exist, the watchdog is a no-op — zero coupling.
    let watchdog_cfg = crate::watchdog::WatchdogConfig::load();
    let watchdog_proposals_path = watchdog_cfg.resolved_proposals_path();
    if watchdog_cfg.watch.is_empty() {
        info!("[Watchdog] No processes configured — health watchdog is a no-op.");
    } else {
        info!("[Watchdog] Monitoring {} process(es). Proposals → {}",
            watchdog_cfg.watch.len(), watchdog_proposals_path.display());
    }

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

        // ── Phase 3 Epoch IX: NREM Dream Consolidation ───────────────────────
        // Fire every 6 hours. On each tick:
        //   1. Scan all manifold blocks with CRS ≥ 0.85 (high-confidence memories).
        //   2. OP_ADD-superpose their q-vectors (equal weights, then L2-normalize).
        //   3. Write the resulting centroid to `~/.engram/ego.leg3`.
        //   4. Call StoreHandle::refresh_ego_q() so the live server picks it up.
        // Effect: the Ego's interpretive frame evolves nightly, causing Ego-resonant
        // memories to surface higher in recall over time (emergent long-term consolidation).
        let mut nrem_interval = tokio::time::interval(Duration::from_secs(6 * 3600));
        nrem_interval.tick().await; // skip the immediate first tick at startup

        // If ego.leg3 does not exist yet, trigger an immediate NREM pass to seed it.
        let ego_path = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".engram").join("ego.leg3"))
            .unwrap_or_default();
        if !ego_path.exists() {
            info!("[NREM] ego.leg3 missing on startup — triggering immediate genesis consolidation.");
            run_nrem_consolidation(&store);
        }

        // ── Integration Inbox Scanner (Phase 5) ──────────────────────────────
        // Poll ~/.engram/stalks/default/inbox/ every 5 seconds for
        // `integration_req_*.json` files written by the Cockpit when the operator
        // clicks INTEGRATE on an escalation report.
        // Format: { "concept": "...", "text": "...", "source": "escalation_id" }
        // On success: writes the .leg block, then DELETES the request file.
        let mut inbox_interval = tokio::time::interval(Duration::from_secs(5));

        // The inbox dir is adjacent to the default stalk — always exists.
        let inbox_dir = {
            let lock = store.lock().unwrap();
            PathBuf::from(lock.store_path()).join("inbox")
        };
        std::fs::create_dir_all(&inbox_dir).ok();
        info!("Integration inbox watching: {}", inbox_dir.display());

        // ── System Health Watchdog ────────────────────────────────────────────
        // Every 5 minutes, check each process in ~/.engram/watchdog.toml.
        // If the file doesn't exist, watch list is empty and this is a no-op.
        let mut health_interval = tokio::time::interval(Duration::from_secs(5 * 60));
        health_interval.tick().await; // skip first tick on startup

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

                _ = nrem_interval.tick() => {
                    run_nrem_consolidation(&store);
                }

                _ = health_interval.tick() => {
                    run_health_watchdog(&watchdog_cfg, &watchdog_proposals_path);
                }

                _ = inbox_interval.tick() => {
                    // ── Integration Inbox: sweep and process ─────────────────────────
                    if let Ok(entries) = std::fs::read_dir(&inbox_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let fname = path.file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("");
                            if !fname.starts_with("integration_req_") || !fname.ends_with(".json") {
                                continue;
                            }
                            match std::fs::read_to_string(&path) {
                                Ok(raw) => {
                                    if let Ok(req) = serde_json::from_str::<serde_json::Value>(&raw) {
                                        let concept = req["concept"].as_str().unwrap_or("").to_string();
                                        let text    = req["text"].as_str().unwrap_or("").to_string();
                                        let source  = req["source"].as_str().unwrap_or("").to_string();

                                        if concept.is_empty() || text.is_empty() {
                                            error!("[INBOX] Malformed integration request {}: concept/text missing", fname);
                                            std::fs::remove_file(&path).ok();
                                            continue;
                                        }

                                        let mut lock = store.lock().unwrap();
                                        match lock.remember(&concept, &text) {
                                            Ok(_) => {
                                                info!("[INBOX] Integrated escalation → Engram: '{}' (from {})", concept, source);
                                                std::fs::remove_file(&path).ok();
                                            }
                                            Err(e) => {
                                                error!("[INBOX] remember() failed for '{}': {}", concept, e);
                                                // Leave the file for retry on next tick
                                            }
                                        }
                                    } else {
                                        error!("[INBOX] JSON parse error in {}", fname);
                                        std::fs::remove_file(&path).ok();
                                    }
                                }
                                Err(e) => {
                                    error!("[INBOX] Failed to read {}: {}", fname, e);
                                }
                            }
                        }
                    }
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

                                // .engramignore: skip paths owned by ouroboros daemon
                                let path_str = path.to_string_lossy();
                                let is_ignored = engramignore.iter()
                                    .any(|pat| path_str.contains(pat.as_str()));

                                if allowed_exts.contains(&ext)
                                    && !path_str.contains("/target/")
                                    && !path_str.contains("/.git/")
                                    && !is_ignored
                                {
                                    if let Ok(content) = std::fs::read_to_string(path) {
                                        let mut lock = store.lock().unwrap();

                                        // ── Namespace separation ────────────────────────────────
                                        // If ENGRAM_LINKED_WORKSPACE is set, AST blocks from that
                                        // workspace go into a dedicated '<workspace_name>_ast' stalk.
                                        // This keeps auto-ingested code blocks separate from agent
                                        // episodic memories in the default stalk.
                                        let linked_ws = std::env::var("ENGRAM_LINKED_WORKSPACE").ok();
                                        let (is_linked, ast_stalk) = match &linked_ws {
                                            Some(ws) if path_str.contains(ws.as_str()) => {
                                                let name = std::path::Path::new(ws)
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or("linked")
                                                    .to_lowercase();
                                                (true, format!("{}_ast", name))
                                            }
                                            _ => (false, String::new()),
                                        };
                                        if is_linked {
                                            lock.set_active_stalk(&ast_stalk);
                                        }

                                        let items = engram_ast::extract_ast_items(path.to_str().unwrap_or(""), &content);

                                        if !items.is_empty() {
                                            for item in items {
                                                let mut block = lock.encode(&item.embed_label());

                                                // Map 2D file coordinates into the 3D logophysical bounding box
                                                block.aabb_min = [item.start_pos.0 as f32, item.start_pos.1 as f32, 0.0];
                                                block.aabb_max = [item.end_pos.0 as f32,   item.end_pos.1 as f32,   0.0];

                                                // Store full unbroken source in ProvLog
                                                engram_core::storage::write_provlog(&mut block, &item.full_source);

                                                // ── Genesis Shadow Anchor ──────────────────────
                                                // OP_ADD(block.q, v_cybernetics_768) → L2 normalize
                                                // Biases the AST block toward the genesis coordinate
                                                // system within the 768-dim shadow manifold.
                                                // Both vectors are 768-dim so dimensions match.
                                                if let Some(ref shadow) = shadow_cybernetics {
                                                    apply_shadow_anchor(&mut block, shadow);
                                                }

                                                if let Err(e) = lock.store(&item.concept, block) {
                                                    error!("Daemon failed to auto-sync AST {}: {}", item.concept, e);
                                                } else {
                                                    debug!("Daemon: Auto-synced AST {} (shadow_anchor={})",
                                                        item.concept, shadow_cybernetics.is_some());
                                                }
                                            }
                                        } else {
                                            // Fallback chunking
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

                                            if let Err(e) = lock.remember(&concept_name, &content[..end]) {
                                                error!(
                                                    "Daemon failed to auto-sync file {}: {}",
                                                    path.display(),
                                                    e
                                                );
                                            } else {
                                                debug!(
                                                    "Daemon: Auto-synced fallback {}",
                                                    path.display()
                                                );
                                            }

                                        // Restore default namespace after linked-workspace block
                                        if is_linked {
                                            lock.set_active_stalk("default");
                                        }
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

// ── Phase 3 Epoch IX: NREM Dream Consolidation ───────────────────────────────
//
// Called every 6 hours by the circadian timer inside spawn().
//
// Algorithm:
//   1. List all concepts in the active manifold.
//   2. For each concept, fetch_block() and inspect CRS.
//   3. Blocks with CRS ≥ NREM_CRS_THRESHOLD contribute their q-vector to a
//      running OP_ADD accumulator (equal weights — mass proportional to signal).
//   4. L2-normalize the accumulated centroid to keep it on the unit hypersphere.
//   5. Mint a fresh HolographicBlock (Leg3Pointer::mint()), write the centroid
//      into its q-field, mark it ZEDOS_EPISODIC, and persist it to ego.leg3.
//   6. Call StoreHandle::refresh_ego_q() — the live recall path immediately
//      picks up the updated interpretive frame.
//
// Safety: the store mutex is released between fetches so the server stays
// responsive. We accumulate into a CPU-side array, never holding the lock
// across the (slow) list() traversal.

const NREM_CRS_THRESHOLD: f32 = 0.85;

fn run_nrem_consolidation(store: &crate::store::SharedStore) {
    info!("[NREM] Starting dream consolidation pass (CRS threshold = {})…", NREM_CRS_THRESHOLD);

    // Collect all concept names while holding the lock briefly.
    let concepts: Vec<String> = {
        let lock = store.lock().unwrap();
        lock.list()
    };

    let mut accumulator = [engram_core::Complex32::new(0.0_f32, 0.0_f32); 8192];
    let mut contributors: u32 = 0;

    for concept in &concepts {
        // Skip internal bookkeeping concepts.
        if concept.starts_with('_') { continue; }

        let block_opt: Option<engram_core::types::Leg3Pointer> = {
            let lock = store.lock().unwrap();
            lock.fetch_block(concept)
        };

        if let Some(block) = block_opt {
            if block.crs_score >= NREM_CRS_THRESHOLD {
                for i in 0..8192 {
                    accumulator[i].re += block.q[i].re;
                    accumulator[i].im += block.q[i].im;
                }
                contributors += 1;
            }
        }
    }

    if contributors == 0 {
        info!("[NREM] No blocks above CRS threshold — ego.leg3 not updated.");
        return;
    }

    // L2-normalize the accumulator onto the unit hypersphere.
    let norm_sq: f32 = accumulator.iter().map(|c| c.re * c.re + c.im * c.im).sum();
    let norm = norm_sq.sqrt();
    if norm < f32::EPSILON {
        info!("[NREM] Accumulator degenerate (zero norm) — ego.leg3 not updated.");
        return;
    }
    let inv = 1.0 / norm;
    for c in &mut accumulator {
        c.re *= inv;
        c.im *= inv;
    }

    // Mint a fresh 256KB block and write the NREM centroid into its q-field.
    let mut ego_block = engram_core::types::Leg3Pointer::mint();
    ego_block.q = accumulator;
    ego_block.zedos_tag = engram_core::types::ZEDOS_EPISODIC;
    ego_block.crs_score = 1.0; // Ego anchor — always pinned.
    ego_block.energetics.crs = 1.0;
    ego_block.superposition_count = contributors;

    // Write to ~/.engram/ego.leg3 — the canonical ego narrative tensor path.
    let ego_path = match std::env::var("HOME") {
        Ok(h) => std::path::PathBuf::from(h).join(".engram").join("ego.leg3"),
        Err(_) => {
            error!("[NREM] HOME env var not set — cannot write ego.leg3");
            return;
        }
    };

    match engram_core::storage::write_block(&ego_path, &ego_block) {
        Ok(_) => {
            info!("[NREM] ego.leg3 updated from {} high-signal blocks (norm={:.4}). Refreshing live Ego gate…",
                contributors, norm);
            // Hot-swap the Ego q-vector into the live StoreHandle.
            let mut lock = store.lock().unwrap();
            lock.refresh_ego_q();
            info!("[NREM] Ego gate refreshed.");
        }
        Err(e) => {
            error!("[NREM] Failed to write ego.leg3: {}", e);
        }
    }
}

// ── System Health Watchdog ────────────────────────────────────────────────────
//
// Config-driven: reads ~/.engram/watchdog.toml for process names to monitor
// and where to write agency proposals. If the config file doesn't exist this
// function is a no-op — Engram has zero coupling to any consumer project.
//
// Called every 5 minutes from the daemon select! loop.

fn run_health_watchdog(
    cfg: &crate::watchdog::WatchdogConfig,
    proposals_path: &std::path::Path,
) {
    if cfg.watch.is_empty() {
        return; // no-op — watchdog.toml absent or empty
    }

    for process in &cfg.watch {
        if !crate::watchdog::is_process_alive(&process.name) {
            info!(
                "[Watchdog] '{}' not detected — minting agency proposal.",
                process.name
            );
            crate::watchdog::mint_proposal(process, proposals_path);
        } else {
            tracing::trace!("[Watchdog] '{}' alive ✓", process.name);
        }
    }
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
