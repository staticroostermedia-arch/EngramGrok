// ── int8_raytracer.wgsl ────────────────────────────────────────────────────────
// Poincaré Disk Hyperbolic Distance — INT8 Quantized K-NN Search
//
// Originally authored by Randall / Command Center codebase (April 2026).
// Ported verbatim into engram-gpu as the default search kernel.
//
// Architecture:
//   - Binding 0: query     — [u32; 96]    (384 INT8 values packed 4-per-u32)
//   - Binding 1: database  — [u32]        (96 u32s per block × N blocks)
//   - Binding 2: scores    — [f32]        (one similarity score per block)
//   - Binding 3: config    — { num_blocks: u32, _pad: [u32; 3] }
//
// Each workgroup processes 256 blocks in parallel.
// Score formula: similarity = 1 / (1 + arcosh(1 + 2·||u-v||² / ((1-||u||²)·(1-||v||²))))
// Maps Poincaré distance ∈ [0, ∞) → similarity ∈ (0, 1].

@group(0) @binding(0) var<storage, read>       query:    array<u32, 96>;
@group(0) @binding(1) var<storage, read>       database: array<u32>;   // packed, 96 per block centroid
@group(0) @binding(2) var<storage, read_write> scores:   array<f32>;

struct Config { num_blocks: u32 }
@group(0) @binding(3) var<uniform> config: Config;

// Unpack one i8 lane from a packed u32 and convert to f32.
// Bytes are stored little-endian: lane 0 = bits[0..7], lane 1 = bits[8..15], etc.
fn unpack_i8(packed: u32, idx: u32) -> f32 {
    let bits = extractBits(packed, idx * 8u, 8u);
    // Sign-extend: if bit-7 is set (value >= 128), the i8 is negative
    let signed_val = select(f32(bits), f32(bits) - 256.0, bits >= 128u);
    return signed_val;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= config.num_blocks) { return; }

    let block_offset = idx * 96u;

    var d_sq      = 0.0f;
    var norm_u_sq = 0.0f;
    var norm_v_sq = 0.0f;

    for (var i = 0u; i < 96u; i = i + 1u) {
        let q_packed  = query[i];
        let db_packed = database[block_offset + i];

        for (var j = 0u; j < 4u; j = j + 1u) {
            let u = unpack_i8(q_packed,  j);
            let v = unpack_i8(db_packed, j);

            // Dequantize: INT8 [-128,127] → float [-1.0, 1.0]
            let u_f = u / 127.0f;
            let v_f = v / 127.0f;

            let diff = u_f - v_f;
            d_sq      += diff * diff;
            norm_u_sq += u_f * u_f;
            norm_v_sq += v_f * v_f;
        }
    }

    // Poincaré ball boundary clamp — prevents singularity at unit sphere perimeter.
    // L2-normalised MiniLM embeddings land just inside the ball after INT8 rounding.
    let r2_u = min(norm_u_sq, 0.9999f);
    let r2_v = min(norm_v_sq, 0.9999f);

    let num = 2.0f * d_sq;
    let den = max((1.0f - r2_u) * (1.0f - r2_v), 0.0001f);

    let delta = num / den;
    let arg   = 1.0f + delta;

    // arcosh(x) = ln(x + sqrt(x²-1)), numerically stable via log
    let hyp_dist = log(arg + sqrt(max(arg * arg - 1.0f, 0.0f)));

    // Invert: smaller distance → higher similarity score (max = 1.0 for identical vectors)
    scores[idx] = 1.0f / (1.0f + hyp_dist);
}
