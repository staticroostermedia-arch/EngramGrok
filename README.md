# Engram — A Local Memory Backend for AI Agents

[![Build Status](https://github.com/staticroostermedia-arch/engram/actions/workflows/rust.yml/badge.svg)](https://github.com/staticroostermedia-arch/engram/actions)
[![MCP](https://img.shields.io/badge/MCP-Native-blue)](https://github.com/modelcontextprotocol)
[![Glama](https://glama.ai/mcp/servers/staticroostermedia-arch/engram/badge)](https://glama.ai/mcp/servers/staticroostermedia-arch/engram)

> **Engram acts like a localized hippocampus for your AI.**

Engram gives your AI agent long-term memory that works associatively. It acts as a lightweight, **local vector database** that seamlessly stores thoughts, facts, and context, allowing models to recall semantic meaning instead of just explicit keywords. 

Best of all: 
- 🔒 **100% Local**: No cloud, no external vector hosting required.
- ⚡ **Lightning Fast**: Built in Rust. Stream optimizations directly from disk to your queries.
- 🛠 **Native MCP Integration**: Plug-and-play with the Model Context Protocol (MCP).

---

## 🚀 Quick Start

To install Engram, simply compile it from the source!

```bash
cargo install engram-server --git https://github.com/staticroostermedia-arch/engram
```

Then, add it to your agent’s MCP configuration file (like Claude Desktop or Google Antigravity):

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp"]
    }
  }
}
```

Restart your IDE or chat client, and your agent instantly gains access to over 20 embedded memory tools!

---

## 🧰 How It Works

Engram acts as a highly optimized semantic memory layer. 

Instead of traditional flat-search string matching, Engram intelligently blends Hybrid Structural Accumulation constraints with **Neural Embeddings** to accurately retrieve answers to natural language questions based on context. 

As a conversational agent stores snippets of code, project rules, or chat records via the `remember` tool, Engram silently maps them across a semantic vector space. When the agent uses `recall`, it retrieves highly-clustered arrays of relevant concepts.

### Included Tools

Here are just a few of the capabilities you get out of the box:

- `remember` — Encode text and natively store it as a persistent knowledge block.
- `recall` — Perform localized semantic similarity checks and return top-k insights.
- `mcp_engram_set_namespace` — Isolate memory spaces by project so your active code bases never cross-contaminate.
- `mcp_engram_summarize` — Generate an immediate snapshot digest of core project priorities (perfect for session boots!).
- `mcp_engram_forget_old` — Autophagy logic that automatically handles background memory-cleanup for extremely stale artifacts.

---

## 🌐 Project Namespaces ("Stalks")

Working on multiple apps? Isolate context quickly and seamlessly:

```
mcp_engram_set_namespace("my_coding_project")
```
When switched, your memory retrievals will prioritize the semantic mappings isolated to that specific workspace.

---

## 💻 IDE Integration Configurations

You can plug Engram quickly into your favorite tools:

### Claude Desktop
```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp"]
    }
  }
}
```

### Google Antigravity IDE
```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["mcp"]
    }
  }
}
```

---

## ⚙️ Fast & Easy Setup

Engram is fully optimized on Rust, maintaining dynamic acceleration arrays using internal data processing constraints to drastically reduce query latency when filtering down large-scale vector embeddings. Because the entire storage is encapsulated to your local machine (`~/.engram`), it is completely private and accessible even offline!

---

## 📄 License
This software is licensed under **AGPL-3.0-only**.

*(Commercial licenses available for Cloud/Enterprise upon reaching out to staticroostermedia@gmail.com)*
