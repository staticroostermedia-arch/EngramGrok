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
