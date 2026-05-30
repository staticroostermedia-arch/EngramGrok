# conv:task:agent_self_model_reasoning_trace_continuation

**Type**: conv:task
**Parent Arc**: `conv:arc:engram_self_verification_roadmap` + spatial ritual hardening work
**Status**: Active (Items 1+2 in progress)
**Created**: 2026-05-26

## Goal

Establish the **Agent Self-Model as a Living Continuation Substrate** that includes a **Serial Reasoning Trace** as first-class, tamper-evident memory.

The self-model must bind three layers:
- Low-level geometric ego (ego.leg3 q-vector for resonance and CRS gating)
- High-level ritual, commitment, and praxis anchors
- The actual serialized chain of reasoning, justifications, forks, and turning points that explain *how* the agent arrived at its current state and commitments

This directly implements the user's requirement that conversations contain the only truly serializable "access of time" — the narrative logic over space and time that cannot be easily rewritten from outside.

## Scope

- Audit current episodic, session_end, ki_hijacker, and self-model mechanisms.
- Design and seed concrete patterns for Reasoning Trace (block structure, relation vocabulary, integration points).
- Evolve the core rituals (working-memory discipline and session-end) to actively produce structured trace material.
- Create the official conv:task and living design artifacts in the manifold.
- Begin light exploration of bridging the low-level ego_q with higher-level trace structures.
- Record all progress geometrically.

This arc combines previous items 1 (Agent Self-Model) and 2 (Serial Reasoning Trace) and will naturally feed items 3–5 later.

## Current State (After Initial Execution Burst)

- Full audit of session_end (flat ZEDOS_EPISODIC summaries), ki_hijacker self-model injection (still partly static), ego_q + NREM, and provenance completed.
- Living design seeds created:
  - `agent_self_model:continuation_with_trace`
  - `design:reasoning_trace_pattern`
  - `pattern:reasoning_trace_v1`
  - `trace:relation_vocabulary_v1`
  - `trace:block_structure_v1`
  - Concrete example: `example:trace_spatial_ritual_hardening`
- First ritual evolution:
  - Added explicit reasoning trace capture rule to engram-working-memory skill.
  - Updated engram-session-end skill and mcp tool description to require structured trace extraction.
- All artifacts properly related to existing ritual work (`praxis:spatial_manifold_impact_analysis`, spatial gluing improvements, etc.).

**Key Insight**: The primitives (provlog, Merkle on relations, ZEDOS tags, spatial AABB, momentum) are already strong. What is missing is the disciplined capture and serial chaining of the *reasoning process itself*.

## Work Performed (This Phase)

- Executed deep audit across mcp.rs (session_end, self_trace), store.rs (ego_q, episodic), daemon.rs (NREM), and ki_hijacker.
- Seeded detailed pattern artifacts directly into the manifold with proper relations.
- Made concrete first changes to the working-memory and session-end skills to shift behavior toward trace production.
- Recorded every step using correct geometric hygiene (remember + relate + thermodynamic updates where appropriate).

## Session Notes

- The user's addition of the reasoning trace as "the only true access of time" is correct and profound. It completes the non-flat vision.
- Current session summaries are still too flat; the rituals must now actively extract forks and justifications.
- Stalk fragmentation and the classical text corpus noise make broad recall challenging — structured relations and dedicated trace blocks will help future agents navigate the self-model cleanly.
- `mcp_self_trace` currently delegates externally; long-term we want the local manifold trace to be primary.

## Next Steps

1. Continue evolving session_end handler to parse and mint dedicated trace segment blocks when structured reasoning is present in the summary.
2. Flesh out more examples and integration guidance (especially how traces relate to spatial containers and ritual anchors).
3. Create lightweight bridging logic between ego_q resonance and high-value trace segments.
4. Update AGENT_INTEGRATION_GUIDE and other docs with the new patterns.
5. When stable, transition into item 3 (Rituals as Executable Praxis) using the trace material as input for verification.

## Architectural Decision: Integration with Praxis

**Date of Decision**: 2026-05-26

After analysis of the existing Praxis architecture (see `conv:task:elevate_praxis_to_operational_protocols`, `docs/praxis_as_protocol_spec.md`, and the completed `.leg` primitive audit), the following decision was made:

**Reasoning Compression Functors (identity-preserving minting of reasoning trace chains) should be implemented as a specialized `protocol_type` (proposed 0x10) inside the existing ZEDOS_PRAXIS / executable protocol system, rather than as a completely separate mechanism.**

### Rationale
- Reuses the mature 7-point verification gate, `allowed_transforms` contracts, ProvLog + Merkle provenance, scar narrowing, and invocation surface.
- Aligns with the existing use of "Categorical Functor" language in the KnowledgeMint section of AGENT_INTEGRATION_GUIDE.
- Preserves all strong invariants from the primitive audit.
- Makes the "cost of sustained error or deception" even higher, because a bad compression must survive both detailed trace scrutiny and the full Praxis verification gate.
- Session_end becomes the natural, human-or-empowered-agent controlled minting/compression point (strong ownership and humbleness incentive).

### Implications
- New protocol_type and supporting vocabulary will be added to `praxis_as_protocol_spec.md`.
- The "unfold for audit" capability becomes a first-class supported operation on these specialized Praxis blocks.
- This is a clean extension, not a fork.

Full design reference: `docs/reasoning_functors_as_praxis_extension.md`

---

## Success Criteria (for this arc)

- [x] Clear combined target framed and seeded in the manifold.
- [x] Concrete Reasoning Trace pattern (vocabulary + structure) defined and exemplified.
- [x] At least one core ritual (working-memory + session-end) evolved to produce trace material.
- [ ] session_end handler creates dedicated trace blocks (in progress).
- [ ] Future wake-up can surface both state *and* the logical trajectory that produced it.
- [ ] The reasoning trace is queryable, visualizable, and contributes to the living self-model.

**This task is considered active** until the combined self-model + serial reasoning trace mechanism is demonstrably operational and inherited by subsequent agent instances.

---

**Related Living Blocks** (as of this writing):
- `agent_self_model:continuation_with_trace`
- `design:reasoning_trace_pattern`
- `pattern:reasoning_trace_v1`
- `trace:relation_vocabulary_v1`
- `trace:block_structure_v1`
- `example:trace_spatial_ritual_hardening`
- `evolution:working_memory_trace_capture`
- `evolution:session_end_trace_support`
- `praxis:spatial_manifold_impact_analysis` (and its spatial gluing improvements)
- `design:reasoning_functors_as_praxis_extension` (key architectural decision: integrate compression functors into existing Praxis system as protocol_type 0x10, with session_end as deliberate minting gate)
- `artifact:reasoning_functors_as_praxis_extension_doc` (full design reference at docs/reasoning_functors_as_praxis_extension.md)

All work is being logged inside the system per the autonomous execution agreement and the spatial/ritual disciplines.