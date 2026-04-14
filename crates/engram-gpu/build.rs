//! Build script for engram-gpu.
//!
//! When the `cuda-kernels` feature is enabled AND `nvcc` is found in PATH,
//! compiles the CUDA kernels to a static library and links it.
//!
//! On machines without CUDA, the build succeeds silently — the CudaBackend
//! automatically falls back to the CPU BVH path.

fn main() {
    // Only compile kernels when feature is enabled
    if std::env::var("CARGO_FEATURE_CUDA_KERNELS").is_err() {
        return;
    }

    // Check nvcc is available
    let nvcc = which_nvcc();
    let Some(nvcc_path) = nvcc else {
        println!("cargo:warning=nvcc not found — skipping CUDA kernel compilation.");
        println!("cargo:warning=The CudaBackend will use CPU BVH and cosine fallback.");
        return;
    };

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

fn which_nvcc() -> Option<std::path::PathBuf> {
    // Check CUDA_HOME first, then PATH
    if let Ok(cuda_home) = std::env::var("CUDA_HOME") {
        let candidate = std::path::Path::new(&cuda_home).join("bin").join("nvcc");
        if candidate.exists() { return Some(candidate); }
    }
    // Walk PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join("nvcc");
            if candidate.exists() { return Some(candidate); }
        }
    }
    None
}
