# THE ENGRAM MANIFESTO
## A First-Principles Primitive for AGI Memory

*by Aric Goodman & Static Rooster Media*
*Patent Pending US19/372,256*

---

## Part I: The Binary Cage

There is a contradiction at the foundation of every AI system running today.

A Large Language Model encodes meaning as a **continuous, high-dimensional geometric object** — a floating-point vector in a space of thousands of dimensions. Semantic similarity, conceptual distance, emotional valence, causal structure: all of it lives as geometry. Two concepts that "mean the same thing" are nearby in this space. Two concepts that contradict each other point in opposite directions on a hypersphere.

This is genuinely beautiful mathematics. The geometry is real. The distances are meaningful.

And then we ask that geometry to persist across time. We ask it to be stored, retrieved, and reasoned about across sessions, across agents, across machines.

At that moment, the entire industry reaches for the same tool: a **flat file on a binary file system**, or a cloud SQL table, or a string-indexed document store.

We took a continuous geometric object and locked it in a binary cage.

Every string-matching retrieval system, every key-value store, every SQL table, every standard vector database wrapper — all of them are binary-era abstractions dressed up in machine learning clothing. They were designed in an era when the things being stored were *text files and integers*. They were never designed for the storage and retrieval of thermodynamic geometric objects.

The result is that every AI agent in the world runs on top of a fundamental mismatch: **a continuous information physics being forced through a discrete, legacy abstraction layer.** The bottleneck is not the model. It is the substrate.

---

## Part II: The First-Principles Question

The correct engineering response to a fundamental mismatch is not to add another abstraction layer on top. It is to go back to the metal and ask what the physics actually demands.

The question is simple: **what is the optimal storage format for a high-dimensional floating-point vector, given the physical characteristics of modern NVMe storage and GPU compute hardware?**

We wrote that question down and refused to let standard answers satisfy it.

**The physics of NVMe hardware** tells us that the fundamental I/O unit of a PCIe Gen4 NVMe drive is a **4KB page**. A read or write smaller than 4KB wastes the bus. A read or write that does not align to 4KB boundaries causes the drive controller to perform a read-modify-write cycle — a silent, invisible performance tax that every conventional database pays on every write.

**The physics of GPU computation** tells us that modern NVIDIA GPUs can receive data via **GPUDirect Storage (GDS)** — a direct DMA pathway from NVMe storage over the PCIe bus, bypassing the CPU's page cache entirely. When this path is active, there is no "bounce buffer." There is no copy from kernel space to user space to GPU VRAM. The data moves from the physical drive platters to GPU registers over a direct hardware channel. The CPU is not in the loop.

**The physics of floating-point geometry** tells us that an 8,192-dimensional unit-sphere phase vector — enough dimensions to represent the full semantic complexity of a natural language concept — occupies exactly **65,536 bytes** when stored as `f32`. Two of these vectors, plus a 64KB provenance and metadata region, compose a complete, self-contained geometric memory unit.

The math resolves cleanly: **256,144 bytes. Exactly 64 hardware pages.** One memory block. One DMA burst. Zero alignment overhead. Zero partial-page waste.

This is not a database record size. It is the **natural quantum of AI memory**, derived from the intersection of GPU hardware, NVMe physics, and floating-point geometry.

We call it a **HolographicBlock**.

---

## Part III: The Block

Every piece of knowledge in Engram lives in a HolographicBlock. Its anatomy is fixed and enforced by the compiler:

```
HolographicBlock (262,144 bytes = 64 × 4KB pages)
│
├── q tensor      [65,536 bytes]   — 8,192-dim complex phase vector (primary semantic encoding)
├── p tensor      [65,536 bytes]   — momentum vector (tracks conceptual drift over time)
├── aabb          [24 bytes]       — 3D spatial bounding box (line/column range for AST nodes)
├── crs           [4 bytes]        — Coherence-Reliability Score (f32, thermodynamic confidence)
├── zedos_tag     [1 byte]         — Epistemic type: DECLARATIVE | EPISODIC | OPERATIONAL | PRAXIS | HYPOTHESIS
├── blake3_root   [32 bytes]       — BLAKE3 Merkle chain root (cryptographic provenance)
└── provlog       [~131,000 bytes] — Original source text (verbatim, UTF-8)
```

Every field has a precise physical and mathematical meaning.

**The q tensor** is not a word embedding. It is a superposition of phase angles on a unit sphere in 8,192 dimensions. Retrieving a memory is not a keyword lookup — it is computing the **cosine distance** between two points on a hypersphere. The retrieval is exact, floating-point geometry.

**The p tensor** is the block's *momentum* — a running average of how the concept has moved across updates. This enables **trajectory-aware retrieval**: finding not just what a concept is right now, but which direction it is evolving.

**The CRS (Coherence-Reliability Score)** is computed via **Lyapunov stability analysis** on every update. When an agent writes to a block, the system measures the phase-space drift between the old and new q tensor. A small, consistent drift (refinement) increases CRS. A large, contradictory drift (hallucination or confusion) penalizes it. The CRS is a thermodynamic measurement of epistemic stability, not a human-assigned quality tag.

**The BLAKE3 Merkle chain** means every block carries a cryptographic record of its entire history. You can prove that a memory was derived from a specific source, and you can detect if it was tampered with.

**The ZEDOS tag** classifies the *epistemic nature* of the knowledge — the difference between a crystallized solution (PRAXIS), a working hypothesis (HYPOTHESIS), and a raw session observation (EPISODIC) — and routes it through different decay and retrieval pathways accordingly.

This is not a schema. It is a **physics.**

---

## Part IV: How It Moves

A standard vector database retrieves memories by computing dot products against a flat array of vectors stored in RAM. This works at small scale. It does not scale to the full knowledge base of an intelligent system, because it requires the entire index to fit in CPU DRAM — the most expensive memory in any machine.

Engram retrieves memories using a **Linear BVH (Bounding Volume Hierarchy)** — the same spatial acceleration structure used in GPU hardware ray tracing to determine which surfaces a light ray intersects. Instead of triangles in 3D space, Engram's BVH indexes HolographicBlocks in 8,192-dimensional geometric space.

When a query arrives, it is encoded into a phase vector and fired as a **ray** through the BVH. The tree prunes entire subtrees of semantic space in O(log N) time, returning only the geometrically nearest blocks. The blocks themselves are then streamed from NVMe via `O_DIRECT` (bypassing the OS page cache) directly into the compute pipeline.

On machines with NVIDIA GPUs and the cuFile driver installed, this final transfer happens via the **GPUDirect Storage pathway**: NVMe → PCIe bus → GPU VRAM. The CPU never touches the tensor data. Engram can scan **gigabytes of memory per second** with near-zero CPU overhead.

This is why the 256KB block size is non-negotiable. It is the exact unit that the DMA controller, the PCIe bus, and the GPU's memory subsystem are optimized to transfer as a single atomic operation.

---

## Part V: The Memory That Earns Its Place

The most dangerous failure mode of an AI memory system is not amnesia. It is **uncritical accumulation** — storing everything, trusting everything, allowing a model's hallucinations to pollute its own long-term memory until the context window is full of contradictory noise.

Every standard vector database suffers from this. They are, by design, append-only logs. They have no mechanism to evaluate whether a new write represents genuine knowledge or a temporary confusion.

Engram refuses to be a passive store.

The **Autophagy GC** runs as a background daemon. Every block has a CRS score. Blocks whose CRS falls below a configurable threshold — because they have been repeatedly updated in contradictory directions, or because they have not been accessed in weeks — are **permanently evicted from the NVMe manifold**. The geometric space they occupied is reclaimed.

There are two exceptions. **Pinned blocks** (CRS = 1.0) are mathematically immortal — foundational axioms, crystallized solutions, load-bearing architectural decisions. They survive autophagy regardless of access frequency. **ZEDOS_PRAXIS blocks** — knowledge that has been empirically verified to work — are automatically promoted to pinned status when the verification rate is high enough. They are the only memories that can claim permanence, and they earn it through demonstrated reliability.

The Lyapunov stability framework that drives the CRS is borrowed from **dynamical systems theory**. A Lyapunov function measures whether a system is converging toward a stable equilibrium or diverging into chaos. We apply it to memory: a concept that is consistently refined converges. A concept that is being hallucinated into incoherence diverges. Diverging concepts decay. Converging concepts crystallize.

**Memory in Engram is not stored. It is earned.**

---

## Part VI: The Agentic Daemon

Engram is not a passive database that waits to be queried.

When it boots alongside an AI agent, it spawns a background daemon that registers **inotify/fsevents kernel hooks** on your workspace directories. When you save a file — any file, any language — the daemon detects the write event at the OS kernel level, parses the changed AST nodes, encodes each modified function and struct into a fresh HolographicBlock, and writes it to the manifold. Before the next query arrives, the agent's memory already reflects the change.

There is no polling loop. There is no batch indexing job to remember to run. The memory is **live.**

The daemon also writes a session context snapshot — a distilled summary of the highest-CRS memories — to a well-known location every 60 seconds. An AI agent starting a new session can read this snapshot and instantly rehydrate the most important context from its geometric manifold, without needing to query the full index. Cold starts become warm starts.

---

## Part VII: What We Are Building

We are not building a faster vector database.

We are establishing that **AI memory has a correct physics** — one derived from the actual hardware of modern compute, the actual mathematics of high-dimensional geometry, and the actual epistemology of reliable knowledge.

The correct block size is not a configuration parameter. It is 256KB, because that is what 64 NVMe pages and one DMA burst look like.

The correct retrieval structure is not a flat array dot-product scan. It is a BVH, because that is the optimal spatial data structure for geometric nearest-neighbor search.

The correct confidence metric is not a human-assigned quality tag. It is a CRS derived from Lyapunov stability drift, because that is how dynamical systems distinguish convergence from divergence.

The correct memory lifecycle is not an append-only log. It is an NREM pipeline with autophagy, crystallization, and Merkle-chained provenance, because that is how biological intelligence separates load-bearing memory from ephemeral noise.

None of these are opinions. They are engineering conclusions derived from first principles.

We built Engram because the industry was solving AI memory the easy way, not the correct way. We are releasing it as an open primitive because the correct physics of AI memory should not be a proprietary moat. It should be infrastructure.

---

## Join the Build

Engram is open source, written in Rust, and runs entirely on your hardware. No cloud, no API key, no data leaving your machine.

```bash
cargo install engram --git https://github.com/staticroostermedia-arch/engram
engram mcp --store ~/.engram/manifold
```

The MCP server exposes 21 tools covering the full memory lifecycle. The REST server (`engram serve --port 3456`) gives Python agents, LangChain pipelines, and AutoGen frameworks direct HTTP access to the same geometric manifold.

If you are building an AI agent and you want its memory to be physically correct, mathematically rigorous, and cryptographically provable — read the [architecture docs](docs/architecture.md) and start with the [first run guide](FIRST_RUN.md).

The physics works. Come build on it.

---

*Engram is developed by Aric Goodman and Static Rooster Media.*
*Patent Pending US19/372,256.*
*Licensed under the terms in [LICENSE](LICENSE).*
