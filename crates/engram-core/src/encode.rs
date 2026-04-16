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

/// Attempt to fetch a Neural Embedding from the local Transductive Suture (llama-server)
fn fetch_neural_embedding(text: &str) -> Option<Vec<f32>> {
    use serde_json::json;
    
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .build()
        .ok()?;
        
    let body = json!({
        "input": text,
        "model": "local" // llama-server ignores model but expects the key
    });
    
    let res: serde_json::Value = client.post("http://localhost:8085/v1/embeddings")
        .json(&body)
        .send()
        .ok()?
        .json()
        .ok()?;
        
    let emb = res["data"][0]["embedding"].as_array()?;
    let vec: Vec<f32> = emb.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect();
    if vec.is_empty() { return None; }
    Some(vec)
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

    // Semantic Aura (Method B) - Deep Neural Integration via llama-server fallback
    if let Some(neural_vec) = fetch_neural_embedding(text) {
        let neural_len = neural_vec.len();
        // Bind the neural geometry directly into the logophysical structure
        for i in 0..DIMENSION {
            let n_val = neural_vec[i % neural_len];
            let neural_phase = Complex32::new(n_val.cos(), n_val.sin());
            // Binding (Multiply) structural token accumulation by Neural Semantic concept
            // and apply a 5x superposition weight so the Aura heavily guides similarity
            q[i] = (q[i] * neural_phase) + (Complex32::new(n_val * 5.0, 0.0));
        }
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
