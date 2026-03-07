//! MCP server for LocalGPT Gen — exposes gen tools + core tools over stdio JSON-RPC.
//!
//! This allows external CLI backends (gemini-cli, claude cli, codex) and
//! MCP-capable editors (VS Code, Zed, Cursor) to drive the Bevy scene.
//!
//! Exposed tools:
//! - All gen tools (spawn, modify, camera, audio, behaviors, world, etc.)
//! - memory_search, memory_get (LocalGPT memory system)
//! - web_fetch, web_search (research during scene building)
//!
//! Protocol: MCP (Model Context Protocol) over stdio, JSON-RPC 2.0.
//! One JSON message per line on stdin/stdout.

use std::sync::Arc;

use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info};

use localgpt_core::agent::tools::Tool;
use localgpt_core::config::Config;

use crate::gen3d::GenBridge;

/// Run the MCP stdio server loop.
///
/// Reads JSON-RPC messages from stdin, dispatches to gen tools via the bridge,
/// and writes responses to stdout. Runs until stdin is closed.
pub async fn run_mcp_server(bridge: Arc<GenBridge>, config: Config) -> Result<()> {
    let tools = create_mcp_tools(bridge, &config)?;

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    info!("MCP server ready, waiting for initialize...");

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            // stdin closed
            info!("MCP server: stdin closed, shutting down");
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                debug!("MCP server: ignoring non-JSON line: {}", e);
                continue;
            }
        };

        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        // Notifications (no id) — handle but don't respond
        if id.is_none() {
            match method {
                "notifications/initialized" => {
                    info!("MCP server: client initialized");
                }
                "notifications/cancelled" => {
                    debug!("MCP server: received cancellation");
                }
                _ => {
                    debug!("MCP server: ignoring notification '{}'", method);
                }
            }
            continue;
        }

        let id = id.unwrap();

        let response = match method {
            "initialize" => handle_initialize(&params),
            "tools/list" => handle_tools_list(&tools),
            "tools/call" => handle_tools_call(&tools, &params).await,
            "ping" => Ok(json!({})),
            _ => Err(json!({
                "code": -32601,
                "message": format!("Method not found: {}", method),
            })),
        };

        let response_msg = match response {
            Ok(result) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result,
            }),
            Err(error) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": error,
            }),
        };

        let mut out = serde_json::to_string(&response_msg)?;
        out.push('\n');
        stdout.write_all(out.as_bytes()).await?;
        stdout.flush().await?;
    }

    Ok(())
}

fn handle_initialize(_params: &Value) -> Result<Value, Value> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "localgpt-gen",
            "version": env!("CARGO_PKG_VERSION"),
        }
    }))
}

fn handle_tools_list(tools: &[Box<dyn Tool>]) -> Result<Value, Value> {
    let tool_defs: Vec<Value> = tools
        .iter()
        .map(|t| {
            let schema = t.schema();
            json!({
                "name": schema.name,
                "description": schema.description,
                "inputSchema": schema.parameters,
            })
        })
        .collect();

    Ok(json!({ "tools": tool_defs }))
}

/// Create the combined tool set for the MCP server:
/// gen tools + safe core tools (memory_search, memory_get, web_fetch, web_search).
///
/// CLI tools (bash, read_file, write_file, edit_file) are excluded because
/// external CLI backends already have their own file/shell tools.
fn create_mcp_tools(bridge: Arc<GenBridge>, config: &Config) -> Result<Vec<Box<dyn Tool>>> {
    use localgpt_core::agent::tools::create_safe_tools;
    use localgpt_core::memory::MemoryManager;

    // Core tools: memory_search, memory_get, web_fetch, web_search
    let memory = MemoryManager::new_with_agent(&config.memory, "gen-mcp")?;
    let memory = Arc::new(memory);
    let mut tools = create_safe_tools(config, Some(memory))?;

    // Gen tools: all 28 scene manipulation tools
    tools.extend(crate::gen3d::tools::create_gen_tools(bridge));

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
    info!(
        "MCP server: {} tools available: {:?}",
        tools.len(),
        tool_names
    );

    Ok(tools)
}

async fn handle_tools_call(tools: &[Box<dyn Tool>], params: &Value) -> Result<Value, Value> {
    let tool_name = params.get("name").and_then(|n| n.as_str()).ok_or_else(|| {
        json!({
            "code": -32602,
            "message": "Missing 'name' parameter",
        })
    })?;

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let tool = tools
        .iter()
        .find(|t| t.name() == tool_name)
        .ok_or_else(|| {
            json!({
                "code": -32602,
                "message": format!("Unknown tool: {}", tool_name),
            })
        })?;

    let args_str = serde_json::to_string(&arguments).unwrap_or_default();

    match tool.execute(&args_str).await {
        Ok(output) => Ok(json!({
            "content": [{
                "type": "text",
                "text": output,
            }]
        })),
        Err(e) => Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Error: {}", e),
            }],
            "isError": true,
        })),
    }
}
