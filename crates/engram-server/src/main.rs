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
//! **REST mode** — HTTP server on localhost. Used by custom integrations.
//!
//! ```sh
//! engram serve [--port 3456] [--store ~/.engram/manifold]
//! ```

mod mcp;
mod serve;
mod store;
pub mod daemon;
pub mod ki_hijacker;
pub mod watchdog;
pub mod scout;
pub mod scout_supervisor;

use clap::{Parser, Subcommand};
use store::open_store;
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

    // Boot scout daemon in background — only for HTTP serve mode.
    // In MCP mode the scout is not needed and its port-8088 startup
    // noise on stderr would corrupt the JSON-RPC protocol stream.
    if let Commands::Serve { .. } = cli.command {
        scout_supervisor::boot();
    }

    let store = open_store(&cli.store);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    match cli.command {
        Commands::Mcp { no_genesis } => {
            if !no_genesis {
                match store.lock().unwrap().seed_genesis() {
                    Ok(msg)  => tracing::info!("{msg}"),
                    Err(e)   => tracing::warn!("Genesis seed failed: {e}"),
                }
            }
            let _guard = rt.enter();

            // ── Boot file-watcher daemon (AST auto-ingest) ────────────────────
            store::StoreHandle::boot_daemon(store.clone());

            // ── Boot KI Hijacker — Logophysical Antigravity Bridge ────────────
            //
            // Every 60s, queries the manifold for top-N CRS + hot-session
            // memories and writes them to:
            //   ~/.gemini/antigravity/knowledge/active_engram_context/artifacts/context.md
            //
            // Antigravity reads this KI at session start. This means the
            // agent always wakes up with its own geometric memory injected
            // into its context window — no explicit recall calls needed.
            let _hijacker = ki_hijacker::spawn(store.clone());
            tracing::info!("[KI_HIJACKER] Logophysical Antigravity Bridge spawned (MCP mode).");

            mcp::run(store)?;
        }
        Commands::Serve { port, no_genesis } => {
            if !no_genesis {
                match store.lock().unwrap().seed_genesis() {
                    Ok(msg)  => tracing::info!("{msg}"),
                    Err(e)   => tracing::warn!("Genesis seed failed: {e}"),
                }
            }

            let _guard = rt.enter();

            // ── Boot file-watcher daemon ──────────────────────────────────────
            store::StoreHandle::boot_daemon(store.clone());

            // ── Boot KI Hijacker ──────────────────────────────────────────────
            let _hijacker = ki_hijacker::spawn(store.clone());
            tracing::info!("[KI_HIJACKER] Logophysical Antigravity Bridge spawned (REST mode).");

            rt.block_on(serve::run(store, port))?;
        }
    }

    Ok(())
}
