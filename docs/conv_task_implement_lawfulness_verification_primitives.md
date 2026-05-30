# conv:task:implement_lawfulness_verification_primitives

**Type**: Active Implementation Task (conv:task)
**Parent Arc**: `conv:arc:engram_self_verification_roadmap`
**Status**: In Progress
**Created**: 2026-05-25
**Current Focus**: Delivering usable `mcp_engram_verify_block_lawfulness` and `mcp_engram_verify_manifold_integrity` tools.

## Goal
Make the first version of the lawfulness verification primitives actually functional and usable by agents via the MCP interface.

This is the foundational capability that enables credible long-sleep self-verification.

## Current State (2026-05-25)
- Spec exists: `LAWFULNESS_VERIFICATION_PRIMITIVES.md`
- Tool definitions + handlers added to MCP schema in `mcp.rs`
- Storage methods (`get_block_lawfulness_summary`, `verify_manifold_integrity`) + supporting types implemented in `store.rs` and compiling cleanly
- Added extra sanity checks (permissive PRAXIS contracts + high-CRS drift warnings)
- Integrated into wake_up protocol (Step 3.5 now explicitly calls the new verification tools as part of the checkpoint ritual)
- This task document + parent arc created inside the system using proper `conv:*` ontology
- All changes documented as part of the development checkpoint process

## Open Questions / Risks (to be updated via remember/update)
- How much historical chain data do we actually need to store for meaningful long-term verification?
- Performance on large manifolds
- Threat model and what "good enough" verification looks like for different risk levels
- Integration with the existing `verify_behavior` and scar mechanics

## Next Concrete Steps
1. Enhance output quality and add more contract/Merkle checks to the verification methods.
2. Add usage guidance + examples to `AGENT_INTEGRATION_GUIDE.md` and strengthen the wake-up ritual.
3. Test the tools against the real ~149k block manifold (primary stalk).
4. Consider adding a lightweight "deep audit" mode for critical Praxis/Genesis blocks.

## Checkpoint Expectation
This task should be reviewed using the ritual defined in `conv:task:engram_development_checkpoints` at least once per significant milestone (e.g., "first working end-to-end verification", "first integration into wake-up guidance").

---

**This is the current primary execution focus for the roadmap.** All related decisions, blockers, and insights should be linked back to this task and the parent arc.