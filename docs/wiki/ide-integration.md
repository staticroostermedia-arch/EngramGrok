# IDE Integration: Claude Desktop, Cursor, and Antigravity

Engram boots natively as a **Model Context Protocol (MCP)** background service. Instead of forcing you to memorize CLI commands or setup API endpoints, Engram exposes its memory tools natively directly to your LLM agent.

## How it works
The MCP interface runs perfectly over standard operating system pipelines (Stdio). When you boot your IDE, it launches the `engram` binary silently in the background.

Whenever you type something like *"Hey Claude, how does the user login script work?"*, the Agent autonomously fires the `mcp_engram_recall` tool inside its tool-chest, querying your `.leg3` NVMe geometry map instantly.

## Supported Tools
As of Phase 2, the Engram background daemon exposes the following explicitly Agent-executable tools:
- `mcp_engram_remember`: Embed and lock text into memory natively.
- `mcp_engram_recall`: Perform semantic extraction and K-NN bounding box sweeps.
- `mcp_engram_watch_workspace`: Commands the Daemon to attach an OS-level file-watch thread against a codebase.
- `mcp_engram_pin`: Prevent a critical context from being garbage collected by Autophagy.
- `mcp_engram_remember_solution`: Creates a perfectly crystallized Bug/Resolution tag linked mathematically inside the graph to stop future agents from failing the same compilation.
- `mcp_engram_relate`: Link nodes geometrically via OP_BIND circular convolutions.
- `mcp_engram_context_for_file`: Actively pre-load the highest correlation file contexts when an Agent opens a new `.rs` or `.ts` script.

## Setup Instructions

### 1. Install Engram globally
First, confirm Engram is built and mapped to your system `$PATH`:
```bash
cargo install engram --git https://github.com/staticroostermedia-arch/engram
```
*Note: Make sure `~/.cargo/bin` is in your environment PATH.*

### 2. Configure Claude Desktop
Add these blocks to your `~/.config/claude-desktop/claude_desktop_config.json`:
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
Restart Claude Desktop. The Engram tools will automatically appear via the `hammer` tooling icon.

### 3. Configure Google Antigravity
Inside the Google Deepmind Antigravity environment, Engram fits in natively via the `mcpServers` block located in your AppData directory (`~/.gemini/antigravity/mcp_config.json`):

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
Reload the IDE window. The agent now possesses total persistence across sessions!
