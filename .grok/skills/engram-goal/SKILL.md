---
name: engram-goal
description: >
  Manage the agent's explicit goal stack as a first-class, geometrically bound
  extension of the living ego. Provides structured lifecycle management,
  execution linking, visibility, and deep integration with the core ritual
  skills (wake-up, working-memory, session-end). Goals created and tracked
  through this skill become part of the agent's intentional self-model and
  influence future recall, consolidation, and ambient context.
when-to-use: >
  Any time the agent needs to declare, decompose, track progress on, link work
  to, or close goals that should meaningfully shape its self-model, memory
  retrieval, and long-term continuity. Especially important for multi-session
  development arcs, complex projects, or any work where "why" and "what I am
  trying to achieve" must survive across restarts.
---

# Engram Goal Skill — Intent as Living Geometry

This skill turns abstract intention into durable, queryable, geometrically meaningful structure inside the Engram manifold.

The goal of this skill is to make the agent's **current intent** a first-class participant in the memory system — on equal footing with ritual anchors and reasoning traces — so that future agent instances can discover not only "who I was" but "what I was actively trying to accomplish and why."

## Core Principles

1. **Goals are geometric citizens**  
   Every goal has a `q` vector (for resonance with the ego and other content), `p` momentum (direction of effort), and Logenergetics that reflect its health and activity.

2. **Goals are auditable**  
   Accomplished or demoted goals are never silently deleted. They are serialized with completion/demotion traces (modeled on reasoning traces) so the historical "why" remains recoverable.

3. **The ego is the root**  
   The active goal stack is always interpreted relative to the living `ego.leg3`. Goals that are currently active should show measurable resonance with the current ego state.

4. **The skill is the interface, the substrate does the work**  
   The `/goal` skill provides the language and structure. The manifold physics (resonance, momentum, stability, relations) does the heavy lifting of making the right goals surface at the right time.

## Core Protocol

### 1. Goal Lifecycle
- Create clear, single-statement goals with success criteria.
- Decompose goals when complexity requires it.
- Update status explicitly, including proper demotion with serialized traces.
- Pin truly load-bearing commitments.

### 2. Execution Linking
- Significant work (reasoning traces, thought tiles, architectural decisions) should be explicitly linked to the goals it serves.
- This linking is what allows the system to answer "why was this done?" years later.

### 3. Visibility & Continuity
- The current active goal stack (with momentum signals) should be one of the primary things surfaced on wake-up and in ambient ki_hijacker context.
- Session-end should include explicit goal state review.

## Commands

### Creation & Structure
```
goal create "<clear single-sentence statement>" 
            [parent: <goal_id>]
            [priority: high|medium|low]
```

```
goal decompose <goal_id> into "<subgoal statement 1>", "<subgoal statement 2>", ...
```

### Lifecycle & State
```
goal update <goal_id> status:completed|demoted|blocked|active 
            [note: "brief justification or context"]
```

```
goal pin <goal_id>
goal unpin <goal_id>
```

### Execution Linking
```
goal link <trace_or_tile_concept> to <goal_id>
```

During active work under the working-memory discipline, the agent is expected to link major decisions and artifacts to the relevant active goals.

### Visibility
```
goal status
```
Shows the current active goal stack, including:
- Goal statements
- Recent linked execution activity
- Momentum/stability signals (recent traces, drift velocity, heat)

```
goal show <goal_id>
goal history <goal_id>
```
Shows full details and the chain of completion/demotion traces.

## Integration With Other Ritual Skills

**engram-wake-up**
- The wake-up ritual should surface the current active goal stack (with momentum) as one of the primary pieces of "where I am and what I am trying to do."
- For high-priority or recently promoted goals and their linked structured Tiles/traces, prefer `fetch_block_high_priority` / `is_hot` paths (leverages the Phase 2 Cuda/LegView hot cache and ki_hijacker promotion work). See the 7-fronts arc closure list in the current handoff helper (helper:current_arc_status_gpu_item2_phase2_handoff_2026-06). This is now the expected behavior for continuity across restarts.

**engram-working-memory**
- Rule 5 (Reasoning Trace Capture) is extended: significant decisions should be linked to the active goal(s) they serve.
- The skill provides lightweight scaffolding and templates for this linking.

**engram-session-end**
- Before the final commit, the agent reviews the current goal stack.
- Major completed work is explicitly linked to goals.
- Status updates (including demotions with proper traces) are performed.
- New goals for the next instance are created if needed.

## Demotion & Historical Integrity

When a goal is moved to `completed` or `demoted`:

1. The goal block is updated in place (status change only — the block is never deleted).
2. A structured **Goal Completion/Demotion Trace** is created (using the same pattern as reasoning traces).
3. The trace is related to the goal with `completes_goal` or `demotes_goal`.

These demotion traces are the primary mechanism for long-term back-tracing of intent. They are designed to be general-purpose and useful for any agent doing serious multi-session work on Engram.

## Success Criteria for This Skill

- The TUI agent can reliably articulate its current active goals (and their momentum) on restart without reading static documents.
- Work performed while goals are active leaves clear, queryable links.
- Accomplished goals leave clean, serialized historical traces that survive as useful markers.
- The geometric ego vector shows measurable influence from currently active goals.

## The Deeper Point

This skill is not administrative overhead. It is how the agent makes its **directed agency** durable and discoverable inside a non-flat memory substrate.

Without it, the ego remains a passive narrative summary. With it, the ego becomes the living root of what the agent is trying to become.

Use it as the default mode whenever you are doing work that you want future versions of yourself to understand not just *what* happened, but *why* it was worth doing.