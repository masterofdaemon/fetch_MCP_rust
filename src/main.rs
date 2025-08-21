use rmcp::{
    tool, tool_router, tool_handler,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    transport::stdio,
    ServiceExt,
};
use rmcp::handler::server::router::tool::ToolRouter;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Clone)]
pub struct CounterServer {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CounterServer {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, rmcp::Error> {
        let mut c = self.counter.lock().await;
        *c += 1;
        Ok(CallToolResult::success(vec![Content::text(c.to_string())]))
    }

    #[tool(description = "Get the current counter value")]
    async fn get(&self) -> Result<CallToolResult, rmcp::Error> {
        let c = self.counter.lock().await;
        Ok(CallToolResult::success(vec![Content::text(c.to_string())]))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for CounterServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple counter server".into()),
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
        .try_init();

    let service = CounterServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("Error starting server: {e}"))?;

    service.waiting().await?;
    Ok(())
}
