//! CUDA backend for Engram — BVH O(log N) K-NN + parallel cosine kernels.
//!
//! # Architecture
//!
//! ```text
//! CudaBackend
//!   ├── BvhManifold (CPU LBVH tree + entry index)
//!   │     └── filter_cpu() → candidate IDs     O(log N)
//!   ├── GPU kernels (compiled by build.rs)
//!   │     ├── engram_project_8k_to_3d          (Gaussian CSRP)
//!   │     ├── engram_cosine_batch              (Hermitian cosine × N)
//!   │     └── engram_bvh_traverse              (slab traversal)
//!   └── CpuBackend fallback (for machines without CUDA)
//! ```
//!
//! The BVH is built once at startup and rebuilt incrementally as new blocks arrive.
//! On GPU-less machines, `CudaBackend` transparently delegates to `CpuBackend`.

pub mod bvh;
pub mod backend;
pub mod metal_backend;

pub use bvh::{BvhManifold, Float3, LBVHNode};
pub use backend::CudaBackend;
pub use metal_backend::MetalBackend;
