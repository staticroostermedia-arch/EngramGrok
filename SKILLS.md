# Enram Agent Skills & Rituals

**For AI agents (Grok, Claude, custom, etc.) using the Enram MCP server:**

This is the canonical entry point for the published ritual protocols.

Load the files in `docs/skills/` (or symlink/copy them into your context):

- [docs/skills/README.md](docs/skills/README.md) — Overview, when-to-use, quickstart loop for external agents.
- [docs/skills/engram-wake-up.md](docs/skills/engram-wake-up.md) — Geometric continuation ritual (session_start, living anchors via momentum/relations, Phase 1.5 lawfulness, rehydrate, spatial hygiene, bind continuation).
- [docs/skills/engram-working-memory.md](docs/skills/engram-working-memory.md) — Runtime discipline (geometric first: momentum/relation/spatial, recall before derive, update-preferred, traces/scars/tiles, Code Edit Ritual with AABB, hot promotion, escalation for meta-work).
- [docs/skills/engram-session-end.md](docs/skills/engram-session-end.md) — Terminal handoff (crystallize traces/goals, COMPRESS, anchors, produce continuation target for next wake).
- [docs/skills/engram-thought-tiles.md](docs/skills/engram-thought-tiles.md) — Structured offload (mandatory for meta, promote_hot for re-hydration).

## Declarative Process Sheaf

The rituals are also defined as first-class processes in `processes/*.toml` (agent:engram.* names, gluing, H¹ handlers for subvisor etc.). Dynamically loaded at `session_start`.

See:
- `processes/ritual/wake-up.toml`, `session-end.toml`, etc.
- `processes/monitor/subvisor.toml` (governance).
- `docs/RITUALS.md` for full overview + Code Edit + sub-agent patterns.

## Examples

- `examples/hello-engram-agent.py` (tiny self-contained demo that loads skills + walks one full loop).
- `examples/mcp_client.py` (MCP usage with rituals).
- `examples/ritual_verify.md` (Code Edit + working-memory).
- `examples/spatial_geosphere_demo.py` (spatial passive + geosphere).
- `docs/examples/sub_agent_governance.md` (subvisor H¹, narrow prompts, escalation, doom-loop prevention).
- `docs/examples/item1_goal_trace_examples.md`

## Full Cycle Demo

See `docs/examples/full_ritual_cycle.md` (or the python equiv) for a complete runnable narrative: wake → heavy meta-work (tiles for plan, traces, spatial pre/post edits, sub-agent call with governance) → session-end (handoff) → simulated next wake (rehydrate from bundle, continue with momentum).

## Usage for Your Agent

1. Connect to engram MCP (see README + MCP_TOOLS_REFERENCE.md).
2. At session start: call `mcp_engram_session_start` + load/follow `docs/skills/engram-wake-up.md`.
3. For work: follow `engram-working-memory.md` (use `context_for_file` + `recall_in_file` for edits, `record_reasoning_trace`, `update` for refinements, mint tiles for meta, `scar` on friction).
4. At end: follow `engram-session-end.md`.

This gives you the same geometric continuation, self-model, and lawfulness we use internally.

**Dogfooding**: The best way to use Enram is to use these rituals *on your own development/meta-work*. The manifold will compound your agent's capability.

See also `docs/GEOMETRIC_MEMORY.md`, `docs/RITUALS.md`, `docs/MCP_TOOLS_REFERENCE.md`, `docs/GITHUB_MVP_PREP_PLAN.md` (execution history + sub-agent governance lessons).

---

*This top-level index makes discovery trivial for agents and humans. All content is in the public repo surface (no .grok/ dependency).*