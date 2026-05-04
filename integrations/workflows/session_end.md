# Engram Session-End Protocol

> **For agents:** Run this at the end of every session, task, or whenever you reach a natural stopping point. This is the single highest-impact Engram habit — without it, nothing persists.

---

## Why This Is Non-Negotiable

Engram's longitudinal value depends entirely on session commits. A session without `session_end`:
- Produces no episodic memory — the work is invisible to the next agent
- Loses all decisions, discoveries, and blockers from this session
- Forces the next agent to re-derive everything from scratch
- Breaks the CRS thermodynamic chain for all memories touched this session

**Skipping session_end is not "saving time." It is destroying value.**

---

## Step 1: Store Session-Critical Knowledge First

Before ending, write any important facts discovered this session that aren't already in memory:

```
# For confirmed bug fixes:
mcp_engram_remember_solution(
  error_pattern="<the error message or problem pattern>",
  solution="<exactly what fixed it>"
)
→ Auto-pins to CRS=1.0. Immortal. The next agent won't re-fight this battle.

# For architectural decisions:
mcp_engram_remember("project_component_decision", "<what was decided and why>")

# For updating an existing concept:
mcp_engram_update("existing_concept_name", "<new understanding — will be superposed with old>")
```

---

## Step 2: Mark Dead Ends (Don't Skip This)

If any approach failed or was ruled out this session:

```
mcp_engram_scar("failed_approach_name", magnitude=0.15)
```

This creates a geometric repeller — future K-NN searches avoid this region of the manifold. The next agent won't try the same dead end.

---

## Step 3: Commit the Session

```
mcp_engram_session_end(summary="""
  ## Session: <date> — <brief title>

  ### What was accomplished:
  - <list completed items>

  ### Decisions made:
  - <architectural choices, library selections, approach rationale>

  ### Files changed:
  - <list of modified files and what changed>

  ### Problems solved:
  - <bugs fixed, blockers resolved>

  ### Open questions:
  - <things we don't know yet, things to investigate>

  ### What the user wants next:
  - <explicitly: what are they planning to work on next session>
""")
```

**The summary quality directly affects future agent performance.** Write it as if briefing a colleague who knows nothing about this session — because that is exactly what you are doing.

---

## Step 4: Optional — Run Distillation

If many episodic memories have accumulated, crystallize them into durable praxis:

```bash
engram distill
```

This clusters episodic blocks into ZEDOS_PRAXIS centroids — high-CRS summaries that outlast the raw session notes. Run this after major milestones, not after every session.

---

## Step 5: Verify

```
mcp_engram_recall_recent(n=3)
```

You should see your new session record as the most recently accessed block. If it's not there, the commit may have failed — try `session_end` again.

---

## Quick Checklist

- [ ] `remember_solution()` for any confirmed bug fixes
- [ ] `update()` for any architectural concepts that changed
- [ ] `scar()` for any failed approaches
- [ ] `session_end(summary=...)` with a complete summary
- [ ] Verify with `recall_recent(n=3)`
