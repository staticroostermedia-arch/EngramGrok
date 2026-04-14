//! Build script for engram-gpu.
//!
//! When the `cuda-kernels` feature is enabled AND `nvcc` is found in PATH,
//! compiles the CUDA kernels to a static library and links it.
//!
//! On machines without CUDA, the build succeeds silently — the CudaBackend
//! automatically falls back to the CPU BVH path.

fn main() {
    // Check for CUDA compilation first
    let build_cuda = std::env::var("CARGO_FEATURE_CUDA_KERNELS").is_ok();
    let build_rocm = std::env::var("CARGO_FEATURE_ROCM_KERNELS").is_ok();

    if !build_cuda && !build_rocm {
        return;
    }

    if build_cuda {
        if let Some(nvcc_path) = which_compiler("nvcc", "CUDA_HOME") {
            compile_cuda(&nvcc_path);
        } else {
            println!("cargo:warning=nvcc not found — skipping CUDA kernel compilation.");
        }
    }

    if build_rocm {
        if let Some(hipcc_path) = which_compiler("hipcc", "ROCM_PATH") {
            compile_rocm(&hipcc_path);
        } else {
            println!("cargo:warning=hipcc not found — skipping ROCm kernel compilation.");
        }
    }
}

fn compile_cuda(nvcc_path: &std::path::Path) {

    let kernel_dir = std::path::Path::new("kernels");
    let out_dir    = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let kernels = ["arkade_8k.cu", "bvh_traverse.cu"];
    let mut obj_files = Vec::new();

    for kernel in &kernels {
        let src   = kernel_dir.join(kernel);
        let stem  = kernel.trim_end_matches(".cu");
        let obj   = out_dir.join(format!("{stem}.o"));
        let ptx   = out_dir.join(format!("{stem}.ptx"));

        // Compile to PTX then to .o using nvcc
        let status = std::process::Command::new(&nvcc_path)
            .args([
                "-O3",
                "--gpu-architecture=sm_75",     // Turing+ (RTX 2000, T4, A100, RTX 5060)
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
            println!("cargo:warning=CUDA kernel compilation failed for {kernel}");
            continue;
        }
        obj_files.push(obj);
    }

    if !obj_files.is_empty() {
        // Create static archive from all .o files
        let lib_path = out_dir.join("libengram_kernels.a");
        let mut ar = std::process::Command::new("ar");
        ar.arg("crs").arg(&lib_path);
        for obj in &obj_files { ar.arg(obj); }
        ar.status().expect("failed to exec ar");

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static=engram_kernels");
        println!("cargo:rustc-link-lib=dylib=cuda");
        println!("cargo:rustc-link-lib=dylib=cudart");
    }
}

fn compile_rocm(hipcc_path: &std::path::Path) {
    let kernel_dir = std::path::Path::new("kernels");
    let out_dir    = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // For now we map arkade_8k.hip which implements HIP equivalents of Cuda Kernels
    let kernels = ["arkade_8k.hip"];
    let mut obj_files = Vec::new();

    for kernel in &kernels {
        let src   = kernel_dir.join(kernel);
        let stem  = kernel.trim_end_matches(".hip");
        let obj   = out_dir.join(format!("{stem}.o"));

        let status = std::process::Command::new(hipcc_path)
            .args([
                "-O3",
                "-fPIC",
                "-c",
                src.to_str().unwrap(),
                "-o", obj.to_str().unwrap(),
            ])
            .status()
            .expect("failed to exec hipcc");

        if !status.success() {
            println!("cargo:warning=ROCm kernel compilation failed for {kernel}");
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
    }
}

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
