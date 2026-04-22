//! Storage layer — read/write `.leg` blocks from NVMe.
//!
//! # File naming
//!
//! Blocks are stored as `<concept>.leg` in the manifold directory.
//! The file is exactly BLOCK_SIZE (256KB), 4096-byte aligned for O_DIRECT I/O.

// Cap'n Proto generated bindings for the ProvLog schema (schema/provlog.capnp).
// Generated at build time by build.rs via capnpc.
pub mod provlog_capnp {
    include!(concat!(env!("OUT_DIR"), "/provlog_capnp.rs"));
}

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

/// Read the ProvLog text from a block's payload field (Capn Proto).
///
/// Falls back to UTF-8 parsing for legacy blocks without Cap'n Proto framing.
pub fn read_provlog(block: &HolographicBlock) -> String {
    let mut slice = &block.payload[..];
    if let Ok(message) = capnp::serialize::read_message(&mut slice, capnp::message::ReaderOptions::new()) {
        if let Ok(plog) = message.get_root::<provlog_capnp::prov_log::Reader>() {
            if let Ok(text) = plog.get_source_text() {
                if let Ok(string) = text.to_string() {
                    return string;
                }
            }
        }
    }

    // Fallback: raw UTF-8 up to first null (for legacy non-unified stalks prior to the Great Reminting)
    let end = block.payload.iter().position(|&b| b == 0).unwrap_or(block.payload.len());
    String::from_utf8_lossy(&block.payload[..end]).into_owned()
}

/// Write ProvLog text into a block's payload field using Cap'n Proto.
pub fn write_provlog(block: &mut HolographicBlock, text: &str) {
    let mut message = capnp::message::Builder::new_default();
    {
        let mut plog = message.init_root::<provlog_capnp::prov_log::Builder>();
        plog.set_source_text(text);
        plog.set_text_data(());
        
        // Export Thermodynamic factors from Engram directly into the Capnp block schema
        plog.set_ego_coherence(block.energetics.crs);
        
        // Map Praxis logic
        if block.zedos_tag == crate::types::ZEDOS_PRAXIS {
            plog.set_is_praxis(true);
        }
    }

    let mut buf = Vec::new();
    capnp::serialize::write_message(&mut buf, &message).unwrap();
    let len = buf.len().min(block.payload.len());
    
    // Wipe previous trailing bytes to avoid Cap'n Proto parse confusion
    block.payload.fill(0);
    block.payload[..len].copy_from_slice(&buf[..len]);
}
