fn main() {
    let q = "OODA loop logophysics quasi-orthogonal causal bindings";
    let backend = engram_core::CpuBackend::new("/home/a/Documents/CodeLand/data/holograms/static/genesis");
    let res = engram_core::VsaBackend::recall(&backend, q, 5);
    println!("=== SEMANTIC RAY-CASTER RESULTS ===");
    for r in res {
        println!("[{}] (crs={:.3}) -> {}", r.concept, r.score, &r.text.replace("\n", " ")[..std::cmp::min(100, r.text.len())]);
    }
}
