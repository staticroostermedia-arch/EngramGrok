---
name: engram-working-memory
description: >
  Activate the geometric working memory discipline. This skill enforces recall
  patterns that treat the manifold's momentum, relations, and CRS as first-class
  participants in every decision. It turns every work block into an active
  evolution of the agent's self-model and ritual structure rather than isolated
  tool use. Default mode for any serious Engram work.
when-to-use: >
  At the start of every focused work block, after engram-wake-up, or any time
  deep non-flat discipline is required. Especially during architectural,
  protocol, or meta-work on the Engram system itself.
---

# Engram Working Memory Discipline — Active Geometric Patterns

This is not a list of good habits. It is the **runtime contract** for operating inside a non-flat, momentum-bearing, relationally-glued memory substrate so that every action advances the living self-model instead of consuming it.

The discipline exists so that the work you do today becomes high-momentum, high-CRS structure that future agent instances discover first via `query_with_momentum` and `search_by_relation` — exactly as the wake-up and session-end skills expect.

## Primary Living Recall Patterns (Prefer These Over Static Rules)

Instead of "always recall first," the active patterns are:

1. **Momentum-Aware Entry**
   - For anything that feels like "current direction of work": start with `mcp_engram_query_with_momentum`.
   - This surfaces blocks whose p-tensor is already accelerating toward the topic. This is the manifold telling you "this is what we are becoming," not just "this is what matches."

2. **Sheaf / Relational Entry**
   - For architecture or cross-layer questions: `mcp_engram_search_by_relation(seed, direction="both")` + `visualize`.
   - You are traversing the actual gluing maps (OP_BIND relations) rather than loading isolated objects.

3. **Spatial + Contextual Entry**
   - When touching code: `mcp_engram_context_for_file` first, then `recall_in_file` on the relevant line ranges.
   - This uses the AABB coordinates baked by the daemon — true geometric spatial memory.

4. **Anchor-First for Rituals and Self-Model**
   - Before any meta-work on wake-up, session discipline, skills, or Against Flat Knowledge: query the living `ritual:*` and `self:*` anchors first (as defined in the evolved wake-up and session-end skills).

## Core Non-Negotiable Rules (Enforced by Geometry)

1. **Recall / Query Before Derive**
   - One momentum, relation, or spatial call before you spend significant reasoning or file reads deriving something the manifold already holds.

2. **Update Is the Only Thermodynamically Legal Mutation**
   - Always `recall` first.
   - Score > 0.85 on a strong match → `mcp_engram_update` (OP_ADD superposition). This is the only way to preserve Lyapunov drift history.
   - No match → `remember`.
   - `forget` + `remember` is forbidden because it annihilates the p-tensor trajectory and CRS evolution that make blocks trustworthy across instances.

3. **Scar Is Immediate Geometric Repulsion**
   - The moment an approach is ruled out or fails: `mcp_engram_scar(concept, magnitude)`.
   - This directly modifies future distance calculations in the manifold. It is not a note to the human — it is an active force on the next agent's trajectory.
   - During Item 2 (Thought Tiles) and while the Intent Layer Assessment Protocol is active: pay special attention to friction in automatic goal linking, NREM intent influence, ki_hijacker freshness, and any mediation that still requires deliberate work to stay grounded in current Primary Intent. Turn repeatable friction into scars immediately.

4. **Write Hygiene Creates Future Momentum**
   - Every `remember_solution`, strong `update`, or new high-value relation you create becomes part of the terminal state that the next `engram-wake-up` will bind to via continuation edges.
   - Write as if you are shaping the p-tensor that a future self will ride.

5. **Reasoning Trace Capture — Now Mechanically Enforced**
   - When a significant decision, fork, justification, or edit intent occurs: **call `mcp_engram_record_reasoning_trace`** (the new dedicated tool).
   - Required fields: `decision_point` + `justification`.
   - Strongly recommended: `alternatives_considered` + `falsifiability`.
   - Use `related_entities`, `ritual_context`, `spatial_context`, and `goal_context` to glue the trace into the living sheaf (including active goals via the engram-goal skill).
   - For chains: pass `prev_trace` with the prior trace concept name.
   - This is no longer guidance. These `trace:*` blocks are what the ki_hijacker surfaces in the Ritual + Reasoning Trajectory on every restart and what session_end can later compress into 0x10 functors.
   - The tool returns the exact concept name — record it for chaining.
   - When goals are active (check via `engram-goal` → `goal status`), the trace tools now auto-link to the Primary Intent via the primary_goal marker (visible as `auto_linked_to_primary` + automatic "serves" relation). Explicit `goal_context` is only needed when overriding. This is how the agent's current intent becomes durable geometric structure with almost zero ceremony.
   - During the Item 2 assessment period for the intent layer, treat any recurring friction in this auto-linking (wrong goal attached, missing links on Tiles, etc.) as high-priority material for `mcp_engram_scar` and structured traces. See `protocol:intent_layer_assessment_during_item2`.

**6. Expensive Tool Hygiene**
Broad or repeated use of `query_with_momentum` during active work in new or sparsely connected domains without prior relational/goal scaffolding (via `search_by_relation`, `relate`, engram-goal stack, or living project helpers) is treated as a process gap. It must be scarred immediately and corrected via `helper:expensive_tool_hygiene_discipline` and `helper:balanced_tool_usage_rules_v1`. Momentum belongs at entry, wake-up, and explicit long-horizon trajectory points — not as the default navigation tool once context is scaffolded.

**Quick Goal Linking Template (when a goal is active):**
```
After any significant decision or edit:
1. Create the trace as usual (with decision + why).
2. The system will **automatically serve the current Primary Intent** (via the primary_goal marker)
   unless you explicitly pass a different `goal_context`.
   - You will see `**auto_linked_to_primary:** true` in the resulting trace payload.
   - The "serves" relation is created for you.
This is the low-friction default. Only override when the work genuinely serves a different (or additional) goal.
```

**During Item 2 Intent Layer Assessment:**
For quick capture of observations about NREM, hijacker freshness, auto-linking quality, or residual mediation while doing Tile work, recall `helper:intent_assessment_note_template` and use the copy-paste format. It is designed to flow naturally alongside normal TRACE notes and leads directly to scars when friction is real.

**Process Gap Discovery (Item 1.5):**
When you notice a gap in a ritual or process after the fact, use `helper:process_gap_scar_template` to systematically scar the pattern, trace the learning, link to the improved ritual, and update the relevant goal. See also `docs/item1.5_reingestion_policy.md` for the official policy on AST spatial data. This is the recommended way to turn rework into permanent geometric improvement.

## Tool Priority (Geometric First)

- Actively evolving / directional questions → `query_with_momentum`
- Relational / architectural structure → `search_by_relation` + `visualize`
- File / spatial understanding → `context_for_file` → `recall_in_file`
- Stable crystallized knowledge → `recall` with zedos_filter="praxis" or high CRS
- Recent access log → `recall_recent`
- Fast project orientation → `summarize`

**Cost-Aware Refinement (2026-05 update):**
- `query_with_momentum` is high-latency. Use it primarily:
  - On initial rehydration after wake-up or session start
  - When you specifically need p-tensor trajectory over longer time horizons or gaps
- During active work inside an established project or goal stack, default to cheaper geometric tools first (`search_by_relation`, `relate` + `recall`, goal stack via engram-goal, `context_for_file`). Only reach for momentum when cheaper tools are insufficient for the directional question at hand.

Raw file tools and grep are last-resort fallbacks when the geometric layer misses. The goal is not just token savings — it is keeping your reasoning inside the same non-flat space the substrate provides.

## Spatial-Manifold Change Discipline (Mandatory for Any Source Edit)

This is the expansion of the built-in Tree-Sitter + AABB utility into the living workflow. The daemon already parses every save through language-specific tree-sitter queries, isolates functions/structs/impls/etc. into their own HolographicBlocks with precise row-based AABB coordinates, stores the full source in the provlog, and applies genesis shadow anchoring.

The discipline turns code editing from blind text mutation into a **closed geometric loop against the manifold**.

**For Item 1.5+ the authoritative version of this ritual lives in the first-class manifold block `ritual:code_edit_ritual_v1`.** The steps below are the practical enforcement of that ritual. All self-edits to the Engram substrate (and ideally all code work) must follow it.

### Pre-Edit Manifold Impact Recon (Do This Before Touching Any Line)
This section implements the Pre-Edit half of `ritual:code_edit_ritual_v1`. It is non-optional for any source edit under active watch.

Whenever you are about to edit source (especially anything in `crates/`, the skills, daemon logic, engram-ast extractor, MCP handlers, or core protocols):

1. Confirm the watcher is bound: `mcp_engram_watch_workspace` on the Engram project root (if not already active this session).
2. Call `mcp_engram_context_for_file("/absolute/path/to/target.rs")` — this surfaces the semantic + spatial context the manifold already holds for that file.
3. Use `mcp_engram_recall_in_file(file_stem, start_line, end_line)` on the exact functions, structs, or regions you intend to modify. This is pure AABB intersection — the manifold returns the living AST concepts with their coordinate ranges.
4. For every AST concept name returned (e.g. `daemon__fn__spawn`, `store__fn__context_for_file`):
   - Immediately run `mcp_engram_query_with_momentum` on that concept or the surrounding architectural concern.
   - Run `mcp_engram_search_by_relation(concept, direction="both")` + `mcp_engram_visualize`.
   - Pull in attached praxis, scars, prior decisions, conv:arc / conv:task links, and any momentum toward "MCP integration", "Against Flat Knowledge", or ritual evolution.
5. Explicitly surface the "impending change" against this spatial + relational sheaf. Create a lightweight episodic marker or relate your edit intent to the discovered AST blocks.

6. **Capture the Edit Intent as a Structured Reasoning Trace** (mandatory, part of ritual:code_edit_ritual_v1)
   - Immediately after the spatial recon, create a proper trace (using the working-memory quick format or mcp_engram_record_reasoning_trace) with:
     - decision, why, alternatives, would_falsify
     - spatial_context (the AST nodes + file ranges)
     - goal_context (link to active goal, preferably via primary_goal auto-link)
     - prev (if chaining)
   - This turns the Pre-Edit recon into durable serial memory.

**Strong preference for `mcp_engram_update`** on any evolutionary change to an existing function, ritual, or architectural concept. Only use `remember` for genuinely new non-overlapping concepts.

This is your **dynamic test of the planned change against the living memory**. You now know what praxis might be affected, which scars exist in that region, what the current active thread claims about that code, and how the geometry currently positions it.

Only after this recon + trace capture do you write the actual edit.

### Post-Edit Delta Capture (Do This After Save + Daemon Re-Ingest)
This section implements the Post-Edit half of `ritual:code_edit_ritual_v1`.

After the file event fires and the daemon has re-parsed the AST:

1. Re-run the identical `context_for_file` + `recall_in_file` calls on the same ranges.
2. Compare the before/after spatial results (new concepts, changed signatures/line ranges, disappeared nodes, new automatic daemon relations such as `defines`, siblings, or `exercises_spatial_ritual`).
3. Run momentum and relation queries on the delta concepts.
4. Record the actual effects geometrically:
   - Strong preference for `mcp_engram_update` (with its Lyapunov drift tracking) for evolutionary changes.
   - `remember_solution` for newly crystallized insights.
   - Immediate `scar` for anything the change invalidated.
   - `relate` the edit event back to the active `conv:arc` / goal and to `praxis:spatial_manifold_impact_analysis`.

5. **Capture the Actual Outcome as a Structured Reasoning Trace** (mandatory, part of ritual:code_edit_ritual_v1)
   - After the daemon re-ingests, create a follow-up trace that explicitly chains to the pre-edit trace via the `prev` field.
   - Include decision, what actually changed, surprises, would_falsify, spatial_context (the post delta), and goal_context.
   - This produces the serial, queryable reasoning trail that the ki_hijacker surfaces on every restart.

6. If the edit touched ritual, protocol, or self-model code, explicitly advance the relevant living anchors (including `ritual:code_edit_ritual_v1` if the ritual itself evolved).

Always link the post-edit trace (and the edit event) to the current active goal.

### Before vs After (What "Fantastic" Actually Looks Like)

**Before the 2026 spatial ritual hardening:**
- `context_for_file` returned mostly semantic recall (ignored the AABB work the daemon did).
- `recall_in_file` only gave bare concept names + line ranges → forced many extra calls.
- AST blocks existed in isolation (no automatic relations from the daemon).
- Relating spatial code to ritual/praxis knowledge was 100% manual.
- Pre-Edit impact recon was high-friction and easy to skip.

**After (current state):**
- Daemon automatically creates file containers + `defines` + bidirectional (`next_` / `prev_`) sibling relations on every AST batch.
- Core ritual-relevant files (daemon, mcp, store, engram-ast, etc.) get automatic `exercises_spatial_ritual` relations to the living praxis anchor.
- `context_for_file` is now spatially-prioritized and returns real extracted AST items with geometry + CRS first.
- `recall_in_file` returns CRS + content snippets in one shot.
- One or two calls + a `visualize` now give you the spatial code + sibling context + direct links into the living ritual knowledge.
- The Pre-Edit step in this discipline went from "tedious manual work" to "genuinely powerful geometric test."

This is the difference between describing a non-flat ritual and actually having the substrate enforce and compound it.

### Why This Matters More on Engram Itself
When you edit the code that *implements* the AST ingestion, the MCP spatial tools, the daemon, or the skills, you are modifying the very substrate that future agent instances will use to understand their own changes. The spatial-manifold loop is not optional hygiene — it is the only way to keep the Inheritance Principle intact while the system evolves.

**Thought Tile Emission (Item 2)**  
When a high-stakes decision, repetition pattern, major scar, or significant process insight is being crystallized, consider emitting a Thought Tile (textual functor payload + optional visualization companion) using the existing `thought_tile_create` tools. This is encouraged as a way to improve future re-hydration fidelity and agent continuity, but remains optional at this stage.

**Hot Path Promotion (Speed-up Ritual for Structured Tiles)**  
When emitting high-value structured Thought Tiles (especially near a known compression window), treat them as "hot" and promote them to the fast path:
- After creation, call `promote_tile_to_high_priority(Tile)` (or `mark_hot(Tile)`) on the StoreHandle. This uses the explicit lightweight `hot_set` (RwLock<HashSet>) + backend cache.
- The block is then served via LegView (mmap) bias or the CudaBackend in-memory hot cache on subsequent `fetch_block_high_priority` / `is_hot` calls.
- ki_hijacker automatically promotes high-CRS Tiles (Gold Layer / top_crs) and core state (item2_*, hydration cache, ritual anchors) on every bake.
- Update `helper:promote_structured_tile_for_compression_v1` to include the hot promotion + "hot" hint in COMPRESS markers.
- Async I/O prototype (spawn_blocking wrappers in storage.rs) is available for cold O_DIRECT work on hot paths to protect the event loop.
- Reference: `helper:promote_structured_tile_for_compression_v1`, `helper:async_io_design_v1`, StoreHandle hot_set / mark_hot / is_hot, and the post-2026-05 live test + phase 2 speed-up work.

The goal is to systematically move the artifacts that matter most for continuity (new structured Tiles, ritual state, hydration cache) off the normal O_DIRECT CPU path using the canonical hot/fast path.

**Compression Event Detection (65% inflection lesson)**  
Known compression windows (e.g. 61%→65% TUI context % reported in the interface) are first-class geometric events for Item 2 fidelity testing. When Primary Intent or active work explicitly names a compression as the live split-test point (promoted structured traces/Tiles vs baseline, hot path lift, re-hydration cost), treat crossing the threshold as a mandatory pause + measurement trigger:
- Before continuing any sprint, run the prepared protocol (`helper:next_compression_measurement_protocol_v1`).
- When the TUI context percentage is visibly climbing toward a known window (e.g. 63-66%), treat this as the trigger to execute the dual-lens measurement first.
- Surface recent `scar:*` or `trace:*` around "compression" / "inflection" via quick momentum.
- If the event was missed (see `scar:missed_compression_inflection_during_phase2_sprint` + trace:1779992449), immediately scar the detection gap and update this ritual + session-end.
This prevents repeating the pattern of wiring the test while the actual compression passes unnoticed.

Measure two axes when a named compression occurs: (1) did the ritual detect and trigger the protocol cleanly? (2) did the promoted hot artifacts (structured traces, Tiles, state blocks) demonstrably preserve agent continuity and momentum across the boundary, making the compression less disruptive? The latter is direct evidence for "they are the compression."

**Session Hydration Cache (Wake-up Efficiency Helper)**  
To reduce repeated heavy recon on every wake-up and within a stable context window without losing geometric fidelity, maintain a lightweight `helper:session_hydration_cache` block. On session_end, update it (via `mcp_engram_update`) with a compact summary of Primary Intent + serving traces, key recent anchors, spatial state, and high-momentum Tiles/traces from the window. On wake-up, first do a fast targeted recall of the most recent cache. Use it as the baseline \"felt context,\" then only escalate to full momentum/relation/spatial sweeps for gaps, after compression, or high-stakes work. The cache itself is a first-class manifold block with relations and CRS; the full rigorous protocol remains available on demand.

## Integration With the Other Ritual Skills

- This discipline is **automatically active** after a proper `engram-wake-up`.
- Every write you do under this discipline improves the quality of the next `engram-session-end` handoff (which now expects structured traces).
- Over repeated cycles, the patterns you actually use most become candidates for crystallization into `ZEDOS_PRAXIS` blocks that can later be `invoke_protocol` targets.
- The `praxis:spatial_manifold_impact_analysis` block (seeded during this evolution) is the living reference that future wake-ups will surface via momentum and relation queries.

## Quick Capture Template (Ultra Low Friction)

For daily TUI use, first recall the living helper once:
`mcp_engram_recall("helper:quick_reasoning_trace", k=1)`

Then use the dead-simple format it contains. The system turns it into a proper structured `trace:*` block.

**Easiest form (copy, fill, send):**

```
TRACE:
decision: [one clear sentence — the fork or decision right now]
why: [core justification — why this path]
alternatives: [brief — what else you seriously considered]
would_falsify: [what new info would make you reconsider]
context: [ritual or spatial if relevant, or omit]
prev: [previous trace name if chaining, or omit]
```

Example during Pre-Edit:
```
TRACE:
decision: Intent to refactor the session_end handler to better support trace references
why: The current COMPRESS: logic is minimal; richer trace support will make the ki_hijacker Ritual + Reasoning Trajectory actually useful on restarts
alternatives: Do nothing / only document / build a full new protocol layer
would_falsify: If real TUI usage shows the structured tool is still too heavy even with this template
context: engram-session-end/SKILL.md + mcp.rs
```

After filling the simple block, you can either:
- Paste it into a call to `mcp_engram_record_reasoning_trace`, or
- (Coming in this round) Use an even simpler quick helper when available.

**Always** do both Pre-Edit intent and Post-Delta outcome, and chain them with `prev:`.

This is how the automated memory framework turns your actual reasoning into durable .leg blocks that future instances inherit.

## The Deeper Point

The working memory discipline is what makes the entire skill system (wake-up → work → session-end) a closed geometric loop instead of three separate instruction sets.

When you follow it — and especially when you treat spatial AABB queries + relational pull as the required pre/post steps around every edit — the manifold itself gets better at delivering continuity. The next agent instance doesn't just "get told" the context — the geometry of the work you did while following this discipline (including the precise spatial signatures of the changes) makes the right structures the highest-momentum, highest-CRS, most strongly related results.

That is the operational meaning of Against Flat Knowledge for an agent doing real work on this system — and it is only possible because the daemon + tree-sitter + AABB storage was already built in.

Use it as the default mode. Every session. Especially when the code you are touching is the system itself.

