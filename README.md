# RMCP Macro-based MCP Server (stdio)

This is a minimal MCP server built from scratch using the RMCP Rust SDK, using macros for tools and a stdio transport.

What it provides
- Tools:
  - increment: increments an in-memory counter and returns the value
  - get: returns the current counter value
- Transport: stdio (spawnable by MCP-capable clients)

Prerequisites
- Rust toolchain (1.75+ recommended)

Build
```
cargo build --release
```

Run (stdio)
```
cargo run --quiet
```

Integrate with MCP Inspector
```
npx @modelcontextprotocol/inspector
```
Then configure the inspector to spawn this binary via stdio.

Notes
- The server uses RMCP macros: #[tool_router], #[tool], and #[tool_handler].
- You can add more tools inside src/main.rs by adding new #[tool] fns to the CounterServer impl.

