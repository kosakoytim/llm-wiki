# Upgrade: agent-client-protocol 0.10 → 0.11

## Problem

`agent-client-protocol` 0.11 has breaking API changes — 70 compile
errors on bump. Types renamed or restructured.

## Known Breaking Changes

From the compile errors:

- `AgentSideConnection` — moved or renamed
- `SessionNotification` — moved or renamed
- `PromptRequest` — moved or renamed
- `ContentBlock` — moved or renamed
- Many other type/path changes

## Files Affected

- `src/acp.rs` — all ACP types, Agent trait impl, `serve_acp`,
  `WikiAgent`, session management, workflow steps
- `tests/acp.rs` — ACP integration tests

## Steps

- [ ] Read agent-client-protocol 0.11 changelog / migration guide
- [ ] Read https://docs.rs/agent-client-protocol/0.11
- [ ] Update `src/acp.rs` — fix all type paths and API changes
- [ ] Update `tests/acp.rs`
- [ ] `cargo check`
- [ ] `cargo test`
- [ ] `cargo clippy -- -D warnings`
- [ ] Manual: `llm-wiki serve --acp` works

## Documentation Updates

- [ ] `docs/implementation/acp-server.md` — update if API changed
- [ ] `docs/implementation/acp-sdk.md` — update SDK reference
- [ ] `docs/specifications/integrations/acp-transport.md` — update if
  protocol messages changed

## Notes

- Do in a feature branch
- stdio and MCP transports are unaffected
- The ACP workflow steps (`step_search`, `step_read`, `step_report_results`)
  may need signature changes if notification types changed
