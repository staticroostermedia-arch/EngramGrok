// Phase 8 — OptiX RT-Core Smoke Test
//
// Validates the hardware-accelerated BVH pipeline end-to-end:
//   1. Build a GAS from a 3×3×3 grid of synthetic AABBs (27 primitives)
//   2. Issue interior queries (Condition 2) and exterior queries
//   3. Assert hit correctness vs expected primitive indices
//   4. Report build latency and per-query latency
//
// Run with:
//   OPTIX_SDK_PATH=/home/a/optix cargo run -p engram-gpu --example optix_smoke
//
// Without CUDA (CI) a stub main runs and prints a diagnostic — no false failure.

#[cfg(not(engram_backend_cuda))]
fn main() {
    eprintln!("[optix_smoke] CUDA not detected — OptiX smoke test skipped on this platform.");
    eprintln!("[optix_smoke] Set OPTIX_SDK_PATH and rebuild with a CUDA-enabled toolchain.");
}

#[cfg(engram_backend_cuda)]
use std::time::Instant;
#[cfg(engram_backend_cuda)]
use engram_gpu::optix_pipeline::OptixBvhPipeline;
#[cfg(engram_backend_cuda)]
use engram_gpu::bvh::ManifoldEntry;

// AABB half-extent matching AABB_RADIUS in index.rs / optix_intersect.cu
const RADIUS: f32 = 100.0;
// Grid spacing — each primitive is SPACING apart on each axis
const SPACING: f32 = 400.0;
// Grid dimensions (GRID × GRID × GRID primitives)
const GRID: i32 = 3;

#[cfg(engram_backend_cuda)]
fn main() {
    println!("=== OptiX RT-Core Smoke Test ===");

    // ── 1. Build synthetic manifold entries ──────────────────────────────────
    // Lay out a GRID×GRID×GRID lattice. Primitive 0 is at origin.
    // centre(i,j,k) = (i*SPACING, j*SPACING, k*SPACING)
    let mut entries: Vec<ManifoldEntry> = Vec::new();
    for k in 0..GRID {
        for j in 0..GRID {
            for i in 0..GRID {
                let x = i as f32 * SPACING;
                let y = j as f32 * SPACING;
                let z = k as f32 * SPACING;
                entries.push(ManifoldEntry {
                    concept: format!("smoke_{i}_{j}_{k}"),
                    center_3d: engram_gpu::Float3 { x, y, z },
                    file_offset_id: (entries.len() as u64 + 1) * 0x1000,
                    q_quantized: vec![],
                    crs_score: 1.0,
                });
            }
        }
    }
    println!("Primitives: {} ({}³ grid, spacing={SPACING}, radius={RADIUS})",
             entries.len(), GRID);

    // ── 2. Build flat AABB array ──────────────────────────────────────────────
    let aabb_data = OptixBvhPipeline::aabb_from_entries(&entries, RADIUS);
    println!("AABB array: {} × [minX,minY,minZ,maxX,maxY,maxZ]", aabb_data.len());

    // ── 3. Build the OptiX GAS ────────────────────────────────────────────────
    let t_build = Instant::now();
    let pipeline = OptixBvhPipeline::build(&aabb_data);
    let build_ms = t_build.elapsed().as_secs_f64() * 1000.0;

    match &pipeline {
        Some(_) => println!("✅  OptiX GAS built in {build_ms:.2} ms  [RT-Core ACTIVE]"),
        None    => {
            println!("⚠️  OptiX not compiled — falling back to CPU BVH for structural validation.");
            println!("    (set OPTIX_SDK_PATH=/home/a/optix and rebuild to enable RT-Core path)");
            cpu_fallback_test(&entries, &aabb_data);
            return;
        }
    }
    let pipeline = pipeline.unwrap();

    // ── 4. Query suite ───────────────────────────────────────────────────────
    // Each case: (label, query_xyz, expected_primitive_0based)
    let cases: &[(&str, [f32; 3], Option<usize>)] = &[
        // Condition 2: interior hit — query origin exactly at primitive 0 centre
        ("Interior @ prim-0 origin",    [0.0,    0.0,    0.0   ], Some(0)),
        // Condition 2: interior hit — query inside prim-1 (i=1, j=0, k=0)
        ("Interior @ prim-1 centre",    [SPACING, 0.0,   0.0   ], Some(1)),
        // Condition 2: interior hit — prim-13 (centre of grid: i=1,j=1,k=1)
        ("Interior @ grid-centre",      [SPACING, SPACING, SPACING], Some(13)),
        // Condition 1: exterior approach — ray from far-left toward prim-0
        ("Exterior approach prim-0",    [-50.0,  0.0,    0.0   ], Some(0)),
        // Miss: well outside all AABBs (SPACING gap between primitives leaves dead zones)
        ("Miss (gap between prims)",    [SPACING/2.0, SPACING/2.0, 0.0], None),
    ];

    println!("\n{:<35} {:>10}  {:>10}  {}", "Case", "Hits", "Lat(µs)", "Hit IDs (1-based)");
    println!("{}", "-".repeat(80));

    let mut passed = 0usize;
    let mut failed = 0usize;

    for (label, pos, expected_prim) in cases {
        let t0 = Instant::now();
        let hits = pipeline.query_filter_optix(*pos, 8);
        let us = t0.elapsed().as_secs_f64() * 1e6;

        // Convert 1-based hit IDs back to 0-based primitive indices
        let prim_ids: Vec<usize> = hits.iter().map(|&h| h as usize - 1).collect();

        // Evaluate expectation
        let ok = match expected_prim {
            Some(exp) => prim_ids.contains(exp),
            None      => prim_ids.is_empty(),
        };

        let mark = if ok { "✅" } else { "❌" };
        let id_str: String = prim_ids.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        println!("{mark} {:<33} {:>10}  {:>10.1}  [{}]",
                 label, hits.len(), us, id_str);

        if ok { passed += 1; } else { failed += 1; }
    }

    // ── 5. Latency benchmark (warm, 1000 queries) ────────────────────────────
    println!("\n--- Latency Benchmark (1000 interior queries) ---");
    let bench_pos = [0.0f32, 0.0, 0.0];
    let t_bench = Instant::now();
    for _ in 0..1000 {
        let _ = pipeline.query_filter_optix(bench_pos, 8);
    }
    let avg_us = t_bench.elapsed().as_secs_f64() * 1e6 / 1000.0;
    println!("Average query latency: {avg_us:.2} µs  (target: <15 000 µs / <15 ms)");
    let latency_ok = avg_us < 15_000.0;
    println!("Latency gate: {}", if latency_ok { "✅  PASS" } else { "❌  FAIL (>15 ms)" });

    // ── 6. Summary ───────────────────────────────────────────────────────────
    println!("\n=== RESULT: {passed} passed, {failed} failed ===");
    if failed > 0 || !latency_ok {
        std::process::exit(1);
    }
}

// ── CPU path structural validator (when OptiX not compiled) ──────────────────

fn cpu_fallback_test(entries: &[ManifoldEntry], aabb_data: &[[f32; 6]]) {
    println!("\n--- CPU AABB structural validation ({} AABBs) ---", entries.len());
    // Check that the origin falls inside AABB #0 (centre 0,0,0, radius 100)
    let aabb0 = &aabb_data[0];
    let inside = 0.0f32 >= aabb0[0] && 0.0 <= aabb0[3]
              && 0.0f32 >= aabb0[1] && 0.0 <= aabb0[4]
              && 0.0f32 >= aabb0[2] && 0.0 <= aabb0[5];
    println!("Origin inside AABB-0: {}", if inside { "✅ YES" } else { "❌ NO" });
    println!("\nBuild with OPTIX_SDK_PATH=/home/a/optix for hardware acceleration.");
}
