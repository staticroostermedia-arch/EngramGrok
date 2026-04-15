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
