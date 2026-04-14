# Engram

> **Persistent geometric memory for AI agents.**  
> Patent Pending US19/372,256 — Aric Goodman & Static Rooster Media

Engram gives your AI agent a long-term memory that works directly like human associative memory—store anything, retrieve by meaning, not keywords. No vector database. No cloud. No API key. Runs entirely on your machine via the Model Context Protocol (MCP).

---

## 🚀 The AI Native Agentic Daemon

Engram is not just a passive database. When you map it into an LLM IDE (like Antigravity or Cursor), it boots an **Agentic Daemon** that handles geometric awareness completely in the background:

1. **Native OS Watcher:** By calling the `mcp_engram_watch_workspace` JSON tool, the Daemon binds directly to OS-level kernel events (`inotify`/`fsevents`). The moment you or your Agent saves *any* major code file, Engram immediately re-ingests the vector chunk to the manifold. Your contextual window never decays into staleness.
2. **Autophagy Garbage Collection:** Every hour, a deep-asynchronous Tokio thread wakes up and scans your memory manifold. Irrelevant ephemeral states automatically log-decay. If a geometric block drops below `0.40 CRS` (Coherence-Reliability Score), it is permanently forgotten by the system to save compute space.
3. **Project Matrix Pinning:** Using the `mcp_engram_pin` tool, a connecting LLM can permanently bind critical Task Boards or Architectural Decisions to a perfect `1.0 CRS`. The Autophagy Daemon is hardcoded to never decay pinned vectors.

---

## ⚡ The Semantic Ray-Tracer (VSA Arithmetic)

Because Engram uses **Vector Symbolic Architectures** inside the core math engine, you and your agents are capable of performing complex geometrical operations *before* extracting memory.

Instead of hunting for keywords, you can mathematically bind, superpose, and intersect concepts across the hyperdimensional plane using the **Trace** commands:

```bash
# Locate the precise `.rs` chunks where native "CUDA" overlaps contextually with "Memory"
engram-cli trace "cuda" BIND "memory"

# Find code blocks that share equivalence between two combined domains
engram-cli trace "geometric" ADD "tensor"
```
These logophysical math routes are available locally over CLI, or explicitly exposed to AI integrations over the robust `/api/trace` Axum REST route!

---

## 💾 Engineering Limitations: NVMe `O_DIRECT` Block Rules

> [!WARNING]  
> If you are modifying the `engram-core` serialization pipelines, **strictly adhere to the 256KB block rules**.

Every piece of knowledge is stamped into fundamentally isolated **HolographicBlocks** (`.leg` files). We designed these logical containers to be exactly **262,144 bytes (256KB)** in memory space. 

**This is not an arbitrary limitation.**
The exact 256KB payload was specifically optimized for **O_DIRECT physical SSD alignments**. This ensures that geometrical tensors can completely bypass standard Operating System CPU buffer-caches, and be Direct Memory Access (DMA) streamed off of M.2 NVMe drives directly into GPU VRAM for CUDA or ROCm kernel processing. 
If you alter the struct geometry in `types.rs` so that it no longer aligns evenly to modern block bounds, you will break the underlying DMA speeds!

---

## ⚡ Quick Start integration

> **Integration configs** for all supported IDEs are openly tracked in [`integrations/`](integrations/)

### Install

```bash
cargo install engram --git https://github.com/StaticRoosterMedia/engram
```

### Google Antigravity IDE

Engram ships a ready-to-use config for Google Antigravity IDE. Add to `~/.gemini/antigravity/mcp_config.json`:
```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "~/.engram/manifold"],
      "disabled": false
    }
  }
}
```

### Claude Desktop Integration

Add to `~/.config/claude-desktop/claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "~/.engram/manifold"]
    }
  }
}
```
Restart Claude Desktop. You immediately tap into the native tools: `remember`, `recall`, `forget`, `list_concepts`, `mcp_engram_watch_workspace`, `mcp_engram_pin`.

---

## 💻 CLI Operations & Manifold Bootstrapping

If you're bringing Engram into a legacy repository, you do not need to manually parse your code logs. Just utilize the native recursive chunking engine:

```bash
# Bootstrap an entire monolithic repository into geometric logic inside of seconds:
engram-cli ingest /path/to/monolithic-workspace --chunk-size 8000

# Store single ad-hoc memories:
engram-cli remember krebs_cycle "The Krebs cycle converts acetyl-CoA to ATP via 8 enzymatic steps"

# Interrogate memory conceptually:
engram-cli recall "how does cellular respiration produce energy"
# Prints exactly the Krebs Cycle vector, and exact code lines tied to that phase domain.
```

---

## Hardware Support

Engram maps identically across the top hardware architectures natively:

| Backend | Flag | Status | Notes |
|---------|------|--------|-------|
| CPU (Rayon) | Default | ✅ v1.0 | Linear scan natively, works on any machine |
| CUDA (NVIDIA) | `cuda-kernels` | ✅ v2.0 | BVH O(log N) index, NVMe parallel kernel computation |
| ROCm (AMD) | `rocm-kernels` | ✅ v2.0 | Wavefront HIP execution |
| Metal (Apple) | `metal` (Auto) | ✅ v1.0 | macOS MSL dynamic runtime compilation via metal-rs |

---

## License & Patent

This software is licensed under **AGPL-3.0-only**.

The `.LEG` container format is covered by **U.S. Patent Application No. 19/372,256** (pending),  
*Self-Contained Variable File System (.LEG Container Format)*,  
Applicant: **Aric Goodman**, Oregon, USA — Static Rooster Media.

This is a **reference implementation**. Commercial licenses (including for SaaS/cloud use) are available.  
Contact: **StaticRoosterMedia@gmail.com**

See [PATENT-NOTICE.md](PATENT-NOTICE.md) for full details.
