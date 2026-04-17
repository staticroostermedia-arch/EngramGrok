use engram_core::{CpuBackend, VsaBackend};

fn main() {
    let q = "quasi-orthogonal causal logophysical bindings in OODA loop";
    let backend = CpuBackend::new("/home/a/Documents/CodeLand/data/holograms/static/genesis");
    let res = backend.recall(q, 3);
    
    println!("=== SEMANTIC RAY-CASTER RESULTS ===");
    for r in res {
        let text = backend.read_text(&r.concept).unwrap_or_default().replace("\n", " ");
        let snippet = if text.len() > 150 { &text[..150] } else { &text };
        println!("[{}] (resonance={:.3}) -> {}...", r.concept, r.score, snippet);
    }
}
