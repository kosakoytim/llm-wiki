---
title: "Rust Module Architecture"
summary: "Canonical module layout for the llm-wiki binary — what lives where, cross-referenced against all specs."
read_when:
  - Deciding which module a new function belongs in
  - Resolving a discrepancy between a spec's module table and the actual layout
  - Onboarding to the codebase structure
status: active
last_updated: "2025-07-15"
---

# Rust Module Architecture

Single source of truth for the `src/` layout. Each spec's "Rust Module
Changes" table references modules defined here.

---

## Module Map

```
src/
├── main.rs       # CLI entry point — dispatch only, no logic
├── lib.rs        # module declarations
├── cli.rs        # clap Command enum — all subcommands and flags
├── config.rs     # GlobalConfig, WikiConfig, two-level resolution
├── spaces.rs     # Spaces, WikiEntry, resolve_name(), resolve_uri()
├── git.rs        # init_repo(), commit(), current_head(), diff_last()
├── frontmatter.rs # parse/write, scaffold_frontmatter(), validate_frontmatter(),
│                  # generate_minimal_frontmatter()
├── markdown.rs   # read_page(), list_assets(), read_asset(),
│                 # promote_to_bundle(), slug helpers
├── links.rs      # extract_links()
├── ingest.rs     # IngestOptions, validate → git add → commit → index
├── search.rs     # PageRef, PageSummary, PageList, tantivy index,
│                 # search(), list(), rebuild_index(), index_status()
├── lint.rs       # LintReport, MissingConnection, orphan/stub/section/
│                 # connection/untyped-source detection, LINT.md write
├── graph.rs      # build_graph(), render_mermaid(), render_dot(),
│                 # subgraph(), GraphReport, in_degree()
├── server.rs     # WikiServer, startup, stdio + SSE transport wiring
├── mcp.rs        # all MCP tools, resources, prompts
└── acp.rs        # WikiAgent, AcpSession, workflow dispatch
```

---

## Module Responsibilities

| Module        | Owns                                                                                                                                                                                                                  | Referenced by                                                                                                                                                                                                      |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `main.rs`     | CLI dispatch                                                                                                                                                                                                          | —                                                                                                                                                                                                                  |
| `lib.rs`      | Module declarations                                                                                                                                                                                                   | —                                                                                                                                                                                                                  |
| `cli.rs`      | All subcommands and flags                                                                                                                                                                                             | all command specs                                                                                                                                                                                                  |
| `config.rs`   | `GlobalConfig`, `WikiConfig`, `ResolvedConfig`, `ServeConfig`, `LintConfig`, `GraphConfig`, `IndexConfig`, `ReadConfig`, `SchemaConfig`                                                                               | [configuration.md](../commands/configuration.md), [serve.md](../commands/serve.md)                                                                                                                                 |
| `spaces.rs`   | `Spaces`, `WikiEntry`, `resolve_uri()`, `resolve_name()`, `register()`, `remove()`, `load_all()`                                                                                                                      | [spaces.md](../commands/spaces.md), [page-creation.md](../commands/page-creation.md), [read.md](../commands/read.md)                                                                                               |
| `git.rs`      | `init_repo()`, `commit()`, `current_head()`, `diff_last()`                                                                                                                                                            | [init.md](../commands/init.md), [index.md](../commands/index.md), [graph.md](../commands/graph.md)                                                                                                                 |
| `frontmatter.rs` | Frontmatter parse/write, `scaffold_frontmatter()`, `validate_frontmatter()`, `generate_minimal_frontmatter()` | [page-content.md](../core/page-content.md), [frontmatter-authoring.md](../core/frontmatter-authoring.md), [ingest.md](../pipelines/ingest.md), [page-creation.md](../commands/page-creation.md) |
| `markdown.rs` | `read_page()`, `list_assets()`, `read_asset()`, `promote_to_bundle()`, slug helpers | [ingest.md](../pipelines/ingest.md), [asset-ingest.md](../pipelines/asset-ingest.md), [read.md](../commands/read.md), [page-creation.md](../commands/page-creation.md) |
| `links.rs` | `extract_links()` | [lint.md](../commands/lint.md) |
| `ingest.rs`   | `IngestOptions`, `IngestReport`, validate → git add → commit → index pipeline, asset detection                                                                                                                        | [ingest.md](../pipelines/ingest.md), [asset-ingest.md](../pipelines/asset-ingest.md)                                                                                                                               |
| `search.rs`   | `PageRef`, `PageSummary`, `PageList`, `IndexStatus`, `IndexReport`, tantivy index, `search()`, `list()`, `rebuild_index()`, `index_status()`                                                                          | [search.md](../commands/search.md), [list.md](../commands/list.md), [index.md](../commands/index.md)                                                                                                               |
| `lint.rs`     | `LintReport`, `MissingConnection`, all lint checks, `LINT.md` write, `lint_fix()`                                                                                                                                     | [lint.md](../commands/lint.md), [backlink-quality.md](../llm/backlink-quality.md), [source-classification.md](../core/source-classification.md)                                                                    |
| `graph.rs`    | `GraphReport`, `PageNode`, `GraphFilter`, `build_graph()`, `render_mermaid()`, `render_dot()`, `subgraph()`, `in_degree()`                                                                                            | [graph.md](../commands/graph.md), [lint.md](../commands/lint.md)                                                                                                                                                   |
| `server.rs`   | `WikiServer`, startup, stdio + SSE transport wiring | [serve.md](../commands/serve.md) |
| `mcp.rs`      | All MCP tools, MCP resources, MCP prompts | [features.md](../features.md), [serve.md](../commands/serve.md), [session-bootstrap.md](../llm/session-bootstrap.md) |
| `acp.rs`      | `WikiAgent`, `AcpSession`, ACP workflow dispatch                                                                                                                                                                      | [acp-transport.md](../integrations/acp-transport.md), [serve.md](../commands/serve.md)                                                                                                                             |

---

## Modules Removed from Prior Design

| Module         | Status        | Reason                                                                                                       |
| -------------- | ------------- | ------------------------------------------------------------------------------------------------------------ |
| `context.rs`   | Removed       | Body assembly logic dropped; ref logic moved to `search.rs`                                                  |
| `integrate.rs` | Never created | Page creation logic belongs in `markdown.rs` + `frontmatter.rs` + `spaces.rs` |
| `analysis.rs`  | Removed       | `Enrichment`, `QueryResult`, `Analysis` types removed from design; `Asset`, `AssetKind` moved to `ingest.rs` |

---

## Notes

- `frontmatter.rs`, `markdown.rs`, and `links.rs` replace the former monolithic `markdown.rs`
- `server.rs`, `mcp.rs`, and `acp.rs` share the same underlying engine functions —
  no logic is duplicated between them
- `search.rs` owns both search and index management (rebuild, status) —
  they share the tantivy index handle
