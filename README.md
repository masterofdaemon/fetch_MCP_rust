# fetch_mcp_rust

High-performance Rust implementation of the MCP (Model Context Protocol) "fetch" server. It exposes a single tool, `fetch`, that performs HTTP(S) requests with an allowlist, timeouts, and response-size limits — optimized for low memory and CPU overhead.

- Language/runtime: Rust + tokio
- HTTP client: reqwest (rustls)
- Transport: JSON-RPC 2.0 over stdio with Content-Length framing (LSP-style)

## Features
- Fast and lightweight: streaming download with a configurable max-size cap
- Safe-by-default: HTTP(S)-only with URL allowlist
- Configurable: environment flags for allowlist, timeout, and size limits
- Simple interface: MCP initialize, tools/list, and tools/call("fetch")

## Quick start

### Build from source
Requirements: Rust (stable), cargo

```bash
# clone (if not already inside the repo)
# git clone https://github.com/your-org/fetch_mcp_rust
# cd fetch_mcp_rust

# build
cargo build --release

# optional: install to cargo bin
cargo install --path .
```

### Run (standalone, for debugging)
This server communicates via stdio. For manual debugging you can echo a framed JSON-RPC request.

```bash
# Minimal initialize request via Content-Length framing
printf 'Content-Length: 61\r\n\r\n{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  | ./target/release/fetch_mcp_rust
```

## Configuration
Environment variables:
- FETCH_ALLOWLIST: comma-separated URL globs; default: `https://*`
  - Examples: `https://example.com/*,https://*.rust-lang.org/*`
- FETCH_TIMEOUT_MS: per-request timeout in milliseconds; default: `15000`
- FETCH_MAX_BYTES: cap on response body bytes; default: `5242880` (5 MiB)

Note:
- By default, only HTTPS is allowed. HTTP is blocked unless explicitly enabled.
- To allow both protocols, set: `FETCH_ALLOWLIST="https://*,http://*"` (or specify host-specific rules, e.g., `https://example.com/*,http://example.com/*`).

Example:
```bash
export FETCH_ALLOWLIST="https://example.com/*,https://api.github.com/*"
export FETCH_TIMEOUT_MS=10000
export FETCH_MAX_BYTES=2097152
```

## Using with LLMs (MCP clients)
This binary is an MCP server. Any MCP-capable client/agent/LLM runtime that can launch stdio servers can use it. Configure the client to start the server command and pass any needed environment variables.

### Generic MCP client configuration (example)
```json
{
  "mcpServers": {
    "fetch": {
      "command": "/absolute/path/to/target/release/fetch_mcp_rust",
      "env": {
        "FETCH_ALLOWLIST": "https://example.com/*,https://*.rust-lang.org/*",
        "FETCH_TIMEOUT_MS": "15000",
        "FETCH_MAX_BYTES": "5242880"
      }
    }
  }
}
```

Once connected, the client can:
1) call `initialize` (handled automatically by clients)
2) call `tools/list` to discover the `fetch` tool
3) call `tools/call` with `name: "fetch"` and arguments

### tools/call request schema (client side)
- name: "fetch"
- arguments:
  - url (string, required)
  - method (string, default "GET")
  - headers (object<string,string>)
  - body (string for raw text, or object for JSON)
  - timeout_ms (number)
  - max_bytes (number)

### Example tools/call payload
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "fetch",
    "arguments": {
      "url": "https://example.com",
      "method": "GET",
      "timeout_ms": 5000,
      "max_bytes": 65536
    }
  }
}
```

### Example result
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "status": 200,
    "headers": [["content-type", "text/html; charset=UTF-8"], ["content-length", "1256"]],
    "body": {
      "type": "base64",
      "data": "...base64...",
      "truncated": false
    }
  }
}
```

Note: Body is always returned as base64 to safely support binary. If the response exceeds `max_bytes`, `truncated` will be `true` and the body will be clipped to the cap.

## Development

### Run tests
```bash
cargo test
```

### Project structure
- src/main.rs — server entry, JSON-RPC dispatch (initialize, tools/list, tools/call)
- src/mcp/jsonrpc.rs — JSON-RPC 2.0 types
- src/mcp/stdio.rs — Content-Length framing over stdio
- src/config.rs — environment-based configuration and allowlist
- src/fetch_tool.rs — HTTP fetch tool implementation (reqwest)

### Design notes
- Efficient streaming read with a hard cap prevents unbounded memory
- Small connection pools and compression enabled by default
- Allowlist uses globset on full URL string for simplicity; can be tightened to host/path components if needed

## Compatibility
- Platforms: macOS, Linux, Windows (where Rust/reqwest work)
- Protocol: MCP (JSON-RPC 2.0 over stdio). Compatible with any MCP client that can spawn stdio servers.

## Security
- HTTP(S)-only
- Allowlist required to reach hosts; defaults to `https://*` for convenience — restrict in production.
- Timeout and body-size caps mitigate resource abuse.

## License
Dual-licensed under either of:
- Apache License, Version 2.0
- MIT license

## Acknowledgements
Inspired by the reference MCP fetch server from the Model Context Protocol ecosystem.

