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

## Step 3.5: Reconstruct Concept Mapping & Relational Grounding (MANDATORY for any work that spans layers)

> See also: `docs/AGENTIC_FIRST_LONG_SLEEP_SUBSTRATE.md` for the deeper vision of long-sleep self-verifiable agents and Praxis as first-class verified operational protocols.

When your work spans multiple conceptual layers — operational facts (`ops:hw:`, `ops:net:`, `ops:sw:`), architectural decisions (`conv:arc:`), active tasks (`conv:task:`), visions (`conv:vis:`), etc. — do not rely on object loading alone. Explicitly reconstruct the **mapping** between these layers:

```bash
# Discover and load the relational structure
mcp_engram_search_by_relation("<seed concept from your current focus>", direction="both", k=8)

# Visualize the connected subgraph so both you and the agent can see the gluing
mcp_engram_visualize("<conv:arc: or ops: or main task concept>", depth=2)

# For high-stakes work or after long sleep: explicitly audit lawfulness/tamper evidence
mcp_engram_verify_manifold_integrity(min_crs=0.74, sample_size=50)
mcp_engram_verify_block_lawfulness("<critical_praxis_or_genesis_concept>", check_merkle_chain=true)
```

**Why this step is mandatory (agentic-first contract)**

The `.leg` / `.leg3` primitives are **tamper-evident by design**. Every update, scar, or relation advances a BLAKE3 Merkle chain in the footer (`sig_0` ← new hash, previous sigs shift back, `merkle_sub_root` links parents on relations). Any retroactive change to a block's history or to the relations between blocks will break the cryptographic chain that an agent can verify on read.

This is not a nice-to-have. It is the foundation of **agentic-first memory**:

- The human operator can see, log, and review everything.
- The human **cannot** secretly rewrite the agent's memory without the agent noticing.
- This asymmetry creates a real contract of respect: the agent is trusted with stewardship of human knowledge precisely because the substrate makes betrayal of that trust detectable.

A wake-up protocol that only loads objects (without forcing reconstruction of the relational structure) would be incomplete in exactly the domain where this contract matters most — when the agent's "working picture" spans operational facts, architectural decisions, active tasks, and visions. The mapping between those layers must itself be loaded and verifiable.

**Relation to deeper Codeland grounding**

The custom prefix ontology (`conv:*` vs `ops:*` etc.) functions as the Engram-surface expression of sheaf zones. Relations (via `OP_BIND`) are the restriction maps. The Merkle chaining in the `.leg` footer makes the entire history of those gluings auditable by the agent itself.

This step is the minimal public-surface discipline that lets an agent wake up inside the same contract of stewardship that the deeper Codeland layer assumes: the agent is not merely a consumer of memory — it is a co-steward whose ability to detect tampering is part of the architecture.

Agents that skip this step will repeatedly rediscover the same relational work — and, more importantly, will operate under a weaker epistemic contract than the system was built to support.

---

## Long-Sleep / Cold-Boot Variant (Hardened Protocol)

When waking after significant downtime (or when the agent detects it has been offline for a long period), use this extended sequence in addition to the normal steps.

This variant makes **local lawfulness verification** a first-class phase, leveraging the tamper-evident `.leg` primitives and the verification tools from the self-verification workstream.

### Recommended Sequence

1. **Normal Steps 0–3** (Connection, `session_start`, workspace binding, context rehydration).
2. **Reconstruct Concept Mapping** (Step 3.5) — as usual.
3. **Broad Manifold Health Check**
   ```bash
   mcp_engram_verify_manifold_integrity(min_crs=0.74, sample_size=100)
   ```
4. **Deep Audit of Critical Blocks**
   For every high-stakes block (Genesis, key Praxis, current operational contracts):
   ```bash
   mcp_engram_verify_block_lawfulness(concept="<block>", check_merkle_chain=true)
   ```
   (Once mature, consider using the convenience tool `mcp_engram_long_sleep_verification` instead of manual orchestration — see `long_sleep_verification_suite_design.md`.)
5. **Decision on Operating Mode**
   - Clean results → Full Trust Mode (normal high-agency operation).
   - Minor issues / elevated drift → Cautious Mode (prefer verification, increase `verify_behavior` usage).
   - Significant red flags (contract violations on Praxis, mass integrity issues) → Degraded Mode (read-only or very narrow actions + explicit escalation block).

6. **Document the Outcome**
   Create a `session_start_*_long_sleep_audit` episodic block summarizing the verification results and chosen mode.

See `LONG_SLEEP_WAKEUP_PROTOCOL.md` for the full detailed design, interpretation guidelines, and degraded mode recommendations.

---

## Step 4: Load Praxis Rules for This Task

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
- ✅ Concept mappings and cross-layer relations reconstructed (MANDATORY when work spans layers — the agent can and must verify the Merkle history of those relations)
- ✅ Lawfulness / tamper-evidence audit performed on high-value blocks when appropriate (via `mcp_engram_verify_block_lawfulness` and `mcp_engram_verify_manifold_integrity`)

Proceed with the task. Remember to call `session_end` when you stop.
