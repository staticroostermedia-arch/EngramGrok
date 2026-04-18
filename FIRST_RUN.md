# Engram — First Run Guide

> **For new users and AI agents.** Run through this guide exactly once when you first install Engram.
> After completing it, your manifold is seeded, your daemon is watching your workspace,
> and every tool in the MCP server is verified working.

---

## 1. Install the Binary

```bash
# Clone and install from source (recommended — gets the latest physics loop)
git clone https://github.com/staticroostermedia-arch/engram.git
cd engram
cargo install --path crates/engram-server

# Verify:
engram --version
# engram-server 0.4.x
```

---

## 2. Configure Your MCP Client

Add Engram to your IDE's MCP config. In **Antigravity / VS Code**, this is `.antigravity/mcp.json`:

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

> **Store location:** `~/.engram/stalks/` is the default. Each project namespace (stalk)
> gets a subdirectory. You can override with `ENGRAM_STORE` env var.

---

## 3. (Optional but Recommended) Wire a Neural Embedding Server

Engram works out of the box with a pure BLAKE3 hash encoder — no network required.
But if you run a local embedding server (llama.cpp, ONNX MiniLM, nomic-embed), semantic
recall quality improves dramatically.

```bash
# Tell Engram where to find the embedding server:
export ENGRAM_EMBED_URL="http://localhost:8086/v1/embeddings"

# Add to ~/.bashrc to make it permanent:
echo 'export ENGRAM_EMBED_URL="http://localhost:8086/v1/embeddings"' >> ~/.bashrc

# Verify the embedding server is alive:
curl -sf http://localhost:8086/health && echo "✓ Embedding server ready" || echo "⚠ Not running — hash encoder will be used as fallback"
```

> **Without an embedding server:** Engram falls back to BLAKE3 spiral-phase encoding.
> Recall works, but semantic similarity is weaker for paraphrased queries.
> The fallback is completely automatic — nothing breaks.

---

## 4. Store Your First Memory (Verify MCP is Working)

In your AI agent (or via the CLI), run:

```
remember("first_run_test", "Engram is working. This is my first memory block.")
```

Or via CLI:
```bash
engram --store ~/.engram/stalks/ remember first_run_test "Engram is working. This is my first memory block."
```

Expected output:
```
✓ Stored memory: 'first_run_test' (52 chars)
```

Then retrieve it:
```
recall("working first memory", k=3)
```

You should see `first_run_test` in the results with a score > 0.5.

---

## 5. ⚠️ Activate the File Watcher Daemon (Critical — Don't Skip)

**This is the most commonly missed step.**

The Engram MCP server spawns a background daemon that auto-ingests files into memory whenever
you save them. But it **watches nothing until you tell it what to watch.** Without this step:

- `mcp_engram_recall_in_file` returns no results
- AST spatial coordinates (AABB line ranges) are never populated
- Code-aware spatial search doesn't work

**Fix: tell the daemon which directories to watch:**

```
mcp_engram_watch_workspace("/path/to/your/project")
```

Run this for every project you work in. For example:

```
mcp_engram_watch_workspace("/home/user/Documents/MyProject")
mcp_engram_watch_workspace("/home/user/Documents/AnotherProject")
```

Expected output:
```
✓ Agentic Daemon now recursively watching: /home/user/Documents/MyProject
```

> **Make this permanent:** Add `mcp_engram_watch_workspace` calls to your IDE's session
> start / wake-up workflow so they run automatically at the start of every session.
> The daemon does not persist watch state across server restarts.

---

## 6. Ingest Your Workspace (Seed the Manifold)

The file watcher only picks up files you save *after* it starts watching. To ingest your
existing codebase right now:

```bash
# Ingest a project directory (AST extraction for .rs, .py, .ts, etc. + chunking for everything else)
engram ingest /path/to/your/project

# Example:
engram ingest /home/user/Documents/MyProject/src

# Expected output:
#   > Starting Engram Ingest: /home/user/Documents/MyProject/src
#   [ast] my_project__fn__main              ← pub fn main (lines 1-50)
#   [ast] my_project__struct__Config        ← pub struct Config (lines 52-80)
#   ...
#   ✓ INGESTION COMPLETE
#     Files processed : 42
#     AST items minted: 156  (one block per pub fn/struct/enum/trait)
#     Chunk blocks    : 23   (non-code files)
#     Total blocks    : 179
```

After ingestion, test spatial recall:

```
mcp_engram_recall_in_file("main", start_line=0, end_line=100)
```

You should see all AST concepts defined in `main.rs` within lines 0–100.

---

## 7. Verify the Full Physics Loop

Run these three checks to confirm all Phase 8 features are active:

### Check 1: ZEDOS tag filtering works
```
recall("crystallized solutions", k=5, zedos_filter="praxis")
```
→ Should return only PRAXIS-tagged blocks (or "No memories found" if manifold is empty — that's OK)

### Check 2: Momentum search works
```
mcp_engram_query_with_momentum("your project topic here", k=5)
```
→ Should return `Momentum-weighted results for '...'` with `(momentum score: X.XXX, drift: X.XXX)`

### Check 3: Composite scoring is active
```
recall("any query", k=3)
```
→ Result lines should show `(score: X.XXX, crs: X.XXX, dv: X.XXX, depth: N, tag: DECLARATIVE)`
→ If you see the `dv:` and `depth:` fields, composite scoring is live

---

## 8. Genesis Blocks (Auto-Seeded)

On first boot, Engram seeds alignment genesis blocks — foundational PRAXIS memories that
define the system's operational context. These are pinned at CRS=1.0 and never decay.

Check they exist:
```
mcp_engram_genesis(action="status")
```

If missing:
```
mcp_engram_genesis(action="reseed")
```

---

## 9. Namespaces (Multi-Project Setup)

Engram supports isolated namespaces (stalks) so different projects don't pollute each other:

```
mcp_engram_set_namespace("my_project")    # creates + switches to this namespace
mcp_engram_set_namespace("work_project")  # switch to another project's memory

mcp_engram_list_namespaces()              # see all namespaces
```

> **Default namespace:** `default`. All memories without an explicit namespace go here.

---

## 10. Session End (Epistemic State Tracking)

At the end of a work session, call:

```
mcp_engram_session_end("One-sentence summary of what we accomplished today.")
```

This calculates the aggregate CRS of your session's work and records the epistemic state
(confidence vs. frustration ratio) as a PRAXIS block. Useful for understanding which sessions
were productive vs. stuck.

---

## Quick Reference: The Tools That Matter

| Tool | When to Use |
|------|-------------|
| `remember(concept, text)` | Save a fact, decision, or code pattern |
| `recall(query, k)` | Find semantically similar memories |
| `recall(query, k, zedos_filter="praxis")` | Only crystallized solutions |
| `mcp_engram_query_with_momentum(query)` | Find concepts *trending toward* your query |
| `mcp_engram_recall_in_file(file_stem, start_line, end_line)` | Spatial code search by file + line range |
| `mcp_engram_watch_workspace(path)` | **Activate the daemon** (do this every session) |
| `mcp_engram_update(concept, new_text)` | Update a stale memory (triggers drift tracking) |
| `mcp_engram_scar(concept, magnitude)` | Mark a failed approach as hostile |
| `mcp_engram_pin(concept)` | Prevent a concept from ever decaying |
| `mcp_engram_session_end(summary)` | Record session epistemic state |
| `mcp_engram_genesis(action="status")` | Check alignment anchor blocks |

---

## Common Failure Modes

| Symptom | Cause | Fix |
|---------|-------|-----|
| `recall_in_file` returns nothing | Daemon not watching workspace | Call `mcp_engram_watch_workspace(path)` |
| Low recall quality | No embedding server | Set `ENGRAM_EMBED_URL` + start llama.cpp |
| `recall` returns everything with score ~0.85 | CRS weighting compressed range | Normal — composite scoring keeps cosine dominant |
| Memories accumulate too fast | No autophagy threshold set | Run `mcp_engram_forget_old(min_crs_threshold=0.20)` periodically |
| AABB coordinates all zero | Files ingested before daemon was watching | Re-run `engram ingest /your/project` |

---

*First-run guide complete. Your manifold is ready.*
