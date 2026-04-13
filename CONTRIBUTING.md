# Contributing to llm-wiki

## Prerequisites

- Rust stable (minimum version: 1.75) — install via [rustup](https://rustup.rs/)
- `cargo`
- `git`

## Build

```bash
cargo build
cargo check          # faster: type-checks without linking
```

## Test

```bash
cargo test                            # all unit tests
cargo test --test integration_test    # integration tests only
```

## Lint

```bash
cargo clippy -- -D warnings    # must pass with zero warnings
cargo fmt --check               # verify formatting matches rustfmt.toml
cargo fmt                       # auto-format
```

## Run locally

```bash
cargo run -- ingest analysis.json    # ingest an analysis document
cargo run -- ingest -                # read analysis JSON from stdin
cargo run -- search "scaling laws"   # full-text search
cargo run -- context "how does MoE scaling work?"
cargo run -- serve                   # start MCP server (stdio)
cargo run -- serve --sse :8080       # start MCP server (SSE)
```

## Project structure

```
src/
├── main.rs         CLI entry point
├── cli.rs          clap Command enum — all subcommands
├── analysis.rs     Analysis JSON schema (primary LLM↔wiki interface)
├── markdown.rs     PageFrontmatter schema
├── config.rs       Per-wiki WikiConfig (.wiki/config.toml)
├── ingest.rs       Deserialise Analysis JSON → integrate
├── integrate.rs    Write pages + contradictions, git commit
├── search.rs       tantivy index build + query
├── context.rs      Assemble top-K pages as Markdown context
├── lint.rs         Structural audit: orphans, stubs, contradictions
├── graph.rs        petgraph concept graph → DOT/Mermaid
├── contradiction.rs Contradiction page read/list/filter
├── git.rs          commit, diff, log via git2
├── server.rs       rmcp WikiServer (Phase 4)
└── registry.rs     Multi-wiki registry (Phase 6)
```

See [`docs/dev/architecture.md`](docs/dev/architecture.md) for the full module dependency graph and design principles.

## Commit message format

```
<type>: <short description>

[optional body]
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`

Examples:
- `feat: implement wiki ingest write loop`
- `fix: handle append action on missing slug`
- `test: add PageFrontmatter serde round-trip`
- `chore: update tantivy to 0.23`

## Branch naming

```
phase-<N>/<short-description>
```

Examples: `phase-1/ingest-write-loop`, `phase-2/tantivy-search`

## Pull request process

1. One phase per PR.
2. CI must pass (check, clippy, fmt, tests).
3. Add a `CHANGELOG.md` entry under `[Unreleased]`.
4. Reference the relevant phase task file in the PR description.

## Changelog

All changes go in `CHANGELOG.md` under `[Unreleased]` using [Keep a Changelog](https://keepachangelog.com/) format.

Sections: `Added`, `Changed`, `Fixed`, `Removed`.

## Code style

`rustfmt.toml` is authoritative. Run `cargo fmt` before committing.
Do not manually format code — let rustfmt decide.

## No LLM dependency rule

PRs **must not** add `rig-core` or any other LLM client crate as a dependency.
The wiki engine has zero LLM calls. The `wiki` binary is a pure Rust tool that
manages Markdown files, git history, and tantivy search indexes. All intelligence
is supplied by an external LLM that calls the wiki via CLI or MCP.

This is a hard rule — PRs that add an LLM dependency will not be merged.
