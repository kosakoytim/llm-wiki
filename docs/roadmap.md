---
title: "Roadmap"
summary: "Development roadmap for llm-wiki — from focused engine to skill registry."
status: draft
last_updated: "2025-07-17"
---

# Roadmap

Three deliverables, four phases. The engine (`llm-wiki`), the skills
(`llm-wiki-skills`), and the type schemas (`schemas/`) evolve together
but release independently.

## Phase 0 — Specification Rationalization ✓

Completed. Fresh specifications written from the design documents.
All specs reviewed and marked ready.

See [decisions/rationalize-specs.md](decisions/rationalize-specs.md)
for the full record of what was done.

## Phase 1 — Focused Engine

Fresh implementation from the specifications. The current codebase
(`src/`, `tests/`) moves to `code-ref/` as reference material. New
`src/` built against the specs, pulling implementation patterns and
complete modules from `code-ref/` where they still apply.

Phase 1 uses base frontmatter fields only (`title`, `type`, `summary`,
`status`, `tags`, etc.) with hardcoded field-to-index mapping. No JSON
Schema validation, no `x-index-aliases`, no `x-graph-edges`, no skill
registry features. The dynamic type system comes in Phase 2.

### Step 0: Codebase reset ✓

Modules: `code-ref/` (created), `src/` (emptied), `tests/` (emptied)
Pulls from: nothing — this is a move
Tests: `cargo check` on empty lib
Commit: `chore: move src/ and tests/ to code-ref/, start fresh`

- Move `src/` to `code-ref/src/`
- Move `tests/` to `code-ref/tests/`
- Create minimal `src/lib.rs` (empty) and `src/main.rs` (prints version)
- Update `Cargo.toml`: remove `schemars`, add `frontmatter = "0.4"`,
  add `jsonschema = "0.28"`, add `agent-client-protocol-tokio = "0.1"`,
  keep all other deps
- Verify `cargo check` passes

### Step 1: slug.rs — Slug and WikiUri types ✓

Modules: `src/slug.rs`, `src/lib.rs`
Pulls from: `code-ref/src/markdown.rs` (slug_for, resolve_slug,
resolve_read_target, title_case), `code-ref/src/spaces.rs` (resolve_uri)
Tests: unit tests for Slug::from_path, Slug::resolve, Slug::title,
WikiUri::parse, WikiUri::resolve
Commit: `feat: add slug.rs — Slug, WikiUri types and resolution`

Changes from code-ref:
- Extract slug logic from `markdown.rs` into `Slug` newtype with
  `TryFrom<&str>` validation (no `../`, no extension, no leading `/`)
- Extract URI parsing from `spaces.rs` into `WikiUri` struct
- `Slug::resolve` checks flat then bundle (same logic as resolve_slug)
- `resolve_read_target` moves here (asset fallback)
- `Slug::title` replaces `title_case` (same logic)
- Pure data transformation — only I/O is `is_file`/`is_dir` checks

### Step 2: config.rs — Two-level config loading ✓

Modules: `src/config.rs`, `src/lib.rs`
Pulls from: `code-ref/src/config.rs` (almost entirely)
Tests: unit tests for load/save/resolve, set_global_config_value,
set_wiki_config_value, missing file returns defaults
Commit: `feat: add config.rs — GlobalConfig, WikiConfig, resolution`

Changes from code-ref:
- Remove `LintConfig` from all structs (lint moves to skills)
- Remove `SchemaConfig` and `load_schema` (schema.md eliminated)
- Add `defaults.output_format: String` (default: `"text"`)
- Add `index.memory_budget_mb: u32` (default: `50`)
- Add `index.tokenizer: String` (default: `"en_stem"`)
- Add `graph: Option<GraphConfig>` to `WikiConfig` (overridable)
- Remove lint keys from set_wiki_config_value
- Add new keys to get/set dispatchers

### Step 3: frontmatter.rs — Untyped frontmatter parsing ✓

Modules: `src/frontmatter.rs`, `src/lib.rs`
Pulls from: `code-ref/src/frontmatter.rs` (parse_frontmatter split
logic, write_frontmatter, scaffold_frontmatter,
title_from_body_or_filename)
Tests: unit tests for parse (BOM, no frontmatter, valid YAML),
write round-trip, scaffold, title extraction
Commit: `feat: add frontmatter.rs — untyped BTreeMap parsing`

Changes from code-ref:
- Replace `PageFrontmatter` struct with `ParsedPage { frontmatter:
  BTreeMap<String, serde_yaml::Value>, body: String }`
- Add convenience methods: `title()`, `page_type()`, `status()`,
  `tags()`, `superseded_by()`
- Remove `Claim` struct, `BUILT_IN_TYPES`, `validate_frontmatter`
  (validation moves to type_registry in Phase 2)
- `scaffold_frontmatter` returns `BTreeMap` using `Slug::title`
- `write_frontmatter` serializes `BTreeMap` back to YAML
- `generate_minimal_frontmatter` returns `BTreeMap`

### Step 4: git.rs — Git operations ✓

Modules: `src/git.rs`, `src/lib.rs`
Pulls from: `code-ref/src/git.rs` (all functions)
Tests: integration tests with tempdir — init, commit, commit_paths,
current_head, changed_wiki_files, changed_since_commit
Commit: `feat: add git.rs — git2 wrappers for init, commit, diff`

Changes from code-ref:
- Pull all functions as-is: init_repo, commit, commit_paths,
  current_head, changed_wiki_files, changed_since_commit,
  collect_md_changes, ChangedFile
- Handle empty commits gracefully (no-op, not error) — check if
  tree matches parent tree before creating commit
- Remove `diff_last` (not used in new design)
- Add default signature fallback: try git config first, fall back
  to `llm-wiki <llm-wiki@localhost>`

### Step 5: links.rs — Wiki-link extraction ✓

Modules: `src/links.rs`, `src/lib.rs`
Pulls from: `code-ref/src/links.rs` (extract_wikilinks)
Tests: unit tests for `[[slug]]` extraction, deduplication
Commit: `feat: add links.rs — wiki-link extraction from body text`

Changes from code-ref:
- Keep `extract_wikilinks` as-is (15 lines, works correctly)
- Rewrite `extract_links` to work with `ParsedPage` instead of
  `PageFrontmatter` — read `sources` and `concepts` from BTreeMap
- Used at ingest time to populate `body_links` index field

### Step 6: type_registry.rs — Hardcoded base type registry ✓

Modules: `src/type_registry.rs`, `src/lib.rs`
Pulls from: nothing (new module, Phase 1 is hardcoded)
Tests: unit tests for known_type, base_fields, validate_base
Commit: `feat: add type_registry.rs — hardcoded base type registry`

Phase 1 scope: no JSON Schema, no x-index-aliases, no x-graph-edges.
The registry knows the base fields and validates only that `title`
exists and `type` is non-empty.

```rust
struct TypeRegistry {
    known_types: HashSet<String>,  // built-in type names
}

impl TypeRegistry {
    fn new() -> Self;  // populates with built-in types
    fn is_known(&self, type_name: &str) -> bool;
    fn validate_base(&self, fm: &BTreeMap<String, Value>) -> Result<Vec<String>>;
}
```

Validation: title required, type defaults to "page" if missing,
unknown types produce warnings (not errors) in loose mode.

### Step 7: index_schema.rs + search.rs — Tantivy schema and search ✓

Modules: `src/index_schema.rs`, `src/search.rs`, `src/lib.rs`
Pulls from: `code-ref/src/search.rs` (build_schema, build_document,
search, list, search_all, index_status, index_check, rebuild_index,
update_index, collect_changed_files, open_index, RecoveryContext,
all return types)
Tests: integration tests with tempdir — rebuild, search, list,
update, status, recovery
Commit: `feat: add index_schema.rs + search.rs — tantivy index and BM25 search`

`index_schema.rs` — hardcoded schema for Phase 1:
- Fields: slug (STRING|STORED), title (TEXT|STORED), summary
  (TEXT|STORED), body (TEXT|STORED), type (STRING|STORED), status
  (STRING|STORED), tags (TEXT|STORED), uri (STRING|STORED),
  body_links (STRING|STORED, multi-valued)
- `IndexSchema` struct holds `Schema` + field handles
- Uses `en_stem` tokenizer for text fields (from config)

`search.rs` — adapted from code-ref:
- `build_document` uses `ParsedPage` instead of `PageFrontmatter`
- `search` adds `--type` filter via BooleanQuery (already in code-ref)
- `list` unchanged
- `rebuild_index` uses `ParsedPage` and `IndexSchema`
- `update_index` uses `collect_changed_files` (already in code-ref)
- `index_status` + `index_check` merged into one `index_status`
  that returns all health info
- Add `body_links` field population at index time via
  `links::extract_wikilinks`

### Step 8: ingest.rs — Ingest pipeline ✓

Modules: `src/ingest.rs`, `src/lib.rs`
Pulls from: `code-ref/src/ingest.rs` (structure, normalize_line_endings,
process_file pattern)
Tests: integration tests — ingest file, ingest folder, dry_run,
auto_commit, missing frontmatter generation
Commit: `feat: add ingest.rs — validate, index, commit pipeline`

Changes from code-ref:
- Use `ParsedPage` instead of `PageFrontmatter`
- Use `TypeRegistry::validate_base` instead of `validate_frontmatter`
- Remove `SchemaConfig` parameter
- After validation + commit, call index update (incremental)
- `IngestReport` adds `format` support (text/json output)
- Accept slug or URI as input (not just path) — resolve via
  `WikiUri::parse` + `Slug::resolve`

### Step 9: graph.rs — Concept graph from index ✓

Modules: `src/graph.rs`, `src/lib.rs`
Pulls from: `code-ref/src/graph.rs` (PageNode, GraphFilter,
GraphReport, subgraph, in_degree, wrap_graph_md, render patterns)
Tests: integration tests — build graph, filter by type, subgraph
extraction, mermaid/dot rendering
Commit: `feat: add graph.rs — concept graph from tantivy index`

Changes from code-ref:
- `build_graph` reads from tantivy index instead of walking filesystem
  — no file I/O, uses searcher to iterate all documents
- Edges come from indexed `sources`, `concepts`, `body_links` fields
- Phase 1: untyped edges (`LabeledEdge { relation: String }`) with
  hardcoded relations: `sources` → "fed-by", `concepts` → "depends-on",
  `body_links` → "links-to"
- `render_mermaid` adds node titles as labels, type as CSS class
- `render_dot` adds node labels and type attributes
- `GraphFilter` adds `relation: Option<String>` field
- `--type` filter on nodes, `--relation` filter on edges

### Step 10: markdown.rs — Page I/O ✓

Modules: `src/markdown.rs`, `src/lib.rs`
Pulls from: `code-ref/src/markdown.rs` (read_page, list_assets,
read_asset, promote_to_bundle, create_page, create_section)
Tests: integration tests — read, write, create page/section,
bundle operations
Commit: `feat: add markdown.rs — page read, write, create`

Changes from code-ref:
- Remove `slug_for`, `resolve_slug`, `resolve_read_target`, `title_case`
  (moved to `slug.rs`)
- Use `Slug` type for all slug parameters
- Use `ParsedPage` and `frontmatter::write_frontmatter` with BTreeMap
- Add `write_page(slug, content, wiki_root)` — writes content to
  resolved path (new function for `wiki_content_write`)
- `create_page` accepts optional `--name` and `--type` overrides
- `read_page` adds supersession notice when `superseded_by` is set

### Step 11: spaces.rs — Space management ✓

Modules: `src/spaces.rs`, `src/lib.rs`
Pulls from: `code-ref/src/spaces.rs` (resolve_name, register, remove,
load_all, set_default), `code-ref/src/init.rs` (init, ensure_structure)
Tests: integration tests — create, list, remove, set_default,
re-run behavior
Commit: `feat: add spaces.rs — space create, list, remove, set-default`

Changes from code-ref:
- Merge `init.rs` into `spaces.rs` as `create()` function
- Remove `schema.md` generation from `ensure_structure`
- Add `schemas/` directory creation (empty for Phase 1, populated
  in Phase 2)
- Remove URI resolution (moved to `slug.rs`)
- `create` generates `wiki.toml` with name + description only
  (no `[types.*]` until Phase 2)
- Commit message: `create: <name>` (was `init: <name>`)

### Step 12: engine.rs — Engine composition ✓

Modules: `src/engine.rs`, `src/lib.rs`
Pulls from: nothing (new module)
Tests: integration tests — engine startup, tool dispatch reads,
on_ingest write path
Commit: `feat: add engine.rs — Engine struct and EngineManager`

New module that composes everything:

```rust
struct Engine {
    config: GlobalConfig,
    type_registry: TypeRegistry,
    // Per-wiki: index reader, wiki_root, repo_root
    spaces: HashMap<String, SpaceState>,
}

struct SpaceState {
    wiki_root: PathBuf,
    repo_root: PathBuf,
    index_path: PathBuf,
    index: tantivy::Index,
    reader: tantivy::IndexReader,
    schema: IndexSchema,
}

struct EngineManager {
    engine: Arc<RwLock<Engine>>,
    config_path: PathBuf,
}
```

Startup sequence:
1. Load GlobalConfig
2. Build TypeRegistry (hardcoded for Phase 1)
3. For each registered wiki: open/build tantivy index
4. Assemble Engine, wrap in EngineManager

EngineManager methods:
- `on_ingest(wiki, paths)` — incremental index update
- `on_wiki_added/removed` — stub returning "restart required"
- `on_config_change` — stub returning "restart required"

### Step 13: cli.rs — Clap subcommand hierarchy ✓

Modules: `src/cli.rs`, `src/main.rs`, `src/lib.rs`
Pulls from: `code-ref/src/cli.rs` (structure pattern),
`code-ref/src/main.rs` (dispatch pattern, init_logging)
Tests: `cargo check`, manual CLI smoke test
Commit: `feat: add cli.rs — new subcommand hierarchy with --format`

New hierarchy per implementation/cli.md:
- `Commands::Spaces { SpacesAction::Create|List|Remove|SetDefault }`
- `Commands::Config { ConfigAction::Get|Set|List }`
- `Commands::Content { ContentAction::Read|Write|New|Commit }`
- `Commands::Search` with `--type`, `--format`
- `Commands::List` with `--format`
- `Commands::Ingest` accepting slug|uri, with `--format`
- `Commands::Graph` with `--relation`
- `Commands::Index { IndexAction::Rebuild|Status }` (no Check)
- `Commands::Serve`
- Remove: `Commands::Lint`, `Commands::Instruct`, `Commands::Commit`
  (commit moves under Content)
- `--format text|json` on: search, list, ingest, index rebuild,
  index status, spaces list, config list
- `--wiki` global flag

`main.rs` dispatches to engine via EngineManager. Pulls logging
setup from code-ref/src/main.rs.

### Step 14: mcp/ — MCP server with 15 tools ✓

Modules: `src/mcp/mod.rs`, `src/mcp/tools.rs`, `src/lib.rs`
Pulls from: `code-ref/src/mcp/mod.rs` (ServerHandler impl pattern,
resource listing), `code-ref/src/mcp/tools.rs` (argument helpers,
tool dispatch pattern, ToolResult, collect_page_uris)
Tests: integration tests for each of the 15 tools via direct
function calls (not transport)
Commit: `feat: add mcp/ — 15 MCP tools with ServerHandler`

Changes from code-ref:
- `McpServer` holds `Arc<RwLock<Engine>>` instead of `WikiServer`
  with separate fields
- Rename tools: `wiki_init` → `wiki_spaces_create`,
  `wiki_read` → `wiki_content_read`, `wiki_write` → `wiki_content_write`,
  `wiki_new_page` + `wiki_new_section` → `wiki_content_new`,
  `wiki_commit` → `wiki_content_commit`
- Remove: `wiki_lint`, `wiki_index_check`, all prompts
- Add `--type` parameter to search (already in code-ref)
- Add `--section`, `--name`, `--type` to `wiki_content_new`
- Tool handlers call `EngineManager` for mutations
- Keep argument helpers (arg_str, arg_bool, arg_usize, arg_str_req)
- Keep ToolResult struct and panic isolation
- Remove prompt_list, get_prompt_content, INSTRUCTIONS injection

### Step 14-bis: ops.rs — Shared business logic

Modules: `src/ops.rs`, `src/mcp/handlers.rs`, `src/main.rs`, `src/lib.rs`
Pulls from: `src/mcp/handlers.rs` (complete handler logic),
`src/main.rs` (CLI dispatch logic)
Tests: existing tests must still pass, add `tests/ops.rs` for
direct ops function calls
Commit: `refactor: extract ops.rs — shared business logic for CLI and MCP`

CLI (`main.rs`) and MCP (`mcp/handlers.rs`) duplicate the same
business logic — wiki resolution, config loading, module calls,
EngineManager mutations. The only differences are argument parsing
and output formatting. This step extracts the shared logic into
`src/ops.rs` so both layers become thin adapters.

See [decisions/ops-module.md](docs/decisions/ops-module.md)
for the decision record.

#### What moves to ops.rs

Everything between "args parsed" and "result ready to format":

```rust
// Spaces
pub fn spaces_create(path, name, desc, force, set_default, config_path) -> Result<CreateReport>
pub fn spaces_list(config: &GlobalConfig) -> Vec<WikiEntry>
pub fn spaces_remove(name, delete, config_path) -> Result<()>
pub fn spaces_set_default(name, config_path) -> Result<()>

// Config
pub fn config_get(config_path, key) -> Result<String>
pub fn config_set(config_path, key, value, global, wiki_name) -> Result<String>
pub fn config_list(config_path, global) -> Result<String>

// Content
pub fn content_read(engine, uri, wiki_flag, no_frontmatter, list_assets) -> Result<ContentResult>
pub fn content_write(engine, uri, wiki_flag, content) -> Result<WriteResult>
pub fn content_new(engine, uri, wiki_flag, section, bundle, name, type_) -> Result<String>
pub fn content_commit(engine, wiki_name, slugs, message) -> Result<String>

// Search + List
pub fn search(engine, query, type_, no_excerpt, top_k, include_sections, all, wiki_name) -> Result<Vec<PageRef>>
pub fn list(engine, type_, status, page, page_size, wiki_name) -> Result<PageList>

// Ingest (read + mutation)
pub fn ingest(engine, manager, path, dry_run, wiki_name) -> Result<IngestResult>

// Index
pub fn index_rebuild(manager, wiki_name) -> Result<IndexReport>
pub fn index_status(engine, wiki_name) -> Result<IndexStatus>

// Graph
pub fn graph_build(engine, wiki_name, format, root, depth, type_, relation) -> Result<GraphResult>
```

#### What stays in CLI (`main.rs`)

- Argument parsing (clap structs)
- `--format` text/json output switching
- `println!` / `print!` output
- `init_logging`
- `EngineManager::build` (CLI builds per invocation)

#### What stays in MCP (`mcp/handlers.rs`)

- Argument parsing (JSON map → arg helpers)
- `Content::text` wrapping
- `collect_page_uris` + resource notifications
- `ToolResult` / panic isolation

#### Migration

1. Create `src/ops.rs` — extract functions from `mcp/handlers.rs`
   (MCP has the more complete logic, e.g. search recovery context)
2. Update `src/mcp/handlers.rs` — parse args, call `ops::*`, wrap result
3. Update `src/main.rs` — parse args, call `ops::*`, format output
4. Fix CLI divergences as a side effect (search/list missing recovery)
5. Add `tests/ops.rs` — test ops functions directly
6. Rewrite `docs/implementation/cli-mcp-comparison.md` as
   `docs/decisions/ops-module.md` — record the decision and final design
7. Update `docs/implementation/rust.md` project layout

#### Fixes included

- CLI search and list gain auto-recovery (was missing, MCP had it)
- Single source of truth for all business logic

### Step 15: acp.rs — ACP agent

Modules: `src/acp.rs`, `src/lib.rs`
Pulls from: `code-ref/src/acp.rs` (WikiAgent struct, AcpSession,
streaming helpers, Agent trait impl, serve_acp)
Tests: unit tests for dispatch_workflow, session management
Commit: `feat: add acp.rs — WikiAgent with ACP transport`

Changes from code-ref:
- `WikiAgent` holds `Arc<EngineManager>` instead of separate
  `GlobalConfig` + `Vec<WikiEntry>`
- `resolve_wiki` uses `engine.resolve_wiki_name` + `engine.space`
  instead of iterating `Vec<WikiEntry>`
- `run_research` calls `ops::search` and `ops::content_read`
  instead of direct `crate::search::search` /
  `crate::markdown::read_page`
- Remove `WikiServer::index_path_for` — index path comes from
  `SpaceState` via Engine
- Remove `run_lint` (lint is a skill)
- Remove `INSTRUCTIONS` injection at initialize
- Remove `crate::server::INSTRUCTIONS` reference
- Remove ingest/crystallize placeholder strings
- Replace keyword dispatch with `llm-wiki:` prefix convention
- `serve_acp` accepts `Arc<EngineManager>` instead of
  `Arc<GlobalConfig>`
- Keep: AcpSession, streaming helpers (send_message, send_tool_call,
  send_tool_result), make_tool_id, serve_acp connection setup

### Step 16: server.rs — Transport wiring

Modules: `src/server.rs`, `src/lib.rs`
Pulls from: `code-ref/src/server.rs` (serve_stdio, serve_sse,
serve function, heartbeat, ACP thread supervision)
Tests: `cargo check`, manual smoke test with `llm-wiki serve`
Commit: `feat: add server.rs — stdio + SSE + ACP transport wiring`

Changes from code-ref:
- Build `EngineManager` at startup instead of `WikiServer`
- Share `Arc<EngineManager>` across all transports (MCP + ACP)
- `McpServer::new(manager.clone())` for MCP transports
- `serve_acp(manager.clone())` for ACP transport
- Remove `INSTRUCTIONS` constant and schema.md injection
- Remove `WikiServer` struct (replaced by `McpServer` in mcp/mod.rs)
- Keep: serve_stdio, serve_sse with retry, serve() orchestration,
  heartbeat task, ACP thread with supervision loop
- Startup sequence per spec: load config → mount wikis → check
  staleness → start transports → log summary

### Step 17: Integration tests

Modules: `tests/` (one file per module)
Pulls from: `code-ref/tests/` (test patterns, tempdir setup)
Tests: full integration test suite
Commit: `test: add integration tests for all 15 tools`

One test file per module:
- `tests/slug.rs` — slug resolution, URI parsing
- `tests/config.rs` — load, save, resolve, get/set
- `tests/frontmatter.rs` — parse, write, scaffold
- `tests/git.rs` — init, commit, diff
- `tests/search.rs` — rebuild, search, list, update, status
- `tests/ingest.rs` — ingest file/folder, dry_run, auto_commit
- `tests/graph.rs` — build, filter, render
- `tests/markdown.rs` — read, write, create
- `tests/spaces.rs` — create, list, remove, set_default
- `tests/mcp.rs` — all 15 tools via direct call
- `tests/acp.rs` — session management, dispatch

All tests use `tempfile::tempdir()`. No real paths.

### Step 18: Cleanup and CI

Modules: `Cargo.toml`, `.github/workflows/ci.yml`, `clippy.toml`,
`rustfmt.toml`
Pulls from: `code-ref/` (CI workflow)
Tests: `cargo fmt -- --check`, `cargo clippy -- -D warnings`,
`cargo test`
Commit: `chore: CI pipeline, clippy clean, fmt clean`

- Ensure `cargo fmt -- --check` passes
- Ensure `cargo clippy -- -D warnings` passes
- Remove unused dependencies from `Cargo.toml`
- Update CI workflow for new structure
- Verify `cargo build --release` produces working binary

### What MUST work at the end of Phase 1

- [ ] `llm-wiki spaces create/list/remove/set-default`
- [ ] `llm-wiki config get/set/list`
- [ ] `llm-wiki content read/write/new/commit`
- [ ] `llm-wiki search` with `--type` filter and `--format`
- [ ] `llm-wiki list` with `--type`, `--status`, `--format`
- [ ] `llm-wiki ingest` with `--format`
- [ ] `llm-wiki graph` with `--format`, `--root`, `--depth`, `--type`
- [ ] `llm-wiki index rebuild/status`
- [ ] `llm-wiki serve` (stdio + SSE)
- [ ] `llm-wiki serve --acp`
- [ ] All 15 MCP tools working
- [ ] Integration tests for each tool

### Skills (llm-wiki-skills)

- [ ] Create the `llm-wiki-skills` git repository
- [ ] Set up Claude Code plugin structure
- [ ] Write the 10 initial skills:
  - `bootstrap` — session orientation
  - `ingest` — source processing workflow
  - `crystallize` — distil session into wiki pages
  - `research` — search, read, synthesize
  - `lint` — structural audit + fix
  - `graph` — generate and interpret concept graph
  - `frontmatter` — frontmatter authoring reference
  - `skill` — find and activate wiki skills
  - `write-page` — create page of any type
  - `configure-hugo` — configure wiki for Hugo rendering
- [ ] Test with `claude --plugin-dir ./llm-wiki-skills`

### Milestone

Engine binary with 15 tools. Skills repo with 10 skills. Claude Code
plugin installable. `llm-wiki serve` + plugin = working system.

## Phase 2 — Type System

JSON Schema validation per type. Type registry in `wiki.toml`.
`schema.md` eliminated.

### Engine (llm-wiki)

- [ ] Add `[types.*]` section to `wiki.toml`
- [ ] Add `schemas/` directory to wiki repo layout
- [ ] Ship default JSON Schema files:
  - `base.json` — required: `title`, `type`
  - `concept.json` — extends base, adds `read_when`, `sources`,
    `concepts`, `confidence`, `claims`
  - `paper.json` — extends base, adds `read_when`, `sources`,
    `concepts`, `confidence`, `claims`
  - `skill.json` — standalone, uses `x-index-aliases`
  - `doc.json` — extends base, adds `read_when`, `sources`
  - `section.json` — extends base
- [ ] Implement JSON Schema validation on `wiki_ingest`
- [ ] Implement `x-index-aliases` — resolve field aliases at ingest
- [ ] `llm-wiki spaces create` generates default `wiki.toml` with
  `[types.*]` entries and `schemas/` directory
- [ ] `wiki_config list` returns type names + descriptions
- [ ] Schema change detection via `schema_hash` in `state.toml`
- [ ] Per-type hashes for partial rebuild

### Skills (llm-wiki-skills)

- [ ] Update `frontmatter` skill with type-specific guidance
- [ ] Update `bootstrap` skill to read types from `wiki_config`
- [ ] Update `ingest` skill to reference type validation

### Milestone

Type-specific JSON Schema validation on ingest. Field aliasing for
skill and doc pages. Custom types addable via `wiki.toml` + schema file.

## Phase 3 — Typed Graph

`x-graph-edges` in type schemas. Typed nodes and labeled edges.
`wiki_graph` filters by relation.

### Engine (llm-wiki)

- [ ] Implement `x-graph-edges` parsing from JSON Schema files
- [ ] At ingest: read edge declarations, index edges with relation labels
- [ ] At graph build: petgraph nodes get `type` label, edges get
  `relation` label
- [ ] `wiki_graph --relation <label>` — filter edges by relation
- [ ] Mermaid and DOT output include relation labels
- [ ] Warn on ingest when edge target has wrong type

### Default edge declarations

| Schema | Field | Relation | Target types |
|--------|-------|----------|-------------|
| `concept.json` | `sources` | `fed-by` | All source types |
| `concept.json` | `concepts` | `depends-on` | `concept` |
| `concept.json` | `superseded_by` | `superseded-by` | Any |
| `paper.json` | `sources` | `cites` | All source types |
| `paper.json` | `concepts` | `informs` | `concept` |
| `paper.json` | `superseded_by` | `superseded-by` | Any |
| `skill.json` | `document_refs` | `documented-by` | `doc` |
| `skill.json` | `superseded_by` | `superseded-by` | Any |
| `doc.json` | `sources` | `informed-by` | All source types |
| `doc.json` | `superseded_by` | `superseded-by` | Any |

Body `[[wiki-links]]` get a generic `links-to` relation.

### Skills (llm-wiki-skills)

- [ ] Update `graph` skill with relation-aware instructions
- [ ] Update `lint` skill to detect type constraint violations

### Milestone

Labeled graph edges. Relation-filtered graph output. Type constraint
warnings on ingest.

## Phase 4 — Skill Registry

The wiki becomes a full skill registry.

### Engine (llm-wiki)

- [ ] Verify `wiki_search --type skill` works end-to-end with
  `x-index-aliases`
- [ ] Verify `wiki_list --type skill` returns skill-specific metadata
- [ ] Verify `wiki_graph` renders skill edges correctly
- [ ] Cross-wiki skill discovery: `wiki_search --type skill --all`

### Skills (llm-wiki-skills)

- [ ] Finalize `skill` skill — find, read, activate wiki skills
- [ ] Document the skill authoring workflow
- [ ] Add example wiki skills to the README

### Milestone

Wiki as skill registry. Agents discover skills via search, read them
via `wiki_content_read`, activate them by injecting the body into
context.

## Future

Ideas that don't fit in the four phases:

- `wiki_diff` — changes between two commits for a page
- `wiki_history` — git log for a specific page
- `wiki_search` facets — type/status/tag distributions alongside results
- `wiki_export` — static site, PDF, or EPUB
- Cross-wiki links — `wiki://<name>/<slug>` resolved in graph and search
- Webhook on ingest — notify external systems
- `wiki_watch` — filesystem watcher that auto-ingests on save
- Skill composition — `extends` field for wiki skills
- Confidence propagation — compute concept confidence from source graph
- Persistent graph index — avoid rebuilding petgraph on every call

## Related: llm-wiki-hugo-cms

A separate project that renders a wiki as a Hugo site. The wiki is the
CMS, Hugo is the renderer. See
[decisions/three-repositories.md](decisions/three-repositories.md) for
why it's a separate repo.
