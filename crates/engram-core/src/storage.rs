//! Storage layer — read/write `.leg` blocks from NVMe.
//!
//! # File naming
//!
//! Blocks are stored as `<concept>.leg` in the manifold directory.
//! The file is exactly BLOCK_SIZE (256KB), 4096-byte aligned for O_DIRECT I/O.

use crate::types::{HolographicBlock, BLOCK_SIZE};
use std::io::{Read, Write};
use std::path::Path;

#[cfg(target_os = "linux")]
use std::os::unix::fs::OpenOptionsExt;

/// Read a `.leg` block from disk using O_DIRECT (bypasses OS page cache).
///
/// The block is heap-allocated (256KB — never stack) and returned as a Box.
pub fn read_block<P: AsRef<Path>>(path: P) -> std::io::Result<Box<HolographicBlock>> {
    let mut file = {
        let mut opts = std::fs::OpenOptions::new();
        opts.read(true);
        #[cfg(target_os = "linux")]
        opts.custom_flags(libc::O_DIRECT);
        opts.open(path.as_ref())?
    };

    // Heap-allocate — 256KB must never go on the stack
    let mut block: Box<HolographicBlock> = unsafe {
        let layout = std::alloc::Layout::new::<HolographicBlock>();
        let ptr = std::alloc::alloc_zeroed(layout) as *mut HolographicBlock;
        if ptr.is_null() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "Failed to allocate HolographicBlock (256KB)",
            ));
        }
        Box::from_raw(ptr)
    };

    let ptr = block.as_mut() as *mut HolographicBlock as *mut u8;
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, BLOCK_SIZE) };
    file.read_exact(slice)?;
    Ok(block)
}

/// Write a `.leg` block to disk using O_DIRECT.
///
/// Applies Toryx periodic boundary conditions before writing:
/// `q[8191] = q[0]` and `p[8191] = p[0]`.
pub fn write_block<P: AsRef<Path>>(path: P, block: &HolographicBlock) -> std::io::Result<()> {
    let mut file = {
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(target_os = "linux")]
        opts.custom_flags(libc::O_DIRECT);
        opts.open(path.as_ref())?
    };

    // Heap-copy to apply Toryx PBC without modifying the caller's block
    let mut boxed: Box<HolographicBlock> = unsafe {
        let layout = std::alloc::Layout::new::<HolographicBlock>();
        let ptr = std::alloc::alloc_zeroed(layout) as *mut HolographicBlock;
        if ptr.is_null() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "Failed to allocate HolographicBlock for write (256KB)",
            ));
        }
        Box::from_raw(ptr)
    };
    unsafe { std::ptr::copy_nonoverlapping(block, &mut *boxed, 1); }

    // Toryx Periodic Boundary Condition: close the standing spiral
    boxed.q[8191] = boxed.q[0];
    boxed.p[8191] = boxed.p[0];

    let ptr = boxed.as_ref() as *const HolographicBlock as *const u8;
    let slice = unsafe { std::slice::from_raw_parts(ptr, BLOCK_SIZE) };
    file.write_all(slice)?;
    file.sync_all()?;
    Ok(())
}

/// Read the ProvLog text from a block's payload field.
///
/// Returns UTF-8 text up to the first null byte, or an empty string if
/// the payload contains no valid UTF-8.
pub fn read_provlog(block: &HolographicBlock) -> String {
    let end = block.payload.iter().position(|&b| b == 0).unwrap_or(block.payload.len());
    String::from_utf8_lossy(&block.payload[..end]).into_owned()
}

/// Write ProvLog text into a block's payload field.
///
/// Truncates silently if `text.len() > payload capacity (122,584 bytes)`.
pub fn write_provlog(block: &mut HolographicBlock, text: &str) {
    let bytes = text.as_bytes();
    let len = bytes.len().min(block.payload.len());
    block.payload[..len].copy_from_slice(&bytes[..len]);
}
