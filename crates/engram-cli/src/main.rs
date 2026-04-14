use clap::{Parser, Subcommand};
use engram_core::backend::{CpuBackend, VsaBackend};

#[derive(Parser)]
#[command(name = "engram", about = "Persistent geometric memory for AI agents", version)]
struct Cli {
    /// Directory to store .leg memory blocks
    #[arg(long, default_value = "~/.engram/manifold", env = "ENGRAM_STORE")]
    store: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store a concept in persistent memory
    Remember {
        /// Concept identifier (e.g. "krebs_cycle")
        concept: String,
        /// Text to encode and store
        text: String,
    },
    /// Find semantically similar memories
    Recall {
        /// Query text
        query: String,
        /// Number of results to return
        #[arg(short, default_value_t = 5)]
        k: usize,
    },
    /// Delete a concept from memory
    Forget {
        concept: String,
    },
    /// List all stored concepts
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let store_path = shellexpand::tilde(&cli.store).into_owned();
    let backend = CpuBackend::new(&store_path);

    match cli.command {
        Commands::Remember { concept, text } => {
            backend.remember(&concept, &text)?;
            println!("✓ Remembered: {concept}");
        }
        Commands::Recall { query, k } => {
            let results = backend.recall(&query, k);
            if results.is_empty() {
                println!("No memories found.");
            } else {
                for (i, mem) in results.iter().enumerate() {
                    println!("[{}] {} (score: {:.3}, crs: {:.3})", i + 1, mem.concept, mem.score, mem.crs);
                    if !mem.provlog.is_empty() {
                        let preview: String = mem.provlog.chars().take(120).collect();
                        println!("    {}", preview);
                    }
                }
            }
        }
        Commands::Forget { concept } => {
            backend.forget(&concept)?;
            println!("✓ Forgotten: {concept}");
        }
        Commands::List => {
            let concepts = backend.list();
            if concepts.is_empty() {
                println!("No memories stored in {store_path}");
            } else {
                println!("{} memories:", concepts.len());
                for c in &concepts {
                    println!("  · {c}");
                }
            }
        }
    }

    Ok(())
}
