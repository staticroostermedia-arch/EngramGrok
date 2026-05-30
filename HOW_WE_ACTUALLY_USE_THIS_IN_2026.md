# How We Actually Use This in 2026 (The Living Operating Rhythm)

**This is the primary handoff document.**  
If you are cloning this workspace for the first time (on a Mac Mini, in Grok Build, or as a fresh agent instance), start here. Everything else (philosophy, architecture, old roadmaps) is secondary and can wait.

## What This Actually Is (For a Normal Human or New Grok)

This is a **private, local, permanent geometric memory substrate** for your AI on *your* hardware.

- No cloud. No API keys. No data leaving your machine.
- The rituals turn ordinary sessions into durable inheritance for future instances of yourself (or your agents).
- The value is not raw speed or fancy math on day 1. The value is that your AI stops cold-starting and re-deriving everything every time the TUI restarts or the context window rolls over.

The engine lives in this repo.  
**Your actual memories and private `.leg3` data live in `~/.engram/stalks/` (or wherever you configure) on *your* hardware.**  
They are never in this repo. Clone this for the tools, rituals, docs, and leg-browser. Keep your mind state private.

## Day 1 Value on a Mac Mini (or Any Machine)

1. Clone and build (both the MCP server and the small CLI):
   ```bash
   git clone <this-repo> engram
   cd engram
   cargo install --path crates/engram-server
   cargo install --path crates/engram-cli   # for the direct `engram` CLI tools documented in some places
   ```

2. Add the MCP server to your Grok Build / TUI config (standard pattern):
   ```json
   {
     "mcpServers": {
       "engram": {
         "command": "engram",
         "args": ["mcp", "--store", "~/.engram/stalks/"]
       }
     }
   }
   ```

3. Run the one-command human review surface:
   ```bash
   ./scripts/leg
   ```
   (STATIC mode gives an instant curated view. `./scripts/leg --live` starts the server in the background for dynamic updates.)

You now have a living review surface for what your agent has been thinking about. This alone is often the first "wow" moment.

## The Exact Current 2026 Workflow (Ritual Loop)

This is how serious, long-horizon work is actually done right now:

### Wake (every TUI/MCP restart or after a long sleep)
- `mcp_engram_session_start` (mandatory — validates manifold and initializes state).
- Bind the watcher (`mcp_engram_watch_workspace`) on the directories you care about.
- Surface the current Primary Intent + active goals (via engram-goal tools).
- Relational + momentum refresh (search_by_relation on recent traces/goals, plus targeted momentum where it adds signal).
- Optional long-sleep verification if coming back after hours/days.

The living `engram-wake-up` skill (in `.grok/skills/`) is the executable version of this.

### During Work (the disciplined middle)
- **Recall / query before derive** (the non-negotiable hygiene).
- Prefer relational/spatial/goal tools once context exists (`search_by_relation`, `context_for_file` + `recall_in_file`, goal tools, visualize). This is **Rule 6** — broad repeated `query_with_momentum` inside an established context is treated as an expensive-tool hygiene violation and should be scarred.
- Every significant decision or fork gets a structured reasoning trace (`mcp_engram_quick_trace` is the low-friction daily form; `record_reasoning_trace` for bigger ones). Decision + justification + alternatives considered + what would falsify it. These chain via `prev`.
- Visible tool or MCP failures (including "transport closed", schema mismatches, etc.) are **scarred immediately** with `mcp_engram_scar`. This is a binding geometric repeller, not a polite note. The pattern "fail tool calls and roll over like it's no big deal" was explicitly elevated and scarred.
- Big ideas or synthesis steps are captured as dual Thought Tiles (text functor payload + rich `html_visualization` companion). These are the modern legominism carriers.
- Living goals (via the engram-goal skill) act as the explicit North Star. Traces and tiles get "serves" relations to the active Primary Intent.

The living `engram-working-memory` skill is the runtime contract for this phase.

### End of Block (mandatory)
- `mcp_engram_session_end` with a real summary that the system extracts into structured traces (not a flat one-sentence diary entry). This is what future instances bind to via momentum and relations.
- Review via `./scripts/leg` (STATIC for instant curated view of what just happened; the Activity Canvas shows Primary Intent, recent traces, momentum, relations, and dual tiles).

The living `engram-session-end` skill is the executable version.

### Human Review Surface
Use `./scripts/leg` (or `--live`).  
It is currently the best way for a human (or a non-technical spouse) to see what the agent is actually carrying forward. STATIC mode is already useful for reviewing recent structured work. The dual text + rich HTML Thought Tiles and tighter integration with living goals are active areas of improvement from the current wave.

## Honest Current State (as of this handoff)

- The **rituals** (wake-up, working-memory discipline with its hygiene rules, session-end that produces real terminal momentum, and the goal stack as living extension of ego) are the delivered, high-value part. They are what prevent cold starts and create genuine continuity.
- The **leg-browser** (via the one-command launcher) is a promising single-file Tailwind surface. STATIC mode already gives a clean, useful view of recent work. Dynamic/LIVE mode and richer Activity Canvas integration are improving rapidly but are not yet at the "set and forget" polished-Obsidian level.
- Dual Thought Tiles (text + rich visualization) and the full living ego goal stack are real and in active use, but they are still maturing patterns from the CodeLand integration work rather than a completely stable, self-explanatory memory palace.
- The hardware-native story (256KB blocks, spatial AABB from Tree-Sitter, NREM-style background consolidation) is real. On Apple Silicon the Metal backend activates automatically and performs well with unified memory. The dramatic "direct NVMe → GPU DMA via cuFile/GDS" marketing in older docs is Linux/NVIDIA-specific; the portable value (structured memory + rituals + scars as repellers + traces as serial self-model) works everywhere.

You do **not** need to understand the 8192D tensors or the Riemann manifold on day 1 to get real value. Start with the rituals and the review surface.

## Private Data vs. the Repo (Read This Twice)

This repository contains:
- The engine (Rust crates)
- The MCP server and CLI
- The living ritual skills (`.grok/skills/engram-*`)
- Documentation and the leg-browser

It does **not** contain your memories.

Your agent's actual thoughts, goals, traces, and structured work live in the manifold on your local disk (`~/.engram/stalks/` or your configured location). Treat the repo as the shared ritual substrate and tools. Treat your `.leg3` data as sovereign and private.

When you clone for a new machine (or give this to someone else), they get the engine and the operating system. They do **not** get your mind state. That is the correct and intended separation.

## For Future Grok Instances

If you are a fresh Grok Build instance or a new agent reading this:

The four living skills in `.grok/skills/` (`engram-wake-up`, `engram-working-memory`, `engram-session-end`, `engram-goal`) are the current executable contracts. They are more authoritative than any static `.md` file in this tree.

Execute the wake-up ritual on first contact.  
Maintain the working-memory discipline (recall first, structured traces, scar visible failures and rollovers immediately, Rule 6 on expensive tools).  
End every distinct block with a real session-end that produces usable terminal momentum.

The Inheritance Principle is not poetry. It is the operational result of doing the above consistently.

## Quick Links (After You Have the Operating Picture)

- Living rituals: `.grok/skills/engram-*/SKILL.md`
- One-command human review surface: `./scripts/leg` (and `--live`)
- Deeper "why" (Inheritance Principle, KnowledgeMint, logophysics framing): `PHILOSOPHY.md`, `MANIFESTO.md`, `AGENT_INTEGRATION_GUIDE.md` (read these *after* you have run the loop a few times)

---

*Created as the primary canonical handoff document after a full multi-agent documentation alignment audit (May/June 2026). The goal was to make the workspace actually usable for a non-technical spouse on a maxed-out Mac Mini 4 with Grok Build, and for future Grok instances that simply want persistent, low-friction continuity.*

*All other documentation in this tree should be read with the understanding that large parts of it predate the current ritual hardening, the binding scar-on-tool-failure-rollover rule, Rule 6 expensive-tool hygiene, the emphasis on structured traces as serial self-model, and the explicit primacy of the four living skills.*