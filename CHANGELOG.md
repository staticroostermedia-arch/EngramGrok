# Changelog

All notable changes to Engram (geometric non-flat memory substrate).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2026-06

### Added
- Full GitHub MVP prep for public representation (feat/mvp-github-prep-2026-06 branch):
  - Enhanced README with geometric Memory Model section, comparison table (vs mem0/Letta/chroma/qdrant/ragflow/milvus), badges, runnable examples section, links to new docs, 55+ MCP updates, build hygiene notes.
  - New docs/: GEOMETRIC_MEMORY.md (HolographicBlock, VSA, sheaf/H¹, spatial AABB, invariants), RITUALS.md (wake/working-memory/session-end + Code Edit Ritual v1 + sub-agent governance + lawfulness), MCP_TOOLS_REFERENCE.md (categorized 55+ tools).
  - examples/: mcp_client.py (improved runnable), ritual_verify.md, spatial_geosphere_demo.py (force/context/geosphere/momentum + ritual).
  - .github/: PULL_REQUEST_TEMPLATE.md (full ritual/spatial/manifold/verify/build/current-build checklist), ISSUE_TEMPLATE/bug_report.md + feature_request.md (engram-specific checks).
  - Enhanced .github/workflows/rust.yml (matrix ubuntu/macos, clippy/fmt, feat/docs branches, mcp-harness-and-ritual job).
  - Cargo.toml metadata: expanded description (geometric/non-flat/sheaf/rituals/MCP/spatial/continuation/lawfulness/256KB), keywords (ai-memory,geometric-memory,mcp,rituals,...), categories.
  - Enhanced SECURITY.md (manifold/ritual disclosure, verify/spatial/scar/subvisor/continuation, build hygiene).
  - CHANGELOG.md (this), start of AGENTS.md/CLAUDE.md.
- All changes under full engram-working-memory + Code Edit Ritual (pre context_for_file + record_reasoning_trace, post delta trace + relate, spatial force/ingest, engram dogfood remember/relate/promote/scar to goal:1780419540...).
- Traces, progress records, praxis solutions for prep arc (see manifold for trace:17804... series).

### Changed
- Public surface now explicitly represents current MVP uniques (geometric sheaf + rituals + subvisor + spatial + continuation + lawfulness + process sheaf) vs flat vector/RAG clones.
- CI triggers expanded; build hygiene enforced (target/debug/engram preferred).

### Fixed
- Gaps vs popular memory GitHub best practices (hero/comparison/examples/templates/CI/docs/AGENTS/CHANGELOG) identified via sub-agent recon + supervisor + narrow audit (see GITHUB_MVP_PREP_PLAN.md for details, scars for sub-agent governance lessons).

See [docs/GITHUB_MVP_PREP_PLAN.md](docs/GITHUB_MVP_PREP_PLAN.md) for full execution log, sub-agent IDs, gap matrix, success criteria.

## [0.4.0] - 2026-06 (MVP Sheaf / Rituals / Geometric Substrate)

- Process Architecture Sheaf: declarative processes/*.toml (7: ritual/wake-up, nrem-consolidation, spatial-recon, momentum-query, session-end, manifold-health, monitor/subvisor), dynamic loader in mcp.rs (registers process:engram.* at session_start), category gluing/H¹ (OP_ADD/OP_GEOMETRIC_PRODUCT/OP_INVERT/OP_IS_SYMBOLIC_OF).
- Subvisor (monitor): OP_INVERT + H¹ for sub-agent oversight, loop detect via tool graph, geometric enforce, scar repetitive (governance from sub-agent doom loop scars).
- Rituals first-class: engram-wake-up (Phases 0-5 + lawfulness metric:wake_up_verification + continuation bundle), engram-working-memory (momentum/relational/spatial entry, Code Edit Ritual v1 pre/post + trace A/D/R + goal, expensive tool hygiene, scar), engram-session-end (crystallize + COMPRESS 0x10 + handoff).
- Spatial (Item 1.5): watch_workspace, context_for_file, recall_in_file, force_spatial_ingest, AABB from tree-sitter on save; item1.5_spatial_ingestion_state_engram; bootstrap notes.
- MCP: 55+ tools (memory, spatial, graph/sheaf, verify/lawfulness, goals/tiles, session/continuation, process).
- Geometric core: HolographicBlock .leg3 (256KB, q 8192D, p momentum, CRS, Merkle, provlog), VSA, geosphere/symplectic, non-flat vs flat (momentum + relations + scar/verify/continuation vs append-log).
- Other: goal stack as geometric, thought tiles + hot promotion, verify_manifold_integrity + block lawfulness + genesis, NREM/ego.leg3, continuation bundles, harness-gate, lawfulness-metrics skill.
- GitHub prep initiated on feat branch; current build hygiene (target/debug).

See prior handoff docs, MANIFESTO, design/process_architecture_sheaf, 2026-06_Substrate_CS_Gap_Closure_Roadmap.md, .grok/skills/ for full.

## Earlier
See git log and in-manifold traces (trace:* , goal:*) for pre-sheaf history. Primary objective: engram_mvp_v1 (harness continuity, Against Flat Knowledge via geometric sheaf).