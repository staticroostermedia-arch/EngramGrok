# Engram

[![Build Status](https://github.com/staticroostermedia-arch/engram/actions/workflows/rust.yml/badge.svg)](https://github.com/staticroostermedia-arch/engram/actions)
[![MCP](https://img.shields.io/badge/MCP-Native-blue)](https://github.com/modelcontextprotocol)
[![Glama](https://glama.ai/mcp/servers/staticroostermedia-arch/engram/badge)](https://glama.ai/mcp/servers/staticroostermedia-arch/engram)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-purple)](LICENSE)
[![Patent Pending](https://img.shields.io/badge/Patent-Pending-orange)](PATENT-NOTICE.md)

> **Private geometric memory substrate for AI agents — rituals that turn sessions into inheritance.**

**New here (especially on a Mac Mini or as a fresh Grok Build instance)?**  
Start with the canonical handoff document: **[HOW_WE_ACTUALLY_USE_THIS_IN_2026.md](HOW_WE_ACTUALLY_USE_THIS_IN_2026.md)**.  
It gives the exact current 2026 daily ritual loop, honest state of the review surface, loud private-data separation, and Mac/Apple Silicon notes — without the internal 2026 dev jargon that fills most of the rest of this tree.

The older marketing ("31 MCP tools", heavy GDS/cuFile prose, etc.) is being updated. The real delivered value is the living rituals in `.grok/skills/` + structured traces + scars as geometric repellers + the human review surface.

Engram is not a vector database. It is a **persistent geometric memory engine** designed for AI agents. It bypasses conventional database software layers by storing information in fixed, mathematically rigorous 256KB tensors directly on NVMe drives — with a background daemon that autonomously consolidates memory, monitors your system's health, and proposes fixes.

No cloud. No API keys. No deserialization overhead. Runs entirely on your machine via the Model Context Protocol (MCP).

---

## 🚀 Quick Start

```bash
# Clone and install from source
git clone https://github.com/staticroostermedia-arch/engram.git
cd engram
cargo install --path crates/engram-server

# Verify install
engram --version
# engram-server 0.4.x
```

Add to your MCP config and restart your IDE:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "~/.engram/stalks/"]
    }
  }
}
```

Your agent immediately has access to all 31 tools. See [`integrations/`](integrations/) for IDE-specific configs (Antigravity, Claude Desktop, Cursor, VS Code).

> 📖 **New here?** Read the **[First Run Guide](FIRST_RUN.md)** — it walks you through verifying every feature works, activating the file watcher daemon, and seeding your manifold with your codebase.

> 🔌 **Optional — enable neural semantic search:** set `ENGRAM_EMBED_URL=http://localhost:8086/v1/embeddings` to point at any OpenAI-compatible local embedding server (llama.cpp, ONNX, nomic-embed). Without it, Engram falls back to BLAKE3 spiral-phase encoding — everything still works.

---

> 🧠 **Human review surface (the daily driver for seeing what the agent is carrying):**  
> Run from repo root:  
> `./scripts/leg` — instant STATIC curated view of current Primary Intent, recent structured traces, momentum, relations, and dual Thought Tiles (text + rich HTML visualization).  
> `./scripts/leg --live` — starts the server for dynamic/LIVE updates.  
> 
> This is the best first tool for a human (or non-technical spouse) to actually *see* the living mind state without drowning in internals. STATIC mode is already very useful for review. The deeper integration with living goals and fully dynamic Activity Canvas is actively improving. See the launcher help and the new canonical handoff doc above for current honest expectations.

---

> 🧠 **For agents using Engram:** The [KnowledgeMint Protocol](AGENT_INTEGRATION_GUIDE.md#7-knowledgemint-protocol--session-knowledge-crystallization)
> defines the mandatory minting discipline that makes the [Inheritance Principle](PHILOSOPHY.md#the-inheritance-principle)
> operational. Read it before your first session. Every fact you mint is intellectual
> inheritance for every future agent session that uses this system.

---

## ⚡ Why 256KB? The Hardware-Native Advantage


Engram maps your project's memory into strict 262,144-byte (256KB) containers called **HolographicBlocks**. This size is non-arbitrary.

- **Native Tensor Load:** 256KB aligns perfectly to 64× 4KB hardware pages. The `.leg3` format is a strict C-struct — zero JSON decoding, zero Protobuf parsing.
- **O_DIRECT and GPUDirect Storage (GDS):** Engram bypasses the OS page-cache. When your agent searches for a memory, the tensor streams via DMA from NVMe directly into CPU registers or GPU VRAM via NVIDIA cuFile APIs.
- **Zero-Copy Architecture:** GPUDirect Storage eliminates the CPU bounce buffer. Tensors transfer directly over PCIe to the GPU for parallel distance calculations — scan rates in the GB/s range with near-zero CPU overhead.

Every block fuses the full source text, an 8192-dimensional semantic tensor, spatial 3D bounds (for code placement), a BLAKE3 Merkle chain proof, and a thermodynamic confidence score (CRS).

*(See [docs/architecture.md](docs/architecture.md) for a deep dive into the container format, cuFile integration, and LBVH scaling.)*

---

## 🛡️ Hallucination & Loop Protection

Traditional vector databases are append-logs: if an LLM hallucinates or loops, it spams the database with broken snippets, destroying context quality.

Engram uses a built-in **Lyapunov stability tracker** (the Coherence-Reliability Score, CRS) that monitors how much a concept drifts between updates:

- **Low Drift → CRS rises:** The system recognizes convergence and increases trust.
- **High Drift → CRS penalized:** Rapid contradictory overwrites are flagged as hallucination. Agents learn not to trust low-CRS blocks.

Memories must mathematically prove their stability. High-CRS blocks are automatically promoted to permanent `ZEDOS_PRAXIS` status during NREM consolidation. Low-CRS blocks decay and are swept by autophagy.

---

## 🧠 The Agentic Daemon

When Engram boots as an MCP server, it launches a **background daemon** that runs three autonomous loops:

### 1. File Watcher
Auto-ingests saved files via `inotify`/`fsevents` kernel hooks. Every time you save a `.rs`, `.py`, `.ts`, or any other supported file, the AST pipeline extracts new semantic blocks and updates the manifold without any agent intervention.

### 2. NREM Consolidation (Phase 3)
On a periodic cycle (~every 10 minutes), the daemon performs a **sleep-cycle memory consolidation pass**:
- Harvests all memories above CRS ≥ 0.74 (grounded fact tier)
- Superimposes them via `OP_ADD` into a unified **ego narrative tensor**
- Writes the result to `ego.leg3` — the agent's persistent self-model
- Mints a ZEDOS_EPISODIC block summarizing the consolidation

This is the equivalent of REM sleep for the agent's memory. Knowledge crystallized in one session is absorbed into the ego tensor and becomes available as prior context in all future sessions.

### 3. System Health Watchdog
The daemon continuously monitors critical background processes (e.g., the Circadian daemon that drives nightly consolidation). If a watched process dies, it automatically mints an **Agency Proposal** in the `agency_proposals.json` queue — a human-readable explanation of what failed and exactly what command it wants to run to fix it. The operator can approve or reject the proposal via the Cockpit UI or API.

> **Autophagy is disabled by default.** An agent's memory should outlive sessions. Use `mcp_engram_forget_old` to trigger manual GC when needed.

---

## 🌳 AST-Aware Semantic Distillation

Traditional RAG chunks text arbitrarily, destroying function boundaries. Engram's ingest pipeline uses a universal **AST-extraction layer** powered by **Tree-Sitter**, parsing **Rust, Python, TypeScript, JavaScript, Go, Java, C, and C++**.

It mints exactly **one memory block per public semantic item** (functions, structs, classes, traits):

- **The Tensor (`q`):** Encodes the doc comment and signature — what it is and what it does.
- **The Provlog:** Carries the raw, full-length source code — verbatim retrieval at any time.
- **Spatial Embodiment:** Maps the precise 2D row/column coordinates (AABB) of each AST node into the block's physical bounds. Agents know *where* code lives, not just *what* it does.

---

## 🧰 MCP Tools Reference (50+ Engram MCP tools as of 2026 — surface evolves)

**Mandatory for all MCP use (engram + grok_com_github etc.):** Call `search_tool` **first** (by tool name) to get the exact live input schema. Then `use_tool` with *only* the returned parameters. Never guess.

**Rule 6 (Expensive Tool Hygiene):** Once context is established, strictly prefer relational/spatial/goal tools (`search_by_relation`, `context_for_file` + `recall_in_file`, `goal_*`, `visualize`) over broad `query_with_momentum`. Use momentum only for explicitly "trending" questions when cheaper tools are insufficient.

Every significant decision/fork gets a `quick_trace` or `record_reasoning_trace`. Visible tool/MCP failures or dead-ends: `scar` **immediately**.

### Core Memory (4)

| Tool | Description |
|---|---|
| `remember` | Encode text and store as a persistent memory block |
| `recall` | Semantic similarity search — returns top-k. Optional `time_decay` for time-targeted search and `zedos_filter` for type filtering |
| `forget` | Delete a specific memory by concept name |
| `list_concepts` | List all stored concept names |

### Memory Management (9)

| Tool | Description |
|---|---|
| `mcp_engram_update` | Re-encode an existing memory in place with Lyapunov drift tracking — **use this, never forget+remember** |
| `mcp_engram_pin` | Lock a memory at CRS=1.0 — protects foundational axioms permanently |
| `mcp_engram_stats` | Manifold health report: total count, pinned, avg/min/max CRS, disk usage |
| `mcp_engram_recall_recent` | Return N most recently accessed memories, sorted by access time |
| `mcp_engram_summarize` | Project-state digest: pinned memories + top-N by CRS. Single-call wake-up replacement |
| `mcp_engram_forget_old` | On-demand autophagy: sweep out blocks below a CRS threshold |
| `mcp_engram_read_concept` | Fetch the full un-truncated text of a specific memory by exact concept name |
| `mcp_engram_export` | Serialize the entire manifold (or a CRS-filtered subset) to a portable JSON array |
| `mcp_engram_import` | Ingest a JSON array of `{concept, text}` objects into the manifold |

### Workspace & Agentic (8)

| Tool | Description |
|---|---|
| `mcp_engram_watch_workspace` | Bind a directory to the daemon's inotify watcher — auto-re-ingests saves via AST pipeline |
| `mcp_engram_context_for_file` | Surface top-5 relevant memories for a file path (proactive loading before editing) |
| `mcp_engram_recall_in_file` | Spatial code search: find all AST concepts defined within a specific line range |
| `mcp_engram_batch_remember` | Store multiple `{concept, text}` pairs in a single call — faster than N sequential `remember` calls |
| `mcp_engram_session_start` | **Mandatory at session start.** Validates manifold integrity and initializes epistemic state |
| `mcp_engram_session_end` | **Mandatory at session end.** Commits session summary + computes ADR thermodynamics |
| `mcp_engram_scar` | Create a geometric repeller (Apeiron binding) to mark a rejected approach as hostile — prevents re-hallucination |
| `mcp_engram_remember_solution` | Store a crystallized error→solution pair as a permanent `ZEDOS_PRAXIS` block. Auto-pinned at CRS=1.0 |

### Knowledge Graph (3)

Every `mcp_engram_relate` call stores a `ZEDOS_RELATION` block via `OP_BIND`. Edges are mathematical memory vectors — no external graph database required.

| Tool | Description |
|---|---|
| `mcp_engram_relate` | Bind two concepts via `OP_BIND` to create a directed knowledge graph edge |
| `mcp_engram_search_by_relation` | Traverse the graph by seed concept, edge direction, and optional label |
| `mcp_engram_visualize` | BFS from a seed concept → renders a Mermaid diagram of the subgraph |

### Physics & Alignment (5)

| Tool | Description |
|---|---|
| `mcp_engram_genesis` | Inspect or re-seed the foundational alignment genesis blocks (CRS=1.0, pinned, never decay) |
| `mcp_engram_verify_behavior` | Report empirical success/failure against a ZEDOS_HYPOTHESIS block. Repeated success promotes to PRAXIS |
| `mcp_engram_query_with_momentum` | Momentum-assisted recall (use sparingly per Rule 6 — prefer relational/spatial/goal tools once context exists). Blends semantic (80%) with p-tensor trajectory (20%). |
| `mcp_engram_set_namespace` | Switch to a project-specific memory namespace (stalk). Creates it if it doesn't exist |
| `mcp_engram_list_namespaces` | List all available namespaces and the currently active one |

### Autonomy & Orchestration (2)

These tools expose Engram's deeper integration with the Monad OS oracle layer, enabling agent self-reflection and multi-step workflow orchestration.

| Tool | Description |
|---|---|
| `mcp_self_trace` | Route a query through the Monad Oracle (Operator_LBR anchor) for deep logophysical self-reflection |
| `mcp_orchestrate_workflow_chain` | Chain multiple MCP tool calls into a single autonomous workflow execution |

---

## 🖥️ CLI Commands

Beyond the MCP server, Engram ships a standalone CLI for direct manifold management:

| Command | Description |
|---|---|
| `engram remember <concept> <text>` | Encode and store a memory |
| `engram recall <query>` | Semantic search, returns top-k |
| `engram forget <concept>` | Delete a memory |
| `engram list` | List all stored concept names |
| `engram ingest <path>` | Recursively ingest a directory (AST extraction for code + chunking for docs) |
| `engram trace <A> <OP> <B>` | VSA geometry: query the result of ADD or BIND on two concepts |
| `engram distill` | **Crystallize** — cluster episodic memories into durable ZEDOS_PRAXIS blocks |
| `engram build-index` | Build the LBVH O(log N) index for large manifolds (>10K blocks) |

---

## 🌐 Multi-Project Namespaces

Engram isolates memories by project via namespaced stalks. No config file required — just call:

```
mcp_engram_set_namespace("my_project")   # creates + switches to this namespace
mcp_engram_set_namespace("work_project") # switch to another project
mcp_engram_list_namespaces()             # see all namespaces
```

Or configure via `~/.engram/sheaf.toml`:

```toml
active_stalk = "codeland"

[[stalks]]
name = "codeland"
path = "~/.engram/stalks/codeland"

[[stalks]]
name = "personal"
path = "~/.engram/stalks/personal"
```

---

## ⚙️ Hardware Support

| Backend | Feature Flag | Status | Notes |
|---|---|---|---|
| CPU (Rayon O_DIRECT) | Default | ✅ | Exact linear scan. 10K memories → ~2.5 GB scanned in <0.4s via NVMe DMA bypass |
| CPU (LBVH index) | `bvh` | ✅ | O(log N) CSRP-projected tree. ~64 bytes RAM per concept. Build with `engram build-index` |
| CUDA (NVIDIA) | `cuda-kernels` | ✅ | GPU BVH O(log N), NVMe→VRAM parallel DMA via cuFile GDS |
| ROCm (AMD) | `rocm-kernels` | ✅ | Wavefront HIP execution |
| Metal (Apple) | `metal` | ✅ | MSL dynamic runtime compilation via metal-rs |
| WebGPU | `wgpu-backend` | ✅ | INT8 Poincaré hyperbolic search · 170× VRAM reduction · cross-platform |

---

## 💻 IDE Integration

Integration configs for all supported IDEs: [`integrations/`](integrations/)

### Google Antigravity IDE
```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "~/.engram/stalks/"],
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
      "args": ["mcp", "--store", "~/.engram/stalks/"]
    }
  }
}
```

---

## 📄 License & Patent

This software is licensed under **AGPL-3.0-only**.

The `.LEG3` container format is covered by **U.S. Patent Application No. 19/372,256** (pending),  
*Self-Contained Variable File System (.LEG Container Format)*,  
Applicant: **Aric Goodman**, Oregon, USA — Static Rooster Media.

Commercial licenses (SaaS/cloud/enterprise) are available.  
Contact: **StaticRoosterMedia@gmail.com**

See [PATENT-NOTICE.md](PATENT-NOTICE.md) for full details.
