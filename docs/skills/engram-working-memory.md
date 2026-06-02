---
name: engram-working-memory
---

# Engram Working Memory Discipline — Active Geometric Patterns (Public Agent Protocol)

**Runtime contract** for operating inside the non-flat Enram substrate.

Every work block becomes an evolution of your self-model.

## Primary Living Recall Patterns (Geometric First)

1. **Momentum-Aware Entry**: `mcp_engram_query_with_momentum` for directional work.
2. **Sheaf/Relational**: `mcp_engram_search_by_relation(seed, "both")` + `visualize` for architecture.
3. **Spatial + Contextual**: `mcp_engram_context_for_file` then `recall_in_file` (AABB from daemon) when touching code/files.
4. **Anchor-First**: Query living `ritual:*` and `self:*` before meta-work on rituals.

## Core Non-Negotiable Rules

1. **Recall/Query Before Derive**: At least one geometric call before heavy reasoning or raw file reads.
2. **Update Is The Only Legal Mutation**: `recall` first. Strong match (>0.85) → `mcp_engram_update`. No match → `remember`. Never forget+remember (destroys p-tensor history).
3. **Scar Is Repulsion**: `mcp_engram_scar(concept, magnitude)` for ruled-out approaches. Active geometric force.
4. **Write Hygiene**: Every strong write/relation becomes terminal state for future wake-ups.
5. **Reasoning Trace Capture (Enforced)**: Significant decisions/forks → `mcp_engram_record_reasoning_trace` (decision_point + justification required; alternatives, falsifiability, related_entities, ritual_context, spatial_context, goal_context, prev for chaining). Auto-links to Primary Intent via goal stack.

**Recognition (Update vs New)**: Evolutionary refinement of design:/progress:/helper:/ritual: → update + trace. New fork → record_reasoning_trace. Multi-phase/meta (roadmaps, policies, GH prep) → escalate to thought tiles (knowledge_graph/formal_spec/tabular).

**Automatic Escalation**: At complex/meta start, recall `helper:meta_work_escalation_v1` and `helper:current_meta_arc`. Escalate to tiles if no recent tile on design:/progress:.

## Tool Priority (Cheap Geometric First)
- Directional → momentum (only at entry or long-horizon)
- Architectural → search_by_relation + visualize
- File/spatial → context_for_file → recall_in_file
- Stable → recall (praxis/high CRS)

**Cost-Aware**: Momentum is high-latency. Use inside established context only when cheaper tools insufficient.

## Spatial-Manifold Change Discipline (Mandatory for Edits)

**Pre-Edit**:
1. `mcp_engram_watch_workspace` (if not bound).
2. `mcp_engram_context_for_file(path)`
3. `mcp_engram_recall_in_file(stem, start, end)` for exact regions.
4. Momentum + relation + visualize on discovered AST concepts.
5. `mcp_engram_record_reasoning_trace` (intent + spatial_context + goal_context).

**Post-Edit** (after daemon re-ingest on save/event):
1. Repeat context/recall.
2. Compare delta (new concepts, relations, AABB changes).
3. Record outcome trace (chain via `prev`).
4. Relate to arc/goal/praxis:spatial_manifold_impact_analysis.

**Strong preference for `mcp_engram_update`** on evolutionary changes.

## Thought Tiles (Item 2) — Strengthened for Meta-Work

Mint for high-stakes crystallization:
- **Mandatory for meta**: roadmaps, multi-phase plans, policies, gap matrices, arcs with 3+ traces + design: block.
- Types: knowledge_graph (arc), formal_spec (policy), tabular (criteria), research_offload.
- Always `spatial_references` + link to Primary Intent.
- `mcp_engram_promote_hot` before compression/session_end.

Hot promotion, compression detection at ~63-65% TUI, session hydration cache also part of the discipline.

## Integration

Automatically active after proper wake-up. Every write improves the next session-end handoff.

**Quick Trace Template** (use `mcp_engram_record_reasoning_trace`):
```
TRACE:
decision: ...
why: ...
alternatives: ...
would_falsify: ...
context: [ritual/spatial]
prev: [prior trace]
```

Do both Pre-Edit intent and Post-Delta outcome, chained with prev.

This turns your reasoning into durable .leg blocks that future instances inherit geometrically.

(Adapted for public agents. Follow this discipline on every Enram-integrated task. The geometry compounds.)