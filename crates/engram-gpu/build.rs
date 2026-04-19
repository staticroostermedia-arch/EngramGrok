//! Build script for engram-gpu.
//!
//! AUTO-DETECTION: This script probes the host machine for available GPU toolchains
//! and automatically compiles the best available backend. No feature flags required.
//!
//! Priority order:
//!   1. CUDA  (NVIDIA) — if `nvcc` found in PATH or CUDA_HOME
//!   2. ROCm  (AMD)    — if `hipcc` found in PATH or ROCM_PATH
//!   3. Metal (Apple)  — if target_os = "macos" (no compiler probe needed, uses metal-rs)
//!   4. WebGPU         — cross-platform fallback (wgpu, always compiled on non-macOS)
//!   5. CPU            — always available baseline
//!
//! Emitted cfg flags (used in lib.rs / server/store.rs):
//!   engram_backend_cuda   — CUDA kernels compiled and linked
//!   engram_backend_rocm   — ROCm/HIP kernels compiled and linked
//!   engram_backend_metal  — Metal backend active (macOS only)
//!   engram_backend_wgpu   — WebGPU wgpu backend active
//!   engram_backend_cpu    — CPU baseline (always set)

fn main() {
    // ── Always emit CPU baseline ─────────────────────────────────────────────
    println!("cargo:rustc-cfg=engram_backend_cpu");

    // ── Probe CUDA ────────────────────────────────────────────────────────────
    if let Some(nvcc_path) = which_compiler("nvcc", "CUDA_HOME") {
        println!("cargo:warning=engram: CUDA detected (nvcc: {}). Compiling GPU kernels.", nvcc_path.display());
        println!("cargo:rustc-cfg=engram_backend_cuda");
        compile_cuda(&nvcc_path);
        return; // CUDA is top priority — skip other GPU backends
    }

    // ── Probe ROCm ────────────────────────────────────────────────────────────
    if let Some(hipcc_path) = which_compiler("hipcc", "ROCM_PATH") {
        println!("cargo:warning=engram: ROCm detected (hipcc: {}). Compiling HIP kernels.", hipcc_path.display());
        println!("cargo:rustc-cfg=engram_backend_rocm");
        compile_rocm(&hipcc_path);
        return;
    }

    // ── Metal (macOS) ─────────────────────────────────────────────────────────
    // metal-rs links automatically via Cargo.toml target cfg — just emit the cfg flag.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:warning=engram: macOS detected. Activating Metal backend.");
        println!("cargo:rustc-cfg=engram_backend_metal");
        return;
    }

    // ── WebGPU fallback ───────────────────────────────────────────────────────
    // wgpu is cross-platform and requires no native toolchain.
    // Activate if a GPU is detectable via runtime env hints, or unconditionally
    // as a better-than-CPU fallback on Linux/Windows without CUDA/ROCm.
    println!("cargo:warning=engram: No CUDA/ROCm/Metal detected. Activating WebGPU (wgpu) backend.");
    println!("cargo:rustc-cfg=engram_backend_wgpu");
}

fn compile_cuda(nvcc_path: &std::path::Path) {
    let kernel_dir = std::path::Path::new("kernels");
    let out_dir    = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let kernels = ["arkade_8k.cu", "bvh_traverse.cu"];
    let mut obj_files = Vec::new();

    for kernel in &kernels {
        let src = kernel_dir.join(kernel);
        if !src.exists() {
            println!("cargo:warning=engram: kernel source {} not found — skipping.", src.display());
            continue;
        }
        let stem = kernel.trim_end_matches(".cu");
        let obj  = out_dir.join(format!("{stem}.o"));

        let status = std::process::Command::new(nvcc_path)
            .args([
                "-O3",
                "--gpu-architecture=sm_75",     // Turing+ (RTX 2000)
                "--generate-code=arch=compute_75,code=sm_75",
                "--generate-code=arch=compute_86,code=sm_86",  // Ampere (RTX 3000)
                "--generate-code=arch=compute_89,code=sm_89",  // Ada (RTX 4000/5000)
                "-Xcompiler", "-fPIC",
                "-c",
                src.to_str().unwrap(),
                "-o", obj.to_str().unwrap(),
            ])
            .status()
            .expect("failed to exec nvcc");

        if !status.success() {
            println!("cargo:warning=engram: CUDA kernel compilation failed for {kernel}");
            continue;
        }
        obj_files.push(obj);
    }

    if !obj_files.is_empty() {
        let lib_path = out_dir.join("libengram_kernels.a");
        let mut ar = std::process::Command::new("ar");
        ar.arg("crs").arg(&lib_path);
        for obj in &obj_files { ar.arg(obj); }
        ar.status().expect("failed to exec ar");

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static=engram_kernels");
        println!("cargo:rustc-link-lib=dylib=cuda");
        println!("cargo:rustc-link-lib=dylib=cudart");
        println!("cargo:warning=engram: CUDA kernels compiled and linked successfully.");
    } else {
        // Kernels failed to compile — fall back to CPU BVH, still emit cuda cfg
        // so the runtime probe path is used (dlopen libcuda.so for queries).
        println!("cargo:warning=engram: CUDA kernel compilation produced no objects. Using CPU BVH + runtime CUDA probe.");
    }
}

fn compile_rocm(hipcc_path: &std::path::Path) {
    let kernel_dir = std::path::Path::new("kernels");
    let out_dir    = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let kernels = ["arkade_8k.hip"];
    let mut obj_files = Vec::new();

    for kernel in &kernels {
        let src = kernel_dir.join(kernel);
        if !src.exists() {
            println!("cargo:warning=engram: kernel source {} not found — skipping.", src.display());
            continue;
        }
        let stem = kernel.trim_end_matches(".hip");
        let obj  = out_dir.join(format!("{stem}.o"));

        let status = std::process::Command::new(hipcc_path)
            .args(["-O3", "-fPIC", "-c", src.to_str().unwrap(), "-o", obj.to_str().unwrap()])
            .status()
            .expect("failed to exec hipcc");

        if !status.success() {
            println!("cargo:warning=engram: ROCm kernel compilation failed for {kernel}");
            continue;
        }
        obj_files.push(obj);
    }

    if !obj_files.is_empty() {
        let lib_path = out_dir.join("libengram_rocm_kernels.a");
        let mut ar = std::process::Command::new("ar");
        ar.arg("crs").arg(&lib_path);
        for obj in &obj_files { ar.arg(obj); }
        ar.status().expect("failed to exec ar");

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static=engram_rocm_kernels");
        println!("cargo:rustc-link-lib=dylib=amdhip64");
        println!("cargo:warning=engram: ROCm kernels compiled and linked successfully.");
    }
}

fn which_compiler(binary_name: &str, env_home: &str) -> Option<std::path::PathBuf> {
    // Check dedicated env home first (e.g. CUDA_HOME=/usr/local/cuda)
    if let Ok(home) = std::env::var(env_home) {
        let candidate = std::path::Path::new(&home).join("bin").join(binary_name);
        if candidate.exists() { return Some(candidate); }
    }
    // Walk PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(binary_name);
            if candidate.exists() { return Some(candidate); }
        }
    }
    None
}
