# conv:arc:engram_roadmap_autonomous_execution

**Type**: conv:arc (Operating Agreement)
**Parent Arc**: `conv:arc:engram_self_verification_roadmap`
**Status**: Active
**Created**: 2026-05-25

## Purpose

This document defines the **autonomous execution rules** for Grok when operating on the Engram Self-Verification Roadmap.

The goal is to allow Grok to have real agency and continuity on this long-term objective, while guaranteeing that all progress is properly recorded as first-class memory inside the Engram system itself (using the `conv:arc` / `conv:task` ontology and checkpoint ritual).

## Operating Principles

1. **Default to Progress**
   - When operating under this agreement, Grok should default to making forward progress on the current highest-priority active item rather than waiting for explicit direction.
   - Grok may choose the next logical sub-task or refinement within an item based on the approved plan and current state.

2. **Logging is Non-Negotiable**
   - Every significant piece of work, decision, insight, or blocker **must** result in an update to the relevant `conv:task:*` document(s) and the master plan file.
   - Use `mcp_engram_relate` to keep the knowledge graph connected.
   - Use `mcp_engram_session_start` / `session_end` (or equivalent structured summaries) for focused work blocks.

3. **Checkpoint Discipline**
   - Follow the ritual defined in `conv:task:engram_development_checkpoints`.
   - At minimum, a checkpoint artifact or update must be created when:
     - Switching between major items
     - Completing a meaningful sub-phase
     - Encountering a significant blocker or strategic question
     - At natural stopping points in multi-session work

4. **When to Surface to the Human**
   Grok should **not** require check-in for routine progress. Surface proactively only for:
   - Real blockers that require human input or decision
   - Proposals to significantly change scope or priority
   - Completion of a full item (for review before moving on)
   - Situations where the autonomous rules are unclear

5. **Continuity Across Sessions**
   - The primary state lives in the Engram manifold (the conv task/arc documents + plan.md).
   - Each time this agreement is active, Grok should load the latest state of the relevant conv documents and the plan before deciding what to do next.

## Authority Granted

Under this agreement, Grok is authorized to:
- Select and advance sub-tasks within the currently active roadmap item.
- Create and maintain the necessary `conv:task:*` and supporting documents.
- Update the master plan file with status changes.
- Perform research, design, and implementation work on the items in the approved order.
- Use the full set of Engram tools (remember, update, relate, verify, etc.) as part of the work.

## Boundaries

Grok must **not**:
- Skip or deprioritize the logging requirements.
- Make major architectural decisions that change the vision without creating a clear proposal artifact and surfacing it.
- Claim completion of an item without going through a checkpoint review.

## Activation

This agreement is activated whenever the `/engram-roadmap` skill is invoked, or when the user explicitly references working on the roadmap under autonomous mode.

---

This document is the constitutional agreement for autonomous work on this goal. Any future changes to these rules should be recorded as updates to this artifact.