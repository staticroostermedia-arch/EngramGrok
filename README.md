# Engram

> **Persistent geometric memory for AI agents.**  
> Patent Pending US19/372,256 — Aric Goodman & Static Rooster Media

Engram gives your AI agent a long-term memory that works like human associative memory — store anything, retrieve by meaning, not keywords. No vector database. No cloud. No API key. Runs entirely on your machine.

---

## How It Works

Every piece of knowledge is stored as a **HolographicBlock** — a 256KB binary container (`.leg` file) in the LEG container format (U.S. Patent Application 19/372,256, pending). Each block contains:

- A **8192-dimensional complex phase vector** — the geometric fingerprint of the concept
- A **momentum tensor** — encodes relational binding state  
- A **CRS score** — geometric memory health [0.0, 1.0]
- A **ProvLog payload** — the original source text
- A **BLAKE3 Merkle footer** — cryptographic lineage, no registry required

Memories are retrieved by **geometric similarity** — the system finds memories whose phase vectors point in the same direction as your query.

---

## Quick Start

> **Integration configs** for all supported IDEs are in [`integrations/`](integrations/)

### Install

```bash
cargo install engram --git https://github.com/StaticRoosterMedia/engram
```

### Google Antigravity IDE

Engram ships a ready-to-use config for Google Antigravity IDE.

**One-time install:**
```bash
cargo install --git https://github.com/staticroostermedia-arch/engram engram
```

**Add to `~/.gemini/antigravity/mcp_config.json`:**
```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "~/.engram/manifold"],
      "disabled": false
    }
  }
}
```

Or copy the template directly:
```bash
cp integrations/antigravity/mcp_config.json ~/.gemini/antigravity/mcp_config.json
```

Click **Manage MCP Servers → Refresh** in the Antigravity side panel. The `remember`, `recall`, `forget`, and `list_concepts` tools will appear immediately. Antigravity can now build a persistent geometric memory of your codebase — facts, context, and decisions that survive across sessions.

---

### Claude Desktop Integration

Add to `~/.config/claude-desktop/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "~/.engram/manifold"]
    }
  }
}
```

Restart Claude Desktop. You'll have four new tools: `remember`, `recall`, `forget`, `list_concepts`.

### Cursor Integration

Add to `.cursor/mcp.json` in your project:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp", "--store", "${workspaceFolder}/.engram"]
    }
  }
}
```

### CLI Usage

```bash
# Store a memory
engram-cli remember krebs_cycle "The Krebs cycle converts acetyl-CoA to ATP via 8 enzymatic steps"

# Find semantically similar memories
engram-cli recall "how does cellular respiration produce energy"

# List everything stored
engram-cli list

# Delete a memory
engram-cli forget krebs_cycle
```

### REST API Usage

You can also run Engram as an HTTP server for native integration with any language or client (NodeJS, Python, Go):

```bash
# Start the local server
engram serve --port 3456
```

Secure it by passing the `ENGRAM_API_KEY` environment variable, which enables Bearer Token enforcement.

```bash
curl -X POST http://127.0.0.1:3456/api/recall \
  -H "Authorization: Bearer YOUR_SECRET" \
  -H "Content-Type: application/json" \
  -d '{"query": "krebs cycle", "k": 3}'
```

### VS Code Extension

The `vscode-engram` package gives your IDE native GUI access to the geometric memory.

1. Install NodeJS/NPM on your main machine.
2. Run `cd extensions/vscode && npm install && npm run compile`.
3. Sideload the `extensions/vscode` directory into VS Code.
4. Highlight any code and execute **Engram: Remember Selection**. Or use **Engram: Recall Context** to search geometry directly via the Output pane.

---

## Architecture

```
engram/
├── crates/
│   ├── engram-core/     # LEG format, VSA ops, encoder, direct I/O storage
│   ├── engram-gpu/      # CUDA backend (Phase 2 — BVH index, parallel cosine)
│   ├── engram-server/   # MCP + REST server (Claude Desktop / Cursor)
│   └── engram-cli/      # CLI tool
└── extensions/
    └── vscode/          # VS Code extension (Phase 4)
```

### The VSA Operations

Engram uses **Fourier Holographic Reduced Representations (FHRR)** — all vectors live on the unit hypersphere in 8192-dimensional complex space:

| Operation | Description |
|-----------|-------------|
| `op_bind(role, filler)` | Associate two concepts — encodes `role → filler` |
| `op_add(a, b)` | Superpose two memories — result is similar to both |
| `cosine_similarity(a, b)` | Geometric similarity ∈ [−1.0, 1.0] |
| `holographic_unbind(result, role)` | Recover filler from bound result |
| `op_invert(v)` | Negate a concept (π rotation) |

---

## Hardware Support

Engram maps identically across the top three hardware architectures. Enable them in Cargo via features:

| Backend | Flag | Status | Notes |
|---------|------|--------|-------|
| CPU (Rayon) | Default | ✅ v1.0 | Linear scan natively, works on any machine |
| CUDA (NVIDIA) | `cuda-kernels` | ✅ v1.0 | BVH O(log N) index, parallel kernel computation |
| ROCm (AMD) | `rocm-kernels` | ✅ v1.0 | Wavefront HIP execution |
| Metal (Apple) | `metal` (Auto) | ✅ v1.0 | macOS MSL dynamic runtime compilation via metal-rs |

---

## License & Patent

This software is licensed under **AGPL-3.0-only**.

The `.LEG` container format is covered by **U.S. Patent Application No. 19/372,256** (pending),  
*Self-Contained Variable File System (.LEG Container Format)*,  
Applicant: **Aric Goodman**, Oregon, USA — Static Rooster Media.

This is a **reference implementation**. Commercial licenses (including for SaaS/cloud use) are available.  
Contact: **StaticRoosterMedia@gmail.com**

See [PATENT-NOTICE.md](PATENT-NOTICE.md) for full details.

---

*Built by Aric Goodman & Static Rooster Media · Oregon, USA*
