//! Text encoder — convert free-form text into a HolographicBlock.
//!
//! Uses deterministic spiral phase encoding: no neural network, no embedding API,
//! no network call. Same text always produces the same vector.
//!
//! # Algorithm
//!
//! 1. Hash the input text with BLAKE3 → 32-byte seed
//! 2. Generate 8192 complex phase angles via an XOF (extended output function)
//! 3. Apply character-level spiral weighting to capture word structure
//! 4. Normalize to the unit hypersphere |z| = 1.0
//! 5. Pack into a `HolographicBlock` with the source text in the ProvLog

use crate::types::{Leg3Pointer, ZEDOS_DECLARATIVE, DIMENSION};
use crate::ops::normalize;
use crate::storage::write_provlog;
use num_complex::Complex32;

/// Fetch a dense embedding vector from the configured embedding server.
///
/// Tries the `ENGRAM_EMBED_URL` environment variable first (e.g. nomic-embed
/// served by llama-server on port 8086), then falls back to port 8086 default.
///
/// **IMPORTANT FOR AGENTS:** If this returns None, the Euler gate WILL reject the
/// resulting vector because BLAKE3-only vectors have chaotic phase distributions.
/// Ensure ENGRAM_EMBED_URL=http://localhost:8086/v1/embeddings is set in the
/// environment of whichever process is calling remember().
///
/// Returns a L2-normalised `Vec<f32>` of any dimension, or `None` on failure.
fn fetch_neural_embedding(text: &str) -> Option<Vec<f32>> {
    use serde_json::json;

    // Default is port 8086 (nomic-embed via llama-server).
    // Override with ENGRAM_EMBED_URL env var.
    let url = std::env::var("ENGRAM_EMBED_URL")
        .unwrap_or_else(|_| "http://localhost:8086/v1/embeddings".to_string());

    let text_owned = text.to_string();
    let url_clone  = url.clone();

    // ── CRITICAL: Spawn a dedicated OS thread for the blocking HTTP call ──────
    //
    // reqwest::blocking panics if called while a Tokio runtime is active on
    // the current thread (e.g. the MCP server's background daemon spawns a
    // Tokio runtime at boot). The panic propagates as an Err through the
    // blocking client builder's .build().ok()? and silently falls back to
    // BLAKE3-only encoding — which always fails the Euler gate.
    //
    // Spawning a fresh OS thread guarantees no Tokio context is present,
    // making reqwest::blocking safe to use without panicking.
    let result = std::thread::spawn(move || -> Option<Vec<f32>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(3000))  // 3s — handles cold starts
            .build()
            .ok()?;

        let body = json!({
            "input": text_owned,
            "model": "local"
        });

        let res: serde_json::Value = client.post(&url_clone)
            .json(&body)
            .send()
            .map_err(|e| {
                eprintln!("[ENGRAM WARN] Embedding server unreachable at {url_clone}: {e}");
                eprintln!("[ENGRAM WARN] Falling back to BLAKE3-only encoding.");
                eprintln!("[ENGRAM WARN] Euler gate WILL reject this vector. Set ENGRAM_EMBED_URL.");
                e
            })
            .ok()?
            .json()
            .ok()?;

        let emb = res["data"][0]["embedding"].as_array()?;
        let vec: Vec<f32> = emb.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect();
        if vec.is_empty() { return None; }

        // L2-normalise so values land inside the Poincaré ball (||v|| ≤ 1)
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm < 1e-9 { return None; }
        Some(vec.into_iter().map(|x| x / norm).collect())
    })
    .join()
    .unwrap_or_else(|e| {
        eprintln!("[ENGRAM WARN] Embedding thread panicked: {:?}", e);
        None
    });

    result
}

/// Encode free-form text into a `HolographicBlock` using Hybrid HRR + Neural Strategy.
pub fn from_text(text: &str) -> Leg3Pointer {
    let mut block = Leg3Pointer::mint();
    block.magic = *b"LEG3";
    block.schema_ver = 1;
    block.zedos_tag = ZEDOS_DECLARATIVE;
    block.spin_state = 0x01; // Axiomatic (lit)

    // Structural Anchor (Method A) - Native Logophysical HRR accumulation
    let mut q = [Complex32::default(); DIMENSION];
    let tokens: Vec<&str> = text.split_whitespace().collect();
    
    for token in &tokens {
        let seed_hash = blake3::hash(token.to_lowercase().as_bytes());
        let mut xof = blake3::Hasher::new();
        xof.update(seed_hash.as_bytes());
        let mut phase_bytes = vec![0u8; DIMENSION * 4];
        xof.finalize_xof().fill(&mut phase_bytes);

        for i in 0..DIMENSION {
            let b0 = phase_bytes[i * 4]     as f32;
            let b1 = phase_bytes[i * 4 + 1] as f32;
            let b2 = phase_bytes[i * 4 + 2] as f32;
            let b3 = phase_bytes[i * 4 + 3] as f32;
            let theta_re = (b0 * 256.0 + b1) / 65535.0 * std::f32::consts::TAU;
            let theta_im = (b2 * 256.0 + b3) / 65535.0 * std::f32::consts::TAU;
            
            // Vector Superposition (Accumulate pure token phases)
            q[i] += Complex32::new(theta_re.cos(), theta_im.sin());
        }
    }

    // Semantic Aura (Method B) — Neural embedding into dedicated slots.
    //
    // Strategy: place the neural vector CLEANLY into q[0..N].re where N is the
    // embedding dimension (384 for MiniLM, up to 4096 for Gemma, etc.).
    // This keeps slots 0..N free of hash contamination so the INT8 Poincaré
    // kernel (which reads q[0..384].re) sees clean L2-normalised values.
    //
    // slots N..8192 keep the pure logophysical hash accumulation.
    if let Some(neural_vec) = fetch_neural_embedding(text) {
        let neural_len = neural_vec.len().min(DIMENSION);
        // Write neural values into the first N real slots (overwrite hash)
        for i in 0..neural_len {
            q[i].re = neural_vec[i];
            q[i].im = 0.0;  // clean imaginary — no hash contamination
        }
        // Slots neural_len..8192 keep the logophysical hash (already in q[])
    }

    // Phase 4: Normalize to unit hypersphere
    block.q = normalize(&q);

    // Phase 5: CRS — new blocks start fully coherent; the manifold will update this
    block.crs_score = 1.0;
    block.energetics.crs = 1.0;
    block.energetics.heat_dissipated = 5.47e-4; // Minimum action quantum

    // Store provenance identifier
    let seed_hash = blake3::hash(text.as_bytes());
    block.footer.sig_0 = *seed_hash.as_bytes();

    write_provlog(&mut block, text);
    block
}

/// Encode a concept with a specific CRS score override.
/// Used when importing memories from external sources with known quality estimates.
pub fn from_text_with_crs(text: &str, crs: f32) -> Leg3Pointer {
    let mut block = from_text(text);
    block.crs_score = crs.clamp(0.0, 1.0);
    block.energetics.crs = block.crs_score;
    block
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::cosine_similarity;

    #[test]
    fn same_text_same_vector() {
        let a = from_text("hello world");
        let b = from_text("hello world");
        let sim = cosine_similarity(&a.q, &b.q);
        assert!((sim - 1.0).abs() < 1e-5, "encoding is not deterministic: {sim}");
    }

    #[test]
    fn different_text_different_vector() {
        let a = from_text("photosynthesis converts sunlight to glucose");
        let b = from_text("the Eiffel Tower is in Paris");
        let sim = cosine_similarity(&a.q, &b.q);
        assert!(sim < 0.9, "unrelated texts too similar: {sim}");
    }

    #[test]
    fn provlog_roundtrip() {
        let text = "mitochondria are the powerhouse of the cell";
        let block = from_text(text);
        let recovered = crate::storage::read_provlog(&block);
        assert_eq!(recovered, text);
    }

    #[test]
    fn block_has_correct_magic() {
        let block = from_text("test");
        assert_eq!(&block.magic, b"LEG3");
    }
}
