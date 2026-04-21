# Study: rmcp 1.x Upgrade + ACP Bridge

## Context

Two related tasks:

1. **Upgrade rmcp 0.1 → 1.x** — fixes the `paste` audit warning
   (RUSTSEC-2024-0436), replaces SSE with Streamable HTTP
2. **Evaluate `agent-client-protocol-rmcp`** — bridge that wraps our
   rmcp `ServerHandler` into ACP sessions, exposing MCP tools to ACP
   clients without duplication

They must be studied together because the bridge crate depends on
rmcp 1.2+. We can't adopt it without upgrading rmcp first.

## Phase 1: rmcp 0.1 → 1.x

### Breaking Changes

| rmcp 0.1 | rmcp 1.x |
|---|---|
| `transport-sse-server` feature | `transport-streamable-http-server` |
| `SseServer::serve(addr)` | `StreamableHttpService::new(config, factory)` (tower) |
| `PaginatedRequestParam` | `PaginatedRequestParams` |
| `CallToolRequestParam` | `CallToolRequestParams` |
| `ReadResourceRequestParam` | `ReadResourceRequestParams` |
| `ResourceUpdatedNotificationParam` | `ResourceUpdatedNotificationParam` (unchanged) |
| `list_tools(&self, PaginatedRequestParam, ...)` | `list_tools(&self, Option<PaginatedRequestParams>, ...)` |
| `list_resources(&self, PaginatedRequestParam, ...)` | `list_resources(&self, Option<PaginatedRequestParams>, ...)` |
| `ServerHandler: Sized + 'static` | `ServerHandler: Sized + Send + Sync + 'static` |

`Tool`, `CallToolResult`, `ServerInfo`, `ServerCapabilities`,
`Implementation`, `RawResource`, `ResourceContents`, `AnnotateAble`
are unchanged.

### Cargo.toml

```toml
# Before
rmcp = { version = "0.1", features = ["server", "transport-io", "transport-sse-server"] }

# After
rmcp = { version = "1", features = ["server", "transport-io", "transport-streamable-http-server"] }
```

### Files

| File | Change |
|---|---|
| `Cargo.toml` | Bump + feature rename |
| `src/mcp/mod.rs` | Rename `Param` → `Params`, `Option<>` wrappers on list methods |
| `src/mcp/tools.rs` | Likely unchanged (`Tool::new` API same) |
| `src/mcp/handlers.rs` | Likely unchanged (`CallToolResult` same) |
| `src/server.rs` | Replace `SseServer::serve` with `StreamableHttpService` |
| `src/cli.rs` | `--sse` flag → `--http` (keep `--sse` as hidden alias?) |
| `src/config.rs` | `serve.sse` / `serve.sse_port` → `serve.http` / `serve.http_port` |
| `audit.toml` | Remove `RUSTSEC-2024-0436` ignore |

### Transport Migration

Old (SSE):
```rust
let sse_server = SseServer::serve(addr).await?;
let _ct = sse_server.with_service(move || server.clone());
```

New (Streamable HTTP — tower service):
```rust
let config = StreamableHttpServerConfig::default()
    .with_cancellation_token(cancel_token.clone());
let service = StreamableHttpService::new(config, move || server.clone());
// Mount as axum/tower route, bind with hyper
```

The new transport is a tower `Service` — needs an HTTP server
(axum or hyper) to bind it. More setup than `SseServer::serve` but
more flexible.

### StreamableHttpServerConfig

| Field | Default | Notes |
|---|---|---|
| `stateful_mode` | `true` | Session per client, SSE priming for reconnection — equivalent to our current SSE behavior |
| `json_response` | `false` | When `true` + stateless: pure JSON, no SSE framing. Not relevant for us (we want stateful). |
| `cancellation_token` | new token | **Replaces our `watch` channel for HTTP shutdown.** Cancel the token → all sessions terminate. |
| `allowed_hosts` | `["localhost", "127.0.0.1", "::1"]` | We bind `0.0.0.0` — need to widen or disable for non-local access. |
| `sse_keep_alive` | 15s | SSE ping interval (stateful mode only) |
| `sse_retry` | 3s | SSE retry hint for client reconnection |

Decisions needed:
- Keep `stateful_mode: true` (default) — matches current SSE behavior
- `allowed_hosts`: expose as `serve.http_allowed_hosts` config key
  (default: `["localhost", "127.0.0.1", "::1"]`)
- Wire `cancellation_token` to our ctrl_c handler instead of `watch` channel

### Config Rename Strategy

Option A: Rename `sse` → `http` everywhere (breaking for existing configs).
Option B: Accept both, prefer `http`, warn on `sse` (backward compat).

Recommendation: **Option A** — no production configs exist yet.

### Full Rename Surface

| Location | Current | After |
|---|---|---|
| `src/config.rs` `ServeConfig` | `sse: bool`, `sse_port: u16` | `http: bool`, `http_port: u16` |
| `src/config.rs` `set_global_config_value` | `serve.sse`, `serve.sse_port` | `serve.http`, `serve.http_port` |
| `src/config.rs` `get_config_value` | `serve.sse`, `serve.sse_port` | `serve.http`, `serve.http_port` |
| `src/config.rs` `set_wiki_config_value` | `serve.sse`, `serve.sse_port` (global-only guard) | `serve.http`, `serve.http_port` |
| `src/config.rs` defaults | `default_sse_port()` | `default_http_port()` |
| `src/cli.rs` `Serve` | `--sse [PORT]` | `--http [PORT]` |
| `src/server.rs` | `serve_sse()`, `sse_enabled`, `resolved_port` | `serve_http()`, `http_enabled` |
| `tests/config.rs` | `set_global_sets_serve_keys` etc. | Update key names |
| Config spec | `docs/specifications/model/global-config.md` | Update |
| Server spec | `docs/specifications/engine/server.md` | SSE → Streamable HTTP |
| IDE guide | `docs/guides/ide-integration.md` | Update SSE section |
| README | Technology table | Update rmcp description |

## Phase 2: ACP Bridge Evaluation

### How `agent-client-protocol-rmcp` Works

```rust
use agent_client_protocol_rmcp::McpServerExt;

let mcp = McpServer::from_rmcp("llm-wiki", || crate::mcp::McpServer::new(manager.clone()));
```

Creates a duplex stream internally, connects rmcp service to one end,
ACP MCP protocol to the other. Handles `list_tools` / `call_tool`
transparently.

### The Problem

The bridge is designed for the **Proxy** pattern:

```rust
Proxy.builder()
    .with_mcp_server(mcp)
    .connect_to(conductor)
    .await
```

We are an **Agent**, not a Proxy. There's no upstream agent to proxy to.

### Investigation Results

`with_mcp_server` exists on `Builder` but is constrained:

```rust
/// Only applicable to proxies.
pub fn with_mcp_server(...)
where
    Host::Counterpart: HasPeer<Agent> + HasPeer<Client>,
```

For `Agent.builder()`, `Host::Counterpart = Client`. Only `Conductor`
satisfies both `HasPeer<Agent> + HasPeer<Client>`. So:

- `Agent.builder().with_mcp_server()` — **does not compile**
- `Proxy.builder().with_mcp_server()` — works (needs conductor)
- `with_handler()` is generic but `McpServer.into_handler_and_responder()`
  has the same role constraint

**Verdict: Defer.** MCP tools in ACP sessions requires the Proxy
pattern + conductor. Not worth the architecture change.

### Investigation Steps

- [x] Check if `Agent.builder().with_mcp_server()` exists — yes, but
  role-constrained to Proxy/Conductor only
- [x] Check if `with_handler` can accept an `McpServer` — same constraint
- [ ] ~~Check if ACP clients send MCP tool calls to agents directly~~ — moot
- [ ] ~~Test Zed MCP tool discovery from ACP agent~~ — moot

### Outcome

**Deferred.** `Agent.builder()` does not support `with_mcp_server` —
the trait bound `Client: HasPeer<Agent>` is not satisfied. Only
`Proxy.builder()` / `Conductor` can use it.

MCP and ACP remain separate transports. ACP handles prompts via
`run_research`; MCP tools are accessed via stdio/HTTP transports.

### Decision Criteria

| Criterion | Weight |
|-----------|--------|
| No architecture change (stay Agent, no conductor) | High |
| MCP tools accessible from ACP sessions | Medium |
| No tool definition duplication | Medium |
| Minimal new dependencies | Low |

## Execution Order

1. Create branch `feat/upgrade-rmcp`
2. Bump rmcp to 1.x, fix compile errors (Phase 1)
3. Replace SSE transport with Streamable HTTP
4. Rename config keys (`sse` → `http`), add `http_allowed_hosts`
5. Remove `RUSTSEC-2024-0436` from `audit.toml`
6. `cargo check && cargo test && cargo clippy && cargo audit`
7. Update docs

## Documentation Updates

| Doc | Change |
|---|---|
| `docs/decisions/rmcp-streamable-http.md` | New — SSE → Streamable HTTP, config rename, bridge deferred |
| `docs/specifications/engine/server.md` | SSE → Streamable HTTP |
| `docs/specifications/integrations/mcp-clients.md` | Update client config |
| `docs/specifications/model/global-config.md` | `serve.sse*` → `serve.http*`, add `http_allowed_hosts` |
| `docs/implementation/mcp-server.md` | Update transport setup |
| `docs/implementation/acp-sdk.md` | Update bridge status (deferred) |
| `docs/implementation/rust.md` | Update rmcp version in dependency table |
| `docs/guides/ide-integration.md` | Update SSE section |
| `README.md` | Update technology table |
| `CHANGELOG.md` | rmcp 1.x, Streamable HTTP, config rename |
| `docs/roadmap.md` | Move to completed |

## Related Projects

### llm-wiki-skills

Add a prompt at `docs/prompts/update-mcp-transport.md`:

- SSE transport replaced by Streamable HTTP in rmcp 1.x
- `--sse` flag renamed to `--http`
- Config keys: `serve.sse` → `serve.http`, `serve.sse_port` → `serve.http_port`
- New config key: `serve.http_allowed_hosts`
- Update `setup` skill with new serve command
- Update any skill that references SSE endpoints or `--sse` flag
