---
title: "Migrate integration tests to pytest"
summary: "Replace bash test scripts with a pytest suite under tests-integration/. Eliminates false-positive grep patterns, provides automatic teardown, and gives structured JSON inspection without jq."
status: accepted
date: "2026-05-04"
---

# Migrate integration tests to pytest

## Decision

Add a `tests-integration/` directory at repo root containing a pytest suite managed by `uv`. Port all bash integration sections to Python test files under `tests-integration/engine/`, `tests-integration/mcp/`, and `tests-integration/acp/`. Bash scripts are not deleted during migration — each section is removed after its pytest equivalent passes in CI.

## Context

The bash test harness in `docs/testing/scripts/` accumulated structural limits over v0.1–v0.4:

- `grep -v` negative assertions always output lines → false-positive tests silently passed
- No guaranteed teardown — state leaked between runs when a test failed mid-section
- JSON inspection required `jq` piped through bash; subtle to read and write
- `run_nocheck` existed solely to paper over unreliable exit-code semantics
- ACP tests required tmpfiles + `sleep 0.2` polling + `kill $pid` because bash lacks async I/O
- No parametrization, no structured output, no diff on failure

A false-positive `grep -v` bug was shipped in the initial integration scripts and only caught during v0.4.1 gap analysis.

## Rationale

- **Correctness.** `assert "x" not in result.stdout` cannot false-positive; `grep -v` can.
- **Isolation.** pytest `tmp_path` creates a fresh directory per test and deletes it on teardown — no shared state between tests.
- **Simplicity.** ACP is JSON-RPC 2.0 NDJSON over stdio — Python `asyncio.create_subprocess_exec` replaces 120 lines of `acp-helpers.sh`. MCP uses the official `mcp` Python SDK.
- **Coverage.** `@pytest.mark.parametrize`, per-file runs, JUnit XML output — all available without additional infrastructure.

## Architecture

```
tests-integration/
  pyproject.toml      # uv project: pytest + pytest-asyncio + mcp
  Makefile            # install / test-engine / test-acp / test-mcp via uv
  .gitignore          # .venv/, __pycache__, uv.lock
  conftest.py         # WikiEnv fixture — boots two wikis per test from tests/fixtures/
  engine/             # CLI subprocess tests (sections 01–18)
  mcp/                # MCP stdio tests via mcp.client.stdio
  acp/                # ACP stdio tests via asyncio.create_subprocess_exec
```

`tests/fixtures/` stays in place — `tests/engine.rs` hardcodes `"tests/fixtures/wikis/alt-root"` as a relative path from the crate root.

`tests-integration/` is excluded from `cargo publish` via `Cargo.toml` `exclude`.

Root `Makefile` delegates with `$(MAKE) -C tests-integration test-engine BINARY=$(DEBUG_BIN)`.

## Consequences

- `pyproject.toml` + `Makefile` + `conftest.py` added under `tests-integration/`.
- All Python artefacts (`.venv/`, `uv.lock`, `__pycache__`) gitignored.
- Bash scripts deleted section-by-section as pytest equivalents pass in CI.
- `validate-py-engine`, `validate-py-acp`, `validate-py-mcp` targets added to root `Makefile`.
