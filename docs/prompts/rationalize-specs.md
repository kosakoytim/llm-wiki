# Rationalize llm-wiki Specifications

## Context

The llm-wiki project has gone through a design rethink. The new design
is captured in these documents:

- **`docs/overview.md`** — project introduction, design principles,
  architecture, core concepts
- **`docs/focused-llm-wiki-design.md`** — the focused engine: 16 MCP
  tools, skills in a separate repo, wiki as skill registry
- **`docs/type-specific-frontmatter.md`** — JSON Schema type profiles
  in `wiki.toml`, `x-index-aliases`, `x-graph-edges`, `schema.md`
  eliminated
- **`docs/roadmap.md`** — phased roadmap from spec cleanup through
  skill registry

The old specifications in `docs/specifications/` were written before
this rethink. They have been moved in bulk to
`docs/specifications/archive/` as reference material.

## Your Task

Write fresh specifications in `docs/specifications/` from the design
documents. Pull content from the archive when useful, but write each
spec against the new design — don't patch old prose.

## Read first

Read these documents in order before writing anything:

1. `docs/overview.md`
2. `docs/focused-llm-wiki-design.md`
3. `docs/type-specific-frontmatter.md`
4. `docs/roadmap.md`

The archive is at `docs/specifications/archive/` — consult it for
detail that the design docs summarize but don't fully specify (e.g.,
CLI flag details, MCP tool parameter types, index rebuild behavior).

## Target layout

```
docs/specifications/
├── README.md                        ← index with tables per section
├── features.md                      ← canonical feature inventory
│
├── model/                           ← data model and knowledge structure
│   ├── repository-layout.md         ← repo structure, wiki root, slug resolution
│   ├── page-content.md              ← page format, flat vs bundle, body conventions
│   ├── frontmatter.md               ← THE frontmatter reference (all fields, per-type templates)
│   ├── epistemic-model.md           ← why types carry epistemic distinctions
│   └── type-system.md               ← JSON Schema profiles, wiki.toml registry, aliases, graph edges, default types
│
├── tools/                           ← the 16 MCP/ACP/CLI tools
│   ├── overview.md                  ← tool surface summary, design principle, global flags
│   ├── space-management.md          ← init, spaces, config (5 tools)
│   ├── content-operations.md        ← read, write, new-page, new-section, commit (5 tools)
│   └── search-and-index.md          ← search, list, ingest, graph, index-rebuild, index-status (6 tools)
│
├── engine/                          ← engine behavior contracts
│   ├── index-management.md          ← tantivy index: fields, staleness, rebuild
│   ├── graph.md                     ← petgraph: typed nodes, labeled edges, rendering
│   ├── ingest-pipeline.md           ← validate → alias → index → commit flow
│   └── server.md                    ← serve: transports, multi-wiki, resilience
│
├── integrations/                    ← how external tools connect
│   ├── mcp-clients.md               ← Cursor, VS Code, Windsurf, generic MCP config
│   └── acp-transport.md             ← ACP for Zed / VS Code agent panel
│
└── archive/                         ← old specs (read-only reference)
    └── ... (all previous spec files)
```

## Rules

### Single source of truth

Each concept defined in exactly one place:

| Concept | Defined in | Other files do |
|---------|-----------|---------------|
| Frontmatter fields | `model/frontmatter.md` | Reference, not redefine |
| Type system | `model/type-system.md` | Reference, not repeat JSON Schema examples |
| Tool surface (16 tools) | `tools/overview.md` | Tool group files add parameter detail |
| Ingest pipeline | `engine/ingest-pipeline.md` | Reference the flow, not repeat it |
| Skill inventory | `docs/focused-llm-wiki-design.md` §8 | Reference, not repeat |

### Design over code

Specs describe *what* and *why*. Not *how* in Rust.

- No Rust struct definitions, function signatures, or pseudocode
- Yes: CLI examples (`llm-wiki search "query" --type skill`)
- Yes: MCP tool parameters (what the tool accepts and returns)
- Yes: data models as field tables (PageRef, PageSummary, IngestReport)

### No dead references

These things no longer exist — do not reference them:

- `schema.md` → replaced by `wiki.toml` `[types.*]` + `schemas/`
- `llm-wiki instruct` → replaced by `llm-wiki-skills` repo
- `wiki_lint` tool → now a skill
- `wiki_context` / `wiki_ask` tools → removed, `wiki_search` covers both
- `wiki_index_check` tool → folded into `wiki_index_status`
- `source-summary` type → use specific source types
- `integrate_file` / `integrate_folder` → removed
- `src/instructions.md` → removed, content in skills

### Frontmatter on every spec

Every spec file must have:

```yaml
---
title: "..."
summary: "..."
read_when:
  - ...
status: active
last_updated: "2025-07-17"
---
```

## Spec-by-spec instructions

### README.md

Index page with two parts:

**Part 1 — Specification status table.** Every target spec file listed
with its status:

| Status | Meaning |
|--------|---------|
| `ready` | Written, reviewed, aligned with design docs |
| `proposal` | Draft exists, needs review or completion |
| `plan` | Not yet written, scope defined in this prompt |

The table:

| Spec | Section | Status | Description |
|------|---------|--------|-------------|
| `model/repository-layout.md` | model | plan | Repo structure, wiki root, slug resolution |
| `model/page-content.md` | model | plan | Page format, flat vs bundle, body conventions |
| `model/frontmatter.md` | model | plan | THE frontmatter reference — all fields, per-type templates |
| `model/epistemic-model.md` | model | plan | Why types carry epistemic distinctions |
| `model/type-system.md` | model | plan | JSON Schema profiles, wiki.toml registry, aliases, graph edges, default types |
| `tools/overview.md` | tools | plan | Tool surface summary, design principle, global flags |
| `tools/space-management.md` | tools | plan | init, spaces, config (5 tools) |
| `tools/content-operations.md` | tools | plan | read, write, new-page, new-section, commit (5 tools) |
| `tools/search-and-index.md` | tools | plan | search, list, ingest, graph, index-rebuild, index-status (6 tools) |
| `engine/index-management.md` | engine | plan | Tantivy schema, field mapping, staleness, versioning, rebuild |
| `engine/graph.md` | engine | plan | Petgraph: typed nodes, labeled edges, rendering |
| `engine/ingest-pipeline.md` | engine | plan | Validate → alias → index → commit flow |
| `engine/server.md` | engine | plan | Transports (stdio, SSE, ACP), multi-wiki, resilience |
| `integrations/mcp-clients.md` | integrations | plan | Cursor, VS Code, Windsurf, generic MCP config |
| `integrations/acp-transport.md` | integrations | plan | ACP for Zed / VS Code agent panel |
| `features.md` | root | plan | Canonical feature inventory with roadmap phase |

Update the status as you write each spec. When all specs are `ready`,
the rationalization is complete.

**Part 2 — Section index.** One table per section (`model/`, `tools/`,
`engine/`, `integrations/`) with file name (linked) and one-line
description. Same as the status table but without the status column —
this is the permanent index for readers.

Keep both parts. Part 1 is the progress tracker (remove it when all
specs are ready). Part 2 is the permanent navigation.

### features.md

Canonical feature inventory. One line per feature, link to the spec
that defines it, mark by roadmap phase (0–4). Remove features that
moved to skills (lint, crystallize, session bootstrap, backlink
quality, instruct).

### model/repository-layout.md

Source: `docs/overview.md` §The Wiki Repository +
archive `core/repository-layout.md`.

Cover:
- Repo structure (`wiki.toml`, `schemas/`, `inbox/`, `raw/`, `wiki/`)
- Wiki root vs repo root
- Slug resolution (flat file vs bundle)
- No `schema.md` — `wiki.toml` is the config
- Index storage (`~/.llm-wiki/indexes/<name>/`)

### model/page-content.md

Source: archive `core/page-content.md` (non-frontmatter parts).

Cover:
- Flat page vs bundle page
- Body conventions (Markdown, `[[wiki-links]]`, asset references)
- Section index pages

Do NOT repeat frontmatter fields — reference `model/frontmatter.md`.

### model/frontmatter.md

Source: `docs/focused-llm-wiki-design.md` §3 +
archive `core/frontmatter-authoring.md`.

This is THE frontmatter reference. Cover:
- Required fields (`title`, `summary`, `read_when`, `status`, `type`,
  `last_updated`)
- Optional fields (`tldr`, `tags`, `sources`, `concepts`, `confidence`,
  `claims`, `owner`, `superseded_by`)
- Field-by-field guide with conventions per type
- Per-type templates (concept, paper, article, query-result, skill)
- The accumulation contract (read before write, preserve list values)
- Common mistakes

### model/epistemic-model.md

Source: `docs/overview.md` §The Epistemic Model +
archive `core/epistemic-model.md`.

Cover:
- Knowledge types vs source types vs query-results
- Why the `type` field carries epistemic distinctions
- Provenance and auditability

Keep it concise — the type taxonomy detail belongs in
`model/type-system.md`.

### model/type-system.md

Source: `docs/type-specific-frontmatter.md` (the whole document).

Cover:
- `wiki.toml` `[types.*]` registry with `schema` + `description`
- JSON Schema Draft 2020-12 per type in `schemas/`
- `x-index-aliases` — field aliasing for uniform indexing
- `x-graph-edges` — typed directed edges with relation labels
- Default types (15 types in 4 categories)
- Custom types (add schema + register in `wiki.toml`)
- Canonical index fields table
- Backward compatibility

### tools/overview.md

Source: `docs/focused-llm-wiki-design.md` §2.

Cover:
- Design principle (stateful access only)
- The 16 tools in three groups (table)
- Global flags (`--wiki <name>`)
- What was removed and why (lint, instruct, context, ask, index-check)

Do NOT repeat individual tool parameters — those go in the group files.

### tools/space-management.md

Source: archive `commands/init.md`, `commands/spaces.md`,
`commands/configuration.md`.

Cover 5 tools: `wiki_init`, `wiki_spaces_list`, `wiki_spaces_remove`,
`wiki_spaces_set_default`, `wiki_config`.

For each tool:
- Description (one line)
- CLI interface with flags
- MCP tool parameters (what it accepts)
- Return type (what it returns)
- Examples

### tools/content-operations.md

Source: archive `commands/read.md`, `commands/page-creation.md`,
`commands/commit.md`.

Cover 5 tools: `wiki_read`, `wiki_write`, `wiki_new_page`,
`wiki_new_section`, `wiki_commit`.

Same structure as space-management.

Note: `wiki_read` surfaces a supersession notice when `superseded_by`
is set.

### tools/search-and-index.md

Source: `docs/focused-llm-wiki-design.md` §3 + §7 +
archive `commands/search.md`, `commands/list.md`, `commands/graph.md`,
`commands/index.md`, `commands/serve.md`.

Cover 6 tools: `wiki_search`, `wiki_list`, `wiki_ingest`,
`wiki_graph`, `wiki_index_rebuild`, `wiki_index_status`.

Same structure as space-management.

Key additions vs archive:
- `wiki_search` has `--type` filter
- `wiki_graph` has `--relation` filter, labeled edges
- `wiki_ingest` does JSON Schema validation + alias resolution

### engine/index-management.md

Source: `docs/focused-llm-wiki-design.md` §3 +
`docs/type-specific-frontmatter.md` §5–§6 +
archive `core/index-management.md`, `core/index-integrity.md`.

This is the most under-specified area. The design docs define *what*
gets indexed and *how aliases work*, but not the tantivy schema itself
or how it evolves. This spec must fully define both.

Cover:

#### Tantivy schema definition

- The fixed set of tantivy fields, their types, and their options:

  | Tantivy field | Type | Options | Source |
  |--------------|------|---------|--------|
  | `slug` | `STRING` | `STORED`, not indexed | Path-derived |
  | `uri` | `STRING` | `STORED`, not indexed | Computed |
  | `title` | `TEXT` | `STORED`, tokenized | Frontmatter or alias |
  | `summary` | `TEXT` | `STORED`, tokenized | Frontmatter or alias |
  | `read_when` | `TEXT` | tokenized | Frontmatter or alias (each entry joined) |
  | `tldr` | `TEXT` | tokenized | Frontmatter |
  | `body` | `TEXT` | tokenized | Markdown body |
  | `type` | `STRING` | `STORED`, `FAST` | Frontmatter |
  | `status` | `STRING` | `STORED`, `FAST` | Frontmatter |
  | `tags` | `STRING` | multi-valued, `FAST` | Frontmatter |
  | `sources` | `STRING` | multi-valued | Frontmatter |
  | `concepts` | `STRING` | multi-valued | Frontmatter |
  | `superseded_by` | `STRING` | `STORED` | Frontmatter |
  | `owner` | `STRING` | `STORED`, `FAST` | Frontmatter |
  | `confidence` | `STRING` | `STORED`, `FAST` | Frontmatter |
  | `last_updated` | `DATE` | `STORED`, `FAST` | Frontmatter |
  | `extra_text` | `TEXT` | tokenized | Unrecognized string fields |
  | `extra_keywords` | `STRING` | multi-valued | Unrecognized list-of-string fields |
  | `edges_json` | `STRING` | `STORED` | Serialized edge data from `x-graph-edges` |

  Specify the exact tantivy field options (`STORED`, `FAST`,
  `INDEXED`, tokenizer name) for each field.

#### Alias resolution

- How `x-index-aliases` maps source fields to canonical tantivy fields
  at ingest time
- Precedence: if both alias source and canonical field exist, canonical
  wins
- Aliases are resolved before indexing — tantivy never sees the
  original field names

#### Unrecognized field handling

- String values → concatenated into `extra_text` (tokenized, BM25
  searchable)
- List-of-string values → each entry added to `extra_keywords`
  (keyword, exact match)
- Objects, numbers, booleans → stored in a JSON blob field but not
  searchable
- The mechanism: after alias resolution, iterate remaining frontmatter
  fields not in the canonical set, classify by YAML type, index
  accordingly

#### Graph edge storage

- How `x-graph-edges` data is stored in the index
- Option A: each edge field (`sources`, `concepts`, etc.) is already
  indexed as keywords — the relation label and target types are looked
  up from the schema at graph-build time, not stored in the index
- Option B: an `edges_json` stored field contains the pre-computed
  edge list with relation labels, so graph building doesn't need to
  re-read schemas
- Decide and specify which option. Document the tradeoff (A is simpler
  but slower graph builds; B is faster but duplicates schema info in
  the index)

#### Schema versioning and migration

- The tantivy schema is fixed — it doesn't change when the wiki owner
  adds custom types or fields. Custom fields go into `extra_text` /
  `extra_keywords`.
- If the engine adds a new canonical field in a future release, the
  index schema version changes
- `state.toml` in the index directory stores:
  - `schema_version` — engine's index schema version
  - `indexed_commit` — last git commit that was indexed
  - `page_count` — number of indexed pages
  - `built_at` — ISO timestamp
- On startup, if `schema_version` doesn't match the engine's current
  version → auto-rebuild required
- `wiki_index_status` reports schema version mismatch
- `wiki_index_rebuild` always rebuilds from scratch with the current
  schema

#### Staleness detection

- Compare `indexed_commit` in `state.toml` against `git rev-parse HEAD`
- If they differ, the index is stale
- Stale index: search/list still work (they return what's indexed) but
  results may be incomplete
- `wiki_index_status` reports staleness
- Configurable auto-rebuild on stale (default: off)

#### Corruption recovery

- If tantivy files are corrupted (missing, truncated, unreadable),
  the engine detects this on first query
- Recovery: delete the index directory, rebuild from committed files
- `wiki_index_rebuild` is always safe — it creates a fresh index

#### Relationship between JSON Schema and tantivy schema

- JSON Schema (in `schemas/`) defines **validation** — what fields
  are required, what values are valid
- Tantivy schema (in the engine) defines **indexing** — what fields
  exist in the search index, how they're tokenized
- `x-index-aliases` bridges the two — it tells the engine which
  JSON Schema field maps to which tantivy field
- `x-graph-edges` bridges the two — it tells the engine which
  JSON Schema field is a graph edge and what relation it carries
- The tantivy schema is fixed and engine-defined. The JSON Schemas
  are wiki-defined and can vary per type. The aliases and edge
  declarations are the glue.

### engine/graph.md

Source: `docs/type-specific-frontmatter.md` §11 +
`docs/focused-llm-wiki-design.md` §2 (wiki_graph).

Cover:
- Petgraph with typed nodes and labeled edges
- Edge sources: frontmatter slug fields + body `[[wiki-links]]`
- `x-graph-edges` declarations per type schema
- Relation labels (`fed-by`, `depends-on`, `informs`, `superseded-by`,
  `links-to`)
- Target type constraints and ingest warnings
- How the graph is built from the tantivy index (reference
  `engine/index-management.md` for how edges are stored)
- Rendering: Mermaid and DOT output with filters
  (`--type`, `--relation`, `--root`, `--depth`)
- Graph metrics the engine can compute (orphan nodes, hub pages,
  missing connections)

### engine/ingest-pipeline.md

Source: `docs/focused-llm-wiki-design.md` §3 (wiki_ingest) +
`docs/type-specific-frontmatter.md` §7 +
archive `pipelines/ingest.md`, `pipelines/asset-ingest.md`.

Cover:
- The full flow: parse YAML → read `type` → lookup schema in
  `wiki.toml` → load JSON Schema → validate → resolve aliases →
  resolve graph edges → index in tantivy → commit to git
- What the engine validates vs what the author writes
- Frontmatter defaults for missing fields (`status: active`,
  `type: page`, `last_updated: today`)
- How alias resolution feeds into indexing (reference
  `engine/index-management.md` for the tantivy field mapping)
- How `x-graph-edges` are processed at ingest time (reference
  `engine/index-management.md` for edge storage decision)
- Auto-commit vs explicit commit (`ingest.auto_commit` config)
- Asset handling (co-located in bundles, non-`.md` files)
- IngestReport return type (fields table, not Rust struct)
- Dry-run mode
- Error handling: what causes rejection vs warning

### engine/server.md

Source: archive `commands/serve.md`, `core/server-resilience.md`,
`core/logging.md`.

Cover:
- Three transports: stdio (always), SSE (opt-in), ACP (opt-in)
- Multi-wiki: all registered wikis mounted at startup
- MCP resource namespacing (`wiki://<name>/<slug>`)
- Startup sequence
- Failure isolation between transports
- Config defaults

### integrations/mcp-clients.md

Source: archive `integrations/mcp-clients.md`.

Cover:
- Configuration for Cursor, VS Code, Windsurf
- Generic MCP client config
- The `llm-wiki-skills` plugin (reference, not repeat)

Update: reference `llm-wiki-skills` repo instead of the old
`claude-plugin.md` content.

### integrations/acp-transport.md

Source: archive `integrations/acp-transport.md`.

Cover:
- ACP transport for Zed / VS Code agent panel
- Same 16 tools as MCP
- Session-oriented, streaming

## Process

### Step 1 — Set up

```bash
# Archive is already in place
ls docs/specifications/archive/

# Create new directories
mkdir -p docs/specifications/{model,tools,engine,integrations}
```

### Step 2 — Write specs in order

Write in this order (each file may reference earlier ones):

1. `README.md`
2. `model/repository-layout.md`
3. `model/page-content.md`
4. `model/frontmatter.md`
5. `model/epistemic-model.md`
6. `model/type-system.md`
7. `tools/overview.md`
8. `tools/space-management.md`
9. `tools/content-operations.md`
10. `tools/search-and-index.md`
11. `engine/index-management.md`
12. `engine/graph.md`
13. `engine/ingest-pipeline.md`
14. `engine/server.md`
15. `integrations/mcp-clients.md`
16. `integrations/acp-transport.md`
17. `features.md`

Write `features.md` last — it links to all other specs.

### Step 3 — Verify

```bash
# No dead references
grep -rn "schema\.md\|llm-wiki instruct\|wiki_lint\|wiki_context\|wiki_ask\|wiki_index_check\|source-summary\|integrate_file\|integrate_folder\|src/instructions" docs/specifications/ --include="*.md" | grep -v archive/

# All new files exist
ls docs/specifications/model/
ls docs/specifications/tools/
ls docs/specifications/engine/
ls docs/specifications/integrations/

# Every file has frontmatter
for f in $(find docs/specifications -name "*.md" -not -path "*/archive/*"); do
  head -1 "$f" | grep -q "^---" || echo "MISSING FRONTMATTER: $f"
done
```

### Step 4 — Clean up old directories

Once all specs are written and verified, remove the empty old
directories:

```bash
rmdir docs/specifications/core docs/specifications/commands \
      docs/specifications/pipelines docs/specifications/llm 2>/dev/null
```
