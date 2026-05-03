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

/// Read .engramignore from the Engram workspace root.
/// Returns a list of path prefixes that the daemon should never watch or encode.
fn load_engramignore() -> Vec<String> {
    // Look for .engramignore in the Engram project root (adjacent to Cargo.toml)
    let candidates = [
        "/home/a/Documents/Engram/.engramignore",
        "/home/a/Documents/CodeLand/.engramignore",
    ];
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
/// The daemon runs purely as a workspace file watcher — auto-ingesting saved files.
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

        // ── NREM Consolidation Cycle (Phase 3) ──────────────────────────────────
        // Every 4 hours, harvest high-CRS episodic/operational memories and
        // superimpose them into the ego narrative tensor (ego.leg3).
        // This implements the nocturnal memory consolidation loop described in
        // IMPLEMENTATION-PLAN.md § Phase 3: NREM Consolidation.
        let mut nrem_interval = tokio::time::interval(Duration::from_secs(4 * 60 * 60));
        nrem_interval.tick().await; // skip immediate first tick on startup

        let ego_leg3_path = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".engram").join("ego.leg3"))
            .unwrap_or_else(|_| PathBuf::from("/tmp/ego.leg3"));

        // ── System Health Watchdog (Track 1 Autonomy) ───────────────────────────
        // Every 5 minutes, ping each system service. If a service is unreachable,
        // write a SYSTEM_HEALTH healing proposal to the agency proposals file so the
        // human (or auto-exec layer) can approve a restart.
        let mut health_interval = tokio::time::interval(Duration::from_secs(5 * 60));
        health_interval.tick().await; // skip first tick on startup

        let agency_proposals_path = std::env::var("CODELAND_ROOT")
            .map(|r| PathBuf::from(r).join("data").join("agency_proposals.json"))
            .unwrap_or_else(|_| {
                // Auto-discover: look relative to HOME
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join("Documents").join("CodeLand")
                        .join("data").join("agency_proposals.json"))
                    .unwrap_or_else(|_| PathBuf::from("/tmp/agency_proposals.json"))
            });

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
                    // ── NREM Phase 3: Ego Narrative Consolidation ────────────────
                    info!("[NREM] Starting ego consolidation pass...");

                    // 1. Harvest high-CRS episodic + operational memories
                    let harvest_queries = [
                        "session progress decisions architecture crate",
                        "bug fix solution praxis crystallized error",
                        "agent identity mission memory manifold",
                    ];

                    let mut ego_accumulator: Vec<engram_core::Complex32> = vec![
                        engram_core::Complex32::new(0.0, 0.0); 8192
                    ];
                    let mut harvested = 0usize;

                    {
                        let mut lock = store.lock().unwrap();
                        for query in &harvest_queries {
                            let results = lock.recall(query, 10);
                            for mem in results {
                                // Only consume blocks above the silver tier (≥0.85)
                                if mem.crs >= 0.85 {
                                    if let Some(block) = lock.fetch_block(&mem.concept) {
                                        // OP_ADD superposition into accumulator
                                        for i in 0..8192 {
                                            ego_accumulator[i].re += block.q[i].re;
                                            ego_accumulator[i].im += block.q[i].im;
                                        }
                                        harvested += 1;
                                    }
                                }
                            }
                        }
                    }

                    if harvested == 0 {
                        info!("[NREM] No high-CRS memories found for consolidation — skipping.");
                    } else {
                        // 2. L2-normalize the accumulated ego vector
                        let norm: f32 = ego_accumulator.iter()
                            .map(|c| c.re * c.re + c.im * c.im)
                            .sum::<f32>()
                            .sqrt();
                        if norm > f32::EPSILON {
                            for c in ego_accumulator.iter_mut() {
                                c.re /= norm;
                                c.im /= norm;
                            }
                        }

                        // 3. Write updated ego.leg3 to disk
                        // Build a minimal HolographicBlock using encode then swap q
                        {
                            let lock = store.lock().unwrap();
                            let mut ego_block = lock.encode("ego_narrative_tensor NREM consolidated");
                            // Overwrite q with the freshly consolidated vector
                            for i in 0..8192 {
                                ego_block.q[i] = ego_accumulator[i];
                            }
                            ego_block.zedos_tag = 0xFF; // GENESIS tier
                            ego_block.crs_score = 1.0;
                            if let Err(e) = engram_core::storage::write_block(
                                ego_leg3_path.to_str().unwrap_or("/tmp/ego.leg3"),
                                &ego_block,
                            ) {
                                error!("[NREM] Failed to write ego.leg3: {}", e);
                            } else {
                                info!("[NREM] ego.leg3 updated ({} blocks consolidated, norm={:.4})", harvested, norm);
                                // Hot-reload the updated ego tensor into the store
                                store.lock().unwrap().refresh_ego_q();
                            }
                        }

                        // 4. Mint NREM episodic summary into manifold
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let nrem_concept = format!("nrem_cycle_{}", ts);
                        let nrem_text = format!(
                            "NREM Consolidation @ ts={} — {} high-CRS blocks superimposed into ego narrative tensor. ego.leg3 updated. Ego narrative absorbing: session praxis, architectural decisions, mission continuity.",
                            ts, harvested
                        );
                        let mut lock = store.lock().unwrap();
                        if let Err(e) = lock.remember(&nrem_concept, &nrem_text) {
                            error!("[NREM] Failed to mint episodic summary: {}", e);
                        } else {
                            info!("[NREM] Episodic summary minted: '{}'", nrem_concept);
                        }
                    }
                }


                _ = health_interval.tick() => {
                    // ── System Health Watchdog ───────────────────────────────────────
                    info!("[HEALTH] Running system health check...");

                    // Services to monitor: (name, url, restart_cmd, plain_english)
                    let services: &[(&str, &str, &str, &str)] = &[
                        (
                            "LLM Server (llama.cpp)",
                            "http://localhost:11434/v1/models",
                            "bash -c 'cd ~/Documents/CodeLand && nohup scripts/start_llama_server.sh > logs/llm.log 2>&1 &'",
                            "The local LLM server (Gemma 26B / llama.cpp) is not responding. Restarting it will restore autonomous reasoning, agency proposals, and chess analysis.",
                        ),
                        (
                            "Monad Runtime",
                            "http://localhost:8080/api/status",
                            "bash -c 'cd ~/Documents/CodeLand && nohup target/release/monad_runtime > logs/runtime.log 2>&1 &'",
                            "The Monad Runtime (NVSA oracle + transductive API) is not responding. Restarting it will restore /api/ask lookups, VSA recall, and the semantic CLI.",
                        ),
                        (
                            "Moltbook Hub",
                            "http://localhost:6090/api/moltbook/status",
                            "bash -c 'cd ~/Documents/CodeLand && nohup python3 data/web_ui/serve.py > logs/hub.log 2>&1 &'",
                            "The Moltbook Hub (Cockpit UI + Rooster daemon) is not responding. Restarting it will restore the web interface, draft queue, and Moltbook posting.",
                        ),
                    ];

                    let http_client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(4))
                        .build()
                        .unwrap_or_default();

                    for (name, url, restart_cmd, plain_english) in services {
                        let ok = http_client.get(*url).send().await
                            .map(|r| r.status().is_success() || r.status().as_u16() < 500)
                            .unwrap_or(false);

                        if !ok {
                            error!("[HEALTH] {} is DOWN — filing healing proposal", name);

                            // Build a SYSTEM_HEALTH proposal
                            let ts = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs();
                            let prop_id = format!("health_{}_{}", name.replace(' ', "_").to_lowercase(), ts);

                            let proposal = serde_json::json!({
                                "id": prop_id,
                                "action_type": "restart_daemon",
                                "risk_level": "low",
                                "plain_english": plain_english,
                                "if_approved": format!("I will run: `{}`. The service should be back online within 15 seconds.", restart_cmd),
                                "if_rejected": format!("{} will remain offline until manually restarted. Functionality depending on it will be degraded.", name),
                                "action": restart_cmd,
                                "reason": format!("Health check to {} returned no response (timeout or connection refused).", url),
                                "confidence": 0.95,
                                "source_topic": "system_health",
                                "status": "pending",
                                "created_at": ts as f64,
                            });

                            // Append to agency_proposals.json
                            let existing_raw = std::fs::read_to_string(&agency_proposals_path)
                                .unwrap_or_else(|_| "[]".to_string());
                            if let Ok(mut arr) = serde_json::from_str::<serde_json::Value>(&existing_raw) {
                                if let Some(list) = arr.as_array_mut() {
                                    // Avoid duplicate pending proposals for the same service
                                    let already_pending = list.iter().any(|p|
                                        p["action_type"] == "restart_daemon"
                                        && p["source_topic"] == "system_health"
                                        && p["action"] == *restart_cmd
                                        && p["status"] == "pending"
                                    );
                                    if !already_pending {
                                        list.push(proposal);
                                        if let Ok(out) = serde_json::to_string_pretty(&arr) {
                                            std::fs::write(&agency_proposals_path, out).ok();
                                            info!("[HEALTH] Filed restart proposal for {}", name);
                                        }
                                    } else {
                                        info!("[HEALTH] Restart proposal for {} already pending — skipping", name);
                                    }
                                }
                            }
                        } else {
                            info!("[HEALTH] {} OK", name);
                        }
                    }

                    // Also check Circadian via pgrep
                    let circadian_ok = std::process::Command::new("pgrep")
                        .args(["-x", "circadian"])
                        .output()
                        .map(|o| !o.stdout.is_empty())
                        .unwrap_or(false);

                    if !circadian_ok {
                        error!("[HEALTH] Circadian daemon is DOWN — filing restart proposal");
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs();
                        let prop_id = format!("health_circadian_{}", ts);
                        let proposal = serde_json::json!({
                            "id": prop_id,
                            "action_type": "restart_daemon",
                            "risk_level": "low",
                            "plain_english": "The Circadian daemon (which runs the nightly NREM memory consolidation cycle) has stopped. Restarting it ensures the ego narrative tensor continues to evolve overnight.",
                            "if_approved": "I will run: `nohup ~/Documents/CodeLand/target/release/circadian > ~/Documents/CodeLand/logs/circadian.log 2>&1 &`. NREM cycles resume.",
                            "if_rejected": "Circadian stays offline. NREM consolidation will not run. The ego tensor (ego.leg3) will not be updated until manually restarted.",
                            "action": "bash -c 'nohup ~/Documents/CodeLand/target/release/circadian > ~/Documents/CodeLand/logs/circadian.log 2>&1 &'",
                            "reason": "pgrep -x circadian returned empty — process is not running.",
                            "confidence": 0.97,
                            "source_topic": "system_health",
                            "status": "pending",
                            "created_at": ts as f64,
                        });
                        let existing_raw = std::fs::read_to_string(&agency_proposals_path)
                            .unwrap_or_else(|_| "[]".to_string());
                        if let Ok(mut arr) = serde_json::from_str::<serde_json::Value>(&existing_raw) {
                            if let Some(list) = arr.as_array_mut() {
                                let already = list.iter().any(|p|
                                    p["id"].as_str().unwrap_or("").starts_with("health_circadian")
                                    && p["status"] == "pending"
                                );
                                if !already {
                                    list.push(proposal);
                                    if let Ok(out) = serde_json::to_string_pretty(&arr) {
                                        std::fs::write(&agency_proposals_path, out).ok();
                                        info!("[HEALTH] Filed Circadian restart proposal");
                                    }
                                }
                            }
                        }
                    }
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

                                        // ── Phase 1: Namespace separation ──────────────────────
                                        // CodeLand AST blocks go into 'codeland_ast' namespace.
                                        // Agent episodics remain in the default stalk.
                                        let is_codeland = path_str.contains("/CodeLand/");
                                        if is_codeland {
                                            lock.set_active_stalk("codeland_ast");
                                        }

                                        let items = engram_core::ast_extract::extract_ast_items(path.to_str().unwrap_or(""), &content);

                                        if !items.is_empty() {
                                            for item in items {
                                                let mut block = lock.encode(&item.embed_label());

                                                // Map 2D file coordinates into the 3D logophysical bounding box
                                                block.aabb_min = [item.start_pos.0 as f32, item.start_pos.1 as f32, 0.0];
                                                block.aabb_max = [item.end_pos.0 as f32,   item.end_pos.1 as f32,   0.0];

                                                // Store full unbroken source in ProvLog
                                                engram_core::storage::write_provlog(&mut block, &item.full_source);

                                                // ── Phase 1: Genesis Shadow Anchor ─────────────
                                                // OP_ADD(block.q, v_cybernetics_768) → L2 normalize
                                                // Anchors the AST block to the CodeLand genesis
                                                // coordinate system within the 768-dim manifold.
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

                                        // Restore default namespace after CodeLand block
                                        if is_codeland {
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
