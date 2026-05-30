# conv:task:engram_development_checkpoints

**Status**: Active Ritual Definition
**Type**: conv:task
**Parent Arc**: `conv:arc:engram_self_verification_roadmap`
**Created**: 2026-05-25

## Purpose

Define a lightweight but rigorous recurring checkpoint ritual for serious development work on Engram (especially this self-verification + long-sleep roadmap).

By using the system rigorously while building the system, we generate real high-signal data, discover usability gaps in practice, and create an auditable record of how the design evolved.

## Recommended Ritual

### At the Start of a Focused Work Block
- `mcp_engram_session_start(intent="Working on <specific subtask, e.g. implementing lawfulness verification primitives>")`
- Load the parent arc + current task documents
- If relevant: `mcp_engram_search_by_relation` or `mcp_engram_visualize` on the arc
- Create or update the specific `conv:task:*` for the work being done

### During the Work
- Record significant decisions, insights, and blockers as properly namespaced `conv:task:*` or `conv:arc:*` blocks
- Use `mcp_engram_relate` to connect new concepts back to the parent arc
- When testing a design hypothesis, use `mcp_engram_verify_behavior`

### At Natural Stopping Points (or End of Day)
- `mcp_engram_session_end(summary="...")` — must include:
  - What was accomplished
  - Key decisions (with links)
  - Open questions / risks
  - Next intended step
  - Any Praxis-worthy patterns observed

### Periodic (Every 3–5 Sessions or Major Milestone)
- `mcp_engram_summarize` focused on the roadmap arc
- `mcp_engram_visualize("conv:arc:engram_self_verification_roadmap", depth=2)`
- Light `mcp_engram_verify_manifold_integrity` or targeted block audits on high-CRS work

## Automation Path

This ritual can be made progressively more automatic:
- Small helper scripts / agent skills that enforce the steps
- Future daemon support for "development mode" checkpoints
- Explicit `dev:*` or `process:*` tagged blocks for process hygiene

## Why This Matters

This is a small but concrete demonstration of treating development work itself as "operational protocols" that can be remembered, related, crystallized, and later audited — exactly the pattern we want high-value Praxis blocks to support at scale.

---

**This task is considered active for the duration of the self-verification roadmap.** Any contributor (human or agent) should treat the above as expected hygiene.