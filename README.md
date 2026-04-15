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

## 🗂️ Sheaf Mode — Multi-Project Memory

By default Engram uses a single manifold directory. For developers working across multiple projects (or agents managing multiple codebases simultaneously), **Sheaf Mode** lets you run isolated memory stalks that can still be queried as a unified whole.

In mathematics, a Sheaf is a structure that assigns local data to each region of a space while allowing those regions to be "glued" into a global view. Engram's `.leg` blocks already encode this topology in their `footer.merkle_sub_root` field — the Sheaf routing layer simply exposes it.

### Setup

Create `~/.engram/sheaf.toml`:

```toml
active_stalk = "my-project"   # new memories write here

[[stalks]]
name = "my-project"
path = "~/.engram/stalks/my-project"

[[stalks]]
name = "reference-corpus"
path = "~/.engram/stalks/reference-corpus"
```

Restart `engram mcp`. All recall queries automatically fan out across **all stalks in parallel** and return a merged, re-ranked result set. Each result is prefixed with its stalk name (`my-project::concept_name`) so agents always know the provenance of a memory.

### MCP Tools (Sheaf)

| Tool | Description |
|---|---|
| `mcp_engram_list_stalks` | List all registered stalks and show which is active |
| `mcp_engram_set_active_stalk` | Switch write target to a different stalk at runtime |

> [!TIP]
> Keep your active codebase in one stalk and your persistent reference knowledge (tokenizers, ontologies, prior corpus) in another. The agent always searches both but writes only to the active one — no cross-contamination.

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

## 🚦 Four Deployment Modes

Engram is **model-agnostic** and **IDE-agnostic**. The same geometric manifold powers every configuration. Pick the mode that fits your setup:

| Mode | Setup | Internet? | Use Case |
|---|---|---|---|
| **1 — IDE + Cloud** | IDE + Gemini/Claude + Engram MCP | Required | Standard developer workflow |
| **2 — Hybrid** | IDE + cloud agent + local LLM sharing one manifold | Required for cloud agent | Multi-agent collaboration |
| **3 — IDE + Local, Offline** | IDE + local LLM (Gemma 4 / any GGUF) + Engram MCP | **None** | Enterprise, medical, air-gapped |
| **4 — Headless Standalone** | `nemo-agency` Rust orchestrator + local LLM + Engram REST | **None** | Autonomous agents, CI/CD, scripted |

> **Any OpenAI-compatible local LLM works in Modes 3 and 4** — Gemma 4, Llama 3, Mistral, Qwen, Phi-4, DeepSeek. Swap the `.gguf` file. No code changes.

📖 **[Full deployment guide → `docs/DEPLOYMENT_MODES.md`](docs/DEPLOYMENT_MODES.md)**

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
| CPU (Rayon) | Default | ✅ Stable | Exact linear scan, works on any machine |
| CUDA (NVIDIA) | `cuda-kernels` | ✅ Stable | BVH O(log N) index + GPU cosine kernels. Runtime probe — no compile-time CUDA required |
| ROCm (AMD) | `rocm-kernels` | 🔧 Beta | HIP kernel (`arkade_8k.hip`) is complete. Runtime probe via `libamdhip64.so`. CPU BVH fallback active. GPU dispatch wired in Phase 10 |
| Metal (Apple) | `metal` | 🔧 Beta | MSL kernel (`arkade_8k.metal`) compiled at startup via `metal-rs`. CPU fallback active. GPU dispatch wired in Phase 10 |

---

## License & Patent

This software is licensed under **AGPL-3.0-only**.

The `.LEG` container format is covered by **U.S. Patent Application No. 19/372,256** (pending),  
*Self-Contained Variable File System (.LEG Container Format)*,  
Applicant: **Aric Goodman**, Oregon, USA — Static Rooster Media.

This is a **reference implementation**. Commercial licenses (including for SaaS/cloud use) are available.  
Contact: **StaticRoosterMedia@gmail.com**

See [PATENT-NOTICE.md](PATENT-NOTICE.md) for full details.
