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
mod store;

use clap::{Parser, Subcommand};
use store::open_store;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(
    name    = "engram",
    version = env!("CARGO_PKG_VERSION"),
    about   = "Persistent geometric memory for AI agents — by Aric Goodman / Static Rooster Media",
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
    Mcp,

    /// Run as a REST HTTP server
    Serve {
        /// Port to listen on
        #[arg(long, default_value_t = 3456)]
        port: u16,
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
    let store = open_store(&cli.store);

    match cli.command {
        Commands::Mcp => {
            mcp::run(store)?;
        }
        Commands::Serve { port } => {
            eprintln!("REST server on port {port} — Phase 3b, not yet implemented.");
            eprintln!("Use `engram mcp` for Claude Desktop / Cursor integration.");
        }
    }

    Ok(())
}
