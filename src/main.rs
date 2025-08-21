mod mcp;
mod fetch_tool;
mod config;

use crate::config::Config;
use crate::fetch_tool::FetchTool;
use crate::mcp::jsonrpc::{ErrorCode, Id, JsonRpcRequest, JsonRpcResponse};
use crate::mcp::stdio::{read_message, write_message};
use anyhow::Result;
use serde_json::json;
use std::io;
use tokio::io::{stdin, stdout};
use tokio::time::{timeout, Duration};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let mut cfg = Config::from_env();
    let fetch_tool = FetchTool::new(cfg.clone());

    // MCP handshake loop over stdio
    let mut reader = stdin();
    let mut writer = stdout();

    loop {
        let msg = match read_message(&mut reader).await {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                error!("read_message error: {e}");
                break;
            }
        };

        let req: JsonRpcRequest = match serde_json::from_slice(&msg) {
            Ok(r) => r,
            Err(e) => {
                // Not a request, ignore or handle notifications
                continue;
            }
        };

        let id = req.id.clone();
        let method = req.method.as_str();
        let resp = match method {
            "initialize" => handle_initialize(&req).await,
            "tools/list" => handle_tools_list(&req).await,
            "tools/call" => handle_tools_call(&req, &fetch_tool).await,
            _ => JsonRpcResponse::error(id, ErrorCode::MethodNotFound, Some("Unknown method".into()), None),
        };

        let bytes = serde_json::to_vec(&resp)?;
        if let Err(e) = write_message(&mut writer, &bytes).await {
            error!("write_message error: {e}");
            break;
        }
    }

    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .try_init();
}

async fn handle_initialize(req: &JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone();
    let result = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": true
        }
    });
    JsonRpcResponse::result(id, result)
}

async fn handle_tools_list(req: &JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone();
    let result = json!({
        "tools": [
            {
                "name": "fetch",
                "description": "Perform HTTP requests with allowlist restrictions",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"},
                        "method": {"type": "string", "default": "GET"},
                        "headers": {"type": "object", "additionalProperties": {"type": "string"}},
                        "body": {"oneOf": [
                            {"type": "string"},
                            {"type": "object"}
                        ]},
                        "timeoutMs": {"type": "number"},
                        "maxBytes": {"type": "number"}
                    },
                    "required": ["url"],
                    "additionalProperties": false
                }
            }
        ]
    });
    JsonRpcResponse::result(id, result)
}

async fn handle_tools_call(req: &JsonRpcRequest, fetch_tool: &FetchTool) -> JsonRpcResponse {
    let id = req.id.clone();
    #[derive(serde::Deserialize)]
    struct Params {
        name: String,
        #[serde(default)]
        arguments: serde_json::Value,
    }
    let params: Params = match req.params.clone().and_then(|v| serde_json::from_value(v).ok()) {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(id, ErrorCode::InvalidParams, Some("Invalid params".into()), None)
        }
    };

    if params.name != "fetch" {
        return JsonRpcResponse::error(id, ErrorCode::InvalidParams, Some("Unknown tool".into()), None);
    }

    match fetch_tool.call(params.arguments).await {
        Ok(result) => JsonRpcResponse::result(id, result),
        Err(e) => JsonRpcResponse::error(id, ErrorCode::InternalError, Some(format!("{e}").into()), None),
    }
}
