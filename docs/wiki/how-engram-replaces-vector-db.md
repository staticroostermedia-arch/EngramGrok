# How Engram Replaces Vector Databases for AI Agents

Most AI Agent and Model Context Protocol (MCP) systems currently rely on traditional Vector Databases (like Pinecone, Milvus, or Chroma). These systems require network connections, cloud hosting, API keys, and complex indexing mechanisms.

**Engram replaces this entirely using a completely native file-system approach.**

## The Problem with Vector Databases
When an agent is rapidly iterating on a codebase, traditional Vector DBs suffer from:
1. **Network Latency:** Context retrieval is physically bottlenecked by REST API calls. 
2. **Staleness:** Code changes frequently. Vector DBs must constantly re-compute indices (like HNSW), leading to massive CPU/RAM spikes or outdated context being fed to the agent.
3. **Black Box Storage:** A developer cannot simply open a file explorer and "look" at the semantic graph.

## The Engram Solution: O_DIRECT `.leg3` NVMe Streaming
Engram bypasses Vector Databases by utilizing an advanced native file-system optimization combined with Hyperdimensional Computing (VSA - Vector Symbolic Architectures).

1. **Deterministic Bounding:** Every single semantic vector chunk is stored in a highly strict, immutable `.leg3` block format on your SSD. 
2. **256KB Alignment:** The 262,144 bytes (`256KB`) payload constraint is not arbitrary. It perfectly maps to Linux/macOS block alignments.
3. **Bypassing the Cache:** Engram skips the traditional OS page-cache completely. When an agent queries its memory, Engram streams the arrays directly off the NVMe drive via Direct Memory Access (DMA) (`O_DIRECT` flag in Rust) directly into the processing thread.
4. **TurboQuant B=4 Retrieval:** Because the vectors are structured logarithmically rather than semantically, the core engine accelerates K-Nearest Neighbors via a quantized B=4 codebook, scanning gigabytes of geometrical project state natively in <0.05s on a basic laptop CPU.

## No Cloud. No API Key.
By executing as a headless binary over standard Stdio (MCP Protocol), your IDE is no longer chatting with the cloud to understand your repository. The memory lives adjacent to the code, natively linked directly to your file system kernel watcher (`inotify`).
