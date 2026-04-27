---
title: "Cross-Wiki Links"
summary: "Make wiki:// URIs first-class link targets; resolve them at graph build time; wiki_graph --cross-wiki for unified multi-wiki graph."
status: proposed
last_updated: "2026-04-27"
---

# Cross-Wiki Links

## Problem

Links between pages are bare slugs — `sources: [concepts/moe]`,
`[[concepts/moe]]`. These resolve within a single wiki only. There is no
way for a page in wiki A to link to a page in wiki B.

`wiki://` URIs exist (`wiki://research/concepts/moe`) but are used only
for display and `wiki_content_read` resolution. They are not recognized as
link targets by the graph builder or the index. A page that writes
`sources: [wiki://other/concepts/foo]` gets a broken edge — the graph
sees it as a literal slug, finds no matching node, and silently drops it.

## Current architecture

**Link extraction (`links.rs`):** `extract_links` collects slugs from
frontmatter fields and body `[[wikilinks]]`. All values are treated as bare
slugs — no URI parsing.

**Indexing (`index_manager.rs`):** frontmatter edge fields and body
`[[wikilinks]]` are indexed as raw strings — no URI normalization.

**Graph (`graph.rs`):** `build_graph` reads from a single wiki's tantivy
index. It builds a `slug → NodeIndex` map and resolves edges by slug
lookup. Cross-wiki URIs don't match any slug and are silently dropped.

**URI resolution (`slug.rs`):** `WikiUri::parse` handles `wiki://name/slug`
and `WikiUri::resolve` looks up the wiki name in the global config. This
works for `wiki_content_read` but is not used by the link or graph pipelines.

## Goals

- `wiki://` URIs are valid link targets in frontmatter edge fields and body
  wikilinks, stored as-is in the index.
- Cross-wiki edges are resolved at graph build time from the set of currently
  mounted wikis — no index schema change.
- `wiki_graph(cross_wiki: true)` builds a unified graph across all mounted
  wikis with cross-wiki edges fully resolved.
- Single-wiki `wiki_graph` shows cross-wiki targets as external nodes
  (visually distinct, minimal metadata).
- Lint detects cross-wiki links pointing to unmounted wikis.

## Proposed behavior

### Link syntax

```yaml
sources:
  - concepts/local-concept        # same wiki (unchanged)
  - wiki://other/concepts/foo     # cross-wiki
```

```markdown
See [[wiki://other/concepts/foo]] for details.
```

### Single-wiki graph

Cross-wiki targets rendered as external nodes — visually distinct, no
metadata (title, type) since they are not in the local index. Open
question: show the slug as placeholder title, or omit the node entirely?
Default: show with slug as label, `:::external` CSS class.

```
graph LR
  research/concepts/moe["MoE"]:::concept
  wiki://notes/concepts/foo["notes/concepts/foo"]:::external
  research/concepts/moe -->|fed-by| wiki://notes/concepts/foo
```

### Unified graph (`cross_wiki: true`)

All mounted wikis contribute nodes. Cross-wiki edges fully resolved. Nodes
prefixed with wiki name for disambiguation.

```
graph LR
  research/concepts/moe["MoE"]:::concept
  notes/concepts/foo["foo"]:::concept
  research/concepts/moe -->|fed-by| notes/concepts/foo
```

### Content read

`wiki_content_read(uri: "wiki://other/concepts/foo")` already works.
No change needed.

### Search

Cross-wiki links affect graph topology but not BM25 ranking. No change
needed for `wiki_search`.

## Interaction with existing features

- **Hot reload**: when a wiki is mounted/unmounted, cross-wiki edges
  become resolvable/unresolvable. The graph reflects the current set of
  mounted wikis — no persistent state needed.
- **Cross-wiki search**: `wiki_search(cross_wiki: true)` already merges
  results. No change.
- **Facets**: no impact.
- **Performance**: building a unified graph holds multiple searchers open
  simultaneously. Fine for 2–3 wikis; worth noting as a constraint for
  large deployments.

## Open questions

- Should external nodes in single-wiki graph show a placeholder slug as
  title or be omitted entirely? Current proposal: show with slug label.

## Tasks

### Spec docs

- [ ] `docs/specifications/engine/graph.md` — add "Cross-wiki edges"
  section: URI parsing at graph build time, external node rendering,
  `cross_wiki` flag.
- [ ] `docs/specifications/tools/graph.md` — add `cross_wiki` parameter,
  document external node styling and unified graph behavior.
- [ ] `docs/specifications/model/page-content.md` — document `wiki://`
  URI as valid link target in frontmatter edge fields and body wikilinks.

### Link extraction — `src/links.rs`

- [ ] Update `extract_links` and `extract_wikilinks` to recognize
  `wiki://name/slug` and preserve the full URI rather than treating it as
  a bare slug.
- [ ] Add `ParsedLink` enum: `Local(slug)` vs `CrossWiki { wiki, slug }`
  for downstream consumers.

### Graph — `src/graph.rs`

- [ ] Add `build_graph_cross_wiki` that takes multiple
  `(wiki_name, Searcher, IndexSchema, TypeRegistry)` tuples; builds a
  unified `wiki_name/slug → NodeIndex` map.
- [ ] Tag nodes as `local` vs `external` for rendering.
- [ ] Mermaid renderer: add `classDef external` styling; prefix external
  node IDs with wiki name; show slug as placeholder label.
- [ ] DOT renderer: add `wiki` attribute to external nodes; use `subgraph`
  clusters per wiki.

### CLI / MCP — `src/cli.rs`, `src/tools.rs`, `src/ops/graph.rs`

- [ ] Add `--cross-wiki` flag to `wiki_graph` CLI command.
- [ ] Add `cross_wiki: bool` param to `wiki_graph` MCP tool.
- [ ] When `cross_wiki` is set, call `build_graph_cross_wiki` with all
  mounted wikis.

### Lint — `src/ops/lint.rs`

- [ ] Add `broken-cross-wiki-link` rule: detect `wiki://` URIs pointing
  to wikis not in the config; report as `Warning` (unmounted ≠ wrong).

### Skills — `llm-wiki-skills/`

- [ ] `skills/graph/SKILL.md` — document `cross_wiki` param, external
  node rendering, and the unified multi-wiki graph use case.
- [ ] `skills/content/SKILL.md` — document `wiki://` URI as valid link
  target in frontmatter edge fields and body wikilinks.
- [ ] `skills/lint/SKILL.md` — add broken cross-wiki link detection to
  the rule set (after engine ships the rule).

### Tests

- [ ] Local link resolves in single-wiki graph (no regression).
- [ ] `wiki://other/slug` in frontmatter creates edge in unified graph.
- [ ] `[[wiki://other/slug]]` in body creates edge in unified graph.
- [ ] Cross-wiki link to unmounted wiki is silently dropped in graph.
- [ ] Single-wiki graph renders cross-wiki target as external node.
- [ ] Lint flags `wiki://unmounted/slug` as broken cross-wiki link.
- [ ] Existing test suite passes unchanged.
