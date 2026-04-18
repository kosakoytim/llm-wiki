# Implement Phase 1 — Step by Step

## Context

The Phase 1 plan is finalized in `docs/roadmap.md` (Steps 0–18).
Each step produces a compilable, testable increment. This prompt
drives execution of one step at a time.

## How to use this prompt

1. Tell the agent which step to implement: "Implement Step N"
2. The agent reads the step from `docs/roadmap.md`, reads the
   referenced implementation docs and code-ref/ sources, then
   implements it
3. After each step: `cargo check`, `cargo test`, commit

To resume after a break, say "Continue from Step N".

## Read before every step

These are always relevant — read once, refer back as needed:

- `docs/roadmap.md` — the step you're implementing
- `docs/implementation/rust.md` — project layout, dependencies, code quality
- `docs/implementation/engine.md` — how modules compose at runtime
- `Cargo.toml` — current dependencies

## Per-step reading

Each step references specific implementation docs and code-ref/ files.
Read them before writing code.

| Step | Implementation docs to read | code-ref/ files to pull from |
|------|-----------------------------|------------------------------|
| 0 | `rust.md` | — (move only) |
| 1 | `slug.md` | `code-ref/src/markdown.rs`, `code-ref/src/spaces.rs` |
| 2 | `config-loader.md` | `code-ref/src/config.rs` |
| 3 | `frontmatter-parser.md` | `code-ref/src/frontmatter.rs` |
| 4 | `git.md` | `code-ref/src/git.rs` |
| 5 | — | `code-ref/src/links.rs` |
| 6 | `type-registry.md` | — (new module) |
| 7 | `tantivy.md`, `index-manager.md` | `code-ref/src/search.rs` |
| 8 | — | `code-ref/src/ingest.rs` |
| 9 | `graph-builder.md` | `code-ref/src/graph.rs` |
| 10 | — | `code-ref/src/markdown.rs` |
| 11 | — | `code-ref/src/spaces.rs`, `code-ref/src/init.rs` |
| 12 | `engine.md`, `manager-pattern.md` | — (new module) |
| 13 | `cli.md` | `code-ref/src/cli.rs`, `code-ref/src/main.rs` |
| 14 | `mcp-server.md` | `code-ref/src/mcp/mod.rs`, `code-ref/src/mcp/tools.rs` |
| 15 | `acp-server.md` | `code-ref/src/acp.rs` |
| 16 | — | `code-ref/src/server.rs` |
| 17 | — | `code-ref/tests/` |
| 18 | `rust.md` | `code-ref/.github/` |

## Rules for every step

### Before writing code

1. Read the step description in `docs/roadmap.md`
2. Read the implementation doc(s) listed above — especially the
   "Existing Code" table showing what's reusable
3. Read the code-ref/ source file(s) you'll pull from
4. Read the current `src/lib.rs` to see what modules exist so far
5. Read any `src/` files this step depends on (earlier steps)

### While writing code

- Follow `docs/implementation/rust.md` for style, error handling,
  and testing conventions
- Pull reusable code from code-ref/ directly — don't rewrite what
  works. Adapt to new types (`ParsedPage` instead of
  `PageFrontmatter`, `Slug` instead of bare strings, etc.)
- Note what you pulled and what you changed
- Add the module to `src/lib.rs`
- Write unit tests in-module (`#[cfg(test)] mod tests`)
- Use `anyhow::Result` for public functions
- Use `tempfile::tempdir()` for any filesystem tests

### After writing code

1. Run `cargo check` — must pass
2. Run `cargo test` — must pass
3. Run `cargo clippy -- -D warnings` — fix any warnings
4. Commit with the message from the step description

### What NOT to implement

- JSON Schema validation (Phase 2)
- `x-index-aliases` resolution (Phase 2)
- `x-graph-edges` typed edges (Phase 3)
- Skill registry features (Phase 4)
- Hot reload / file watcher (future)

If a function signature needs a type registry or schema parameter
that doesn't exist yet, use a simple placeholder that Phase 2 will
replace.

## Step-by-step summary

For quick reference — full details are in `docs/roadmap.md`:

```
Step  0: Codebase reset — move src/ to code-ref/, empty src/
Step  1: slug.rs — Slug, WikiUri types and resolution
Step  2: config.rs — GlobalConfig, WikiConfig, two-level resolution
Step  3: frontmatter.rs — untyped BTreeMap parsing, ParsedPage
Step  4: git.rs — git2 wrappers (init, commit, diff)
Step  5: links.rs — [[wiki-link]] extraction from body text
Step  6: type_registry.rs — hardcoded base type registry
Step  7: index_schema.rs + search.rs — tantivy schema and BM25 search
Step  8: ingest.rs — validate, index, commit pipeline
Step  9: graph.rs — concept graph from tantivy index
Step 10: markdown.rs — page read, write, create
Step 11: spaces.rs — space create, list, remove, set-default
Step 12: engine.rs — Engine struct and EngineManager
Step 13: cli.rs + main.rs — clap subcommand hierarchy
Step 14: mcp/ — 15 MCP tools with ServerHandler
Step 15: acp.rs — WikiAgent with ACP transport
Step 16: server.rs — stdio + SSE + ACP transport wiring
Step 17: Integration tests for all modules
Step 18: Cleanup — CI pipeline, clippy clean, fmt clean
```

## Verification at the end

After Step 18, all of these must work:

- `llm-wiki spaces create/list/remove/set-default`
- `llm-wiki config get/set/list`
- `llm-wiki content read/write/new/commit`
- `llm-wiki search` with `--type` filter and `--format`
- `llm-wiki list` with `--type`, `--status`, `--format`
- `llm-wiki ingest` with `--format`
- `llm-wiki graph` with `--format`, `--root`, `--depth`, `--type`
- `llm-wiki index rebuild/status`
- `llm-wiki serve` (stdio + SSE)
- `llm-wiki serve --acp`
- All 15 MCP tools working
- `cargo test` — all tests pass
- `cargo clippy -- -D warnings` — clean
- `cargo fmt -- --check` — clean
