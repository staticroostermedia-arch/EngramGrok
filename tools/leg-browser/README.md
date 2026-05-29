## LEG Browser

A slick, local, read-only HTML viewer for .leg3 HolographicBlocks.

**Strictly respects the core block invariants** (see `leg_block_invariants_guardrail_v1`):
- No writes
- No changes to binary layout or alignment
- Only reads via safe paths or the running daemon

### Quick Start (for reviewing Phase 0 artifacts right now)

```bash
# From the Engram repo root
open tools/leg-browser/index.html
# or
python3 -m http.server 8765 --directory tools/leg-browser
# then visit http://localhost:8765
```

The sidebar is pre-loaded with the most important Phase 0 + Phase 1 review artifacts (mapping table, guardrail, integration handoff, review tiles from the Thought Tile Schema + A/D/R work, and the MCP code deltas).

**Important:** This is a *curated static snapshot* for convenient local/offline human review during the CodeLand integration. It does not automatically sync with the live Engram manifold. New Thought Tiles created via the MCP tools appear in the TUI (ki_hijacker, `recall`, `visualize`, `read_concept`). Use the exact concept names shown in the sidebar + the commands printed in the viewer panes to inspect full live content.

### Live Dynamic Mode (v0.3+ — current)
Open `tools/leg-browser/index.html` (or serve it locally). 

**It auto-probes for a running `engram serve` on port 3456** (the default REST server — see `docs/rest_api.md`).

- When detected: 
  - Connection pill turns emerald LIVE (auto or manual click).
  - **Live Recent + Momentum sidebar** appears and auto-refreshes (`/api/recent` every ~6.5s).
  - Clicking live items loads rich block details via `/api/block/:concept` (includes CRS, type, text, outgoing relations).
  - Hybrid momentum hints via `/api/recall`.
- All original curated Phase 0/1 review tiles (mapping table, guardrail, handoff, A/D/R tiles, etc.) remain as **graceful static fallback** in the lower sidebar when no server or offline.
- Perfect foundation for Obsidian-like experience: graph (Mermaid), full html_visualization Thought Tile injection (iframe srcdoc), 3-pane layout, Agent Activity Canvas, quick switcher, and filters coming in Phases B/C.

```bash
# Terminal A — start the live backend (enables dynamic mode)
engram serve

# Terminal B or browser — open the viewer
open tools/leg-browser/index.html
# or
python3 -m http.server 8765 --directory tools/leg-browser
```

The viewer is **single-file, zero-build, CDN-only** (Tailwind + Font Awesome + future Mermaid). It feels like a native Obsidian vault navigator for the geometric manifold / living sheaf.

### Future increments (under active development)
- Phase B: Full 3-pane multi-layout, perfect `html_visualization_*` payload injection, backlinks pane.
- Phase C: Quick switcher (⌘K), search/filters, Agent Activity Canvas (Primary Intent + serving traces/tiles/goals), dynamic Mermaid graph from relations + `/api/graph` (when backend adds it).
- Every significant change creates dual Thought Tiles (text + rich HTML viz) + full Code Edit Ritual traces, all linked to `goal:1780091465_codeland-integration-2026---systematically-incor` and this handoff.

Built as part of the CodeLand Integration 2026 effort (see `handoff:codeland_integration_2026_plan`) to give humans a first-class way to inspect the living substrate. Follows `ritual:code_edit_ritual_v1` on all edits.