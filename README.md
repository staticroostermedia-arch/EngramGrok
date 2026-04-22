# Engram

[![Build Status](https://github.com/staticroostermedia-arch/engram/actions/workflows/rust.yml/badge.svg)](https://github.com/staticroostermedia-arch/engram/actions)
[![MCP](https://img.shields.io/badge/MCP-Native-blue)](https://github.com/modelcontextprotocol)
[![Glama](https://glama.ai/mcp/servers/staticroostermedia-arch/engram/badge)](https://glama.ai/mcp/servers/staticroostermedia-arch/engram)

> **Hardware-native geometric memory for AI agents — 21 MCP tools.**  

Engram is not a vector database. It is a persistent geometric memory engine designed for AI agents. It bypasses conventional database software layers by storing information in fixed, mathematically rigorous 256KB tensors directly on NVMe drives. No cloud, no API keys, no deserialization overhead. Runs entirely on your machine via the Model Context Protocol (MCP).

---

## 🚀 Quick Start

By default, Engram uses internal geometric hashing. To enable massive-scale semantic code search, point it to any OpenAI-compatible embeddings endpoint (e.g., local llama.cpp or ONNX):

```bash
export ENGRAM_EMBED_URL="http://localhost:8080/v1/embeddings"
cargo install engram --git https://github.com/staticroostermedia-arch/engram
```

Add to your MCP config and restart your IDE:

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

Your agent immediately has access to all 21 tools. See [`integrations/`](integrations/) for IDE-specific configs.

> 📖 **New here?** Read the **[First Run Guide](FIRST_RUN.md)** — it walks you through verifying every feature works, activating the file watcher daemon, and seeding your manifold with your codebase.

---

## ⚡ Why 256KB? The Hardware-Native Advantage

Engram maps your project's memory into strict 262,144 byte (256KB) containers called **HolographicBlocks**. This size is non-arbitrary.

- **Native Tensor Load:** 256KB perfectly aligns to 64× 4KB hardware pages. Because the `.leg` format is a strict C-struct, it requires zero JSON decoding or Protobuf parsing.
- **O_DIRECT and DMA:** Engram bypasses the operating system's page-cache. When your agent searches for a memory, the tensor streams via Direct Memory Access (DMA) from the physical NVMe SSD straight into CPU registers or GPU VRAM.

Every block mathematically fuses the full original source code, 8192-dimensional semantic tensors, spatial 3D bounds (for code placement), and cryptographic BLAKE3 Merkle chain proofs.

*(See [docs/architecture.md](docs/architecture.md) for a deep dive into the container format and LBVH scaling).*

---

## 🛡️ Hallucination & Loop Protection (Volatility Tracker)

Traditional vector databases are "dumb" append-logs: if an LLM hallucinates or gets stuck in a debugging loop, it will spam the database with hundreds of slightly-different, broken code snippets, destroying the context window.

Engram does not blindly accept every update as equally valid. It features a built-in mathematical volatility tracker (using Lyapunov stability equations) that monitors how much a concept "shifts" between updates:
- **Low Drift:** If an agent updates a memory and the semantic meaning barely changes, the system recognizes it as stable/converging. The block's **Coherence-Reliability Score (CRS)** goes up.
- **High Drift:** If an agent rapidly overwrites a memory with wildly different concepts (a hallucination loop), the system recognizes the volatility. The block's CRS is penalized.

Memories must mathematically prove their stability over time. If a block's CRS drops too low, agents know not to trust it.

---

## 🖥️ CLI Commands

Beyond the MCP server, Engram ships a standalone CLI for direct manifold management:

| Command | Description |
|---|---|
| `engram remember <concept> <text>` | Encode and store a memory |
| `engram recall <query>` | Semantic search, returns top-k |
| `engram forget <concept>` | Delete a memory |
| `engram list` | List all stored concept names |
| `engram ingest <path>` | Recursively ingest a directory of text/code files |
| `engram trace <A> <OP> <B>` | VSA geometry: query the result of ADD or BIND on two concepts |
| `engram distill` | **Crystallize** — cluster episodic memories into durable ZEDOS_PRAXIS blocks |

---

## 🌳 AST-Aware Semantic Distillation

Traditional RAG chunks text arbitrarily, destroying function boundaries and context. Engram's ingest pipeline uses a universal **AST-extraction layer** powered natively by **Tree-Sitter**.

It natively parses **Rust, Python, TypeScript, JavaScript, Go, Java, C, and C++**.
Engram mints exactly **one memory block per public semantic item** (functions, structs, classes).

- **The Tensor (`q`):** Encodes the doc comment and signature.
- **The Provlog:** Carries the raw, full-length source code.
- **Spatial Embodiment:** Maps the precise 2D row/column coordinates of the AST node directly into the memory block's physical bounds, allowing the agent to know *where* code lives, not just *what* it does.

---

## 🧰 MCP Tools Reference

Engram exposes **21 tools** across 5 capability groups.

### Core Memory

| Tool | Description |
|---|---|
| `remember` | Encode text and store as a persistent memory block |
| `recall` | Semantic similarity search — returns top-k memories. Optional `time_decay` param for time-targeted search |
| `forget` | Delete a specific memory by concept name |
| `list_concepts` | List all stored concept names |
| `mcp_engram_update` | Re-encode an existing memory in place (uses `op_add` superposition) |
| `mcp_engram_pin` | Lock a memory at CRS=1.0 — protects foundational constraints forever. |

### Memory Intelligence

| Tool | Description |
|---|---|
| `mcp_engram_stats` | Manifold health report: total count, pinned, avg/min/max CRS, disk usage |
| `mcp_engram_recall_recent` | Return N most recently accessed memories, sorted by access time |
| `mcp_engram_summarize` | Project-state digest: pinned memories + top-N by CRS. Single-call `/wake_up` replacement |
| `mcp_engram_forget_old` | On-demand autophagy: manually sweep out low-CRS blocks |

### Workspace & Agentic

| Tool | Description |
|---|---|
| `mcp_engram_watch_workspace` | Tell the daemon to watch a directory; automatically extracts and re-ingests file-saves through the Tree-Sitter AST pipeline |
| `mcp_engram_context_for_file` | Surface top-5 relevant memories for a file path (proactive loading) |
| `mcp_engram_session_end` | Commit session context and natively compute ADR Thermodynamics based on memory coherence |
| `mcp_engram_scar` | Create a geometric repeller using the maximum-entropy Apeiron primitive to mark a rejected thought or dead-end. |

### Knowledge Graph

Every `mcp_engram_relate` call stores a ZEDOS_RELATION block using `op_bind`. Edges are mathematical memory vectors, meaning no external graph database is required.

| Tool | Description |
|---|---|
| `mcp_engram_relate` | Bind two concepts via `op_bind` to build the graph |
| `mcp_engram_search_by_relation` | Traverse the graph by edge direction and label |
| `mcp_engram_visualize` | BFS from a seed concept → outputs a Mermaid diagram |

---

## 🧠 The Agentic Daemon & Autophagy

When Engram boots as an MCP server, it also launches a **background Agentic Daemon** that manages autonomous file system watching via `inotify`/`fsevents` kernel integration. When you save a file, the daemon re-ingests the changed AST components instantly.

> [!NOTE]
> **Autophagy (GC) is Disabled by Default:** We believe an agent's memory should outlive its sessions. If a user steps away from a project for 3 months, their contextual memory shouldn't spontaneously decay. Engram calculates Coherence-Reliability Scores (CRS) continuously, but we deliberately disabled automatic eviction. You must use the `mcp_engram_forget_old` tool to instruct the agent to run manual garbage collection.

---

## 🌐 Multi-Project Namespaces

Use sheaf mode to isolate memories by project. Create `~/.engram/sheaf.toml`:

```toml
active_stalk = "codeland"

[[stalks]]
name = "codeland"
path = "~/.engram/stalks/codeland"

[[stalks]]
name = "personal"
path = "~/.engram/stalks/personal"
```

Then switch namespaces via MCP at any time:
```
mcp_engram_set_namespace("personal")
```

---

## 💻 IDE Integration

> Integration configs for all supported IDEs: [`integrations/`](integrations/)

### Google Antigravity IDE
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

### Claude Desktop / Cursor / VS Code
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

---

## ⚙️ Hardware Support

| Backend | Feature Flag | Status | Notes |
|---|---|---|---|
| CPU (Rayon O_DIRECT) | Default | ✅ | Exact linear scan. At 10K memories scans 2.5 GB in < 0.4s via NVMe DMA bypass |
| CPU (LBVH index) | `bvh` feature | ✅ | O(log N) CSRP-projected tree. ~64 bytes RAM per concept. Build with `engram build-index` |
| CUDA (NVIDIA) | `cuda-kernels` | ✅ | GPU BVH O(log N), NVMe→VRAM parallel DMA |
| ROCm (AMD) | `rocm-kernels` | ✅ | Wavefront HIP execution |
| Metal (Apple) | `metal` | ✅ | MSL dynamic runtime compilation via metal-rs |
| **WebGPU** | **`wgpu-backend`** | ✅ | INT8 Poincaré hyperbolic search · 170× VRAM reduction · cross-platform |

---

## 📄 License & Patent

This software is licensed under **AGPL-3.0-only**.

The `.LEG` container format is covered by **U.S. Patent Application No. 19/372,256** (pending),  
*Self-Contained Variable File System (.LEG Container Format)*,  
Applicant: **Aric Goodman**, Oregon, USA — Static Rooster Media.

Commercial licenses (SaaS/cloud/enterprise) are available.  
Contact: **StaticRoosterMedia@gmail.com**

See [PATENT-NOTICE.md](PATENT-NOTICE.md) for full details.
