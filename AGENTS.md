# AGENTS.md — Engram Editing Guidelines for AI Agents

**Primary Objective**: `goal:engram_mvp_v1` (harness continuity; operationalize "Against Flat Knowledge" via geometric sheaf replacing flat weights/text/vec DBs).

This file + CLAUDE.md + docs/ + .grok/skills/ + processes/*.toml + GITHUB_MVP_PREP_PLAN.md define the contract for any agent editing the Engram substrate or its representation.

## Non-Negotiable: Use Engram Rituals & Working-Memory Discipline

**Dogfooding note:** "Dogfood" means using Engram's MCP tools and rituals (remember/relate/trace/goal/scar/verify/spatial + wake/working/session) to record and evolve the *current task* (e.g. this GitHub prep) as living geometry. The prep work itself becomes high-CRS structure in the manifold. This is the practical meaning of the non-flat system.

- **Always start**: `/engram-wake-up` (or skill) + `/engram-working-memory`.
- **Geometric entry first** (cost-aware):
  1. `mcp_engram_query_with_momentum` (directional/trending only at entry or long-horizon).
  2. `mcp_engram_search_by_relation(seed, direction="both")` + `visualize`.
  3. For code: `mcp_engram_watch_workspace` (absolute root), `mcp_engram_context_for_file(absolute_path)`, `mcp_engram_recall_in_file` (AABB).
  4. Anchor first: ritual:*, goal:*, trace:*, process:engram.* .
- **Recall before derive**. `mcp_engram_update` preferred (Lyapunov drift); never `forget` + `remember`.
- **Every significant decision/fork/edit**: `mcp_engram_record_reasoning_trace` (or `quick_trace`) with `decision_point`, `justification`, `alternatives_considered`, `falsifiability`, `spatial_context`, `goal_context`, `prev_trace` (chain), `related_entities`.
- **Code Edit Ritual v1** (mandatory for crates/, .grok/skills/, mcp.rs, store.rs, daemon, processes/, integrations/):
  - Pre: watch + context + recall_in_file + momentum/relation on AST + intent trace.
  - Edit (search_replace/write after read).
  - Post: re-context/re-recall, delta trace (chained), relate edit to goal/arc/praxis:spatial_manifold_impact_analysis, `remember_solution` or `scar`.
- **Scar immediately** on ruled-out approaches, friction, dead-ends, repetition, doom loops. `mcp_engram_scar(concept, magnitude)`.
- **Session end**: `mcp_engram_session_end(summary=..., prepare_compression=true)`. Include decisions, files, open, next. Advances anchors.
- **Goals**: Use `engram-goal` skill / mcp goal_* ; set primary; decompose; status updates with traces; auto serves via primary_goal.
- **Spatial (Item 1.5)**: Passive by default after `mcp_engram_watch_workspace` (bind + initial walk + fs events). Rich AABB for rs/toml (tables)/md (headings+code+frontmatter) etc. If stale post-edit, `force_spatial_ingest` recovery only. context_for_file / recall_in_file should return topo without manual editor saves (high_prio fallback + parsers). Re-check after changes. See GITHUB_MVP_PREP_PLAN.md passive redesign section.
- **Sub-agent governance** (from subvisor + scars): Narrow one-shot prompts only (one action, MCP geometric first, Primary Objective + negative examples, "report to supervisor", max calls ~20). Kill on "doom loop detected (exploratory stagnation)". Supervisor monitors. No broad FS recon without task_id capture + narrow. See scar:subagent_launch_failure... and processes/monitor.subvisor.toml.
- **Meta-work automatic escalation (2026-06 ritual evolution)**: For design:/progress: multi-phase arcs (roadmaps, policies like this prep), recall `helper:meta_work_escalation_v1` + `helper:current_meta_arc` early. Escalate to update on canonical blocks + tile mint (knowledge_graph etc.) at boundaries. ki_hijacker/wake/session will prompt/surface if gap. Subvisor H1 flags tool patterns. Reconcile via helper:reconcile_step_v1 + trace field. See GITHUB_MVP_PREP_PLAN.md plan section + updated skills. Makes tiles expected for re-hydration bundles.

## Representing Engram (GitHub / Public)

When editing public surface (README, docs/, .github/, examples/, Cargo, CHANGELOG, SECURITY, AGENTS/CLAUDE):
- Highlight uniques: geometric/non-flat (q/p/CRS/Merkle/sheaf/H¹/VSA/spatial/continuation/lawfulness/rituals/subvisor/process tomls) vs flat vector/RAG.
- Emulate popular (badges, quickstarts, comparison tables, examples/cookbooks, .github templates with checklists, AGENTS/CLAUDE, CHANGELOG, CI matrix).
- Dogfood: every edit records trace/remember/relate/update to active goal (e.g. goal:1780419540... during prep), spatial where applicable, verify post. For meta-work (plans, policies): mint Thought Tiles (knowledge_graph/formal_spec/tabular) at decision boundaries with spatial_references. See updated working-memory + thought-tiles skills for recognition heuristics (evolutionary → update; complex arc → tile).
- Current build: always double-check `cargo build && target/debug/engram --version` (use target/debug or cargo run, not stale ~/.local/bin). Note in PRs/templates/plan.
- Link new docs: GEOMETRIC_MEMORY.md, RITUALS.md, MCP_TOOLS_REFERENCE.md.
- Templates enforce ritual (see .github/PULL_REQUEST_TEMPLATE.md full checklist: spatial, manifold verify, trace, engram records, non-flat invariants, current build, atomic commits).

## Process Sheaf & Subvisor

Declarative first-class: processes/*.toml (agent:engram.<type>...) loaded dynamically in mcp.rs:load_process_sheaf at session_start. Category table + H¹ gluing. Subvisor (monitor) uses OP_INVERT/H¹ for tool-graph oversight, loop detect, geometric enforce, scar.

See processes/, design/process_architecture_sheaf_*, mcp.rs, docs/RITUALS.md.

## Lawfulness & Verification

- `mcp_engram_verify_manifold_integrity` (min_crs=0.74, sample), `verify_block_lawfulness`, `genesis`, `spatial_status`.
- Phase 1.5 metrics on wake (engram-lawfulness-metrics skill).
- CRS >=0.74 grounded; pinned for axioms.
- Substrate CS Gap Closure Roadmap (6 phases) in docs/.

## Other

- Use todo_write for complex multi-step (track phases).
- For MCP: ALWAYS `search_tool` first (exact schema), then `use_tool` (qualified name e.g. engram__mcp_engram_...); never guess params.
- Hot promotion for high-value (traces, tiles, plan artifacts): `promote_hot`.
- When in doubt on approach: recall "helper:..." or "praxis:...", search_by_relation on goal/plan, trace the question.
- This AGENTS.md created/updated as part of GitHub MVP prep (dogfood on self).

See also: 
- **Public agent rituals (what other Groks/agents should load)**: root `SKILLS.md` + `docs/skills/` (engram-wake-up.md etc.). 
- Full cycle + sub-agent gov: `docs/examples/full_ritual_cycle.md`, `docs/examples/sub_agent_governance.md`, `examples/hello-engram-agent.py` (loads skills + demo loop).
- Internal TUI: `.grok/skills/engram-*.md`.
- docs/RITUALS.md, docs/GITHUB_MVP_PREP_PLAN.md, docs/MCP_TOOLS_REFERENCE.md, docs/GEOMETRIC_MEMORY.md.
- MANIFESTO.md, PHILOSOPHY.md, CONTRIBUTING.md, .github/PULL_REQUEST_TEMPLATE.md.

**Violations of this contract are scarred immediately.** The manifold will deflect future attempts.