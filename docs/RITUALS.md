# Engram Rituals

Rituals turn the geometric substrate into living self-model and continuity for agents.

**Dogfooding:** When the agent uses these rituals (and the underlying mcp_engram_remember/relate/record_reasoning_trace/update/goal/scar/verify/spatial + thought_tile tools) *on its own work and decisions*, the activity becomes first-class persistent geometry (traces, relations, CRS evolution, structured tiles for compression). The GitHub MVP prep work was dogfooded this way (with 2026-06 correction pass for better update + tile usage) — see GITHUB_MVP_PREP_PLAN.md execution log, tiles, and traces. This is "eating your own dog food" on a non-flat memory system.

**Recognition for update and tiles + automatic escalation (ritual process update 2026-06):** See working-memory "Recognition Heuristics" and "Automatic Escalation", thought-tiles "Recognition triggers" and "Expected for Re-hydration". In practice: recall `helper:meta_work_escalation_v1` + `helper:current_meta_arc` for meta arcs; ki/wake/session/subvisor auto prompt for tile/update; tiles expected (not optional) for bundles/re-hydration. Reconcile step via helper:reconcile_step_v1. Full plan/execution in GITHUB_MVP_PREP_PLAN.md. Subvisor enhanced for H1 meta detection.

## Core Rituals (Skills)

- **engram-wake-up**: Geometric continuation. Query anchors (ritual:*, goal:*, trace:*, process:engram.*), session_start with intent, bind agent_instance_continuation, Phase 1.5 lawfulness metric (verify, genesis, spatial, ki), rehydrate via momentum/relations/hot path/Legominism (handoff compresses_path), surface goal stack + Primary Intent, activate working-memory, spatial hygiene (watch, context, item1.5), record trace.

- **engram-working-memory**: Momentum-aware entry (query_with_momentum for directional), relational/sheaf first (search_by_relation + visualize), spatial/contextual (context_for_file + recall_in_file), anchor-first. Recall before derive. Update (not forget+remember). Scar immediately on ruled-out. record_reasoning_trace (A/D/R + goal/spatial/ritual_context + prev). Expensive tool hygiene (narrow prompts, scaffolding before broad momentum). Code Edit Ritual: pre recon + trace, update-prefer, post delta + trace + relate. Hot promotion. Cost-aware.

- **engram-session-end**: Crystallize (remember_solution, update, scar, batch). Trace chain summary + goal review/demotion traces (completes_goal). Spatial check. COMPRESS markers. mcp_engram_session_end (prepare_compression). Advance anchors (ritual:session_end_anchor, agent_instance_terminal / provides_continuation_for). Verify (recall_recent, verify_block_lawfulness). Signal distillation.

- **engram-goal**: Explicit goal stack as geometric (q/p). Lifecycle, execution linking (traces serve goals), visibility. Primary intent marker.

- **engram-thought-tiles**: Mint tiles (research_offload, formal_spec, etc.) with provenance, compresses_path, hot promotion. Continuation bundle consumption.

- Others: engram-harness-gate (MCP test + pre-binary-swap), engram-lawfulness-metrics (wake-up metrics + trend), engram-substrate-cs (6-phase roadmap), engram-gpu-bvh, engram-substrate-edit (for Rust substrate with spatial + lawfulness).

## Process Architecture Sheaf

Declarative processes/*.toml (two-level naming agent:engram.<type>.<domain>-<action>) registered dynamically at session_start via load_process_sheaf in mcp.rs. Category table (object, morphism OP_*, sheaf_role, h1_handler). Gluing/H¹ for subvisor (monitor for sub-agent governance: narrow one-shot, loop detect via H¹ on tool graph, geometric enforce, scar repetitive).

See processes/ (7 tomls), design/processes/, mcp.rs:90, prior sheaf execution traces.

## Code Edit Ritual (v1)

Mandatory for substrate changes (crates/, skills/, etc.):
1. Pre: watch_workspace, context_for_file, recall_in_file, momentum/relation on AST nodes, intent trace (decision, why, spatial_context, goal_context).
2. Edit (update-prefer).
3. Post: re-context, delta trace (chained prev), relate to goal/arc/praxis:spatial_manifold_impact_analysis, scar if needed.

## Governance for Sub-agents

Narrow one-shot only (single action, mcp geometric first, Primary Objective + negative examples in prompt, "report to supervisor"). Kill on loop/stagnation. Main high-cognition. Subvisor process (OP_INVERT + H¹) for oversight. See scar for past doom loops, subvisor.toml.

## Lawfulness & Metrics

verify_manifold_integrity, verify_block_lawfulness, genesis, spatial_status, ki freshness. metric:wake_up_verification_<iso> + trend. overall_lawful + score.

See SKILL.md files in .grok/skills/, GITHUB_MVP_PREP_PLAN.md, 2026-06_Substrate_CS_Gap_Closure_Roadmap.md, engram-wake-up/SKILL.md (Phase 1.5), working-memory/SKILL.md (Code Edit + hygiene).

For external agents: follow rituals for lawful use of the substrate.