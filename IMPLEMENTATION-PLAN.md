# Engram Implementation Plan

## Phases 1–4: Completed ✅

- [x] Headless conversion, Autophagy GC, TurboQuant BVH, NVSA operator ports, documentation audit (see git history)

---

## Phase 5: MCP Tool Expansion — Full Feature Parity

### Discovery
Audit confirmed the following store-layer functionality **already exists** but is **not yet wired as MCP tools**:
- `AccessIndex.recent(n)` → temporal recall is implemented
- `StoreHandle.update()` → in-place memory update
- `StoreHandle.export_context()` → context export exists
- `SheafConfig` + stalk system → namespace/project isolation is fully implemented
- `HolographicBlock.crs_score` + timestamps → all data for stats is in every block

This means Phase 5 is largely **MCP wiring + aggregation logic**, not new backend work.

---

### Phase 5A: Quick Wins (MCP wiring only)
*All store logic exists — just needs `mcp.rs` handlers.*

- [ ] **`mcp_engram_stats`** — Aggregate manifold health report
  - Total concept count, pinned count, avg/min/max CRS, oldest block timestamp, newest block timestamp, disk usage of manifold path
  - No new store logic needed — iterate `list()`, call `fetch_block()` per concept

- [ ] **`mcp_engram_recall_recent`** — Temporal recall
  - Args: `n_concepts: usize` (default 10), optional `since_hours: f64`
  - Wraps existing `StoreHandle.recent(n)` → returns concepts sorted by `last_accessed_timestamp`
  - Enables session rehydration without knowing concept names in advance

- [ ] **`mcp_engram_set_namespace`** — Switch active project/stalk
  - Args: `namespace: String`
  - Wraps `Backend.set_active_stalk()` — sheaf system already implemented
  - Creates namespace if it doesn't exist

- [ ] **`mcp_engram_list_namespaces`** — List all project namespaces
  - Wraps `StoreHandle.stalk_names()` + `active_stalk_name()`

- [ ] **`mcp_engram_update`** — Update an existing memory in place
  - Args: `concept: String`, `new_text: String`
  - Wraps `StoreHandle.update()` which already handles re-encoding + CRS bump

---

### Phase 5B: Core New Features
*Requires new store logic + MCP handlers.*

- [ ] **`mcp_engram_summarize`** — Project state digest
  - Returns all pinned memories (`CRS = 1.0`) + top N by CRS score as a formatted summary
  - New `StoreHandle.summarize(top_n: usize)` method that iterates blocks, sorts by CRS, formats output
  - This replaces the manual multi-recall `/wake_up` pattern

- [ ] **`mcp_engram_batch_remember`** — Bulk memory ingestion
  - Args: `entries: Vec<{concept: String, text: String}>`
  - Single MCP call, sequential writes to manifold
  - Critical for initializing Engram from an existing knowledge dump

- [ ] **`mcp_engram_export`** — Export manifold to portable JSON
  - Args: optional `namespace: String`, optional `min_crs: f32`
  - Returns: `Vec<{concept, text (ProvLog), crs, created_at, last_accessed, tags}>`
  - Uses `fetch_block()` to read `LegFooter` source text per concept

- [ ] **`mcp_engram_import`** — Import from exported JSON
  - Args: `entries: Vec<{concept, text}>`, optional `namespace: String`
  - Batch writes — enables cross-machine sync and backup restore

- [ ] **`mcp_engram_forget_old`** — On-demand autophagy
  - Args: `min_crs_threshold: f32` (e.g. 0.2), optional `older_than_days: u64`
  - Triggers manual eviction of blocks below threshold without waiting for the daemon
  - Returns count of evicted concepts

---

### Phase 5C: Knowledge Graph Query Layer
*New index structure required for relation queries.*

The `relate` tool stores relation blocks but there is no inverted index for querying them. This phase builds the read side of the knowledge graph.

- [ ] **Relation index** — `relations.json` sidecar file in manifold root
  - Schema: `{ concept_a: { label: [concept_b, ...] } }`
  - Updated atomically on every `relate` call (append-only, no rewrite)
  - Loaded into memory on boot for O(1) relation lookup

- [ ] **`mcp_engram_search_by_relation`** — Knowledge graph traversal
  - Args: `concept: String`, optional `label: String`, optional `direction: "from"|"to"|"both"`
  - Returns all concepts related to the seed by the given label
  - Example: `search_by_relation("authentication", "depends_on")` → all things that depend on auth

- [ ] **`mcp_engram_visualize`** — Render concept graph as Mermaid
  - Args: `seed_concept: String`, `depth: usize` (default 2)
  - BFS traversal of relation index from seed
  - Returns a Mermaid `graph LR` block the agent can embed in markdown
  - No external dependencies — pure string generation

---

### Phase 5D: Session Hooks (Advanced)
*Integrates Engram into the agent session lifecycle.*

- [ ] **`mcp_engram_session_start`** — Snapshot session context
  - Creates a pinned memory: `session_{timestamp}` with a summary of active namespace state
  - Returns the executive digest (replaces manual `/wake_up` recall sequence)

- [ ] **`mcp_engram_session_end`** — Commit session to long-term memory
  - Args: `summary: String` (agent writes what happened this session)
  - Stores with moderate CRS, promotes anything accessed >3 times this session to higher CRS
  - Enables automatic knowledge consolidation between sessions

---

## Phase 6: CLI Enhancements ✅

All core CLI subcommands shipped:
- [x] `engram distill` — cluster manifold memories into ZEDOS_PRAXIS centroids via `bundle()` superposition
- [ ] `engram stats` — print manifold health to stdout
- [ ] `engram export > backup.json` — dump to stdout
- [ ] `engram import backup.json` — restore from file
- [ ] `engram visualize <concept>` — print Mermaid graph to stdout

---

## Phase 5D: Session Hooks 🟡 NEXT

Integrates Engram into the agent session lifecycle. Two new MCP tools:

- [ ] **`mcp_engram_session_start`**
  - Loads manifold digest (summarize internally)
  - Computes session anchor: OP_ADD superposition of top-5 recently accessed blocks
  - Writes `SESSION_START::timestamp` as ZEDOS_EPISODIC (naturally decays)
  - Returns full digest so agent rehydrates in one call — no manual recall loop

- [ ] **`mcp_engram_session_end`** `summary: String`
  - Stores agent-written session summary as ZEDOS_PRAXIS at CRS=0.80
  - CRS promotion sweep: bump all blocks accessed this session by `+0.05 × access_count`
  - Computes session centroid: `bundle(all accessed q-vectors)` → written as `SESSION_CENTROID::timestamp`
  - Session centroid becomes the seed for the *next* session_start
  - Decays the session_start block from this session (now stale)

---

## Phase 7: Ouroboros AST Pipeline 🔴 TODO

Port CodeLand's tree-sitter → phase-vector pipeline as a standalone `engram-ast` crate.
Gives Engram **native code structure awareness** — the AST topology is encoded geometrically,
not as raw text. `mcp_engram_watch_workspace` would automatically use it for `.rs`, `.py`, `.ts` files.

- [ ] `engram-ast` crate: tree-sitter → 8192D phase vector encoder per language
- [ ] Wire into `mcp_engram_watch_workspace` as opt-in backend (`--ast-mode`)
- [ ] `engram-cli ingest --ast` flag to AST-encode a directory instead of chunking by character

---

| Priority | Feature | Effort | Impact |
|---|---|---|---|
| 1 | `stats` | Low (MCP wire only) | High |
| 2 | `recall_recent` | Low (MCP wire only) | High |
| 3 | `summarize` | Medium (new store logic) | High |
| 4 | `set_namespace` / `list_namespaces` | Low (MCP wire only) | High |
| 5 | `update` | Low (MCP wire only) | Medium |
| 6 | `export` / `import` | Medium | High |
| 7 | `batch_remember` | Low | Medium |
| 8 | `forget_old` | Low | Medium |
| 9 | Relation index | Medium | Medium |
| 10 | `search_by_relation` | Medium | Medium |
| 11 | `visualize` | Medium | Medium |
| 12 | Session hooks | High | High |
| 13 | CLI subcommands | Low | Medium |

---

## Versioning

- `v0.1.0` — Initial release
- `v0.2.0` — Phase 5A: stats, recall_recent, namespace, update tools
- `v0.3.0` — Phase 5B: summarize, batch_remember, export, import, forget_old tools
- `v0.4.0` — Phase 5C: relation index, search_by_relation, visualize tools
- `v0.5.0` — Phase 6: `engram distill` CLI command ← **current**
- `v0.6.0` — Phase 5D: session_start / session_end MCP tools
- `v1.0.0` — Phase 7: Ouroboros AST pipeline (stable API)
