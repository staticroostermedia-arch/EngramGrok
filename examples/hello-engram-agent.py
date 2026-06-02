# examples/hello-engram-agent.py
# Tiny self-contained "hello world" for agents using Enram.
# Demonstrates loading the public skills/ and one full ritual loop end-to-end.
#
# Run (after engram MCP is available via the client):
#   PYTHONPATH=integrations/python python examples/hello-engram-agent.py
#
# It reads the public docs/skills/*.md (no .grok/ dep) and walks:
#   wake-up (session_start + anchors) -> working-memory discipline (meta-work with tile) -> session-end (handoff)
#   -> "next instance" rehydrate simulation.
#
# Uses the demo client shim from mcp_client.py patterns. For live, import real EngramClient.
# Current build: target/debug/engram (cargo build; see GITHUB_MVP_PREP_PLAN.md).
# Follows engram rituals exactly as published for external agents.

import os
import sys

# --- Minimal EngramClient shim (copy/adapt from examples/mcp_client.py or integrations/python/engram_client.py) ---
class EngramClient:
    def __init__(self):
        print("[Enram] Connected to MCP (replace with real client for live calls).")
    def session_start(self, intent):
        print(f"[MCP] session_start: {intent}  # loads process sheaf, Phase 1.5 lawfulness, binds continuation")
    def remember(self, concept, text):
        print(f"[MCP] remember {concept}")
    def update(self, concept, new_text):
        print(f"[MCP] update {concept} (evolutionary)")
    def record_reasoning_trace(self, **kwargs):
        print(f"[MCP] record_reasoning_trace: {kwargs.get('decision_point')}")
        return f"trace:{kwargs.get('decision_point', 'demo')[:20]}_demo"
    def thought_tile_create(self, tile_type, payload, spatial_references=None):
        print(f"[MCP] thought_tile_create type={tile_type}")
        return f"tile:{tile_type}_demo"
    def promote_hot(self, concept):
        print(f"[MCP] promote_hot {concept}")
    def relate(self, a, b, label):
        print(f"[MCP] relate {a} ->[{label}] {b}")
    def verify_manifold_integrity(self, **k):
        print("[MCP] verify_manifold_integrity -> healthy (0 issues)")
    def spatial_status(self):
        print("[MCP] spatial_status: passive ingested (watch + events)")
    def session_end(self, summary, prepare_compression=True):
        print(f"[MCP] session_end: COMPRESS markers, handoff produced for next wake. {summary[:80]}...")

client = EngramClient()

# --- Load public skills (the key: agents read the published .md files directly) ---
def load_skill(name):
    path = os.path.join("docs", "skills", f"{name}.md")
    if os.path.exists(path):
        with open(path) as f:
            return f.read()[:500] + "... [truncated for demo; load full in real agent context]"
    return f"(skill {name} not found in docs/skills/ - ensure repo layout)"

print("\n=== Agent loads published skills for Enram rituals ===")
wake = load_skill("engram-wake-up")
wm = load_skill("engram-working-memory")
end = load_skill("engram-session-end")
tiles = load_skill("engram-thought-tiles")
print("Loaded: wake-up, working-memory, session-end, thought-tiles (from docs/skills/)")

# --- Full loop simulation (follow the protocols) ---
print("\n=== 1. WAKE-UP (per engram-wake-up.md) ===")
client.session_start(intent="hello-engram-agent.py demo - full ritual loop for external agents")
# (in real: query momentum for anchors, verify, spatial_status, relate continuation, rehydrate goals/traces/tiles)
client.verify_manifold_integrity(min_crs=0.6)
client.spatial_status()

print("\n=== 2. WORKING-MEMORY + HEAVY META-WORK (per engram-working-memory.md) ===")
# Geometric first: but here we do meta-work so escalate to tile
trace = client.record_reasoning_trace(
    decision_point="Implement richer agent demos for public repo",
    justification="Addresses 'missing items' for external agents to have full rituals + examples. Dogfood the system.",
    alternatives_considered="Just update plan; add minimal py only",
    falsifiability="If agents can't load skills/ and run loop, value is low.",
    spatial_context="docs/skills/ + examples/",
    ritual_context="engram-working-memory + thought-tiles for meta"
)

# Meta-work -> tile (mandatory per heuristics)
tile = client.thought_tile_create(
    tile_type="knowledge_graph",
    payload="Full cycle demo plan: wake->meta(tile, trace, sub-agent governance example)->end->rehydrate. Links to SKILLS.md, sub_agent_governance.md.",
    spatial_references=["examples/hello-engram-agent.py", "docs/skills/", "docs/GITHUB_MVP_PREP_PLAN.md"]
)
client.promote_hot(tile)
client.remember("demo:full_cycle_intent", "Agent now has published skills + governance example + hello script.")

print("\n=== 3. SESSION-END (per engram-session-end.md) ===")
client.session_end(
    summary=f"Demo loop complete. Produced {trace} + {tile}. Handoff ready. Next wake will rehydrate via momentum + hot tiles + continuation bundle. Sub-agent governance patterns documented.",
    prepare_compression=True
)

print("\n=== 4. NEXT INSTANCE REHYDRATE SIM (simulated wake) ===")
print("(Next agent instance would: session_start, follow wake-up.md to surface hot tile + trace via momentum/relations, recall 'demo:full_cycle_intent', continue work with working-memory discipline. No flat reset.)")

print("\n=== Hello complete. ===")
print("Your agent should: 1) Connect engram MCP 2) Load docs/skills/*.md at start 3) Follow exactly for real geometric continuation.")
print("See SKILLS.md (root), docs/skills/README.md, docs/examples/sub_agent_governance.md, and the full cycle doc for more.")