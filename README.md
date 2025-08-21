# Fetch MCP server (Rust, stdio)

A minimal, production-ready MCP server implemented in Rust using the `rmcp` crate and a stdio transport. It exposes a single safe HTTP GET tool you can call from MCP clients to fetch text from URLs, with optional timeout and response-size limits.

Why this repo?
- Clean, idiomatic Rust with async (Tokio) and a tiny surface area.
- Great starter template for building more MCP tools using `rmcp` macros.
- Safe-by-default: timeouts and max-bytes guardrails.

## Features
- RFetch tool: perform HTTP GET and return the response body as text
  - Optional `timeout_secs` (default 15s)
  - Optional `max_bytes` (default 1MB)
  - Follows up to 5 redirects
- Stdio transport: easy to wire into MCP-capable clients

## Quick start
Prerequisites: Rust toolchain (1.75+ recommended)

Build:
```
cargo build --release
```

Run (stdio):
```
cargo run --quiet
```

## Install (from source)
Install the binary to your Cargo bin directory (~/.cargo/bin by default):
```
cargo install --path .
```
Then run it directly (stdio server):
```
~/.cargo/bin/fetch_MCP_rust
```

## Set up in LLM agents (MCP stdio)
Most MCP-capable clients can launch a stdio server by running a command.
Use the installed binary path (e.g., ~/.cargo/bin/fetch_MCP_rust):

- Generic MCP client configuration (conceptual):
  - Command: ~/.cargo/bin/fetch_MCP_rust
  - Args: []

- Cursor (example): add an entry to your MCP servers configuration that runs the binary:
```json
{
  "mcpServers": {
    "fetch-mcp": {
      "command": "~/.cargo/bin/fetch_MCP_rust",
      "args": []
    }
  }
}
```
Restart Cursor if needed so it discovers the server and the `RFetch` tool.

- Warp (example): open Warp AI, go to Tools (or MCP servers) and add a new server:
  - Command: ~/.cargo/bin/fetch_MCP_rust
  - Args: []
After adding, Warp should list the `RFetch` tool for use in the agent.

## Tool API
Tool name: `RFetch`
Description: HTTP GET fetcher that returns response body as text

Parameters (JSON schema):
- `url` (string, required): The URL to fetch
- `timeout_secs` (integer, optional): Request timeout in seconds (default 15)
- `max_bytes` (integer, optional): Max bytes to return (default 1_000_000)

Example calls (from an MCP client):
```json
{
  "name": "RFetch",
  "arguments": { "url": "https://example.com" }
}
```
```json
{
  "name": "RFetch",
  "arguments": { "url": "https://example.com", "timeout_secs": 10, "max_bytes": 65536 }
}
```

## Use with MCP Inspector
1) Install and run the Inspector:
```
npx @modelcontextprotocol/inspector
```
2) In the Inspector, configure a stdio server that spawns this binary (path to your built executable). The server advertises the `RFetch` tool automatically.

## Extend with more tools
This project uses `rmcp` macros — `#[tool_router]`, `#[tool]`, and `#[tool_handler]`. Add additional `#[tool]` functions to the `FetchServer` impl in `src/main.rs` to grow your toolset.

## Contributing
Issues and PRs are welcome. Please keep code idiomatic, documented, and tested. Consider adding examples and integration tests for new tools.

## License
MIT or Apache-2.0 (match your preference for redistribution).

