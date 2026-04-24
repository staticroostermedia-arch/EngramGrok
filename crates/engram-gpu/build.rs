//! Build script for engram-gpu.
//!
//! AUTO-DETECTION: This script probes the host machine for available GPU toolchains
//! and automatically compiles the best available backend. No feature flags required.
//!
//! Priority order:
//!   1. CUDA  (NVIDIA) — if `nvcc` found in PATH or CUDA_HOME
//!      └─ Phase 8: OptiX RT-Core BVH also compiled if OPTIX_SDK_PATH is set
//!   2. ROCm  (AMD)    — if `hipcc` found in PATH or ROCM_PATH
//!   3. Metal (Apple)  — if target_os = "macos" (no compiler probe needed, uses metal-rs)
//!   4. WebGPU         — cross-platform fallback (wgpu, always compiled on non-macOS)
//!   5. CPU            — always available baseline
//!
//! Emitted cfg flags:
//!   engram_backend_cuda   — CUDA kernels compiled and linked
//!   engram_backend_rocm   — ROCm/HIP kernels compiled and linked
//!   engram_backend_metal  — Metal backend active (macOS only)
//!   engram_backend_wgpu   — WebGPU wgpu backend active
//!   engram_backend_cpu    — CPU baseline (always set)
//!   optix_available       — OptiX RT-Core BVH compiled (NVIDIA only)

fn main() {
    // Declare custom cfg keys so rustc doesn't warn about unknown cfgs
    println!("cargo::rustc-check-cfg=cfg(optix_available)");

    // ── Always emit CPU baseline ─────────────────────────────────────────────
    println!("cargo:rustc-cfg=engram_backend_cpu");

    // ── Probe CUDA ────────────────────────────────────────────────────────────
    if let Some(nvcc_path) = which_compiler("nvcc", "CUDA_HOME") {
        println!("cargo:warning=engram: CUDA detected (nvcc: {}). Compiling GPU kernels.", nvcc_path.display());
        println!("cargo:rustc-cfg=engram_backend_cuda");

        // ── Phase 8: OptiX RT-Core BVH — gated on OPTIX_SDK_PATH ──────────
        // Download SDK from https://developer.nvidia.com/optix, then:
        //   export OPTIX_SDK_PATH=/path/to/OptiX-SDK-X.X.X-linux64-x86_64
        if let Ok(sdk) = std::env::var("OPTIX_SDK_PATH") {
            // Validate path before attempting compilation — avoids hard panic on placeholder paths.
            let optix_h = std::path::Path::new(&sdk).join("include").join("optix.h");
            if !optix_h.exists() {
                println!("cargo:warning=engram: OPTIX_SDK_PATH={sdk} — optix.h not found. \
                          Install the real OptiX SDK from https://developer.nvidia.com/optix \
                          and re-export OPTIX_SDK_PATH. Falling back to software BVH.");
                cc::Build::new()
                    .cpp(true)
                    .flag("-std=c++17")
                    .file("src/optix_host.cpp")
                    .compile("optix_host");
            } else {
                println!("cargo:warning=engram: OptiX SDK at {sdk}. Compiling RT-Core BVH.");
                if compile_optix_ptx(&nvcc_path, &sdk) {
                    compile_optix_host(&sdk);
                    println!("cargo:rustc-cfg=optix_available");
                } else {
                    println!("cargo:warning=engram: OptiX PTX compilation failed — software BVH fallback.");
                    cc::Build::new()
                        .cpp(true)
                        .flag("-std=c++17")
                        .file("src/optix_host.cpp")
                        .compile("optix_host");
                }
            }
        } else {
            println!("cargo:warning=engram: OPTIX_SDK_PATH not set — software BVH only.");
            // Compile stub C++ so symbols are always defined (no SDK = stubs return nullptr)
            cc::Build::new()
                .cpp(true)
                .flag("-std=c++17")
                .file("src/optix_host.cpp")
                .compile("optix_host");
        }
        println!("cargo:rerun-if-env-changed=OPTIX_SDK_PATH");

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
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:warning=engram: macOS detected. Activating Metal backend.");
        println!("cargo:rustc-cfg=engram_backend_metal");
        return;
    }

    // ── WebGPU fallback ───────────────────────────────────────────────────────
    println!("cargo:warning=engram: No CUDA/ROCm/Metal detected. Activating WebGPU (wgpu) backend.");
    println!("cargo:rustc-cfg=engram_backend_wgpu");
}

// ── CUDA regular kernels ──────────────────────────────────────────────────────

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
                "--gpu-architecture=sm_75",
                "--generate-code=arch=compute_75,code=sm_75",
                "--generate-code=arch=compute_86,code=sm_86",
                "--generate-code=arch=compute_89,code=sm_89",
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
        println!("cargo:warning=engram: CUDA kernel compilation produced no objects. Using CPU BVH + runtime CUDA probe.");
    }
}

// ── ROCm ─────────────────────────────────────────────────────────────────────

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

// ── OptiX helpers (Phase 8) ───────────────────────────────────────────────────

/// Compile the 4 OptiX device programs to PTX and generate optix_ptx.rs.
/// Returns `true` only if all 4 programs compiled successfully.
fn compile_optix_ptx(nvcc: &std::path::Path, sdk: &str) -> bool {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out = std::path::Path::new(&out_dir);
    let kernel_dir = std::path::Path::new("kernels");

    let programs = [
        ("optix_intersect", "OPTIX_INTERSECT_PTX"),
        ("optix_rg",        "OPTIX_RG_PTX"),
        ("optix_ah",        "OPTIX_AH_PTX"),
        ("optix_ms",        "OPTIX_MS_PTX"),
    ];

    let mut ptx_rs = String::from("// AUTO-GENERATED by build.rs — do not edit\n");
    let mut all_ok = true;

    for (stem, const_name) in &programs {
        let src = kernel_dir.join(format!("{stem}.cu"));
        if !src.exists() {
            println!("cargo:warning=engram: {stem}.cu not found — skipping.");
            all_ok = false;
            continue;
        }
        let ptx_path = out.join(format!("{stem}.ptx"));

        // IMPORTANT: `--ptx` is incompatible with multiple `--generate-code` flags.
        // PTX is virtual ISA — compile for a single virtual architecture.
        // compute_86 (Ampere) is the minimum for RT-Core Gen2 and forward-compatible
        // with Ada (sm_89) and Blackwell (sm_100) via JIT at runtime.
        let status = std::process::Command::new(nvcc)
            .args([
                "--ptx",
                "--gpu-architecture=compute_86",
                &format!("-I{sdk}/include"),
                src.to_str().unwrap(),
                "-o", ptx_path.to_str().unwrap(),
            ])
            .status()
            .expect("nvcc not found");

        if status.success() {
            ptx_rs.push_str(&format!(
                "pub static {const_name}: &str = include_str!(concat!(env!(\"OUT_DIR\"), \"/{stem}.ptx\"));\n"
            ));
        } else {
            println!("cargo:warning=engram: OptiX PTX compile failed for {stem}");
            all_ok = false;
        }
    }

    if all_ok {
        std::fs::write(out.join("optix_ptx.rs"), ptx_rs).unwrap();
    }

    println!("cargo:rerun-if-changed=kernels/optix_intersect.cu");
    println!("cargo:rerun-if-changed=kernels/optix_rg.cu");
    println!("cargo:rerun-if-changed=kernels/optix_ah.cu");
    println!("cargo:rerun-if-changed=kernels/optix_ms.cu");
    println!("cargo:rerun-if-changed=src/optix_host.cpp");
    all_ok
}

/// Compile optix_host.cpp with the OptiX SDK headers enabled.
fn compile_optix_host(sdk: &str) {
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++17")
        .flag(&format!("-I{sdk}/include"))
        .flag("-I/usr/local/cuda/include")
        .flag("-DOPTIX_SDK_AVAILABLE")
        .file("src/optix_host.cpp")
        .compile("optix_host");

    // OptiX stubs use dlopen("libnvoptix.so.1", ...) at runtime — no build-time link needed.
    // Only libdl is required for the dlopen() call itself.
    println!("cargo:rustc-link-lib=dylib=dl");
}

// ── Compiler detection ────────────────────────────────────────────────────────

fn which_compiler(binary_name: &str, env_home: &str) -> Option<std::path::PathBuf> {
    if let Ok(home) = std::env::var(env_home) {
        let candidate = std::path::Path::new(&home).join("bin").join(binary_name);
        if candidate.exists() { return Some(candidate); }
    }
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(binary_name);
            if candidate.exists() { return Some(candidate); }
        }
    }
    None
}
