# Upgrade: rmcp 0.1 → 1.x

## Problem

`rmcp` 0.1 pulls `paste` (unmaintained, RUSTSEC-2024-0436). The only
remaining cargo audit warning. `rmcp` 1.x resolves this.

## Breaking Changes in rmcp 1.x

- `transport-sse-server` feature removed → `transport-streamable-http-server`
- MCP protocol: SSE replaced by Streamable HTTP
- `ServerHandler` trait may have changed
- `Tool` struct / `tool_list()` API may have changed
- `SseServer::serve` → new transport API

## Read First

- rmcp changelog / migration guide: https://github.com/anthropics/rmcp
- rmcp 1.x docs: https://docs.rs/rmcp/latest
- MCP Streamable HTTP spec

## Source Changes

### Cargo.toml

```toml
# Before
rmcp = { version = "0.1", features = ["server", "transport-io", "transport-sse-server"] }

# After
rmcp = { version = "1", features = ["server", "transport-io", "transport-streamable-http-server"] }
```

### src/mcp/mod.rs

- [ ] Update `ServerHandler` impl if trait changed
- [ ] Update `ServerInfo`, `ServerCapabilities` if structs changed
- [ ] Update `list_resources`, `read_resource` if return types changed
- [ ] Update `Peer<RoleServer>` if peer API changed

### src/mcp/tools.rs

- [ ] Update `Tool::new` if constructor changed
- [ ] Update `tool_list()` return type if `Tool` struct changed
- [ ] Update schema helpers if `Arc<Map<String, Value>>` changed

### src/mcp/handlers.rs

- [ ] Update `CallToolResult` if response type changed
- [ ] Update error handling if `McpError` changed

### src/server.rs

- [ ] Replace `SseServer::serve` with Streamable HTTP transport
- [ ] Update `serve_sse` function (rename to `serve_http`?)
- [ ] Update CLI flag: `--sse` → `--http` (or keep `--sse` as alias?)
- [ ] Update retry/backoff logic for new transport
- [ ] Update shutdown integration for new transport

### src/cli.rs

- [ ] Update `Serve` command: `--sse` flag → `--http` (or alias)

### src/config.rs

- [ ] `serve.sse` → `serve.http` (or keep for backward compat?)
- [ ] `serve.sse_port` → `serve.http_port`

## Documentation Updates

### Specifications

- [ ] `docs/specifications/engine/server.md` — update transport table
  (SSE → Streamable HTTP), update CLI examples, update startup sequence
- [ ] `docs/specifications/integrations/mcp-clients.md` — update SSE
  client config (endpoint URL may change)
- [ ] `docs/specifications/tools/overview.md` — update transport list

### Implementation

- [ ] `docs/implementation/mcp-server.md` — update transport setup docs
- [ ] `docs/implementation/mcp-tool-pattern.md` — update if API changed

### Guides

- [ ] `docs/guides/ide-integration.md` — update SSE transport section
- [ ] `docs/guides/ci-cd.md` — update if serve command changed

### README

- [ ] Update Technology table: rmcp version / description
- [ ] Update Quick Start if `--sse` flag changed

## Test

- [ ] `cargo check`
- [ ] `cargo test` — all 357 tests pass
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo audit` — `paste` warning gone
- [ ] Manual: `llm-wiki serve` stdio works
- [ ] Manual: `llm-wiki serve --http` (or new flag) works
- [ ] Manual: MCP client connects via new transport

## llm-wiki-skills Update

Add a prompt in the skills project:

```
docs/prompts/update-mcp-transport.md
```

Content:
- SSE transport replaced by Streamable HTTP in rmcp 1.x
- Update any skill that references SSE endpoints or `--sse` flag
- Update `setup` skill with new serve command
- Update `bootstrap` skill if session init changed

## Notes

- This is the largest single upgrade — rmcp is the MCP protocol layer
- Do it in a feature branch
- The stdio transport is unlikely to change (it's the primary path)
- ACP transport (`agent-client-protocol`) is unaffected
