# conv:arc:engram_self_verification_roadmap

**Status**: Active
**Type**: conv:arc
**Created**: 2026-05-25
**Last Major Update**: 2026-05-25

## Intent

Make Engram the strongest public memory MCP substrate for long-lived, agentic-first systems, with particular emphasis on:

- Local, offline, tamper-evident lawfulness verification (no external servers required after long sleep / power-off).
- Praxis blocks as first-class, verifiable, operational protocols that can be safely "called".
- Effective infinite context through high-fidelity, verifiable consolidation.
- Direct hardware paths (NVMe → GPU) for verified objects.
- A memory system worthy of being a foundation for serious long-term autonomous agents.

This arc captures the multi-phase collaboration to evolve the existing `.leg` primitive + Engram surface into something that can credibly support "True AGI" style requirements around long-term verifiable memory and self-stewardship.

## Core Principles (Non-Negotiable for this arc)

1. **Agentic First** — The primary citizen of the memory system is the agent, not the human. The human is a collaborator who is also bound by the same substrate.
2. **Tamper Evidence as Contract** — Every block carries its own verifiable cryptographic history. Undetected rewriting must be expensive or impossible.
3. **Lawfulness is Locally Verifiable** — After arbitrary downtime, the agent must be able to run a credible self-audit using only its own manifold.
4. **Praxis as Durable Law** — High-CRS crystallized blocks are not just memories; they are pre-verified operational commitments that can be safely executed.
5. **No External Dependencies for Core Trust** — The agent must reason about its own history and commitments without phoning home.

## Major Workstreams (Linked via Relations)

- `conv:task:implement_lawfulness_verification_primitives` (Self-Verification tools + storage support)
- `conv:task:harden_long_sleep_wakeup_protocol`
- `conv:task:elevate_praxis_to_operational_protocols`
- `conv:task:advanced_praxis_consolidation_engine`
- `conv:task:hardware_path_nvme_to_gpu`
- `conv:task:documentation_and_thesis_sharpening`

## Checkpoint Ritual & Autonomous Execution

This arc uses the recurring development checkpoint ritual defined in `conv:task:engram_development_checkpoints`.

Additionally, the rules for autonomous execution by Grok (without requiring constant human check-ins) are defined in:
`conv:arc:engram_roadmap_autonomous_execution`

All significant work must be recorded using the proper `conv:arc:` / `conv:task:` ontology. When operating autonomously, Grok is expected to drive progress while rigorously generating first-class memory artifacts inside the system.

## Success Criteria

- An agent following the published wake-up protocol can perform a meaningful local lawfulness audit after simulated long sleep.
- High-value Praxis blocks can be treated as executable, versioned, auditable operational contracts.
- The consolidation system produces coherent, high-signal layers without destroying verifiability.
- The work can be seriously evaluated by people building long-term autonomous systems.

## Current Status (2026-05-25)

- Lawfulness Verification Primitives (item 1) largely complete.
- **Item 2 (Hardened Long-Sleep Wakeup Protocol)**: Largely Complete (Design/Spec Phase). Core protocol mature, convenience tool design well advanced, good public docs integration, testing considerations documented. Autonomous work has brought it to a solid state.
- **Item 3 (Elevate Praxis to First-Class Operational Protocols)**: Officially activated. Structured phased plan now documented in `conv_task_elevate_praxis_to_operational_protocols.md`.
- Operating under `conv:arc:engram_roadmap_autonomous_execution` agreement. All progress recorded via `conv:task:*` documents and the checkpoint ritual.

## Relations

- Depends on: `conv:arc:leg3_compute_primitive` (the core `.leg` format)
- Enables: Multiple future agentic systems requiring long-term verifiable memory
- Related to: Deeper Codeland sheaf/functor/ops work on grounding and multi-sorted ontologies

---

**This document is itself a first-class memory artifact.** Any agent working on this roadmap should load it (or its high-CRS successor) during wake-up when touching this arc.