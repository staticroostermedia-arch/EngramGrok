//! Memory-mapped read access for `.leg` blocks.
//!
//! `mmap` provides zero-copy read access to blocks — the OS maps the file
//! directly into the process address space. Faster than O_DIRECT for
//! sequential access patterns; prefer `storage::read_block` for random access.

use crate::types::{HolographicBlock, BLOCK_SIZE};
use std::path::Path;

/// A memory-mapped view of a single `.leg` block file.
///
/// The file is opened read-only and mmap'd with `MADV_RANDOM` to hint that
/// we'll access it non-sequentially. The mapping is unmapped on drop.
pub struct LegView {
    ptr: *const u8,
    len: usize,
}

unsafe impl Send for LegView {}
unsafe impl Sync for LegView {}

impl LegView {
    /// Open and mmap a `.leg` block file. Returns an error if the file
    /// doesn't exist or isn't exactly `BLOCK_SIZE` (256KB) bytes.
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        use std::os::unix::io::AsRawFd;
        let file = std::fs::File::open(path.as_ref())?;
        let meta = file.metadata()?;
        if meta.len() as usize != BLOCK_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "LEG file is {} bytes, expected {} (256KB)",
                    meta.len(), BLOCK_SIZE
                ),
            ));
        }
        let fd = file.as_raw_fd();
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                BLOCK_SIZE,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }
        // Hint: random access pattern (BVH refine step)
        unsafe { libc::madvise(ptr, BLOCK_SIZE, libc::MADV_RANDOM); }
        Ok(Self { ptr: ptr as *const u8, len: BLOCK_SIZE })
    }

    /// Borrow the raw bytes of the block.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Borrow the block as a typed `&HolographicBlock`.
    ///
    /// # Safety
    /// The file must be a valid, fully-written LEG block.
    #[inline]
    pub fn as_block(&self) -> &HolographicBlock {
        unsafe { &*(self.ptr as *const HolographicBlock) }
    }
}

impl Drop for LegView {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.ptr as *mut libc::c_void, self.len); }
    }
}
