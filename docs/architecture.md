# Engram Architecture

Engram is a hardware-native geometric memory engine designed for AI agents. It bypasses traditional database paradigms in favor of raw spatial tensors, physical NVMe alignment, and logophysical mathematics.

## The `.LEG` Container Unveiled

The core of Engram is the `HolographicBlock`—stored as a `.leg` file. It is a strictly structured 262,144 byte (256KB) container, aligned physically to 4096-byte page boundaries.

### Why exactly 256KB? 
This size is non-arbitrary. 256KB is exactly 64× 4KB memory pages. By aligning the container directly to the physical SSD page boundaries and using a fixed C-struct layout, Engram completely eliminates parsing overhead (like JSON or Protobuf decoding).
- **Native Tensor Load:** Because the layout is known at compile time, the GPU or CPU treats the `.leg` file as a raw memory buffer. 
- **O_DIRECT and DMA:** The operating system's page cache is bypassed entirely. Tensors are streamed via Direct Memory Access (DMA) from the NVMe drive straight into VRAM or CPU registers. 
- **Zero Serialization:** A read operation is a `mmap` call, not a deserialization loop.

### Internal Layout
1. **`q[8192]` (Knowledge Tensor):** The primary semantic signature (Complex32).
   - **Slots 0..768:** L2-normalized neural embeddings (e.g., from an LLM).
   - **Slots 768..8192:** High-dimensional FHRR vectors created by internal mathematical transforms.
2. **`p[8192]` (Temporal Momentum Tensor):** Tracks the velocity and drift of concepts as they evolve over time.
3. **Payload / Source Code:** The raw markdown or source code payload (e.g., AST-extracted functions).
4. **ZEDOS Tags:** A single byte that enforces the epistemic class of the memory (`DECLARATIVE`, `EPISODIC`, `PRAXIS`, `RELATION`).
5. **LegFooter (Merkle Chain):** A 6-part BLAKE3 hash chain that mathematically proves the provenance of the memory.

---

## The Geometry Engine (VSA)

Engram does not use naive cosine similarity for everything. It employs **Vector Symbolic Architecture (VSA)**, specifically **Fourier Holographic Reduced Representations (FHRR)** on the Bloch hypersphere. This allows algebraic manipulation of ideas:

- **`op_add(A, B)`**: Superposition. Computes the geometric centroid, combining two concepts into a unified representation.
- **`op_bind(A, B)`**: Circular convolution. Computes the Hadamard product in the frequency domain, creating a new vector that is quasi-orthogonal to both parents. This is the foundation of Engram's Knowledge Graph.
- **`op_deduce(A, B)`**: Tracks the rotational matrix $B \cdot conj(A)$ to map logical implication.
- **`op_suspend(A)`**: Binds the concept with the maximum-entropy *Apeiron* vector to explicitly flag it as a "known unknown."

---

## Hallucination Protection: The Volatility Tracker (Lyapunov Stability)

A major problem with autonomous agents is that they hallucinate or get stuck in debugging loops, spamming their database with garbage states. Engram solves this at the storage layer by treating memory updates as a dynamical system, tracking coherence using a Dirichlet belief simplex:

$$ \Phi(v) = w_a \cdot p_a^2 + w_d \cdot p_d^2 + w_r \cdot p_r^2 $$

- **$p_a$ (Affirmation):** The update was geometrically close to the previous state (reinforcement).
- **$p_d$ (Denial):** The update was radically different (surprising / rollback).
- **$p_r$ (Reconciliation):** The update momentum is slowing down (converging).

When a memory is updated, Engram calculates the gradient of this energy function to find the *drift velocity* ($dv$). 
- If $dv < 0.05$, the concept is stable and converging on a canonical truth. The block gains trust.
- If $dv$ is high, the concept is volatile (likely an LLM hallucination loop). The system geometrically penalizes the block's **Coherence-Reliability Score (CRS)**. This mathematical rate-limit prevents garbage from polluting long-term memory.

---

## O(log N) LBVH Indexing & TurboQuant

For large manifolds (50K+ memories), linear scanning becomes suboptimal. 

1. **Gaussian Compressed Sensing Random Projection (CSRP):** Each 8192-D vector is projected into 3D physical space.
2. **Linear Bounding Volume Hierarchy (LBVH):** These 3D coordinates are structured into an LBVH tree. Tree traversal finds candidates in $O(\log N)$ time.
3. **TurboQuant:** To filter the candidates in-memory, Engram applies an SRHT rotation and B4 Lloyd-Max quantization, packing each complex vector into just 8192 bytes.
4. **O_DIRECT Reads:** Only the final top-K candidates trigger actual physical SSD reads.

---

## The ZEDOS Knowledge Graph

Every relationship drawn between two blocks (via `mcp_engram_relate`) creates a `ZEDOS_RELATION` memory.
Because relationships are stored natively using `op_bind` mathematics, you do not need an external graph database. The edges *are* memory blocks, fully searchable, accountable, and visualized dynamically using Breadth-First Search (BFS).
