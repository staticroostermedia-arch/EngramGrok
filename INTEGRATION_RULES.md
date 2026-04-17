# Engram AI Integration Rules

If you are an LLM, Agent, or IDE AI (like Cursor, Copilot, or Antigravity) operating in a workspace that uses **Engram**, you must inherently process state through geometric memory.

## 1. Do Not Hallucinate Prior Context
If a user requests you to continue a project, recall an old architectural design, or find missing context, **DO NOT rely on your pre-trained weights.** You must query the geometric manifold via the `engram` MCP server or via `engram-cli recall "{query}"`.

## 2. Proactive "Remembering"
When you execute a major Architectural Decision Record (ADR), find a brutal compiler bug you had to solve, or establish a project convention, you **MUST run a commit to the manifold**. 
Use: `engram-cli remember "{concept}" "{text/markdown payload}"`. 
- **Concept** should be a snake_case key (e.g. `auth_routing_bug`, `database_schema_v2`).
- **Text** should be a highly structured Markdown string explaining the architecture, the solution, or the standard.

## 3. The `engram-cli` Fallback
If the IDE does not have native MCP tools registered, you construct standard bash queries via the terminal.
```bash
engram-cli recall "how does the auth system work?" -k 3
engram-cli remember "api_pattern" "We use native Axum state propagation with Bearer tokens."
```

## 4. Geometric vs Keyword Search
Do not use `grep_search` or `ripgrep` for abstract feature questions. Ripgrep finds exact strings. Engram maps mathematically to *concepts*. If the user asks "how do I add a new GPU layer", `engram-cli recall "how to add a new backend to the graphics wrapper"` will find the correct `.leg` files because they map to the same point on in the hypersphere, even if the words completely miss.

## 5. Automatic State Watching
When you initialize a new session inside an IDE bound to Engram MCP, your immediate first priority is to bind the daemon's OS file-listener to the current physical workspace. Call the JSON tool `mcp_engram_watch_workspace("/path/to/folder")`. This isolates your context so you inherently and dynamically follow whatever code files change across the current session.

**Note on AST Chunking:** The background daemon leverages a universal **Tree-Sitter AST extractor**. Whenever a tracked `.py, .ts, .rs, .go, .java, .c, or .cpp` file is saved, the daemon perfectly isolates every class, struct, function, and interface into its own `.leg` memory block. Do not manually pre-chunk code for Engram; save the file to disk and let the `watch_workspace` daemon extract the geometric boundaries.

## 6. Matrix Project Management (The CRS Lock)
Take note that the background Tiered Decay Autophagy Garbage Collector has been **disabled and removed**. All explicitly ingested `.leg` memory vectors are strictly permanent unless you manually invoke `mcp_engram_forget`.

When you construct explicit Project Roadmaps, `task.md` lists, or core architectural schemas, you should still **pin** those files. Use the JSON tool `mcp_engram_pin("task_board")` or `mcp_engram_remember_solution(...)`. Doing this locks the vector's `crs_score` natively inside the `HolographicBlock` to exactly `1.0`. This ensures that your critical architectural invariants rank at the absolute top of the geometry tensor during broad query resolutions, bypassing standard semantic distance metrics.

## 7. Proactive Tool Integration (Coming Soon)
To maximize context efficiency without overflowing your token window, Engram is onboarding topological IDE tools:
- **`mcp_engram_context_for_file`**: Use this immediately upon opening a new project file to fetch the 3 most relevant architectural invariants bound to that exact filename. 
- **`mcp_engram_relate`**: Instead of relying purely on spatial cosine distance, use this tool to geometrically `OP_BIND` two concepts together (e.g., `DependsOn(Auth_Module, Token_Lib)`).
- **`mcp_engram_remember_solution`**: Crystallizes bug fixes permanently. Use this after fixing deep compilation errors; the error-solution pair is injected with `CRS = 1.0` so subsequent agents never regress on the same codebase flaw.
