---
title: "Tasks"
summary: "Full implementation task list — one checkbox per concrete action, ordered within each phase."
read_when:
  - Starting implementation work on a phase
  - Checking what remains in the current phase
  - Picking up where a previous session left off
status: active
last_updated: "2025-07-15"
---

# Tasks

Phases match `docs/roadmap.md`. Check off tasks as they are completed.
When all tasks in a phase are done, archive the phase to `docs/archive/`.

---

## Phase 1 — Foundation: Schema + Config + Spaces

### `config.rs`
- [ ] Define `GlobalSection { default_wiki: String }`
- [ ] Define `WikiEntry { name, path, description, remote }`
- [ ] Define `Defaults { search_top_k, search_excerpt, search_sections, page_mode, list_page_size }`
- [ ] Define `ReadConfig { no_frontmatter }`
- [ ] Define `IndexConfig { auto_rebuild }`
- [ ] Define `GraphConfig { format, depth, r#type, output }`
- [ ] Define `ServeConfig { sse, sse_port, acp }`
- [ ] Define `LintConfig { fix_missing_stubs, fix_empty_sections }`
- [ ] Define `ValidationConfig { type_strictness }` — `strict | loose`
- [ ] Define `SchemaConfig { custom_types: Vec<String> }`
- [ ] Define `GlobalConfig` composing all sections
- [ ] Define `WikiConfig { name, description }`
- [ ] Define `ResolvedConfig` — merged global + per-wiki
- [ ] Implement `resolve(global, per_wiki) -> ResolvedConfig`
- [ ] Implement `load_global(path: &Path) -> Result<GlobalConfig>`
- [ ] Implement `load_wiki(wiki_root: &Path) -> Result<WikiConfig>`
- [ ] Implement `load_schema(wiki_root: &Path) -> Result<SchemaConfig>`

### `spaces.rs`
- [ ] Implement `resolve_uri(uri, global) -> Result<(WikiEntry, String)>`
- [ ] Implement `resolve_name(name, global) -> Result<WikiEntry>`
- [ ] Implement `register(entry, force, config_path) -> Result<()>`
- [ ] Implement `remove(name, delete, config_path) -> Result<()>`
- [ ] Implement `load_all(global) -> Vec<WikiEntry>`
- [ ] Implement `set_default(name, config_path) -> Result<()>`

### `git.rs`
- [ ] Implement `init_repo(path: &Path) -> Result<()>`
- [ ] Implement `commit(repo_root, message) -> Result<String>`
- [ ] Implement `current_head(repo_root) -> Result<String>`
- [ ] Implement `diff_last(repo_root) -> Result<Vec<String>>`

### `cli.rs` — Phase 1 commands
- [ ] `wiki init <path> --name --description --force --set-default`
- [ ] `wiki config get <key>`
- [ ] `wiki config set <key> <value> [--global] [--wiki]`
- [ ] `wiki config list [--global] [--wiki]`
- [ ] `wiki spaces list`
- [ ] `wiki spaces remove <name> [--delete]`
- [ ] `wiki spaces set-default <name>`

### `mcp.rs` — Phase 1 tools
- [ ] `wiki_init`
- [ ] `wiki_config`
- [ ] `wiki_spaces_list`
- [ ] `wiki_spaces_remove`
- [ ] `wiki_spaces_set_default`

### `init.rs` logic (called by cli + mcp)
- [ ] Create directory structure (`inbox/`, `raw/`, `wiki/`)
- [ ] Write `README.md`
- [ ] Write `wiki.toml`
- [ ] Write default `schema.md`
- [ ] Run `git init` + initial commit `init: <name>`
- [ ] Register in `~/.wiki/config.toml`

### `tests/config.rs`
- [ ] `load_global` parses a valid `config.toml`
- [ ] `load_global` returns error on malformed TOML
- [ ] `resolve` per-wiki value overrides global value
- [ ] `resolve` falls back to global when per-wiki key absent
- [ ] `load_schema` parses custom types from `schema.md`
- [ ] `load_schema` returns empty custom types when `schema.md` absent

### `tests/spaces.rs`
- [ ] `resolve_uri` parses full URI `wiki://name/slug` correctly
- [ ] `resolve_uri` uses default wiki for short URI `wiki://slug`
- [ ] `resolve_uri` returns error for unknown wiki name
- [ ] `register` appends entry to `config.toml`
- [ ] `register` with `force` updates existing entry
- [ ] `register` errors on duplicate name without `force`
- [ ] `remove` removes entry from `config.toml`
- [ ] `remove` with `--delete` removes directory from disk
- [ ] `remove` errors when wiki is current default

### `tests/git.rs`
- [ ] `init_repo` creates a git repository at path
- [ ] `commit` creates a commit and returns hash
- [ ] `current_head` returns the current commit hash
- [ ] `current_head` matches hash returned by `commit`

### Exit criteria
- [ ] `cargo test` green
- [ ] `wiki init ~/wikis/test --name test` creates structure and registers
- [ ] `wiki config list` shows resolved config
- [ ] `wiki spaces list` shows registered wikis

---

## Phase 2 — Core Write Loop: Ingest + Page Creation

### `frontmatter.rs`
- [ ] Define `PageFrontmatter` struct (all fields per spec)
- [ ] Implement `parse_frontmatter(content: &str) -> Result<(PageFrontmatter, String)>`
- [ ] Implement `write_frontmatter(fm: &PageFrontmatter, body: &str) -> String`
- [ ] Implement `generate_minimal_frontmatter(title: &str) -> PageFrontmatter`
- [ ] Implement `scaffold_frontmatter(slug: &str) -> PageFrontmatter`

### `markdown.rs`
- [ ] Implement `slug_for(path, wiki_root) -> String`
- [ ] Implement `resolve_slug(slug, wiki_root) -> Result<PathBuf>`
- [ ] Implement `read_page(slug, wiki_root, no_frontmatter) -> Result<String>`
- [ ] Implement `list_assets(slug, wiki_root) -> Result<Vec<String>>`
- [ ] Implement `read_asset(slug, filename, wiki_root) -> Result<Vec<u8>>`
- [ ] Implement `promote_to_bundle(slug, wiki_root) -> Result<()>`
- [ ] Implement `create_page(slug, bundle, wiki_root) -> Result<PathBuf>`
- [ ] Implement `create_section(slug, wiki_root) -> Result<PathBuf>`

### `ingest.rs`
- [ ] Define `IngestOptions { dry_run: bool }`
- [ ] Define `IngestReport { pages_validated, assets_found, warnings, commit }`
- [ ] Implement `ingest(path, options, wiki_root) -> Result<IngestReport>`
  - [ ] Walk path, collect `.md` files and assets
  - [ ] Validate each `.md` (title present, YAML valid)
  - [ ] Generate minimal frontmatter for files without it
  - [ ] Set `last_updated` to today
  - [ ] `git add` + commit
  - [ ] Update tantivy index

### `cli.rs` — Phase 2 commands
- [ ] `wiki ingest <path> [--dry-run]`
- [ ] `wiki new page <wiki:// URI> [--bundle] [--dry-run]`
- [ ] `wiki new section <wiki:// URI> [--dry-run]`

### `mcp.rs` — Phase 2 tools
- [ ] `wiki_write`
- [ ] `wiki_ingest`
- [ ] `wiki_new_page`
- [ ] `wiki_new_section`

### `tests/frontmatter.rs`
- [ ] `parse_frontmatter` round-trips all required fields
- [ ] `parse_frontmatter` returns error on invalid YAML
- [ ] `parse_frontmatter` returns error when no frontmatter block
- [ ] `write_frontmatter` produces valid YAML block + blank line + body
- [ ] `generate_minimal_frontmatter` sets title from H1, falls back to filename
- [ ] `generate_minimal_frontmatter` sets status `active`, type `page`
- [ ] `scaffold_frontmatter` derives title from slug segments
- [ ] `scaffold_frontmatter` sets status `draft`, type `page`

### `tests/markdown.rs`
- [ ] `slug_for` flat file returns path without extension
- [ ] `slug_for` bundle `index.md` returns parent directory path
- [ ] `resolve_slug` finds flat `.md` file
- [ ] `resolve_slug` finds bundle `index.md`
- [ ] `resolve_slug` returns error for missing slug
- [ ] `read_page` returns full content including frontmatter
- [ ] `read_page` with `no_frontmatter=true` strips frontmatter block
- [ ] `list_assets` returns empty vec for flat page
- [ ] `list_assets` returns `wiki://` URIs for bundle assets
- [ ] `read_asset` returns raw bytes for a co-located asset
- [ ] `promote_to_bundle` moves `{slug}.md` to `{slug}/index.md`
- [ ] `promote_to_bundle` slug resolves correctly after promotion
- [ ] `create_page` creates flat `.md` with scaffold frontmatter
- [ ] `create_page` with `bundle=true` creates `{slug}/index.md`
- [ ] `create_page` auto-creates missing parent sections
- [ ] `create_section` creates `{slug}/index.md` with section frontmatter

### `tests/ingest.rs`
- [ ] `ingest` validates a valid page and commits
- [ ] `ingest` rejects a page with no `title` field
- [ ] `ingest` rejects a page with invalid YAML frontmatter
- [ ] `ingest` generates minimal frontmatter for a file without it
- [ ] `ingest` sets `last_updated` to today on every page
- [ ] `ingest` with `dry_run=true` does not commit
- [ ] `ingest` on a folder ingests all `.md` files recursively
- [ ] `ingest` detects co-located assets and includes them in `assets_found`
- [ ] `IngestReport` commit hash matches `git HEAD` after ingest

### Exit criteria
- [ ] `wiki new page wiki://test/concepts/foo` creates scaffolded page and commits
- [ ] `wiki ingest wiki/concepts/foo.md` validates, commits, indexes
- [ ] `wiki ingest wiki/` ingests all pages in the tree
- [ ] `--dry-run` shows what would happen without committing

---

## Phase 3 — Frontmatter Validation + Type Taxonomy

### `frontmatter.rs` — validation
- [ ] Define built-in type list (concept, query-result, section, paper, article, documentation, clipping, transcript, note, data, book-chapter, thread)
- [ ] Implement `validate_frontmatter(fm, schema) -> Result<Vec<Warning>>`
  - [ ] Required fields: title, summary, read_when, status, type, last_updated
  - [ ] Type recognized (built-in + custom from schema)
  - [ ] `source-summary` deprecated warning
  - [ ] `strict` mode: unknown type → error
  - [ ] `loose` mode: unknown type → warning

### `ingest.rs` — wire validation
- [ ] Call `validate_frontmatter` on every `.md` during ingest
- [ ] Respect `validation.type_strictness` from resolved config
- [ ] Include warnings in `IngestReport`

### `src/instructions.md`
- [ ] Write `## frontmatter` section (condensed type taxonomy + per-type templates + common mistakes)
- [ ] Write `## help` section
- [ ] Write `## new` section
- [ ] Write `## ingest` section
- [ ] Write `## research` section
- [ ] Write `## lint` section
- [ ] Write `## crystallize` section

### `tests/frontmatter.rs` — Phase 3 additions
- [ ] `validate_frontmatter` passes for a fully valid page
- [ ] `validate_frontmatter` warns on missing `read_when`
- [ ] `validate_frontmatter` warns on missing `summary`
- [ ] `validate_frontmatter` warns on `source-summary` type
- [ ] `validate_frontmatter` in `loose` mode warns on unknown type, does not error
- [ ] `validate_frontmatter` in `strict` mode errors on unknown type
- [ ] `validate_frontmatter` accepts custom type defined in `SchemaConfig`

### Exit criteria
- [ ] `wiki ingest` warns on missing `read_when`
- [ ] `wiki ingest` warns on `source-summary` type
- [ ] `wiki ingest` rejects unknown type in strict mode
- [ ] `wiki instruct frontmatter` prints the frontmatter guide

---

## Phase 4 — Search + Read + Index

### `search.rs`
- [ ] Define `PageRef { slug, uri, title, score, excerpt: Option<String> }`
- [ ] Define `PageSummary { slug, uri, title, r#type, status, tags }`
- [ ] Define `PageList { pages, total, page, page_size }`
- [ ] Define `IndexStatus { wiki, path, built, pages, sections, stale }`
- [ ] Define `IndexReport { wiki, pages_indexed, duration_ms }`
- [ ] Implement tantivy schema with all frontmatter fields
- [ ] Implement `rebuild_index(wiki_root, index_path) -> Result<IndexReport>`
  - [ ] Walk `wiki/`, index all `.md` files
  - [ ] Write `state.toml` with commit hash, page count, built date
- [ ] Implement `index_status(wiki_name, index_path, wiki_root) -> Result<IndexStatus>`
  - [ ] Read `state.toml`, compare commit vs `git HEAD`
- [ ] Implement `search(query, options, index_path) -> Result<Vec<PageRef>>`
- [ ] Implement `list(filter, page, page_size, index_path) -> Result<PageList>`
- [ ] Staleness check before search/list — warn or auto-rebuild per config

### `cli.rs` — Phase 4 commands
- [ ] `wiki search "<query>" [--no-excerpt] [--top-k] [--include-sections] [--all] [--wiki]`
- [ ] `wiki read <slug|uri> [--no-frontmatter] [--list-assets] [--wiki]`
- [ ] `wiki list [--type] [--status] [--page] [--page-size] [--wiki]`
- [ ] `wiki index rebuild [--wiki] [--dry-run]`
- [ ] `wiki index status [--wiki]`

### `mcp.rs` — Phase 4 tools
- [ ] `wiki_search`
- [ ] `wiki_read`
- [ ] `wiki_list`
- [ ] `wiki_index_rebuild`
- [ ] `wiki_index_status`

### `tests/search.rs`
- [ ] `rebuild_index` indexes all pages in `wiki/` and writes `state.toml`
- [ ] `rebuild_index` stores commit hash in `state.toml`
- [ ] `index_status` returns `stale: false` immediately after rebuild
- [ ] `index_status` returns `stale: true` after a new commit
- [ ] `index_status` returns `built: None` when index does not exist
- [ ] `search` returns results ranked by BM25 score
- [ ] `search` with `no_excerpt` returns `PageRef` with `excerpt: None`
- [ ] `search` with `include_sections=false` excludes `type: section` pages
- [ ] `search` with `include_sections=true` includes `type: section` pages
- [ ] `search` `--type paper` filters results to paper pages only
- [ ] `list` returns all pages ordered by slug
- [ ] `list` with `--type concept` returns only concept pages
- [ ] `list` with `--status draft` returns only draft pages
- [ ] `list` pagination returns correct page and total

### Exit criteria
- [ ] `wiki search "MoE scaling"` returns ranked `Vec<PageRef>` with `wiki://` URIs
- [ ] `wiki read wiki://test/concepts/foo` returns full page content
- [ ] `wiki list --type concept` returns paginated concept pages
- [ ] `wiki index status` shows stale/fresh correctly
- [ ] `wiki index rebuild` rebuilds and writes `state.toml`

---

## Phase 5 — Lint + Graph

### `links.rs`
- [ ] Implement `extract_links(content: &str) -> Vec<String>` — frontmatter slugs + body `[[links]]`

### `lint.rs`
- [ ] Define `MissingConnection { slug_a, slug_b, overlapping_terms }`
- [ ] Define `LintReport { orphans, missing_stubs, empty_sections, missing_connections, untyped_sources, date }`
- [ ] Implement `lint(wiki_root, config) -> Result<LintReport>`
  - [ ] Orphan detection — in-degree 0 via petgraph
  - [ ] Missing stub detection — referenced slugs that don't exist
  - [ ] Empty section detection — dirs without `index.md`
  - [ ] Missing connection detection — term overlap heuristic
  - [ ] Untyped source detection — missing or `source-summary` type
- [ ] Implement `write_lint_md(report, repo_root) -> Result<()>` — all 5 sections
- [ ] Implement `lint_fix(wiki_root, config, only) -> Result<()>`
  - [ ] Create stub pages for missing stubs
  - [ ] Create `index.md` for empty sections

### `graph.rs`
- [ ] Define `PageNode { slug, title, r#type }`
- [ ] Define `GraphFilter { root, depth, types }`
- [ ] Define `GraphReport { nodes, edges, output, committed }`
- [ ] Implement `build_graph(wiki_root, filter) -> DiGraph<PageNode, ()>`
- [ ] Implement `render_mermaid(graph) -> String`
- [ ] Implement `render_dot(graph) -> String`
- [ ] Implement `subgraph(graph, root, depth) -> DiGraph<PageNode, ()>`
- [ ] Implement `in_degree(graph, slug) -> usize`

### `cli.rs` — Phase 5 commands
- [ ] `wiki lint [--wiki] [--dry-run]`
- [ ] `wiki lint fix [--only missing-stubs|empty-sections] [--dry-run] [--wiki]`
- [ ] `wiki graph [--format] [--root] [--depth] [--type] [--output] [--dry-run] [--wiki]`

### `mcp.rs` — Phase 5 tools
- [ ] `wiki_lint`
- [ ] `wiki_graph`

### `tests/links.rs`
- [ ] `extract_links` returns slugs from `sources` frontmatter field
- [ ] `extract_links` returns slugs from `concepts` frontmatter field
- [ ] `extract_links` returns slugs from body `[[wikilinks]]`
- [ ] `extract_links` deduplicates repeated slugs
- [ ] `extract_links` returns empty vec for page with no links

### `tests/lint.rs`
- [ ] `lint` detects orphan pages (in-degree 0)
- [ ] `lint` does not flag pages with at least one incoming link as orphans
- [ ] `lint` detects missing stubs (referenced slug does not exist)
- [ ] `lint` detects empty sections (dir without `index.md`)
- [ ] `lint` detects untyped sources (`source-summary` type)
- [ ] `lint` detects untyped sources (missing type on source-like page)
- [ ] `write_lint_md` writes all 5 sections always, even when empty
- [ ] `write_lint_md` shows `_No X found._` for empty sections
- [ ] `lint_fix` creates stub pages for missing stubs
- [ ] `lint_fix` creates `index.md` for empty sections
- [ ] `lint_fix` with `only=missing-stubs` does not touch empty sections

### `tests/graph.rs`
- [ ] `build_graph` creates edges from `sources` frontmatter
- [ ] `build_graph` creates edges from `concepts` frontmatter
- [ ] `build_graph` creates edges from body `[[links]]`
- [ ] `build_graph` skips broken references (missing stubs)
- [ ] `in_degree` returns 0 for orphan page
- [ ] `in_degree` returns correct count for linked page
- [ ] `render_mermaid` produces valid Mermaid `graph TD` block
- [ ] `render_dot` produces valid DOT `digraph` block
- [ ] `subgraph` returns only nodes within depth hops of root
- [ ] `subgraph` with `depth=0` returns only the root node

### Exit criteria
- [ ] `wiki lint` writes `LINT.md` at repository root with all 5 sections
- [ ] `wiki lint fix` creates missing stubs and empty section indexes
- [ ] `wiki graph` outputs Mermaid to stdout
- [ ] `wiki graph --format dot` outputs DOT format
- [ ] `wiki graph --root <slug> --depth 2` outputs subgraph

---

## Phase 6 — MCP Server + Session Bootstrap

### `server.rs`
- [ ] Implement `WikiServer` with all registered wikis mounted at startup
- [ ] Implement startup sequence (load config → mount wikis → check staleness → start stdio)
- [ ] Implement SSE transport (`--sse [:<port>]`)
- [ ] Inject `instructions.md` + `schema.md` at session start

### `mcp.rs` — complete
- [ ] Wire all tools from phases 1–5 into `WikiServer`
- [ ] Add `wiki` param to all tools (target specific wiki)
- [ ] MCP resources namespaced by wiki name (`wiki://<name>/<slug>`)
- [ ] MCP resource update notifications on every ingest
- [ ] Prompts: `ingest_source`, `research_question`, `lint_and_fix`
- [ ] Remove `wiki_context` tool

### `src/instructions.md` — session bootstrap
- [ ] Write `## session-orientation` preamble
- [ ] Write `## linking-policy` preamble
- [ ] Add orientation step to every workflow section

### `cli.rs` — Phase 6 commands
- [ ] `wiki serve [--sse [:<port>]] [--acp] [--dry-run]`
- [ ] `wiki instruct [<workflow>]`

### Exit criteria
- [ ] `wiki serve` starts, all registered wikis accessible via MCP
- [ ] Claude Code can call all MCP tools
- [ ] `wiki instruct crystallize` prints the crystallize workflow
- [ ] `schema.md` is injected alongside instructions at session start

---

## Phase 7 — ACP Transport

### `acp.rs`
- [ ] Add `agent-client-protocol = "0.10"` and `agent-client-protocol-tokio = "0.1"` to `Cargo.toml`
- [ ] Define `AcpSession { id, label, wiki, created_at, active_run }`
- [ ] Define `WikiAgent { spaces, sessions }`
- [ ] Implement `Agent::initialize` — inject `instructions.md` as system context
- [ ] Implement `Agent::new_session`
- [ ] Implement `Agent::load_session`
- [ ] Implement `Agent::list_sessions`
- [ ] Implement `Agent::prompt` — workflow dispatch (ingest, research, lint, crystallize)
- [ ] Implement `Agent::cancel`
- [ ] Implement `serve_acp(spaces) -> Result<()>`

### `server.rs`
- [ ] Start ACP stdio server alongside MCP when `--acp`

### Exit criteria
- [ ] `wiki serve --acp` starts without error
- [ ] Zed agent panel connects and lists sessions
- [ ] `ingest` workflow streams tool calls visibly
- [ ] `research` workflow streams answer

---

## Phase 8 — Claude Plugin

### `.claude-plugin/`
- [ ] Update `plugin.json` to spec
- [ ] Update `marketplace.json` to spec
- [ ] Update `.mcp.json` to spec
- [ ] Write `commands/help.md`
- [ ] Write `commands/init.md`
- [ ] Write `commands/new.md`
- [ ] Write `commands/ingest.md`
- [ ] Write `commands/research.md`
- [ ] Write `commands/crystallize.md`
- [ ] Write `commands/lint.md`
- [ ] Update `skills/llm-wiki/SKILL.md` — remove contradiction workflow
- [ ] Verify `wiki instruct <workflow>` returns correct instructions for all 7 workflows

### Exit criteria
- [ ] `claude plugin add /path/to/llm-wiki` succeeds
- [ ] `/llm-wiki:ingest` triggers the ingest workflow
- [ ] `/llm-wiki:crystallize` triggers the crystallize workflow

---

## Phase 9 — Documentation

- [ ] Rewrite `README.md` — features, workflows, quick start, MCP client setup
- [ ] Rewrite `CONTRIBUTING.md` — module architecture, dev setup, test patterns, release process
- [ ] Rewrite `CHANGELOG.md` — feature-oriented, not a git log

### Exit criteria
- [ ] A new contributor can read `README.md` and run `wiki init` within 5 minutes
- [ ] `CONTRIBUTING.md` references `docs/implementation/rust.md` for dev standards
- [ ] `CHANGELOG.md` describes what the tool can do at each version
