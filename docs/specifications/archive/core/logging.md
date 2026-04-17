---
title: "Logging"
summary: "Structured logging via tracing — stderr for all commands, file rotation for llm-wiki serve, configurable via [logging] section."
read_when:
  - Implementing or extending the logging system
  - Adding tracing calls to a new module
  - Debugging llm-wiki serve in production
  - Understanding the separation between CLI output and operational logs
status: draft
last_updated: "2025-07-17"
---

# Logging

Structured logging via `tracing` and `tracing-subscriber`. All operational
logs go to stderr. Long-running `llm-wiki serve` also writes to rotating log
files under `~/.llm-wiki/logs/`.

---

## 1. Two Concerns, Two Channels

| Concern | Channel | Who reads it |
|---------|---------|-------------|
| CLI user output | `println!` to stdout | Human at the terminal |
| Operational logging | `tracing::*` to stderr + file | Developer debugging issues |

These never mix:

- `println!` — only in `main.rs`, only for CLI command results
- `eprintln!` — only in `main.rs`, only for CLI user-facing warnings
  (stale index, rebuild failure)
- `tracing::*` — everywhere in library code (`server.rs`, `acp.rs`,
  `mcp/tools.rs`, `mcp/mod.rs`)

stdout is the MCP stdio transport — logging must never write to stdout.

---

## 2. Log Levels

| Level | Usage |
|-------|-------|
| `error` | Unrecoverable failures: ACP connection lost, transport crash |
| `warn` | Recoverable failures: git commit failed, index rebuild failed, resource notification failed |
| `info` | Operational milestones: server started, SSE listening, session created |
| `debug` | Per-request detail: tool call ok, prompt complete, search results count |
| `trace` | Not used (reserved for future low-level diagnostics) |

Default filter: `llm_wiki=info,warn`. Override with `RUST_LOG` environment
variable:

```bash
RUST_LOG=llm_wiki=debug llm-wiki serve     # verbose
RUST_LOG=llm_wiki=trace llm-wiki serve     # maximum detail
RUST_LOG=warn llm-wiki serve               # quiet — errors and warnings only
```

---

## 3. Spans

Spans provide request-level context so logs can be correlated across a
single operation:

| Span | Fields | Where |
|------|--------|-------|
| `tool_call` | `tool` | `mcp/tools.rs` — wraps every MCP tool dispatch |
| `mcp_call_tool` | `tool` | `mcp/mod.rs` — wraps the ServerHandler call |
| `acp_new_session` | `session` | `acp.rs` — session creation |
| `acp_prompt` | `session`, `workflow` | `acp.rs` — wraps entire prompt execution |

---

## 4. File Logging

### Default behavior

`llm-wiki serve` writes rotating log files to `~/.llm-wiki/logs/`. The directory
is created automatically on first use.

CLI commands (init, search, lint, etc.) log to stderr only — they are
short-lived and don't need file persistence.

### Rotation

| Setting | Default | Description |
|---------|---------|-------------|
| `log_path` | `~/.llm-wiki/logs` | Directory for log files. Empty string disables file logging. |
| `log_rotation` | `daily` | Rotation schedule: `daily`, `hourly`, or `never` |
| `log_max_files` | `7` | Maximum number of rotated files. `0` = no limit. |
| `log_format` | `text` | Output format: `text` (human-readable) or `json` (machine-parseable) |

Log files are named `wiki.log.YYYY-MM-DD` (daily), `wiki.log.YYYY-MM-DD-HH`
(hourly), or `wiki.log` (never).

### Log format

`text` (default) produces compact human-readable output — one line per
event, span fields appended at the end:

```
2025-07-17T14:32:01Z  INFO llm_wiki::mcp::tools: tool call ok tool="wiki_search"
2025-07-17T14:32:01Z  WARN llm_wiki::mcp::tools: git commit failed error="..." uri="wiki://research/concepts/moe"
```

`json` produces one JSON object per line, suitable for log aggregation:

```json
{"timestamp":"2025-07-17T14:32:01Z","level":"INFO","target":"llm_wiki::mcp::tools","span":{"tool":"wiki_search"},"message":"tool call ok"}
```

The format applies to both stderr and file output.

### Dual output

When file logging is active, logs go to both stderr and the log file.
This means:
- The terminal shows logs in real time (useful during development)
- The file persists logs for post-mortem debugging

### Implementation

Uses `tracing-appender` with `non_blocking` writer to avoid blocking the
MCP/ACP event loop. The `text` format uses `tracing_subscriber::fmt().compact()`:

```rust
let file_appender = tracing_appender::rolling::daily(&log_path, "wiki.log");
let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
```

The `_guard` must be held for the process lifetime — dropping it flushes
and closes the writer.

---

## 5. Configuration

Logging config lives in the `[logging]` section of the global config
(`~/.llm-wiki/config.toml`). It is **global-only** — not per-wiki.

```toml
[logging]
log_path = "~/.llm-wiki/logs"   # directory for log files (empty = stderr only)
log_rotation = "daily"       # daily | hourly | never
log_max_files = 7            # max rotated files (0 = no limit)
log_format = "text"          # text | json
```

### Config keys reference

| Key | Scope | Default | Description |
|-----|-------|---------|-------------|
| `logging.log_path` | global only | `~/.llm-wiki/logs` | Log file directory. Empty string disables file logging. |
| `logging.log_rotation` | global only | `daily` | Rotation schedule: `daily`, `hourly`, `never` |
| `logging.log_max_files` | global only | `7` | Max rotated log files. `0` = unlimited. |
| `logging.log_format` | global only | `text` | Output format: `text` or `json` |

### Resolution

Logging config does not participate in per-wiki resolution. It is read
once at startup from the global config. `RUST_LOG` overrides the log level
filter but not the file output settings.

`llm-wiki config set logging.* --wiki <name>` is rejected with an error:
`"logging.* is a global-only key — use --global"`. Same behavior as
`serve.*` keys.

---

## 6. What Does Not Get Logged

- Page content — never logged (could be large, could be sensitive)
- Frontmatter field values — never logged
- Config file contents — never logged (may contain paths)
- Search queries — logged at `debug` level only (useful for debugging,
  but could be sensitive in shared environments)

---

## 7. Error Handling Rule

No silent error discard in library code. Every fallible operation that is
not propagated via `?` must log the error:

```rust
// Correct:
if let Err(e) = git::commit(&repo_root, &msg) {
    tracing::warn!(error = %e, "git commit failed");
}

// Wrong:
let _ = git::commit(&repo_root, &msg);
```

This rule applies to all `src/` files except `main.rs` (which uses
`eprintln!` for CLI user warnings).
