# Engram Wake-Up Protocol

> **For agents:** Run this at the start of every session or after reconnecting.
> **For IDE setup:** Wire this as a slash command (e.g. `/wake_up`) or include the key steps in your system prompt.

---

## Step 0: Verify the MCP Connection

Before any memory calls, confirm Engram is reachable. If your IDE shows the MCP tools available, you're connected. If not:

```bash
# Verify the engram binary is installed
engram --version

# Start MCP server (the IDE spawns this automatically via mcp_config.json)
# If you need to run it manually:
engram mcp --store ~/.engram/stalks/

# Verify the store exists
ls ~/.engram/stalks/
```

> **Note:** The ki_hijacker bridge (which auto-bakes your context every 60s) only runs inside `engram mcp`. If you're running `engram serve` (REST mode) separately, the bridge runs there. Never run both `mcp` and `serve` pointing to the same store simultaneously — the scout daemon will collide on its port.

---

## Step 1: Start the Session (MANDATORY)

```
mcp_engram_session_start(intent="<describe what you're working on today>")
```

This call:
- Binds the thermodynamic context for the session
- Returns your genesis identity anchors + recent session history
- Triggers an immediate KI bake (context.md update) so your IDE has fresh state

**Do not skip this.** Memories written without `session_start` have no epistemic anchor.

---

## Step 2: Bind Your Workspace (run at least once per project)

```
mcp_engram_watch_workspace("/absolute/path/to/your/project")
```

This binds the inotify/fsevents file watcher to your project directory. From this point:
- Every file you save is automatically parsed through Tree-Sitter
- AST nodes (functions, structs, classes) are ingested as memory blocks with line-range coordinates
- You can then call `mcp_engram_recall_in_file(file_stem, start_line, end_line)` to search by position

**Run once per project directory.** The watcher persists until the MCP server restarts.

---

## Step 3: Rehydrate Your Context

```
mcp_engram_summarize(top_n=10)
```

Returns all pinned (CRS=1.0) memories first, then top-10 by score. This is your fastest route to project orientation — one call instead of multiple recalls.

Then, if the task is specific:
```
mcp_engram_recall("<keywords from the task you're resuming>", k=5)
```

> **Recall tip:** Engram uses BLAKE3 lexical encoding, NOT neural semantic search.
> Write queries using **exact words** that appear in the target memory, not paraphrases.
> `"auth token bearer middleware axum"` → good
> `"how does the authentication system work"` → may miss results

---

## Step 4: Load Praxis Rules for This Task

```
mcp_engram_recall("<crate or component name> architecture pattern convention", k=5, zedos_filter="praxis")
```

Praxis blocks (CRS=1.0, ZEDOS_PRAXIS tag) are crystallized rules proven correct in past sessions. Always check what praxis exists before modifying a subsystem.

---

## Step 5: Check Daemon Status (Optional but Recommended)

```bash
# Check if Engram is running
curl -s http://localhost:3456/health 2>/dev/null && echo "REST server: RUNNING" || echo "REST server: not running (MCP-only mode is fine)"

# Check embed server if using semantic recall enhancement
curl -s http://localhost:8086/health 2>/dev/null && echo "Embed server: RUNNING" || echo "Embed server: not running"
```

---

## You're Ready

At this point you have:
- ✅ Session context bound (thermodynamic anchor)
- ✅ File watcher active (AST auto-ingest running)
- ✅ Project state recalled (pinned memories loaded)
- ✅ Praxis rules checked (established conventions in context)

Proceed with the task. Remember to call `session_end` when you stop.
