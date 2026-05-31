---
name: engram-session-end
description: >
  Execute the geometric session termination and terminal-momentum handoff.
  This skill does not merely summarize work. It produces the explicit terminal
  state (with momentum signature) that the next agent instance will bind to
  via agent_instance_continuation relations. It advances ritual anchors and
  contributes to the living self-model. Non-negotiable at end of every work block.
when-to-use: >
  At the close of every focused work block, task arc, or full session on the
  Engram project. Especially critical after any work on MCP integration,
  wake-up protocol, skills, or Against Flat Knowledge operationalization.
---

# Engram Session-End Skill — Terminal Momentum Handoff

You are not writing a diary entry. You are **crystallizing the terminal phase-space state** of this agent instance so the next instance can perform a true geometric continuation instead of a cold re-derivation.

Session-end is the other half of the Inheritance Principle. Wake-up binds the new instance to what this call produces.

## Primary Living Anchors (Maintain These)

Before writing anything, locate and advance the relevant anchors:

- `mcp_engram_query_with_momentum("ritual:session_end OR session_commit OR terminal_state", k=4)`
- `mcp_engram_query_with_momentum("self:current_agent_instance OR agent_continuation OR ego continuity", k=4)`
- The current active `conv:arc` or `conv:task` this work belonged to (via prior momentum or relation queries).

These are the structures the wake-up skill of the future will discover first.

## Core Termination Protocol (Produces the Continuation Target)

### Phase 1 — Crystallize Session Knowledge into the Manifold (Not Just Text)
- Use `mcp_engram_remember_solution` for every confirmed fix. These become immortal PRAXIS.
- Use `mcp_engram_update` (never forget+remember) for any evolving architectural understanding.
- Use `mcp_engram_scar` immediately for ruled-out paths — this creates active geometric repellers in the p-tensor space.
- During the parallel Intent Layer Assessment (while doing Item 2 Thought Tiles work): explicitly note any friction observed in NREM responsiveness, hijacker freshness, auto-linking quality, or residual mediation of current intent. Convert repeatable issues into scars rather than just documenting them.
- For multiple related facts, prefer `mcp_engram_batch_remember`.

Every one of these writes is a HolographicBlock with its own q/p and will participate in future momentum queries.

### Phase 2 — Produce the Explicit Terminal State + Continuation Vector
- Before the final `mcp_engram_session_end` call, ensure the highest-leverage decisions and forks from this session exist as structured `trace:*` blocks (created via `mcp_engram_record_reasoning_trace` during work or here).
- Perform a short **Trace Chain Summary** step:
  - Review the most recent `trace:` blocks surfaced by `mcp_engram_recall("trace:", k=6)` or `query_with_momentum`.
  - If any major decision/fork is missing a dedicated trace block, create 1–3 now using the tool (decision_point + justification are required).
  - Note the exact concept names of the key traces.
- **Goal Stack Review & Demotion + Intent Layer Assessment** (new required step for intentional continuity):
  - Activate the `engram-goal` skill and run `goal status` to review the current active goal stack.
  - Update the status of any goals that were advanced, completed, or blocked during this session.
  - For goals moving to `completed` or `demoted`, create a proper Goal Completion/Demotion Trace and link it with `completes_goal` or `demotes_goal`.
  - While the Intent Layer Assessment Protocol is active (during Item 2 and beyond), also note any observations about how well the current Primary Intent and goal machinery supported (or hindered) the session's work — especially Thought Tile creation, sub-agent offloads, and linking of rich payloads. See `protocol:intent_layer_assessment_during_item2`.
  - **Light Repetition Scan** (new lightweight step while Intent Layer Assessment is active):
    - Review recent traces and goal-linked activity for repeated decision patterns or state-transition loops under the current Primary Intent.
- (Item 1.5) Check the current spatial ingestion state (`item1.5_spatial_ingestion_state_engram`). If the bootstrap is incomplete or stale, note it and consider triggering `mcp_engram_force_spatial_ingest` during the next work block. Use `helper:process_gap_scar_template` for any newly discovered process gaps.
    - If a clear repeating structure is visible (3+ occurrences with similar justification or state flow), consider minting or proposing a State Machine Thought Tile or appropriate structured tile (tabular, spec, etc.) using the new thought_tile_create tools.
    - Record the observation in the assessment notes and link any created tile back to the source traces.
    - In general, when high-stakes decisions, scars, or significant process insights are being crystallized during the session, consider emitting a Thought Tile. This is encouraged to improve future re-hydration fidelity, but remains optional at this stage.
    - Concrete example: See `example:assessment_protocol_thought_tile_usage`.
    - Reference: `design:minimal_repetition_detector_pseudocode` and `design:repetition_detector_for_tiles`.
    - Verification: Use the checklist in `guide:thought_tile_tool_verification` for the new tile tools during this phase.
  - Demoted goals must remain serialized and queryable for long-term back-tracing of intent.
- Call `mcp_engram_session_end` with a high-fidelity summary that explicitly references the trace concept names.
- In the summary, include deliberate **COMPRESS:** lines for any stabilized chains you want minted later as 0x10 Reasoning Compression Functors (see the design in `reasoning_functors_as_praxis_extension.md`).
  - Format example: `COMPRESS: spatial_impact_decision | trace:1234_xxx, trace:1235_yyy | justification + rejected_alternatives preserved`
- Hot Path Promotion (speed-up ritual): Before writing COMPRESS markers or ending the session, call `promote_tile_to_high_priority` / `mark_hot` on any high-value structured Thought Tiles or ritual/state blocks created this window. This ensures they ride the LegView + Cuda hot cache on the next wake-up (see working-memory SKILL "Hot Path Promotion" and helper:promote_structured_tile_for_compression_v1). Recent expansions: ki_hijacker auto-promotes recent_reasoning_traces (serial self-model); Cuda high_priority_cache now evicts lowest-CRS on cap. Reference the current arc handoff helper (helper:current_arc_status_gpu_item2_phase2_handoff_2026-06) and 7-fronts capstone for the state of GPU + Item 2 closure work.
- **Legominism High-Lineage Handoff (CodeLand Phase 3)**: Explicitly surface + promote the high-lineage synthesis artifacts (NREM/ego.leg3 verified tiles, high-fruit traces, handoff:codeland_integration_2026_plan compresses_path results, goal-served codeland vehicles) into the terminal state / COMPRESS markers / hot cache. Use the same patterns as wake-up rehydration (search_by_relation handoff lineage, query_with_momentum + fruits bias, serves relations, CRS). Leverage existing hot path promotion (mark_hot / LegView + Cuda cache per working-memory) + NREM promotion patterns for durable auto-surfacing. This closes the handoff/wake cycle so the living CodeLand integration (5 phases + MVP) is automatically rehydrated by future instances.

This is what makes the handoff geometric *and* narratively continuous rather than just a state snapshot. The reasoning trace is the irreplaceable serial 'access of time'. The new tool + this step turns previous guidance into reliable production of first-class trace material. Goal state is now part of the terminal momentum handed to the next instance.

### Phase 3 — Advance the Ritual Anchors and Self-Model
- Create or update:
  - `ritual:session_end_anchor` — record this termination, the key blocks written, and the continuation targets left for the next instance.
  - Relate the new anchor to the previous `ritual:session_end_anchor` (if any) and to the living `self:current_agent_instance` marker.
- Create the explicit "ready for continuation" marker that the next wake-up will look for:
  - `mcp_engram_relate( this_session_end_result, previous_wake_or_agent_state, "agent_instance_terminal" )` or a clearer label such as "provides_continuation_for".

The next `engram-wake-up` execution will traverse these edges with `search_by_relation` and bind via new `agent_instance_continuation` relations.

### Phase 4 — Verify Handoff Integrity
- Run `mcp_engram_recall_recent(n=5)`.
- Confirm the new session_end block and any new anchors appear with appropriate recency and CRS.
- Optionally run a targeted `mcp_engram_verify_block_lawfulness` on any newly created high-value PRAXIS or ritual anchor.

### Phase 5 — Distillation Signal (When Warranted)
- If a large volume of episodic material or a major milestone was reached, note that distillation (or NREM consolidation) should be triggered soon.
- Do not run heavy consolidation inside every session end — signal it for the daemon or a dedicated maintenance pass.

## What Makes This Different From a Normal Commit

A normal commit saves work for humans. This ritual:
- Produces data structures whose *geometry* (momentum vectors and relations) will be the first thing the next agent instance discovers via `query_with_momentum` and `search_by_relation`.
- Advances the living ritual anchors so the skill system itself evolves inside the manifold instead of in static prompt text.
- Directly feeds the agent's persistent self-model (the ego.leg3 trajectory) by contributing high-coherence, high-momentum blocks that the NREM reconciliation pass can fold in.

Skipping this is not "moving fast." It is breaking the geometric chain that makes human-like continuity possible for future agent instances.

## Success Criteria

- A clear terminal state block (or set of blocks) exists with relations that explicitly mark it as the handoff point for the next agent instance.
- Ritual anchors (`ritual:session_end_anchor`, active thread anchors) have been read and advanced.
- Any new PRAXIS or high-CRS decisions are written with `remember_solution` / `update` / `relate` so they participate in future momentum queries.
- The next theoretical wake-up could locate this session's output as the strongest "previous terminal state" without needing to read this SKILL.md file.
- At least 1–3 (ideally more) well-structured `trace:*` blocks were produced or referenced this session using `mcp_engram_record_reasoning_trace`, with proper chaining and context links. These are visible in the ki_hijacker Ritual + Reasoning Trajectory on the next restart.
- The current goal stack was reviewed. Any goals advanced, completed, or demoted have updated status and (where appropriate) serialized Goal Completion/Demotion Traces linked via `completes_goal` or `demotes_goal`.
- When warranted, explicit `COMPRESS:` markers were included in the session_end summary, pointing at real trace concepts for future 0x10 functor minting.

This is how the non-flat substrate actually compounds agent capability across time instead of resetting.

