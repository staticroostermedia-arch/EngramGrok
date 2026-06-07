# Engram + OpenAI Codex / CLI Agents

Codex and similar CLI agents connect via MCP the same way as Cursor or Claude Desktop.

## MCP config

Add the `engram` block from [../mcp.engram.template.json](../mcp.engram.template.json):

```json
{
  "mcpServers": {
    "engram": {
      "command": "/path/to/Engram/scripts/engram-grok",
      "args": ["mcp"],
      "env": {
        "ENGRAM_STORE": "~/.engram/stalks/",
        "ENGRAM_PROFILE": "agent"
      }
    }
  }
}
```

Build first: `cargo build -p engram-server` in the Engram repo.

## Agent instructions

Point Codex at:

- [docs/AGENT_MEMORY_CONTRACT.md](../../docs/AGENT_MEMORY_CONTRACT.md)
- [SKILLS.md](../../SKILLS.md)

**Mandatory loop:** `session_start` → work with `context_for_edit` / `recall(anchors)` → `session_end`.

## MCP discipline

Always `search_tool` then `use_tool` with exact schema (Codex harness pattern). Never guess parameter names.