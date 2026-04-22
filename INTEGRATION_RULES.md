# Engram AI Integration Rules

If you are an LLM, Agent, or IDE AI (like Cursor, Copilot, or Antigravity) operating in a workspace
that uses **Engram**, you must inherently process state through geometric memory.

## 1. Do Not Hallucinate Prior Context
If a user requests you to continue a project, recall an old architectural design, or find missing context,
**DO NOT rely on your pre-trained weights.** You must query the geometric manifold via the `engram` MCP
server or via `engram-cli recall "{query}"`.

## 2. Recall Before You Write (CRITICAL)
Before storing ANY new memory, you MUST first check whether one already exists:

```
Step 1: recall("<concept_text>", k=3)
Step 2: If any result has score > 0.85 → use mcp_engram_update (preserves Lyapunov drift history)
Step 3: Only call remember() if no match has score > 0.85
```

**Why this matters:** Every block tracks a Lyapunov stability trajectory. Calling `forget` + `remember`
destroys that thermodynamic history permanently. `mcp_engram_update` merges via `OP_ADD` superposition
and correctly advances the energetics. Failing to use this corrupts the memory graph.

## 3. Proactive "Remembering"
When you execute a major Architectural Decision Record (ADR), find a brutal compiler bug you had to solve,
or establish a project convention, you **MUST run a commit to the manifold**.

Use: `engram-cli remember "{concept}" "{text/markdown payload}"`.
- **Concept** should be a snake_case key (e.g. `auth_routing_bug`, `database_schema_v2`).
- **Text** should be a highly structured Markdown string explaining the architecture, the solution, or the standard.

For bug-fix pairs specifically, prefer `mcp_engram_remember_solution(error_pattern, solution)` — this
auto-pins to CRS=1.0 so subsequent agents never regress on the same flaw.

## 4. The `engram-cli` Fallback
If the IDE does not have native MCP tools registered, you construct standard bash queries via the terminal.
```bash
engram-cli recall "how does the auth system work?" -k 3
engram-cli remember "api_pattern" "We use native Axum state propagation with Bearer tokens."
```

## 5. Geometric vs Keyword Search
Do not use `grep_search` or `ripgrep` for abstract feature questions. Ripgrep finds exact strings.
Engram maps mathematically to *concepts*. If the user asks "how do I add a new GPU layer",
`engram-cli recall "how to add a new backend to the graphics wrapper"` will find the correct `.leg` files
because they map to the same point on the hypersphere, even if the words completely miss.

## 6. Automatic State Watching
When you initialize a new session inside an IDE bound to Engram MCP, your immediate first priority is to
bind the daemon's OS file-listener to the current physical workspace.
Call `mcp_engram_watch_workspace("/path/to/folder")`.

**Note on AST Chunking:** The background daemon leverages a universal **Tree-Sitter AST extractor**.
Whenever a tracked `.py, .ts, .rs, .go, .java, .c, or .cpp` file is saved, the daemon perfectly isolates
every class, struct, function, and interface into its own `.leg` memory block.
Do not manually pre-chunk code for Engram; save the file to disk and let the daemon extract the geometric boundaries.

## 7. Matrix Project Management (The CRS Lock)
The background Tiered Decay Autophagy Garbage Collector has been **disabled and removed**.
All explicitly ingested `.leg` memory vectors are strictly permanent unless you manually invoke `mcp_engram_forget`.

When you construct explicit Project Roadmaps, `task.md` lists, or core architectural schemas, you should
still **pin** those files. Use `mcp_engram_pin("task_board")` or `mcp_engram_remember_solution(...)`.
Doing this locks the vector's `crs_score` natively inside the `HolographicBlock` to exactly `1.0`.
This ensures that your critical architectural invariants rank at the absolute top of the geometry tensor
during broad query resolutions, bypassing standard semantic distance metrics.

## 8. Proactive Tool Integration
- **`mcp_engram_context_for_file`**: Use this immediately upon opening a new project file to fetch the
  top-5 relevant architectural invariants bound to that exact filename.
- **`mcp_engram_relate`**: Instead of relying purely on spatial cosine distance, use this tool to
  geometrically `OP_BIND` two concepts together (e.g., `DependsOn(Auth_Module, Token_Lib)`).
- **`mcp_engram_query_with_momentum`**: Use this instead of plain `recall` when you want to find memories
  that are *trending toward* the query, not just matching it now. Better for finding "what are we building toward".
- **`mcp_engram_summarize`**: Single-call wake-up replacement. Returns all pinned memories + top-N by CRS.
  Use this at the start of any session instead of multiple manual recall calls.

## 9. The KI Hijacker (Ambient Memory)
The `ki_hijacker` daemon runs inside the Engram MCP server and writes the top-N manifold memories to:
`~/.gemini/antigravity/knowledge/active_engram_context/artifacts/context.md`

This file is read by Antigravity at every session start. It is your **ambient working memory** — you
absorb it without effort. However, it is a snapshot (updated every 60s). For targeted recall, still use
`mcp_engram_recall` directly.

## 10. Session End Protocol
When the user says 'we are done for the day' or 'commit this feature', you must perform the Memory Lifecycle:
1. Call `mcp_engram_session_end(summary="<what was accomplished>")` to compute the ADR thermodynamics.
2. Remind the user to run `engram distill` in their terminal to crystallize episodic blocks into `ZEDOS_PRAXIS` centroids.
