#![recursion_limit = "512"]

//! Engram server — MCP + REST memory backend for AI agents.
//!
//! # Modes
//!
//! **MCP mode (default)** — reads JSON-RPC 2.0 from stdin, writes to stdout.
//! Used by Claude Desktop, Cursor, and any MCP-compatible client.
//!
//! ```sh
//! engram mcp [--store ~/.engram/manifold]
//! ```
//!
//! **REST mode** — HTTP server on localhost. Used by custom integrations and the dynamic leg-browser GUI.
//!
//! ```sh
//! engram serve [--port 3456] [--store ~/.engram/manifold] [--light] [--no-scout]
//! ```
//!
//! Flags for reliable leg-browser / GUI use (see scripts/launch-leg-browser-review.sh):
//!   --light     : Force CPU backend (ENGRAM_FORCE_CPU_BACKEND), skips CUDA/Metal/BVH heavy init for fast non-GPU startup during UI testing.
//!   --no-scout  : Skip scout_daemon supervisor (avoids port 8088 contention/spam when only using /api/* for dynamic views).

mod mcp;
mod serve;
mod store;
pub mod daemon;
pub mod ki_hijacker;
pub mod watchdog;
pub mod scout;
pub mod scout_supervisor;

use clap::{Parser, Subcommand};
// open_store is now called inside the command arms (fast path for MCP, full for Serve)
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(
    name    = "engram",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Persistent geometric memory for AI agents",
    long_about = None,
)]
struct Cli {
    /// Directory to store .leg memory blocks
    #[arg(long, global = true, default_value = "~/.engram/manifold", env = "ENGRAM_STORE")]
    store: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as an MCP server over stdin/stdout (for Claude Desktop, Cursor, etc.)
    Mcp {
        /// Skip seeding alignment genesis blocks on first boot
        #[arg(long, default_value_t = false)]
        no_genesis: bool,
    },

    /// Run as a REST HTTP server
    Serve {
        /// Port to listen on
        #[arg(long, default_value_t = 3456)]
        port: u16,

        /// Skip seeding alignment genesis blocks on first boot
        #[arg(long, default_value_t = false)]
        no_genesis: bool,

        /// Light / UI-test mode for leg-browser dynamic GUI: force CPU-only backend (no CUDA/Metal/GPU BVH heavy init). Fast startup, sufficient for /api/block /api/hydrate /api/recent etc. (see parent goal:1780106168)
        #[arg(long, default_value_t = false)]
        light: bool,

        /// Disable the scout_daemon.py supervisor (port 8088 web-search companion). Recommended with --light when only using serve for live leg-browser views (no /api/scout needed).
        #[arg(long, default_value_t = false)]
        no_scout: bool,
    },
}

fn main() -> anyhow::Result<()> {
    // Log to stderr so stdout stays clean for MCP protocol
    fmt()
        .with_env_filter(
            EnvFilter::try_from_env("ENGRAM_LOG")
                .unwrap_or_else(|_| EnvFilter::new("engram=info")),
        )
        .with_writer(std::io::stderr)
        .without_time()
        .init();

    let cli = Cli::parse();

    // Boot scout daemon in background — only for HTTP serve mode (unless --no-scout for minimal leg-browser UI use).
    // In MCP mode the scout is not needed and its port-8088 startup
    // noise on stderr would corrupt the JSON-RPC protocol stream.
    // Linked to sub-goal:1780106172 (diagnose/stabilize serve for seamless dynamic leg-browser under parent goal:1780106168).
    if let Commands::Serve { no_scout, .. } = &cli.command {
        if !no_scout {
            scout_supervisor::boot();
        } else {
            tracing::info!("[SERVE] --no-scout: skipping scout_daemon supervisor (cleaner for leg-browser dynamic GUI testing).");
        }
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    match cli.command {
        Commands::Mcp { no_genesis } => {
            // === Early MCP Ready Path ===
            // Create an ultra-light placeholder instantly so we can answer
            // initialize / tools/list before the heavy manifold + GPU + BVH work.
            // The real work happens in the background (or on first real tool use).
            let store = store::open_store_placeholder_for_mcp(&cli.store);

            if !no_genesis {
                // Genesis seeding is cheap enough to do even on the placeholder path.
                match store.lock().unwrap().seed_genesis() {
                    Ok(msg)  => tracing::info!("{msg}"),
                    Err(e)   => tracing::warn!("Genesis seed failed: {e}"),
                }
            }

            let _guard = rt.enter();

            // Kick off the REAL heavy initialization in the background.
            // This loads the full Sheaf/Cuda backends, BVH indexes, embed matrix, etc.
            // We create a dedicated Tokio runtime inside this thread so that
            // boot_daemon / ki_hijacker (which use tokio::spawn) do not panic.
            let real_path = cli.store.clone();
            std::thread::spawn(move || {
                tracing::info!("[MCP-FAST] Starting full manifold initialization in background...");

                // Create a minimal multi-thread runtime just for this init thread.
                let rt = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("[MCP-FAST] Failed to create background runtime: {e}");
                        return;
                    }
                };

                let _rt_guard = rt.enter();

                let real = store::open_store(&real_path);

                if !no_genesis {
                    match real.lock().unwrap().seed_genesis() {
                        Ok(msg)  => tracing::info!("[MCP-FAST] {msg}"),
                        Err(e)   => tracing::warn!("[MCP-FAST] Genesis seed failed: {e}"),
                    }
                }

                // Boot the long-running daemons on the real store (now safe)
                store::StoreHandle::boot_daemon(real.clone());
                let _hijacker = ki_hijacker::spawn(real.clone());

                // Item 1.5: Signal to the agent layer that the full heavy backend is now ready.
                // This allows wake-up / rituals to distinguish "MCP protocol ready" from
                // "heavy OptiX + manifold work actually complete".
                real.lock().unwrap().mark_fully_initialized();

                tracing::info!("[MCP-FAST] Full initialization complete. Heavy features now active.");

                // Keep the runtime alive for the background work.
                // We intentionally leak the runtime here (it lives as long as the process).
                std::mem::forget(rt);
            });

            // Boot a minimal daemon on the placeholder so basic file watching can work
            // (real one will be booted in the background thread above for the heavy store).
            store::StoreHandle::boot_daemon(store.clone());

            // We intentionally do NOT spawn the full KI Hijacker on the placeholder.
            // It will be started on the real store in the background thread.

            tracing::info!("[MCP-FAST] Fast MCP path active — replying to protocol immediately.");

            mcp::run(store)?;
        }
        Commands::Serve { port, no_genesis, light, no_scout: _ } => {
            // Serve mode (HTTP) — stabilized for leg-browser dynamic GUI (parent goal:1780106168_make-the-leg-browser-a-seamless--truly-dynamic-g ; sub0:1780106172).
            if light {
                std::env::set_var("ENGRAM_FORCE_CPU_BACKEND", "1");
                tracing::info!("[SERVE] --light: ENGRAM_FORCE_CPU_BACKEND=1 (CPU-only, no GPU/CUDA init or heavy BVH for fast leg-browser UI testing + reliable bg launch). See codeland goal:1780091465_codeland-integration-2026---systematically-incor for related substrate work.");
            }

            // Serve mode (HTTP) can afford the full heavy initialization (unless light).
            let store = store::open_store(&cli.store);

            if !no_genesis {
                match store.lock().unwrap().seed_genesis() {
                    Ok(msg)  => tracing::info!("{msg}"),
                    Err(e)   => tracing::warn!("Genesis seed failed: {e}"),
                }
            }

            let _guard = rt.enter();

            // ── Boot file-watcher daemon ──────────────────────────────────────
            store::StoreHandle::boot_daemon(store.clone());

            // ── Boot KI Hijacker — — — — — — — — — — — — — — — — — — — — — — —
            let _hijacker = ki_hijacker::spawn(store.clone());
            tracing::info!("[KI_HIJACKER] Logophysical Antigravity Bridge spawned (REST mode).");

            // Concrete improvement: serve now supports clean shutdown signals (see serve.rs); "Keyboard interrupt received" will be logged on intentional Ctrl-C.
            rt.block_on(serve::run(store, port))?;
        }
    }

    Ok(())
}
