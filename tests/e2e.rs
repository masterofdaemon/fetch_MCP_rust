use std::{net::SocketAddr, time::Duration};

use rmcp::{
    model::{CallToolRequestParam, ClientInfo},
    transport::TokioChildProcess,
    ServiceExt,
};
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;

async fn run_mock_http(addr: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let body = b"hello from mock";
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(headers.as_bytes()).await;
            let _ = stream.write_all(body).await;
            let _ = stream.shutdown().await;
        });
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn e2e_rfetch_happy_path() -> anyhow::Result<()> {
    // Start mock HTTP server on an ephemeral port
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    // hand off listener to a task by converting into std listener then back
    drop(listener); // we will re-bind inside run_mock_http
    tokio::spawn(run_mock_http(local_addr));

    // Locate compiled binary under test
    let bin = match std::env::var("CARGO_BIN_EXE_fetch_MCP_rust") {
        Ok(p) => p,
        Err(_) => {
            // Fallback to target/<profile>/<bin-name>
            let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
            let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
            let mut p = std::path::PathBuf::from(target_dir);
            p.push(&profile);
            let exe = if cfg!(target_os = "windows") { "fetch_MCP_rust.exe" } else { "fetch_MCP_rust" };
            p.push(exe);
            p.to_string_lossy().to_string()
        }
    };

    // Start RMCP service as a child process (stdio transport)
    let transport = TokioChildProcess::new(Command::new(bin))?;
    let client = ClientInfo::default();
    let service = client.serve(transport).await?;

    // Allow a brief moment for server to initialize
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify tool list contains our tool (macro-generated)
    let tools = service.list_tools(Default::default()).await?;
    let names: Vec<String> = tools.tools.iter().map(|t| t.name.to_string()).collect();
    assert!(names.iter().any(|n| n == "RFetch"), "RFetch tool missing: {:?}", names);

    // Call RFetch against the mock server using integer JSON values
    let mut args = serde_json::Map::new();
    args.insert("url".into(), serde_json::Value::String(format!("http://{}", local_addr)));
    args.insert("timeout_secs".into(), serde_json::Value::Number(serde_json::Number::from(5u64)));
    args.insert("max_bytes".into(), serde_json::Value::Number(serde_json::Number::from(10000u64)));

    let result = service
        .call_tool(CallToolRequestParam {
            name: "RFetch".into(),
            arguments: Some(args),
        })
        .await?;

    let combined = format!("{:?}", result);
    assert!(combined.contains("hello from mock"), "unexpected result: {combined}");

    // Gracefully shut down
    let _ = service.cancel().await;
    Ok(())
}
