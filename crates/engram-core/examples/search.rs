use engram_core::{CpuBackend, VsaBackend};

fn main() {
    let q = "quasi-orthogonal causal logophysical bindings in OODA loop";
    let backend = CpuBackend::new("/home/a/Documents/CodeLand/data/holograms/static/genesis");
    let res = backend.recall(q, 3);
    
    println!("=== SEMANTIC RAY-CASTER RESULTS ===");
    for r in res {
        let snippet = if r.provlog.len() > 150 { &r.provlog[..150] } else { &r.provlog };
        println!("[{}] (resonance={:.3}, crs={:.3}) -> {}...", r.concept, r.score, r.crs, snippet);
    }
}
