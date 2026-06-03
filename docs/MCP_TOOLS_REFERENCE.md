# MCP Tools Reference (55+ Tools)

Engram exposes a rich set of tools over MCP for geometric memory operations.

## Core Memory
- remember, recall (with zedos_filter, time_decay), forget, list_concepts, read_concept, batch_remember.

## Management
- update (preferred for evolution, Lyapunov drift), pin, stats, recall_recent, summarize, forget_old, export/import.

## Workspace & Spatial (Item 1.5)
- watch_workspace (mandatory first turn), context_for_file (pre-edit recon), recall_in_file (AABB spatial), force_spatial_ingest, spatial_status.

## Session & Rituals
- session_start (mandatory, intent, binds continuation, loads process sheaf), session_end (mandatory, summary with COMPRESS, prepare_compression for handoff/hot promote).
- scar, remember_solution.
- record_reasoning_trace (A/D/R, goal/spatial/ritual_context, prev).

## Graph / Sheaf
- relate (OP_BIND), search_by_relation, visualize (Mermaid).

## Physics / Lawfulness / Autonomy
- genesis (status/reseed), verify_manifold_integrity, verify_block_lawfulness, verify_behavior.
- query_with_momentum (80% q + 20% p), set_namespace, list_namespaces.
- set_geosphere_frame, get_geosphere_frame, clear_geosphere_frame.
- goal_create, goal_decompose, goal_status, goal_list, goal_update_status, goal_search, goal_set_primary, goal_get_children.

## Thought Tiles & Viz
- thought_tile_create (research_offload, formal_spec, state_machine, etc., auto goal link, compresses_path), thought_tile_write_result, thought_tile_create_visualization.

## Process Sheaf (declarative)
- Dynamic load at session_start from processes/*.toml (ritual/wake-up, nrem-consolidation, harness/spatial-recon, operator/momentum-query, process/session-end, monitor/manifold-health + subvisor for H¹ sub-agent governance).

See also: integrations/ for configs, tools/test-harness, engram-wake-up/SKILL.md (Phase 1.5 metrics), GITHUB_MVP_PREP_PLAN.md, processes/ tomls (category gluing/H¹), mcp.rs (load_process_sheaf).

For external agents: prefix concepts, use session_start/end, follow rituals for lawful continuity.