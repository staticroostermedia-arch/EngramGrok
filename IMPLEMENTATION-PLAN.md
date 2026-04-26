# IMPLEMENTATION PLAN: Monad Agent Memory Pre-Contextualization System
# See full plan: /home/a/.gemini/antigravity/brain/306c13e9-3cea-4a77-99c1-27497f667441/agent_precontextualization_plan.md

## Phase Sequence

1. **Phase 1** (~2h) — `mcp_engram_session_start` returns full hydration payload on first call
   - Files: `engram-server/src/mcp.rs`, `engram-server/src/store.rs`
   - Gate: one call = full context. Agent is not blind after session_start.

2. **Phase 2** (~3h) — `/api/hydrate` REST endpoint on port 8087, alongside MCP
   - Files: `engram-server/src/main.rs`, `engram-server/src/serve.rs`
   - Gate: `curl http://localhost:8087/api/hydrate` returns JSON hydration payload

3. **Phase 3** (~2h) — ki_hijacker: immediate bake, name-based genesis, session trigger
   - Files: `engram-server/src/ki_hijacker.rs`
   - Gate: KI file exists within 1s of server spawn, includes genesis blocks by name

4. **Phase 4** (~1d) — Gemma 4B scout pipeline (DuckDuckGo/Brave → 27B → manifold)
   - Files: `engram-server/src/bin/scout.rs` (new), or extend `monad_nemo`
   - Gate: `mcp_engram_scout({query})` dispatches scout, results appear in KI within 30s

5. **Phase 5** (~1d) — Moltbook episodic memory integration
   - Files: `engram-server/src/moltbook.rs` (new)
   - Gate: every post → ZEDOS_EPISODIC block; history retrievable via MCP

## Key Invariants
- FHRR is lexical (BLAKE3). Use read_concept(name), NOT recall(query) for genesis blocks.
- SharedStore is Arc<Mutex<Store>>. REST + MCP share one lock.
- D1=0.74, D4=0.02. Do not revert D4 to 0.05 (Moltbook depth inflation).
- BVH builds are async. Never call rebuild_bvh() in any query hot path.
- Pinned blocks (CRS=1.0) survive autophagy. Genesis blocks must be pinned.

## Deployment Notes
- Binary installed at: `/home/a/.local/bin/engram` (IDE MCP path — must update this one)
- `cargo install` writes to `~/.cargo/bin/engram` — **always copy to `.local/bin` too**
- Command: `cargo build --release -p engram-server && cp target/release/engram ~/.local/bin/engram`
- IDE must be restarted to unlock the binary before copying (Text file busy otherwise)

## Status
- [x] Recall latency fix (D4 rebalance + async BVH) — commit d605235
- [x] Phase 1 — `session_start` hydration payload (genesis + history + stats)
- [x] Phase 3 — ki_hijacker immediate bake on spawn, Genesis Layer by name
     - Verified: KI baked in <2s, 29KB, 5/5 genesis, 8 gold, 6 hot, 1 episodic
     - Binary deployment fix: .local/bin PATH collision resolved
- [ ] Phase 2 — /api/hydrate REST endpoint
- [ ] Phase 4 — Gemma 4B scout
- [ ] Phase 5 — Moltbook integration
