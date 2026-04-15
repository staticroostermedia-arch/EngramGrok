# Engram Implementation Plan

## Phase 1: Completed Optimizations
- [x] **Headless Conversion**: Strip out chat.html, chat.css, chat.js, and Axum UI routes to ensure a lean, core memory backend.
- [x] **Privacy / PII Scrubber (Moloch Guard)**: Add regex-based PII scrubbing pipeline into the `POST /api/remember` path.
- [x] **TurboQuant Codebook for BVH CPU**: Implemented inline B=4 `TurboQuant` codebook quantization into `ManifoldEntry`, saving 128 NVMe read scans during `K-NN` and providing a ~4x speedup.
- [x] **Manifold Autophagy (Tiered Decay GC)**: Upgraded Engram background `daemon.rs` with `monad_praxis` Tiered Decay GC logic:
    - 2% utility drop for 24-hr stale.
    - 5% utility drop for 7-day stale.
    - Thermodynamic eviction at <= 0.05 CRS floor.

## Phase 2: Agentic IDE Super-Tools
We need to equip Engram with additional context tools previously established in `monad_praxis` that allow external agents to proactively and topologically query the workspace.

- [x] **1. Topological `OP_BIND` Integration (Relations)**
  - Add geometric relationship binding logic inside `engram-core` (or a dedicated `monad_vsa` fallback).
  - Add `mcp_engram_relate` to `mcp.rs` to allow mapping semantic links (e.g., `DependsOn(api, database)`).
  - Modify `.leg3` provenance tracking to list bound relational tags.

- [x] **2. Proactive File-Context Streaming (`context_for_file`)**
  - Implement a `mcp_engram_context_for_file` tool payload.
  - When an agent looks at a specific file path (e.g. `src/bvh.rs`), auto-generate an engram query utilizing the file name and return the Top 3 constraints.
  - Minimizes IDE context limits by returning precision-targeted constraints.

- [x] **3. Invariant Error Crystallization (`remember_solution`)**
  - Add `mcp_engram_remember_solution` MCP tool.
  - Takes `error_trace` and `solution` as arguments.
  - Ingests into the `.leg3` NVMe manifold with `CRS = 1.0` (permanently immune to Autophagy).
  - Ensures subsequent agents immediately recall successful debug fixes.

## Phase 3: Documentation & Release
- [ ] Update `README.md` to document the new `engram` MCP commands and performance benchmarks.
- [ ] Document Autophagy thresholds in `INTEGRATION_RULES.md`.
- [ ] Add explicit instructions on how an external IDE should register the expanded MCP server list.
