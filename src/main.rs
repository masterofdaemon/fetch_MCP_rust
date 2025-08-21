use rmcp::{
    tool, tool_router, tool_handler,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    handler::server::router::tool::ToolRouter,
    handler::server::tool::Parameters,
    transport::stdio,
    ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::future::Future;
use tracing_subscriber::{fmt, EnvFilter};
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchRequest {
    #[schemars(description = "The URL to fetch via HTTP GET")]
    pub url: String,
    #[schemars(description = "Optional request timeout in seconds (default: 15)")]
    pub timeout_secs: Option<u64>,
    #[schemars(description = "Optional maximum bytes to return (default: 1MB)")]
    pub max_bytes: Option<usize>,
}

#[derive(Clone)]
pub struct FetchServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl FetchServer {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }

    #[tool(name = "RFetch", description = "HTTP GET fetcher that returns response body as text")]
    async fn fetch(&self, params: Parameters<FetchRequest>) -> Result<CallToolResult, rmcp::Error> {
        let req = params.0;
        let timeout = std::time::Duration::from_secs(req.timeout_secs.unwrap_or(15));
        let max_bytes = req.max_bytes.unwrap_or(1_000_000);

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;

        let fut = async move {
            let resp = client
                .get(&req.url)
                .send()
                .await
                .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;
            let status = resp.status();
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| rmcp::Error::internal_error(e.to_string(), None))?;
            let limited = if bytes.len() > max_bytes { &bytes[..max_bytes] } else { &bytes[..] };
            let mut body = String::from_utf8_lossy(limited).to_string();
            if !status.is_success() {
                body = format!("HTTP {}\n{}", status.as_u16(), body);
            }
            Ok::<_, rmcp::Error>(CallToolResult::success(vec![Content::text(body)]))
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(res) => res,
            Err(_) => Err(rmcp::Error::internal_error("fetch timeout", None)),
        }
    }
}

#[tool_handler]
impl rmcp::ServerHandler for FetchServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Fetch MCP server: perform HTTP GET requests".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _ = fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with_writer(io::stderr)
        .try_init();

    let service = FetchServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("Error starting server: {e}"))?;

    service.waiting().await?;
    Ok(())
}
