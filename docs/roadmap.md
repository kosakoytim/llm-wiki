---
title: "Roadmap"
summary: "Development roadmap for llm-wiki — from focused engine to skill registry, organized in phases with clear deliverables."
status: draft
last_updated: "2025-07-17"
---

# Roadmap

Three deliverables, four phases. The engine (`llm-wiki`), the skills
(`llm-wiki-skills`), and the type schemas (`schemas/`) evolve together
but release independently.

---

## Phase 0 — Specification Rationalization

Before building, clean up the specification corpus. The current docs
have grown organically and contain repetition, code-level detail that
belongs in implementation, and references to designs that have been
superseded by this roadmap.

### Goals

- **Single source of truth** — each concept defined once, referenced
  everywhere else
- **Design over code** — specs describe *what* and *why*, not *how* in
  Rust. Implementation details belong in code comments and module docs.
- **Aligned with roadmap** — specs reflect the focused engine (16
  tools), type system (JSON Schema + `wiki.toml`), typed graph
  (`x-graph-edges`), skill registry, and `llm-wiki-skills` repo
- **No dead references** — remove references to `schema.md`,
  `llm-wiki instruct`, `wiki_lint` tool, `wiki_context`, and other
  removed features

### Deliverables

- [ ] Audit all files in `docs/specifications/` against the focused
  design doc and this roadmap
- [ ] Remove or consolidate duplicate content (frontmatter fields
  defined in multiple places, tool descriptions repeated across files)
- [ ] Move code-level detail (Rust signatures, struct definitions) out
  of specs into implementation notes or remove entirely
- [ ] Update all `schema.md` references to `wiki.toml` type registry
- [ ] Update all `llm-wiki instruct` references to `llm-wiki-skills`
- [ ] Remove `wiki_lint`, `wiki_context`, `wiki_ask` from tool lists
- [ ] Add `--type` filter to search spec
- [ ] Add `owner` and `superseded_by` to all relevant specs
- [ ] Cross-reference the design docs (`focused-llm-wiki-design.md`,
  `type-specific-frontmatter.md`, `roadmap.md`) from specs instead of
  repeating their content
- [ ] Archive superseded docs in `docs/archive/`
- [ ] Update `docs/specifications/features.md` as the canonical feature
  inventory — one line per feature, link to the spec that defines it

See [`docs/prompts/rationalize-specs.md`](prompts/rationalize-specs.md)
for the session prompt to execute this work.

---

## Phase 1 — Focused Engine

Strip the engine to its core: 16 MCP tools, no embedded LLM prompts,
no lint command, no instruct command. The engine manages files, git,
search, and the concept graph. Nothing else.

### Engine (llm-wiki)

- [ ] Remove `llm-wiki instruct` from the binary
- [ ] Remove `llm-wiki lint` CLI command (moves to skill)
- [ ] Implement the 16 MCP/ACP tools:
  - Space management: `wiki_init`, `wiki_spaces_list`,
    `wiki_spaces_remove`, `wiki_spaces_set_default`, `wiki_config`
  - Content: `wiki_read`, `wiki_write`, `wiki_new_page`,
    `wiki_new_section`, `wiki_commit`
  - Search & index: `wiki_search`, `wiki_list`, `wiki_ingest`,
    `wiki_graph`, `wiki_index_rebuild`, `wiki_index_status`
- [ ] Add `--type` filter to `wiki_search` (tantivy keyword filter
  combined with BM25 query)
- [ ] Add `owner` and `superseded_by` to the base frontmatter schema
- [ ] `wiki_read` surfaces supersession notice when `superseded_by`
  is set
- [ ] Index `owner` as keyword, `superseded_by` as keyword + graph edge
- [ ] Fold `wiki_index_check` into `wiki_index_status`

### Skills (llm-wiki-skills)

- [ ] Create the `llm-wiki-skills` git repository
- [ ] Set up Claude Code plugin structure (`.claude-plugin/plugin.json`,
  `.mcp.json`, `settings.json`)
- [ ] Write the 8 initial skills:
  - `bootstrap` — session orientation
  - `ingest` — source processing workflow
  - `crystallize` — distil session into wiki pages
  - `research` — search → read → synthesize
  - `lint` — structural audit + fix
  - `graph` — generate and interpret concept graph
  - `frontmatter` — frontmatter authoring reference
  - `skill` — find and activate wiki skills
- [ ] Test with `claude --plugin-dir ./llm-wiki-skills`
- [ ] Publish to Claude marketplace

### Milestone

Engine binary with 16 tools. Skills repo with 8 skills. Claude Code
plugin installable. `llm-wiki serve` + plugin = working system.

---

## Phase 2 — Type System

Replace the hardcoded frontmatter schema with JSON Schema validation
per type. Move the type registry from `schema.md` to `wiki.toml`.
Eliminate `schema.md`.

### Engine (llm-wiki)

- [ ] Add `[types.*]` section to `wiki.toml` with `schema` +
  `description` fields
- [ ] Add `schemas/` directory to wiki repo layout
- [ ] Ship default JSON Schema files:
  - `base.json` — shared fields
  - `concept.json` — knowledge pages (extends base)
  - `paper.json` — source pages (extends base, shared by all source
    types)
  - `skill.json` — agent skills (standalone, uses `x-index-aliases`)
  - `doc.json` — reference documents (extends base)
  - `section.json` — section index pages
- [ ] Implement JSON Schema validation on `wiki_ingest` (Rust:
  `jsonschema` crate)
- [ ] Implement `x-index-aliases` — resolve field aliases at ingest
  time before indexing
- [ ] `wiki_init` generates default `wiki.toml` with `[types.*]`
  entries and `schemas/` directory
- [ ] `wiki_config list` returns type names + descriptions
- [ ] Remove `schema.md` from `wiki_init` template
- [ ] Migrate existing wikis: ignore `schema.md`, fall back to
  built-in base schema when no `[types.*]` entries exist

### Default types

Register these in `wiki.toml` on `wiki_init`:

| Type | Schema | Description |
|------|--------|-------------|
| `default` | `base.json` | Fallback for unrecognized types |
| `concept` | `concept.json` | Synthesized knowledge |
| `query-result` | `concept.json` | Saved conclusion |
| `section` | `section.json` | Section index |
| `paper` | `paper.json` | Academic source |
| `article` | `paper.json` | Editorial source |
| `documentation` | `paper.json` | Reference source |
| `clipping` | `paper.json` | Web capture |
| `transcript` | `paper.json` | Spoken source |
| `note` | `paper.json` | Informal source |
| `data` | `paper.json` | Structured data |
| `book-chapter` | `paper.json` | Published excerpt |
| `thread` | `paper.json` | Discussion archive |
| `skill` | `skill.json` | Agent skill |
| `doc` | `doc.json` | Reference document |

### Skills (llm-wiki-skills)

- [ ] Update `frontmatter` skill with type-specific guidance
- [ ] Add `references/type-taxonomy.md` with per-type templates
- [ ] Update `bootstrap` skill to read types from `wiki_config`
- [ ] Update `ingest` skill to reference type validation

### Milestone

Type-specific JSON Schema validation on ingest. Field aliasing for
skill and doc pages. `schema.md` eliminated. Custom types addable via
`wiki.toml` + schema file.

---

## Phase 3 — Typed Graph

Add `x-graph-edges` to type schemas. The concept graph gets typed
nodes and labeled edges. `wiki_graph` can filter by relation type.

### Engine (llm-wiki)

- [ ] Implement `x-graph-edges` parsing from JSON Schema files
- [ ] At ingest: read edge declarations, index edges with relation
  labels
- [ ] At graph build: petgraph nodes get `type` label, edges get
  `relation` label
- [ ] `wiki_graph --relation <label>` — filter edges by relation
- [ ] `wiki_graph` Mermaid output includes relation labels on edges
- [ ] `wiki_graph` DOT output includes relation labels on edges
- [ ] Warn on ingest when edge target page has wrong type (per
  `target_types` constraint)

### Default edge declarations

Add to shipped schemas:

| Schema | Field | Relation | Target types |
|--------|-------|----------|-------------|
| `concept.json` | `sources` | `fed-by` | All source types |
| `concept.json` | `concepts` | `depends-on` | `concept` |
| `concept.json` | `superseded_by` | `superseded-by` | Any |
| `paper.json` | `concepts` | `informs` | `concept` |
| `paper.json` | `superseded_by` | `superseded-by` | Any |
| `skill.json` | `superseded_by` | `superseded-by` | Any |
| `doc.json` | `requires_skills` | `requires` | `skill` |
| `doc.json` | `documents_skill` | `documents` | `skill` |
| `doc.json` | `superseded_by` | `superseded-by` | Any |

Body `[[wiki-links]]` get a generic `links-to` relation.

### Skills (llm-wiki-skills)

- [ ] Update `graph` skill with relation-aware instructions
- [ ] Update `lint` skill to detect type constraint violations

### Milestone

Labeled graph edges. Relation-filtered graph output. Type constraint
warnings on ingest. Graph visualization with semantic edge labels.

---

## Phase 4 — Skill Registry

The wiki becomes a full skill registry. Skills stored as `type: skill`
pages are discoverable, activatable, and relate to knowledge pages
through the typed graph.

### Engine (llm-wiki)

- [ ] Verify `wiki_search --type skill` works end-to-end with
  `x-index-aliases` (name → title, description → summary)
- [ ] Verify `wiki_list --type skill` returns skill-specific metadata
- [ ] Verify `wiki_graph` renders skill → concept edges correctly
- [ ] Cross-wiki skill discovery: `wiki_search --type skill --all`

### Skills (llm-wiki-skills)

- [ ] Finalize `skill` skill — instructions for finding, reading, and
  activating wiki skills
- [ ] Document the skill authoring workflow (write → ingest → search →
  activate)
- [ ] Add example wiki skills to the llm-wiki-skills README

### Documentation

- [ ] Publish skill authoring guide (how to write `type: skill` pages)
- [ ] Publish type authoring guide (how to add custom types with
  JSON Schema)
- [ ] Update README with skill registry documentation

### Milestone

Wiki as skill registry. Agents discover skills via search, read them
via `wiki_read`, activate them by injecting the body into context.
Skills reference knowledge pages through the typed graph.

---

## Future

Ideas that don't fit in the four phases but are worth tracking:

- **`wiki_diff`** — show changes between two commits for a page
- **`wiki_history`** — git log for a specific page
- **`wiki_search` facets** — return type/status/tag distributions
  alongside results
- **`wiki_export`** — generate static site, PDF, or EPUB from wiki
- **Cross-wiki links** — `wiki://<name>/<slug>` links resolved in
  graph and search
- **Webhook on ingest** — notify external systems when pages change
- **`wiki_watch`** — filesystem watcher that auto-ingests on save
- **Skill composition** — wiki skills that reference and extend other
  wiki skills with a formal `extends` field
- **Confidence propagation** — compute concept confidence from source
  confidences via graph traversal

---

## Related Project: llm-wiki-hugo-cms

A separate project that turns a wiki space into a
[Hugo](https://gohugo.io/) site. The wiki is the CMS — Hugo is the
renderer.

### Concept

The wiki stores structured knowledge with rich frontmatter. Hugo needs
Markdown files with frontmatter in `content/`. The bridge is a build
step that reads wiki pages and produces Hugo-compatible content.

```
wiki space                    llm-wiki-hugo-cms              Hugo site
┌──────────┐                  ┌──────────────┐               ┌──────────┐
│ wiki/    │  wiki_read       │ transform    │  hugo build   │ public/  │
│ concepts/│ ──────────────►  │ frontmatter  │ ────────────► │ HTML/CSS │
│ sources/ │  wiki_list       │ resolve URIs │               │ RSS/JSON │
│ skills/  │  wiki_graph      │ generate nav │               │          │
└──────────┘                  └──────────────┘               └──────────┘
```

### What it does

- **Reads** wiki pages via `wiki_list` + `wiki_read` (MCP tools or CLI)
- **Transforms** wiki frontmatter to Hugo frontmatter (map `type` to
  Hugo section, `tags` to Hugo taxonomies, `summary` to Hugo
  `description`, etc.)
- **Resolves** `wiki://` URIs to Hugo-relative URLs
- **Generates** navigation from the wiki's section structure
- **Generates** graph visualizations from `wiki_graph` output
  (Mermaid diagrams embedded in pages)
- **Outputs** Hugo `content/` tree ready for `hugo build`

### What it does not do

- Does not modify the wiki — read-only
- Does not replace Hugo — it generates content for Hugo, not HTML
- Does not embed Hugo themes — bring your own theme
- Does not run Hugo — it produces the input, you run `hugo build`

### Repository structure

```
llm-wiki-hugo-cms/
├── src/                      ← Rust or Python transform logic
├── templates/
│   └── archetypes/           ← Hugo archetype templates per wiki type
├── config/
│   └── mapping.toml          ← wiki type → Hugo section/taxonomy mapping
├── README.md
├── CHANGELOG.md
└── LICENSE
```

### Frontmatter mapping

| Wiki field | Hugo field | Notes |
|------------|-----------|-------|
| `title` | `title` | Direct |
| `summary` | `description` | Hugo uses `description` for meta tags |
| `tags` | `tags` | Hugo taxonomy |
| `type` | Hugo section or `type` | Configurable mapping |
| `status` | `draft` | `draft`/`stub` → `draft: true`; `active` → `draft: false` |
| `last_updated` | `lastmod` | Hugo's last-modified date |
| `owner` | `authors` | Hugo taxonomy |
| `read_when` | Dropped or custom param | Not meaningful for a website |
| `sources` | Custom param or related content | Hugo related content feature |
| `concepts` | Custom param or related content | Hugo related content feature |
| `superseded_by` | Alias or redirect | Hugo alias/redirect mechanism |

### Usage

```bash
# Generate Hugo content from a wiki space
llm-wiki-hugo-cms build --wiki research --output ./hugo-site/content/

# Or via MCP (the tool reads from the running llm-wiki server)
llm-wiki-hugo-cms build --mcp --output ./hugo-site/content/

# Then build the Hugo site
cd hugo-site && hugo build
```

### Why a separate project

- The wiki engine is not a static site generator — it's a knowledge
  base engine
- Hugo has its own ecosystem (themes, shortcodes, taxonomies) that
  doesn't belong in the wiki engine
- The transform is a one-way read-only operation — it doesn't need
  write access to the wiki
- Different release cycle — Hugo themes and mappings evolve
  independently from the engine
