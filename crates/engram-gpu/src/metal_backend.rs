//! `MetalBackend` — implements `VsaBackend` using Apple Metal GPU kernels.
//!
//! On macOS with a Metal-capable GPU, this backend compiles the MSL kernel
//! (`kernels/arkade_8k.metal`) at startup and stores the pipeline for future
//! dispatch. All current query/recall operations delegate to `CpuBackend` while
//! GPU kernel dispatch is completed in Phase 10.
//!
//! On non-macOS targets, this module compiles to a safe no-op stub that panics
//! only if actually instantiated — it is never instantiated on non-macOS.

use engram_core::backend::{CpuBackend, Memory, VsaBackend};
use engram_core::types::Leg3Pointer;
use anyhow::Result;
use std::path::{Path, PathBuf};

// ── macOS implementation ───────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
use metal::*;

#[cfg(target_os = "macos")]
pub struct MetalBackend {
    store_path: PathBuf,
    cpu: CpuBackend,
    #[allow(dead_code)]
    device: Device,
    #[allow(dead_code)]
    command_queue: CommandQueue,
    #[allow(dead_code)]
    library: Library,
    gpu_available: bool,
}

#[cfg(target_os = "macos")]
impl MetalBackend {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path_str = shellexpand::tilde(
            path.as_ref().to_str().unwrap_or("~/.engram/manifold")
        ).into_owned();
        std::fs::create_dir_all(&path_str).ok();

        match Device::system_default() {
            Some(device) => {
                let command_queue = device.new_command_queue();
                let msl_source = include_str!("../kernels/arkade_8k.metal");
                let options = CompileOptions::new();
                match device.new_library_with_source(msl_source, &options) {
                    Ok(library) => {
                        tracing::info!("[engram-gpu] Metal device: {} — MSL kernel compiled OK", device.name());
                        Self {
                            cpu: CpuBackend::new(&path_str),
                            store_path: PathBuf::from(&path_str),
                            device,
                            command_queue,
                            library,
                            gpu_available: true,
                        }
                    }
                    Err(e) => {
                        tracing::warn!("[engram-gpu] Metal MSL compile failed: {} — falling back to CPU", e);
                        // Still need to return a valid struct; use a dummy library by retrying
                        // with an empty source so we can store something in the field.
                        let fallback_lib = device
                            .new_library_with_source("kernel void _noop() {}", &options)
                            .expect("Metal device exists but cannot compile trivial kernel");
                        Self {
                            cpu: CpuBackend::new(&path_str),
                            store_path: PathBuf::from(&path_str),
                            device,
                            command_queue,
                            library: fallback_lib,
                            gpu_available: false,
                        }
                    }
                }
            }
            None => {
                tracing::warn!("[engram-gpu] No Metal device found — CPU fallback active");
                // On macOS without Metal (e.g. CI), we still need valid Device/Queue/Library.
                // This path should be extremely rare; panic here is acceptable.
                panic!("MetalBackend::new() called but no Metal device is available on this machine.");
            }
        }
    }

    pub fn gpu_available(&self) -> bool { self.gpu_available }
}

#[cfg(target_os = "macos")]
impl VsaBackend for MetalBackend {
    fn encode(&self, text: &str) -> Leg3Pointer {
        self.cpu.encode(text)
    }

    fn fetch(&self, concept: &str) -> Option<Box<[engram_core::Complex32; 8192]>> {
        self.cpu.fetch(concept)
    }

    fn fetch_block(&self, concept: &str) -> Option<Leg3Pointer> {
        self.cpu.fetch_block(concept)
    }

    fn query(&self, query: &[engram_core::Complex32; 8192], k: usize) -> Vec<Memory> {
        // Phase 10: dispatch engram_cosine_batch via Metal command encoder.
        // For now: CPU linear scan (identical correctness, lower throughput).
        self.cpu.query(query, k)
    }

    fn store(&self, concept: &str, block: Leg3Pointer) -> Result<()> {
        self.cpu.store(concept, block)
    }

    fn forget(&self, concept: &str) -> Result<()> {
        self.cpu.forget(concept)
    }

    fn list(&self) -> Vec<String> {
        self.cpu.list()
    }
}

// ── Non-macOS stub ────────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
pub struct MetalBackend {
    _dummy: std::marker::PhantomData<()>,
}

#[cfg(not(target_os = "macos"))]
impl MetalBackend {
    pub fn new(_path: impl AsRef<Path>) -> Self {
        panic!("MetalBackend is only available on macOS. Use CudaBackend or CpuBackend instead.");
    }
}

#[cfg(not(target_os = "macos"))]
impl VsaBackend for MetalBackend {
    fn encode(&self, _: &str) -> Leg3Pointer { unreachable!() }
    fn fetch(&self, _: &str) -> Option<Box<[engram_core::Complex32; 8192]>> { unreachable!() }
    fn fetch_block(&self, _: &str) -> Option<Leg3Pointer> { unreachable!() }
    fn query(&self, _: &[engram_core::Complex32; 8192], _: usize) -> Vec<Memory> { unreachable!() }
    fn store(&self, _: &str, _: Leg3Pointer) -> Result<()> { unreachable!() }
    fn forget(&self, _: &str) -> Result<()> { unreachable!() }
    fn list(&self) -> Vec<String> { unreachable!() }
}
