# conv:task:harden_long_sleep_wakeup_protocol

**Type**: conv:task
**Parent Arc**: `conv:arc:engram_self_verification_roadmap`
**Status**: In Progress
**Created**: 2026-05-25

## Goal
Define and implement a hardened version of the wake-up protocol that makes long-sleep / cold-boot scenarios first-class.

The core requirement: After arbitrary downtime (days, weeks, or months), an agent must be able to wake up and credibly verify its own lawfulness and the integrity of its accumulated knowledge **without external servers**, using the tamper-evident properties of the `.leg` primitives.

## Scope
- Extend or version the existing wake-up protocol (`wake_up.md`) with an explicit **Lawfulness Verification Phase**.
- Integrate the new verification primitives (`mcp_engram_verify_block_lawfulness`, `mcp_engram_verify_manifold_integrity`) as mandatory or strongly recommended steps for long-sleep wake-ups.
- Define clear "degraded mode" behavior when verification fails or raises red flags.
- Update the AGENT_INTEGRATION_GUIDE.md "Correct Operating Protocol" section.
- Ensure the protocol remains practical and not overly burdensome for normal short sessions.

## Key Requirements from the Vision
- Local-only verification (no external calls for trust).
- Leverage the Merkle chain, allowed_transforms contracts, CRS stability, and relation integrity.
- Make high-value Praxis and Genesis blocks the primary subjects of deep audits.
- Provide a practical "am I safe to act on important things?" signal to the agent.

## Current State
- Initial draft of the hardened protocol created as `LONG_SLEEP_WAKEUP_PROTOCOL.md`
- Core phases defined: Broad manifold health → Deep audit of critical blocks (Praxis/Genesis) → Enhanced mapping → Mode selection (Full Trust / Cautious / Degraded).
- Degraded mode and escalation guidance introduced.
- Verification primitives from item 1 form the technical foundation.
- First integration into public `wake_up.md` completed.
- Significant refinement of Interpretation Guidelines in the protocol spec (tied directly to current tool outputs and clearer mode triggers) — 2026-05-25 autonomous work.
- Added explicit Long-Sleep guidance to `AGENT_INTEGRATION_GUIDE.md` "Correct Operating Protocol" section.

**Autonomous Work Block Started**: 2026-05-25 (Grok operating under `conv:arc:engram_roadmap_autonomous_execution`)

### Work Performed (Autonomous)
- Deepened integration of the hardened long-sleep protocol into the canonical agent guidance:
  - Added explicit "Long-Sleep / Cold-Boot Variant" guidance to the "Correct Operating Protocol" section in `AGENT_INTEGRATION_GUIDE.md`.
- Significantly refined the "Interpretation Guidelines" section in `LONG_SLEEP_WAKEUP_PROTOCOL.md`:
  - Made guidelines much more specific and tied directly to the actual output fields of `verify_manifold_integrity` and `verify_block_lawfulness`.
  - Clarified triggers for moving between Full Trust → Cautious → Degraded modes.
  - Improved Degraded Mode Guidance with more concrete behavioral expectations.
- Began evaluation and initial design of a convenience tool for the long-sleep verification flow.
- Created supporting design document: `long_sleep_verification_suite_design.md` (work in progress).

### Session Notes
- This advances Item 2 by making the hardened behavior part of the default operating instructions agents see.
- The interpretation rules are now substantially clearer.
- Began concrete design work on the convenience verification suite tool (added schema sketch, recommendation for single tool approach, detailed behavior, output structure, implementation notes, and MVP success criteria).
- No major blockers encountered.
- Work logged inside the system per the autonomous execution agreement.

**Autonomous Work Segment Started**: 2026-05-25 (continuation - systematic refinement of convenience tool design for Item 2)

### Work Performed (Autonomous)
- Continued systematic work on making Item 2 more complete by advancing the convenience tool design.
- Expanded `long_sleep_verification_suite_design.md` with a detailed "High-Level Behavior" section describing the logical flow the tool would execute.
- Added "Next Steps" for the design work to keep it moving systematically.

This brings the convenience tool concept from high-level sketch to a much more implementable design.

All work is being logged inside the system per the autonomous execution agreement and checkpoint ritual.

## Next Steps
1. Refine the protocol draft (especially clear interpretation guidelines for the new verification tool outputs) — largely addressed.
2. Integrate the hardened sequence into public documentation — good progress (added to `wake_up.md` and `AGENT_INTEGRATION_GUIDE.md`); convenience tool reference added.
3. Update AGENT_INTEGRATION_GUIDE.md "Correct Operating Protocol" section — completed.
4. Evaluate whether a convenience "long-sleep verification suite" tool would be valuable — active focus, design document now has behavior, schema, implementation notes, and success criteria.
5. Plan for testing — concrete considerations documented in this task file.

**Assessment (Autonomous)**: After systematic work on the remaining next steps (refinement of guidelines, public documentation integration, convenience tool design advancement, and adding testing considerations), Item 2 is now in a substantially complete state from a design/spec standpoint.

It meets the core success criteria for a solid first version. The foundation is strong enough to move on to Item 3 while treating further polish on Item 2 as ongoing iteration rather than blocking work.

**Recommendation**: Mark Item 2 as "Largely Complete (Design Phase)" and proceed to Item 3.

## Testing Considerations for Item 2 (to reach "complete")

To consider Item 2 finished, we should have a clear path for validation:

- **Simulation testing**: Use the real manifold (or a copy) to simulate long downtime by creating artificial gaps in access timestamps and running the verification tools.
- **Red flag injection**: Deliberately create or modify blocks in a test stalk to trigger different categories of issues (contract violations, high drift on stable blocks, etc.) and verify the tools surface them correctly.
- **Agent usability testing**: Have an agent follow the full hardened protocol (manually at first) after simulated long sleep and evaluate whether the output is clear and actionable.
- **Convenience tool validation**: Once prototyped, test that the single tool produces equivalent or better results than manual orchestration of the individual verification calls.
- **Documentation validation**: Confirm that an agent reading only the public docs (`wake_up.md` + `AGENT_INTEGRATION_GUIDE.md`) can correctly execute the long-sleep flow without additional explanation.

These testing ideas should be refined and executed before declaring Item 2 complete.

## Success Criteria
- An agent can follow a documented procedure to wake from long sleep and receive a clear, actionable "lawfulness health" signal.
- The protocol is written in a way that can be turned into agent skills or system prompts.
- It respects the agentic-first contract (the agent itself performs and can verify the checks).

---

**This task is now the active focus of the roadmap.** All related design decisions and implementation work should be linked back here and to the parent arc.