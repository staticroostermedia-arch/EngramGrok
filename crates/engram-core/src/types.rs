//! Core types for the LEG container format.
//!
//! The [`HolographicBlock`] is the triadic digital container described in
//! **U.S. Patent Application No. 19/372,256** (pending), *Self-Contained Variable
//! File System (.LEG Container Format)*, Applicant: Aric Goodman, Oregon, USA.
//! Filed under 35 U.S.C. § 111(a) — Static Rooster Media.
//!
//! ## Triadic Structure (FIG. 1)
//!
//! | Segment | Patent Element | Fields |
//! |---------|----------------|--------|
//! | Header (100) | Schema ID (110) | `magic`, `schema_ver` |
//! | Header (100) | Allowed Transforms (120) | `allowed_transforms[64]` |
//! | Header (100) | Verification Key (130) | `concept_ref[32]` |
//! | Body   (200) | Type-tagged payload | `zedos_tag` + `payload[122,584]` |
//! | Body   (200) | Geometric tensors | `q[8192]`, `p[8192]` |
//! | Footer (300) | Hash (310) | `footer.sig_0`–`sig_5` (BLAKE3) |
//! | Footer (300) | Link Value (320) | `footer.merkle_sub_root` |
//! | Footer (300) | Error counters (Claim 7) | `footer.error_checks` |
//!
//! Each block verifies itself and reconstructs lineage without any external registry.
//! Exactly 256KB, 4096-byte aligned, safe for O_DIRECT NVMe I/O and CUDA DMA.

use num_complex::Complex32;

/// FHRR phase vector dimension: 8192 complex elements = 64KB
pub const DIMENSION: usize = 8192;

/// LEG block size: exactly 256KB. Enforced at compile time.
pub const BLOCK_SIZE: usize = 262_144;

// ── ZEDOS Epistemic Memory Tags ────────────────────────────────────────────────
// These tags classify what *kind* of memory a block holds.
// Stored in `HolographicBlock::zedos_tag`.

/// Declarative memory: facts, definitions, universal claims.
pub const ZEDOS_DECLARATIVE: u8  = 0xD;
/// Episodic memory: conversation turns, experiential context.
pub const ZEDOS_EPISODIC: u8     = 0xA;
/// Operational memory: tools, executables, code.
pub const ZEDOS_OPERATIONAL: u8  = 0x52;
/// Raw body chunk: unstructured payload.
pub const ZEDOS_BODY: u8         = 0xB0;
/// Verbatim text: lossless source material.
pub const ZEDOS_VERBATIM: u8     = 0xB1;
/// Praxis memory: crystallized learned procedures.
pub const ZEDOS_PRAXIS: u8       = 0x50;
/// Phase M: Relation block — links two concepts via OP_BIND (Merkle-chained).
pub const ZEDOS_RELATION: u8     = 0xE1;

// ── Compile-time size seal ──────────────────────────────────────────────────────
const _: () = assert!(
    std::mem::size_of::<HolographicBlock>() == BLOCK_SIZE,
    "ENGRAM: HolographicBlock has drifted from the 256KB constraint! \
     The LEG format requires an exact 256KB boundary for DMA alignment."
);

// ── Sub-structs ────────────────────────────────────────────────────────────────

/// Energy accounting capsule — tracks computational work per block.
/// Occupies a dedicated 4KB page at offset 0x21000 in the block.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Logenergetics {
    pub ts: u64,
    pub step: u32,
    pub h_in: f32,
    pub h_out: f32,
    pub work_verb: f32,
    pub heat_dissipated: f32,
    pub alpha_a: f32,
    pub alpha_d: f32,
    pub alpha_r: f32,
    pub crs: f32,
    pub dv: f32,
    pub control_action: u32,
    pub tau: f32,
    pub zpl_state: u8,
    pub _pad: [u8; 7],
}

/// Cryptographic footer — BLAKE3 Merkle chain for provenance verification.
/// Occupies the last 256 bytes of the block (offset 0x3FF00).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LegFooter {
    pub error_checks: u32,
    pub _pad: [u8; 28],
    pub sig_0: [u8; 32],
    pub sig_1: [u8; 32],
    pub sig_2: [u8; 32],
    pub sig_3: [u8; 32],
    pub sig_4: [u8; 32],
    pub sig_5: [u8; 32],
    /// BLAKE3 Merkle sub-root of parent block CIDs.
    pub merkle_sub_root: [u8; 32],
}

// ── HolographicBlock ───────────────────────────────────────────────────────────

/// The fundamental unit of Engram memory.
///
/// Exactly 256KB, 4096-byte aligned. Safe for O_DIRECT NVMe I/O and CUDA DMA.
///
/// Memory layout (all offsets exact, compiler-verified):
///
/// ```text
/// Offset 0x00000  [64KB]  q[] — position/knowledge phase vector
/// Offset 0x10000  [64KB]  p[] — momentum/binding tensor  
/// Offset 0x20000  [128KB] metadata, energetics, payload, footer
///   └─ 0x20000   header fields (magic, CRS, timestamps, ...)
///   └─ 0x21000   Logenergetics capsule (64B + padding to 4KB)
///   └─ 0x22000   concept_ref (32B) + zedos_tag (1B) + payload (122,584B)
///   └─ 0x3FF00   LegFooter (256B, Merkle chain)
/// ```
///
/// # Stack Safety
///
/// **Never allocate on the stack.** Use [`Leg3Pointer::mint()`] which forces
/// heap allocation via `Box`. The struct itself is 256KB — one stack frame
/// would consume 12.5% of the default 2MB thread stack.
#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct HolographicBlock {
    // ── Stride 1: Physics & Dynamics (GPU domain) ─────────────────────────────
    /// Position / knowledge tensor. 8192 × Complex32 = 64KB.
    /// The geometric "fingerprint" of the encoded concept.
    pub q: [Complex32; 8192],

    /// Momentum / binding tensor. 8192 × Complex32 = 64KB.
    /// Initialized to (1.0 + 0.0i) — the multiplicative identity.
    pub p: [Complex32; 8192],

    // ── Stride 2: Metadata & Logic (CPU domain) ────────────────────────────────
    pub magic: [u8; 4],
    pub schema_ver: u32,
    pub content_type: u8,
    pub spin_state: u8,
    pub _pad_schema: [u8; 2],
    pub tensor_rank: u32,
    pub superposition_count: u32,
    pub lz4_size: u32,
    pub last_accessed_timestamp: u64,
    pub decay_factor: f32,
    pub _legacy_prev_hash: [u8; 32],
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub context_state: u8,
    pub process_octave: u8,
    pub fail_streak: u8,

    /// Coherence-Reliability Score [0.0, 1.0].
    /// Geometric measure of memory health. Memories below 0.40 are weak.
    /// Memories above 0.74 are fully grounded.
    pub crs_score: f32,

    pub hardware_entropy_seed: u64,
    pub phase_clock_offset: u32,
    pub _pad_entanglement: [u8; 32],
    pub phase_state: u32,
    pub coherence_time: u64,
    pub allowed_transforms: [u8; 64],
    pub _pad_header: [u8; 3872],

    // ── Logenergetics capsule at 0x21000 ──────────────────────────────────────
    pub energetics: Logenergetics,
    pub _pad_energetics: [u8; 4032],

    // ── Payload region at 0x22000 ─────────────────────────────────────────────
    /// Wikidata Q-ID or relation anchor (32 bytes).
    pub concept_ref: [u8; 32],

    /// Epistemic memory tag (ZEDOS_DECLARATIVE, ZEDOS_EPISODIC, etc.)
    pub zedos_tag: u8,
    pub _pad_zedos: [u8; 7],

    /// ProvLog: the human-readable source text for this memory (122,584 bytes).
    pub payload: [u8; 122_584],

    // ── Footer at 0x3FF00 ─────────────────────────────────────────────────────
    pub footer: LegFooter,
}

// ── Leg3Pointer ───────────────────────────────────────────────────────────────

/// Heap-enforced safe handle to a [`HolographicBlock`].
///
/// Always use `Leg3Pointer::mint()` instead of allocating `HolographicBlock` directly.
/// The inner block is 256KB — stack allocation will cause a stack overflow.
///
/// Derefs transparently to `&HolographicBlock` and `&mut HolographicBlock`.
pub struct Leg3Pointer(pub Box<HolographicBlock>);

impl Leg3Pointer {
    /// Allocate a new block on the heap, zero-initialized with LEG3 magic.
    /// Momentum tensor `p` is set to the multiplicative identity (1.0 + 0.0i).
    pub fn mint() -> Self {
        let mut block: Box<HolographicBlock> = unsafe {
            let layout = std::alloc::Layout::new::<HolographicBlock>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut HolographicBlock;
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(ptr)
        };
        for i in 0..DIMENSION {
            block.p[i] = Complex32::new(1.0, 0.0);
        }
        block.magic = *b"LEG3";
        Self(block)
    }

    /// Wrap an existing boxed block.
    #[inline(always)]
    pub fn from_boxed(b: Box<HolographicBlock>) -> Self { Self(b) }

    /// Unwrap into the owned `Box<HolographicBlock>`.
    #[inline(always)]
    pub fn into_inner(self) -> Box<HolographicBlock> { self.0 }

    /// Raw const pointer for CUDA DMA.
    #[inline(always)]
    pub fn as_raw_ptr(&self) -> *const HolographicBlock { &*self.0 }

    /// Raw mut pointer for CUDA DMA.
    #[inline(always)]
    pub fn as_raw_mut_ptr(&mut self) -> *mut HolographicBlock { &mut *self.0 }
}

impl std::ops::Deref for Leg3Pointer {
    type Target = HolographicBlock;
    #[inline(always)]
    fn deref(&self) -> &HolographicBlock { &self.0 }
}

impl std::ops::DerefMut for Leg3Pointer {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut HolographicBlock { &mut self.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    #[test]
    fn block_is_exactly_256kb() {
        assert_eq!(size_of::<HolographicBlock>(), BLOCK_SIZE);
    }

    #[test]
    fn stride_boundaries_exact() {
        assert_eq!(offset_of!(HolographicBlock, q),           0x00000);
        assert_eq!(offset_of!(HolographicBlock, p),           0x10000);
        assert_eq!(offset_of!(HolographicBlock, magic),       0x20000);
        assert_eq!(offset_of!(HolographicBlock, energetics),  0x21000);
        assert_eq!(offset_of!(HolographicBlock, concept_ref), 0x22000);
        assert_eq!(offset_of!(HolographicBlock, payload),     0x22028);
        assert_eq!(offset_of!(HolographicBlock, footer),      261_888);
    }

    #[test]
    fn leg3_pointer_is_pointer_sized() {
        assert_eq!(size_of::<Leg3Pointer>(), size_of::<usize>());
    }
}
