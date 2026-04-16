# Server Resilience Tasks

Implement crash recovery and supervision for `wiki serve` transports.

Reference:
- [Server resilience spec](specifications/core/server-resilience.md)
- [Server resilience analysis](server-resilience.md)
- [ACP SDK reference](implementation/acp-sdk.md)

Current state: any transport failure is terminal for the entire process.
No supervision, no restart, no panic isolation.

---

## Phase 1 — Isolation (prevent cascading failures)

### Task R1 — catch_unwind boundary for MCP tool handlers

**Goal:** A panicking tool handler must not crash the MCP server.

#### Analysis

`tools::call()` dispatches to individual handlers (`handle_search`,
`handle_ingest`, etc.). If any handler panics, the panic propagates
through `call_tool` in `mcp/mod.rs`, kills the tokio task, and may
crash the entire server.

The fix: wrap the dispatch in `catch_unwind` so panics are caught and
returned as error responses.

#### Code changes

- `src/mcp/tools.rs` — in `call()`, wrap the match dispatch:
  ```rust
  let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      match name {
          "wiki_init" => handle_init(server, args),
          // ...
      }
  }));
  match result {
      Ok(r) => { /* existing logic */ }
      Err(_) => {
          tracing::error!(tool = name, "tool handler panicked");
          ToolResult {
              content: err_text("internal error: tool panicked".into()),
              is_error: true,
              notify_uris: vec![],
          }
      }
  }
  ```

#### Tests

- `tests/mcp.rs` — difficult to test panics in integration tests without
  a deliberately panicking tool. Verify manually or add a `#[cfg(test)]`
  panic tool.

#### Exit criteria

- A panicking tool handler returns an error response instead of crashing.
- The MCP server continues serving after a tool panic.
- `cargo test` passes.

---

### Task R2 — catch_unwind boundary for ACP prompt handler

**Goal:** A panicking workflow must not crash the ACP thread.

#### Analysis

The ACP `prompt()` method runs workflow logic (search, lint, etc.)
synchronously. A panic in any workflow kills the ACP thread, which then
kills the process via `acp_thread.join()`.

Same pattern as R1 but in `acp.rs`.

#### Code changes

- `src/acp.rs` — in `prompt()`, wrap the workflow dispatch:
  ```rust
  let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      match workflow {
          "ingest" => { ... }
          // ...
      }
  }));
  let result = match result {
      Ok(r) => r,
      Err(_) => {
          tracing::error!(session = %session_id_str, workflow, "workflow panicked");
          format!("Internal error: workflow panicked")
      }
  };
  ```

#### Tests

- Same limitation as R1 — panic testing is hard in integration tests.

#### Exit criteria

- A panicking workflow returns an error message instead of crashing.
- The ACP thread continues serving after a workflow panic.
- `cargo test` passes.

---

## Phase 2 — Supervision (recover from transport failures)

### Task R3 — ACP thread supervision loop

**Goal:** If the ACP transport crashes, restart it automatically with
exponential backoff.

#### Analysis

The ACP thread currently runs `serve_acp` once. If the connection drops
or the runtime errors, the thread exits and the process follows.

The ACP SDK uses stdio — a broken pipe or protocol error should trigger
a restart, not a process exit. The `!Send` constraint means we must
rebuild the entire runtime + agent on each restart.

#### Code changes

- `src/config.rs` — add `max_restarts: u32` (default 10) and
  `restart_backoff: u32` (default 1) to `ServeConfig`. Add to
  `set_global_config_value` match arms. Add to global-only rejection
  list in `set_wiki_config_value`.
- `src/main.rs` — add to `get_config_value`.
- `src/mcp/tools.rs` — add to `get_value`.
- `src/server.rs` — replace the ACP `thread::spawn` with a supervision
  loop that reads `max_restarts` and `restart_backoff` from the resolved
  config:
  ```rust
  let acp_thread = std::thread::spawn(move || {
      let max_restarts = serve_cfg.max_restarts;
      let initial_backoff = Duration::from_secs(serve_cfg.restart_backoff as u64);
      let max_backoff = Duration::from_secs(30);
      let mut backoff = initial_backoff;
      let mut restarts = 0u32;

      if max_restarts == 0 {
          // No restart — run once, exit on failure
          let global = global_arc.clone();
          let rt = tokio::runtime::Builder::new_current_thread()
              .enable_all()
              .build()
              .expect("failed to build ACP runtime");
          return rt.block_on(crate::acp::serve_acp(global));
      }

      loop {
          let global = global_arc.clone();
          let rt = tokio::runtime::Builder::new_current_thread()
              .enable_all()
              .build()
              .expect("failed to build ACP runtime");
          match rt.block_on(crate::acp::serve_acp(global)) {
              Ok(()) => break Ok(()),
              Err(e) => {
                  restarts += 1;
                  tracing::error!(
                      transport = "acp",
                      error = %e,
                      restart = restarts,
                      "transport crashed",
                  );
                  if restarts >= max_restarts {
                      tracing::error!("ACP max restarts reached, giving up");
                      break Err(e);
                  }
                  std::thread::sleep(backoff);
                  backoff = (backoff * 2).min(max_backoff);
              }
          }
      }
  });
  ```

#### Config

Two new keys in `[serve]` (global-only):

```toml
[serve]
max_restarts    = 10   # 0 = no restart (crash exits)
restart_backoff = 1    # initial backoff in seconds, doubles up to 30s
```

#### Tests

- `tests/config.rs` — new tests:
  - `set_global_config_value_sets_serve_restart_keys` — set
    `serve.max_restarts` and `serve.restart_backoff`, assert values.
  - `set_wiki_config_value_rejects_serve_restart_keys` — set
    `serve.max_restarts` on a WikiConfig, assert error with
    "global-only" message.
- Hard to test supervision loop in unit tests (requires simulating
  transport failure). Verify manually: kill the ACP stdin, observe
  restart in logs.

#### Exit criteria

- ACP transport restarts after a crash.
- Exponential backoff between restarts (configurable initial, 30s cap).
- Stops after `serve.max_restarts` to prevent infinite loops.
- `serve.max_restarts = 0` disables restart (crash exits, pre-resilience behavior).
- Clean shutdown (`Ok(())`) exits the loop without restart.
- `wiki config set serve.max_restarts 5 --global` works.
- `wiki config set serve.max_restarts 5 --wiki research` → error
  (global-only key).
- `cargo test` passes.

---

### Task R4 — SSE transport retry on bind failure

**Goal:** If the SSE port is busy, retry with backoff instead of exiting.

#### Analysis

`SseServer::serve(addr).await` fails immediately if the port is in use.
This is common during development (previous process still running).

#### Code changes

- `src/server.rs` — in `serve_sse`, wrap the bind in a retry loop:
  ```rust
  pub async fn serve_sse(server: WikiServer, port: u16) -> Result<()> {
      let addr: SocketAddr = ([0, 0, 0, 0], port).into();
      let mut backoff = Duration::from_secs(1);
      let max_attempts = 5;

      for attempt in 1..=max_attempts {
          match SseServer::serve(addr).await {
              Ok(sse_server) => {
                  tracing::info!(%addr, "SSE server listening");
                  let _ct = sse_server.with_service(move || server.clone());
                  tokio::signal::ctrl_c().await?;
                  return Ok(());
              }
              Err(e) => {
                  if attempt == max_attempts {
                      return Err(anyhow::anyhow!("SSE bind failed after {max_attempts} attempts: {e}"));
                  }
                  tracing::warn!(
                      %addr, error = %e, attempt,
                      "SSE bind failed, retrying in {backoff:?}",
                  );
                  tokio::time::sleep(backoff).await;
                  backoff = (backoff * 2).min(Duration::from_secs(30));
              }
          }
      }
      unreachable!()
  }
  ```

#### Tests

- Hard to test (requires port conflicts). Verify manually.

#### Exit criteria

- SSE bind failure retries up to `max_attempts` with backoff.
- Final failure produces a clear error message.
- `cargo test` passes.

---

## Phase 3 — Observability (detect hung servers)

### Task R5 — Heartbeat logging

**Goal:** Periodic log entry so silence-based monitors can detect hangs.

#### Analysis

A hung server produces no output. External monitors (systemd watchdog,
launchd KeepAlive) need a signal to detect this.

#### Code changes

- `src/server.rs` — spawn a background task in `serve()` that logs a
  heartbeat every 60 seconds:
  ```rust
  tokio::spawn(async {
      let mut interval = tokio::time::interval(Duration::from_secs(60));
      loop {
          interval.tick().await;
          tracing::debug!("heartbeat");
      }
  });
  ```

  At `debug` level so it doesn't clutter normal logs. Visible with
  `RUST_LOG=llm_wiki=debug`.

#### Tests

- No test needed (background timer).

#### Exit criteria

- `RUST_LOG=llm_wiki=debug wiki serve` produces periodic heartbeat entries.
- Default log level (`info`) does not show heartbeats.
- `cargo test` passes.

---

### Task R6 — SSE health endpoint

**Goal:** HTTP health check for external monitoring.

#### Analysis

The SSE server uses axum via rmcp. Adding a `/health` route requires
access to the axum router, which may not be exposed by the rmcp
`SseServer` API.

#### Code changes

- Investigate whether `SseServer` exposes the axum `Router` for
  customization. If yes, add a `/health` route returning `200 OK`.
  If not, this task is blocked on rmcp upstream.

#### Exit criteria

- `curl http://localhost:8080/health` returns `200 OK`.
- Or: documented as blocked on rmcp API.

---

## Execution order

| Order | Task | Phase | Effort | Dependencies |
|-------|------|-------|--------|-------------|
| 1 | R1 — MCP catch_unwind | Isolation | Small | None |
| 2 | R2 — ACP catch_unwind | Isolation | Small | None |
| 3 | R3 — ACP supervision | Supervision | Medium | R2 |
| 4 | R4 — SSE retry | Supervision | Small | None |
| 5 | R5 — Heartbeat | Observability | Tiny | None |
| 6 | R6 — SSE health | Observability | Small | Investigate rmcp API |

Phase 1 (R1-R2) should be done first — they prevent the most common
crash scenario (a bug in a tool handler taking down the server).
Phase 2 (R3-R4) adds recovery. Phase 3 (R5-R6) adds monitoring.
