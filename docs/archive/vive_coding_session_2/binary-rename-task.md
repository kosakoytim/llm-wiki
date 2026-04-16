# Binary Rename: `wiki` → `llm-wiki`

Rename the CLI binary from `wiki` to `llm-wiki` for uniqueness and
consistency with the crate name and repository.

---

## Rationale

- `wiki` is a generic name — collision risk with other tools
- `llm-wiki` matches the crate name, repo name, and project identity
- Self-documenting — "what's llm-wiki?" answers itself
- Most usage is through MCP (binary name typed once in config), not CLI

---

## Scope

The rename touches every file that references the binary name as a CLI
command. MCP tool names (`wiki_search`, `wiki_read`, etc.) and `wiki://`
URIs are NOT renamed — they are protocol identifiers, not binary names.

### What changes

| Area | Files | Pattern |
|------|-------|---------|
| Binary definition | `Cargo.toml` | `name = "wiki"` → `name = "llm-wiki"` |
| Binstall metadata | `Cargo.toml` | `bin-dir = "wiki"` → `bin-dir = "llm-wiki"` |
| Global directory | `src/main.rs`, `src/server.rs`, `src/config.rs` | `.join(".wiki")` → `.join(".llm-wiki")` |
| Release packaging | `.github/workflows/release.yml` | `wiki` / `wiki.exe` → `llm-wiki` / `llm-wiki.exe` |
| README | `README.md` | All CLI examples, install command, MCP configs |
| CLI spec | `docs/specifications/commands/cli.md` | All `wiki <cmd>` references |
| Command specs (14) | `docs/specifications/commands/*.md` | CLI examples, `~/.wiki/` paths |
| Core specs (7) | `docs/specifications/core/*.md` | CLI references, `~/.wiki/` paths |
| Integration specs (3) | `docs/specifications/integrations/*.md` | MCP client configs |
| Pipeline specs (3) | `docs/specifications/pipelines/*.md` | CLI examples |
| LLM specs (2) | `docs/specifications/llm/*.md` | CLI references |
| Other specs (2) | `docs/specifications/overview.md`, `features.md` | CLI references |
| Instructions | `src/assets/instructions.md` | Workflow examples |
| Improvement docs | `docs/*.md` | CLI references, `~/.wiki/` paths |
| Implementation docs | `docs/implementation/*.md` | CLI references, `~/.wiki/` paths |

### What does NOT change

- MCP tool names: `wiki_search`, `wiki_read`, `wiki_ingest`, etc.
- `wiki://` URI scheme
- `WikiServer`, `WikiAgent`, `WikiConfig` struct names
- `wiki.toml` per-wiki config filename
- `wiki/` directory inside each wiki repository (inbox/, raw/, wiki/)
- `wiki_root` variable names in code
- Error messages like `"wiki already registered"`
- Git commit message prefixes (`ingest:`, `lint:`, `graph:`)

---

## Execution plan

### Step 1 — Binary, packaging, and global directory

- `Cargo.toml`: `[[bin]] name = "llm-wiki"`
- `Cargo.toml`: `bin-dir = "llm-wiki"`
- `.github/workflows/release.yml`: `wiki` → `llm-wiki`, `wiki.exe` → `llm-wiki.exe`
- `src/main.rs`: `.join(".wiki")` → `.join(".llm-wiki")` (2 occurrences)
- `src/server.rs`: `.join(".wiki")` → `.join(".llm-wiki")` (1 occurrence)
- `src/config.rs`: `.join(".wiki")` → `.join(".llm-wiki")` (1 occurrence, log_path default)

### Step 2 — README

- All CLI examples: `wiki init` → `llm-wiki init`, etc.
- Install command: `cargo install llm-wiki`
- MCP client configs: `"command": "llm-wiki"`

### Step 2b — Contributing

- `CONTRIBUTING.md`: CLI references in development workflow examples

### Step 3 — Specifications (bulk sed)

All `docs/specifications/` files:
- `wiki init` → `llm-wiki init`
- `wiki serve` → `llm-wiki serve`
- `wiki search` → `llm-wiki search`
- etc. for all subcommands

Careful: do NOT replace `wiki_search` (MCP tool), `wiki://` (URI),
`wiki.toml` (config file), or `wiki/` (directory).

Safe sed pattern: replace `` `wiki `` (backtick + wiki + space) with
`` `llm-wiki `` and `wiki init`/`wiki serve`/etc. in code blocks.
Also replace `~/.wiki/` with `~/.llm-wiki/` in all docs (~63 occurrences).

### Step 4 — Instructions

`src/assets/instructions.md`: CLI references in workflow examples.

### Step 5 — Other docs

`docs/*.md`: improvement docs, task lists, analysis docs.

### Step 6 — Verify

- `cargo build` — binary is `target/debug/llm-wiki`
- `cargo test` — all tests pass
- `grep -r '"wiki"' Cargo.toml` — only in keywords, not binary name
- `grep -rn 'wiki serve\|wiki init' docs/ src/` — all should be `llm-wiki`
- `grep -rn '~/\.wiki/' docs/ src/` — should be zero (all `~/.llm-wiki/`)
- `grep -rn '"\.wiki"' src/` — should be zero (all `".llm-wiki"`)


### Step 7 — Claude plugin implementation

- claude plugin implementation

---

## Risk

- Missed references — grep verification in Step 6 catches these
- Breaking MCP client configs — users must update `"command": "wiki"`
  to `"command": "llm-wiki"` in their IDE configs
- Breaking existing `~/.wiki/` directory — existing users must rename
  `~/.wiki/` to `~/.llm-wiki/` or the engine creates a fresh one.
  Consider adding a migration note in the changelog.
- README links — internal links should still work (file paths unchanged)

---

## Doc sync — new commands and config keys

While renaming, also update docs to reflect commands and config added
since the last doc sync.

### README CLI reference

The summary table is missing:
- `wiki index status` — already implemented
- `wiki index check` — added in I5
- `wiki config get|set|list` — already in README but verify flags

### `docs/specifications/commands/cli.md`

The `wiki index` section is missing `check`:
```
wiki index rebuild [--wiki] [--dry-run]
wiki index status  [--wiki]
wiki index check   [--wiki]              # NEW
```

### `docs/specifications/commands/index.md`

Missing:
- `wiki index check` subcommand section
- `IndexCheckReport` return type
- `wiki_index_check` MCP tool

### `docs/specifications/features.md`

MCP tool table missing:
- `wiki_index_check` (17th tool)

### `docs/specifications/commands/configuration.md`

Config keys reference table may be missing:
- `index.auto_recovery` — added in I2
- `serve.max_restarts` — added in R3
- `serve.restart_backoff` — added in R3
- `serve.heartbeat_secs` — added in R5
- `logging.*` keys — added in L6

Verify all keys in the table match the code.

### `docs/specifications/commands/serve.md`

Config section may be missing the new `[serve]` keys.

### Verification

After all updates:
- `grep -c 'wiki_' docs/specifications/features.md` should show 17 tools
- `grep 'index check' docs/specifications/commands/cli.md` should match
- All config keys in `set_global_config_value` should appear in
  `configuration.md` §3
