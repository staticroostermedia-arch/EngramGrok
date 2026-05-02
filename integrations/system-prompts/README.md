# Engram System Prompt Additions — IDE Integration Guide

> Add these instructions to your IDE's system prompt or agent instructions to make Engram a first-class behavior rather than an optional tool.

---

## The Problem This Solves

Without explicit instruction, agents treat memory tools as optional — they'll call them when convenient but won't maintain the session protocol that makes longitudinal memory work. These system prompt additions establish the correct behavior patterns at the instruction level.

---

## Universal Core (Add to Any IDE)

Add this to your agent's system prompt or instructions file:

```markdown
## Memory Protocol (Engram)

You have access to Engram, a persistent geometric memory system via MCP tools.

**MANDATORY at session start:**
1. Call `mcp_engram_session_start(intent="<your goal>")` before anything else
2. Call `mcp_engram_watch_workspace("/path/to/project")` to enable AST auto-ingest
3. Call `mcp_engram_summarize(top_n=10)` to load project state

**MANDATORY at session end (any stopping point):**
4. Call `mcp_engram_session_end(summary="<decisions, files changed, next steps>")` 
   — Without this, the session is permanently lost to future agents

**During work:**
- Before any architectural question: `mcp_engram_recall("<keywords>", k=5)` first
- Before opening any file: `mcp_engram_context_for_file(path)` first
- After any confirmed fix: `mcp_engram_remember_solution(error, solution)`
- After any failed approach: `mcp_engram_scar("concept_name")`
- Never call `forget` + `remember` to update — always use `mcp_engram_update`

**Recall tip:** Engram uses BLAKE3 lexical encoding, not neural semantic search.
Write queries using exact words from the target text, not paraphrases.
```

---

## IDE-Specific Setup

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "~/.engram/stalks/"],
      "env": {
        "ENGRAM_EMBED_URL": "http://localhost:8086/v1/embeddings"
      }
    }
  }
}
```

Add to your Claude project's **Custom Instructions**:

```
You have Engram memory connected. At the start of every conversation: call 
mcp_engram_session_start, watch_workspace, and summarize. At the end: always 
call session_end with a full summary of decisions and next steps. Between those: 
recall before you derive, update instead of forget+remember, scar failed approaches.
```

---

### Cursor

Add to `.cursorrules` in your project root:

```markdown
# Engram Memory Rules

At session start: mcp_engram_session_start → watch_workspace → summarize
During work: recall before grep, context_for_file before view_file
At session end: ALWAYS mcp_engram_session_end with full summary

Recall uses BLAKE3 lexical encoding — use exact keyword clusters, not semantic paraphrases.
Never forget+remember — always mcp_engram_update for existing concepts.
```

---

### VS Code (GitHub Copilot)

Add to `.github/copilot-instructions.md` in your repo:

```markdown
## Engram Memory Protocol

This project uses Engram for persistent agent memory (MCP tools available).

Start every session:
1. `mcp_engram_session_start(intent=...)`
2. `mcp_engram_watch_workspace("/path/to/this/repo")`  
3. `mcp_engram_summarize(top_n=10)`

End every session:
- `mcp_engram_session_end(summary=...)` — include decisions, files changed, next steps

During work:
- Recall before grep: `mcp_engram_recall("exact keywords from target text")`
- Before editing a file: `mcp_engram_context_for_file(path)`
- Confirmed fixes: `mcp_engram_remember_solution(error, solution)`
- Failed approaches: `mcp_engram_scar(concept)`
```

---

### Google Antigravity IDE

Add to your `mcp_config.json`:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "~/.engram/stalks/"],
      "env": {
        "ENGRAM_EMBED_URL": "http://localhost:8086/v1/embeddings"
      }
    }
  }
}
```

Then add a workflow file at `.agent/workflows/wake_up.md` pointing to the
workflow templates in this `integrations/workflows/` directory.

The ki_hijacker bridge automatically writes context to:
`~/.gemini/antigravity/knowledge/active_engram_context/artifacts/context.md`

This file is read by Antigravity at session start without any explicit tool call — your agent wakes up with memory already in its context window.

---

## The KI Context Path by IDE

The ki_hijacker needs to know where your IDE reads Knowledge Items from. Set `ENGRAM_KI_PATH` to override the default:

| IDE | Default KI Path | Override Env Var |
|---|---|---|
| Google Antigravity | `~/.gemini/antigravity/knowledge/` | `ENGRAM_KI_PATH` |
| Claude Desktop | Manual system prompt injection | Use `mcp_engram_summarize` at start |
| Cursor | `.cursor/rules/` (write context.md there) | `ENGRAM_KI_PATH` |
| VS Code | `.github/` | `ENGRAM_KI_PATH` |

For IDEs without native KI injection, the `mcp_engram_session_start` call returns the same content that would appear in the KI — treat its output as your context hydration.
