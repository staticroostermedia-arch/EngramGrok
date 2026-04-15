//! Model Context Protocol (MCP) server — JSON-RPC 2.0 over stdio.
//!
//! Implements the MCP specification (protocol version 2024-11-05).
//! Communicates over stdin/stdout, one JSON object per line.
//!
//! # Tools exposed to the LLM
//!
//! | Tool | Arguments | Description |
//! |------|-----------|-------------|
//! | `remember` | `concept: str, text: str` | Encode text and store as a memory |
//! | `recall` | `query: str, k: int` | Find k most similar memories |
//! | `forget` | `concept: str` | Delete a memory |
//! | `list_concepts` | — | List all stored concept names |
//!
//! # Claude Desktop config
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "engram": {
//!       "command": "/path/to/engram",
//!       "args": ["mcp", "--store", "~/.engram/manifold"]
//!     }
//!   }
//! }
//! ```

use crate::store::SharedStore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tracing::{debug, error, info, warn};

// ── JSON-RPC 2.0 types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Request {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct Response {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

impl Response {
    fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0", id, result: Some(result), error: None }
    }
    fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self { jsonrpc: "2.0", id, result: None, error: Some(RpcError { code, message: message.into() }) }
    }
}

// ── MCP tool definitions ──────────────────────────────────────────────────────

fn tool_list() -> Value {
    json!({
        "tools": [
            {
                "name": "remember",
                "description": "Encode text and store it as a persistent memory under a concept name. \
                                Use this to save facts, context, or information for later retrieval.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "concept": {
                            "type": "string",
                            "description": "A unique identifier for this memory (e.g. 'krebs_cycle', 'user_preference_dark_mode')"
                        },
                        "text": {
                            "type": "string",
                            "description": "The text content to encode and store"
                        }
                    },
                    "required": ["concept", "text"]
                }
            },
            {
                "name": "recall",
                "description": "Search persistent memory by semantic similarity. Returns the k most relevant \
                                memories for the given query. Use this to retrieve context before answering questions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language query describing what you want to find"
                        },
                        "k": {
                            "type": "integer",
                            "description": "Number of results to return (default: 5, max: 20)",
                            "default": 5
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "forget",
                "description": "Delete a specific memory by concept name.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "concept": {
                            "type": "string",
                            "description": "The concept name to delete"
                        }
                    },
                    "required": ["concept"]
                }
            },
            {
                "name": "list_concepts",
                "description": "List all concept names currently stored in memory.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "mcp_engram_watch_workspace",
                "description": "Tell the background Agentic Daemon to recursively watch a specific directory for native file-saves.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the workspace folder (e.g. /home/a/Documents/CodeLand)"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "mcp_engram_pin",
                "description": "Lock a project management concept into the manifold so the Autophagy Daemon never decays it.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "concept": {
                            "type": "string",
                            "description": "Concept tag to pin (e.g. 'task_board' or 'system_architecture')"
                        }
                    },
                    "required": ["concept"]
                }
            },
            {
                "name": "mcp_engram_set_active_stalk",
                "description": "Switch the active write-target stalk in Sheaf mode. New memories will be written to this stalk.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the stalk to activate (must be registered in sheaf.toml)"
                        }
                    },
                    "required": ["name"]
                }
            },
            {
                "name": "mcp_engram_list_stalks",
                "description": "List all registered Sheaf stalks and show which one is currently active.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "mcp_engram_status",
                "description": "Inspect a concept's memory health: CRS score, provenance tier, last accessed time, ZEDOS tag, and superposition depth.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "concept": { "type": "string", "description": "Concept key to inspect" }
                    },
                    "required": ["concept"]
                }
            },
            {
                "name": "mcp_engram_recent",
                "description": "Return the N most recently accessed concept names. Zero disk I/O — reads from the in-memory AccessIndex. Call at session start to orient to what changed while you were offline.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "n": { "type": "integer", "description": "Number of concepts to return (default: 10)" }
                    }
                }
            },
            {
                "name": "mcp_engram_update",
                "description": "Merge new information into an existing concept via op_add (superposition). The existing memory is NOT overwritten — the new text is geometrically merged using VSA superposition. Increments superposition_count.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "concept": { "type": "string", "description": "Concept key to update" },
                        "text": { "type": "string", "description": "New information to merge into the existing memory" }
                    },
                    "required": ["concept", "text"]
                }
            },
            {
                "name": "mcp_engram_relate",
                "description": "Bind two concepts via op_bind and store a directional relation block (ZEDOS_RELATION). The relation's merkle_sub_root cryptographically links both parent block signatures.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "concept_a": { "type": "string", "description": "Source concept" },
                        "concept_b": { "type": "string", "description": "Target concept" },
                        "label": { "type": "string", "description": "Relation label (e.g. 'depends_on', 'implements', 'caused_by')" }
                    },
                    "required": ["concept_a", "concept_b", "label"]
                }
            },
            {
                "name": "mcp_engram_remember_solution",
                "description": "Store a crystallized error→solution pair as a ZEDOS_PRAXIS block, auto-pinned to CRS=1.0. Solutions never autophagy. Use after solving a compiler error, a tricky bug, or any pattern you never want to rediscover.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "error_pattern": { "type": "string", "description": "The error or problem pattern (error message, concept, or description)" },
                        "solution": { "type": "string", "description": "The solution or approach that resolved it" }
                    },
                    "required": ["error_pattern", "solution"]
                }
            },
            {
                "name": "mcp_engram_context_for_file",
                "description": "Surface the top 5 most relevant memories for a given file path, enriched with language context from the file extension. Useful for proactive context loading when opening a file.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File path (e.g. /home/user/project/backend.rs)" }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "mcp_engram_export_context",
                "description": "Create a pinned ZEDOS_EPISODIC session summary block (CRS=1.0). The block fingerprints all concepts touched this session in its merkle_sub_root. Call before ending a session to give the next agent instant context.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "2-3 sentence summary of what was built or decided this session" }
                    },
                    "required": ["summary"]
                }
            }
        ]
    })
}

// ── Tool dispatch ─────────────────────────────────────────────────────────────

fn handle_tool_call(name: &str, args: &Value, store: &SharedStore) -> Value {
    match name {
        "remember" => {
            let concept = args["concept"].as_str().unwrap_or("").trim().to_string();
            let text    = args["text"].as_str().unwrap_or("").trim().to_string();

            if concept.is_empty() || text.is_empty() {
                return json!({
                    "content": [{ "type": "text", "text": "Error: concept and text are required." }],
                    "isError": true
                });
            }
            match store.lock().unwrap().remember(&concept, &text) {
                Ok(_) => {
                    info!("remembered: {concept}");
                    json!({
                        "content": [{
                            "type": "text",
                            "text": format!("✓ Stored memory: '{concept}' ({} chars)", text.len())
                        }]
                    })
                }
                Err(e) => json!({
                    "content": [{ "type": "text", "text": format!("Error storing memory: {e}") }],
                    "isError": true
                }),
            }
        }

        "recall" => {
            let query = args["query"].as_str().unwrap_or("").trim().to_string();
            let k = args["k"].as_u64().unwrap_or(5).min(20) as usize;

            if query.is_empty() {
                return json!({
                    "content": [{ "type": "text", "text": "Error: query is required." }],
                    "isError": true
                });
            }

            let results = store.lock().unwrap().recall(&query, k);
            if results.is_empty() {
                return json!({
                    "content": [{ "type": "text", "text": "No memories found." }]
                });
            }

            let mut output = format!("Found {} memories:\n\n", results.len());
            for (i, mem) in results.iter().enumerate() {
                output.push_str(&format!(
                    "**[{}] {}** (similarity: {:.3}, crs: {:.3})\n{}\n\n",
                    i + 1, mem.concept, mem.score, mem.crs,
                    if mem.provlog.is_empty() { "(no text content)" } else { mem.provlog.as_str() }
                ));
            }

            debug!("recall '{}' → {} results", query, results.len());
            json!({ "content": [{ "type": "text", "text": output.trim() }] })
        }

        "forget" => {
            let concept = args["concept"].as_str().unwrap_or("").trim().to_string();
            if concept.is_empty() {
                return json!({
                    "content": [{ "type": "text", "text": "Error: concept is required." }],
                    "isError": true
                });
            }
            match store.lock().unwrap().forget(&concept) {
                Ok(_) => {
                    info!("forgot: {concept}");
                    json!({ "content": [{ "type": "text", "text": format!("✓ Deleted memory: '{concept}'") }] })
                }
                Err(e) => json!({
                    "content": [{ "type": "text", "text": format!("Error: {e}") }],
                    "isError": true
                }),
            }
        }

        "list_concepts" => {
            let concepts = store.lock().unwrap().list();
            if concepts.is_empty() {
                json!({ "content": [{ "type": "text", "text": "No memories stored yet." }] })
            } else {
                let list = concepts.iter()
                    .map(|c| format!("  • {c}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                json!({ "content": [{ "type": "text", "text": format!("{} memories:\n{}", concepts.len(), list) }] })
            }
        }

        "mcp_engram_watch_workspace" => {
            let path = args["path"].as_str().unwrap_or("").trim().to_string();
            let lock = store.lock().unwrap();
            if let Some(daemon) = &lock.daemon {
                let d = daemon.clone();
                let p = path.clone();
                tokio::spawn(async move { d.set_watch_workspace(&p).await; });
            }
            json!({
                "content": [{ "type": "text", "text": format!("✓ Agentic Daemon now recursively watching: {}", path) }]
            })
        }

        "mcp_engram_pin" => {
            let concept = args["concept"].as_str().unwrap_or("").trim().to_string();
            let mut lock = store.lock().unwrap();
            if let Some(mut m) = lock.fetch_block(&concept) {
                m.crs_score = 1.0; // Pinned mathematically
                let _ = lock.store(&concept, m);
                json!({ "content": [{ "type": "text", "text": format!("✓ Pinned concept to CRS 1.0. Autophagy will ignore it.: {}", concept) }] })
            } else {
                json!({ "content": [{ "type": "text", "text": format!("Memory not found: {}", concept) }], "isError": true })
            }
        }

        "mcp_engram_set_active_stalk" => {
            let name = args["name"].as_str().unwrap_or("").trim().to_string();
            let lock = store.lock().unwrap();
            if lock.set_active_stalk(&name) {
                json!({ "content": [{ "type": "text", "text": format!("✓ Active stalk switched to: {}", name) }] })
            } else {
                json!({ "content": [{ "type": "text", "text": format!("Stalk '{}' not found in sheaf.toml — register it first.", name) }], "isError": true })
            }
        }

        "mcp_engram_list_stalks" => {
            let lock = store.lock().unwrap();
            if lock.is_sheaf_mode() {
                let names = lock.stalk_names();
                let active = lock.active_stalk_name();
                let list = names.iter().map(|n| {
                    if n == &active { format!("• {} ← active", n) } else { format!("  {}", n) }
                }).collect::<Vec<_>>().join("\n");
                json!({ "content": [{ "type": "text", "text": format!("Sheaf stalks:\n{}", list) }] })
            } else {
                json!({ "content": [{ "type": "text", "text": "Running in single-store mode. No sheaf.toml detected at ~/.engram/sheaf.toml." }] })
            }
        }

        "mcp_engram_status" => {
            let concept = args["concept"].as_str().unwrap_or("").trim().to_string();
            let mut lock = store.lock().unwrap();
            match lock.status(&concept) {
                Some(report) => json!({ "content": [{ "type": "text", "text": report }] }),
                None => json!({ "content": [{ "type": "text", "text": format!("Concept '{}' not found.", concept) }], "isError": true }),
            }
        }

        "mcp_engram_recent" => {
            let n = args["n"].as_u64().unwrap_or(10) as usize;
            let lock = store.lock().unwrap();
            let entries = lock.recent(n);
            if entries.is_empty() {
                json!({ "content": [{ "type": "text", "text": "No recently accessed concepts yet this session." }] })
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                let list = entries.iter().map(|(concept, ts)| {
                    let secs_ago = now.saturating_sub(*ts);
                    let age = if secs_ago < 60 { format!("{}s ago", secs_ago) }
                              else if secs_ago < 3600 { format!("{}m ago", secs_ago / 60) }
                              else { format!("{}h ago", secs_ago / 3600) };
                    format!("  • {} ({})", concept, age)
                }).collect::<Vec<_>>().join("\n");
                json!({ "content": [{ "type": "text", "text": format!("Recent {} concepts:\n{}", entries.len(), list) }] })
            }
        }

        "mcp_engram_update" => {
            let concept = args["concept"].as_str().unwrap_or("").trim().to_string();
            let text    = args["text"].as_str().unwrap_or("").trim().to_string();
            let mut lock = store.lock().unwrap();
            match lock.update(&concept, &text) {
                Ok(msg)  => json!({ "content": [{ "type": "text", "text": msg }] }),
                Err(e)   => json!({ "content": [{ "type": "text", "text": format!("Update failed: {}", e) }], "isError": true }),
            }
        }

        "mcp_engram_relate" => {
            let concept_a = args["concept_a"].as_str().unwrap_or("").trim().to_string();
            let concept_b = args["concept_b"].as_str().unwrap_or("").trim().to_string();
            let label     = args["label"].as_str().unwrap_or("relates_to").trim().to_string();
            let mut lock = store.lock().unwrap();
            match lock.relate(&concept_a, &concept_b, &label) {
                Ok(msg)  => json!({ "content": [{ "type": "text", "text": msg }] }),
                Err(e)   => json!({ "content": [{ "type": "text", "text": format!("Relate failed: {}", e) }], "isError": true }),
            }
        }

        "mcp_engram_remember_solution" => {
            let error_pattern = args["error_pattern"].as_str().unwrap_or("").trim().to_string();
            let solution      = args["solution"].as_str().unwrap_or("").trim().to_string();
            let mut lock = store.lock().unwrap();
            match lock.remember_solution(&error_pattern, &solution) {
                Ok(msg)  => json!({ "content": [{ "type": "text", "text": msg }] }),
                Err(e)   => json!({ "content": [{ "type": "text", "text": format!("remember_solution failed: {}", e) }], "isError": true }),
            }
        }

        "mcp_engram_context_for_file" => {
            let path = args["path"].as_str().unwrap_or("").trim().to_string();
            let mut lock = store.lock().unwrap();
            let memories = lock.context_for_file(&path);
            if memories.is_empty() {
                json!({ "content": [{ "type": "text", "text": format!("No relevant memories found for: {}", path) }] })
            } else {
                let out = memories.iter().enumerate().map(|(i, m)| {
                    let preview = m.provlog.chars().take(300).collect::<String>();
                    format!("[{}] {} (similarity: {:.3})\n{}", i + 1, m.concept, m.score, preview.trim())
                }).collect::<Vec<_>>().join("\n\n");
                json!({ "content": [{ "type": "text", "text": format!("Context for {}:\n\n{}", path, out) }] })
            }
        }

        "mcp_engram_export_context" => {
            let summary = args["summary"].as_str().unwrap_or("").trim().to_string();
            let mut lock = store.lock().unwrap();
            match lock.export_context(&summary) {
                Ok(msg)  => json!({ "content": [{ "type": "text", "text": msg }] }),
                Err(e)   => json!({ "content": [{ "type": "text", "text": format!("export_context failed: {}", e) }], "isError": true }),
            }
        }

        unknown => json!({
            "content": [{ "type": "text", "text": format!("Unknown tool: {unknown}") }],
            "isError": true
        }),
    }
}


// ── MCP request dispatch ──────────────────────────────────────────────────────

fn dispatch(req: Request, store: &SharedStore) -> Option<Response> {
    let id = req.id.clone();
    let params = req.params.unwrap_or(json!({}));

    // If ID is completely missing, it is a JSON-RPC notification.
    // The MCP client does not expect a response for notifications (e.g. notifications/initialized).
    if id.is_none() {
        return None;
    }

    let response = match req.method.as_str() {
        "initialize" => {
            Response::ok(id, json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "engram",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }))
        }

        "initialized" => {
            // Deprecated fallback (should be caught by is_none above if compliant)
            return None;
        }

        "tools/list" => Response::ok(id, tool_list()),

        "tools/call" => {
            let name = params["name"].as_str().unwrap_or("").to_string();
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let result = handle_tool_call(&name, &args, store);
            Response::ok(id, result)
        }

        "ping" => Response::ok(id, json!({})),

        unknown => {
            warn!("unknown method: {unknown}");
            Response::err(id, -32601, format!("Method not found: {unknown}"))
        }
    };

    Some(response)
}

// ── Server loop ───────────────────────────────────────────────────────────────

/// Run the MCP server, reading from stdin and writing to stdout.
/// Blocks until stdin is closed (i.e. the client disconnects).
pub fn run(store: SharedStore) -> anyhow::Result<()> {
    // ── Boot the Background Worker ─────────────────────────────────
    crate::store::StoreHandle::boot_daemon(store.clone());

    info!("Engram MCP server ready (protocol 2024-11-05)");
    info!("Store: {}", store.lock().unwrap().store_path());

    let stdin  = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => { error!("stdin read error: {e}"); break; }
        };
        if line.trim().is_empty() { continue; }

        debug!("→ {line}");

        let response_opt = match serde_json::from_str::<Request>(&line) {
            Ok(req) => dispatch(req, &store),
            Err(e) => Some(Response::err(None, -32700, format!("Parse error: {e}"))),
        };

        if let Some(response) = response_opt {
            let out_line = serde_json::to_string(&response)?;
            debug!("← {out_line}");
            writeln!(out, "{out_line}")?;
            out.flush()?;
        }
    }

    info!("Engram MCP server shutdown");
    Ok(())
}
