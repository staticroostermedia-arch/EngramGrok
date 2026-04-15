//! `engram-core` — VSA-based persistent geometric memory for AI agents.
//!
//! # The LEG Container Format
//!
//! Engram stores knowledge as **HolographicBlocks** — self-contained 256KB
//! binary containers (`.leg` files) defined by U.S. Patent Application pending,
//! Static Rooster Media. Each block encodes:
//!
//! - A 8192-dimensional complex phase vector (`q`) — the geometric "fingerprint"
//! - A momentum tensor (`p`) — encodes relational binding state  
//! - A Coherence-Reliability Score (`crs`) — geometric memory health [0.0, 1.0]
//! - A provenance payload — the human-readable source text (ProvLog)
//! - A BLAKE3 Merkle footer — cryptographic lineage chain
//!
//! # Operations
//!
//! - [`ops::op_add`] — Superposition: merge two memories (union)
//! - [`ops::op_bind`] — Binding: associate two concepts (role-filler encoding)
//! - [`ops::cosine_similarity`] — Geometric similarity between memories
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use engram_core::backend::{CpuBackend, VsaBackend};
//!
//! let backend = CpuBackend::new("~/.engram/manifold");
//! backend.remember("krebs_cycle", "The Krebs cycle converts acetyl-CoA to ATP").unwrap();
//!
//! let results = backend.recall("how does cellular respiration produce energy", 5);
//! for mem in results {
//!     println!("{}: {:.3}", mem.concept, mem.score);
//! }
//! ```

pub mod types;
pub mod ops;
pub mod encode;
pub mod storage;
pub mod mmap;
pub mod backend;
pub mod index;

pub use types::{
    HolographicBlock, Leg3Pointer, LegFooter, Logenergetics,
    BLOCK_SIZE, DIMENSION,
    // ZEDOS epistemic tags — exposed so downstream storage tools can work with the full format
    ZEDOS_DECLARATIVE, ZEDOS_EPISODIC, ZEDOS_OPERATIONAL,
    ZEDOS_BODY, ZEDOS_VERBATIM, ZEDOS_PRAXIS,
};
pub use ops::{op_add, op_bind, cosine_similarity};
pub use backend::VsaBackend;

/// `Complex32` — a 32-bit complex number. The fundamental unit of the phase vector.
pub use num_complex::Complex32;
