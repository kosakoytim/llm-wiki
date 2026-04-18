# Plan Phase 3 Implementation

## Context

Phase 2 is complete. The engine has a dynamic type registry driven by
JSON Schema files and `wiki.toml` `[types.*]` entries. Frontmatter is
validated per type, `x-index-aliases` resolve field names at ingest
time, and schema change detection triggers index rebuilds.

Phase 3 adds `x-graph-edges` — typed nodes and labeled edges in the
concept graph. The graph builder reads edge declarations from type
schemas and renders relation labels in Mermaid and DOT output.

## Read first

Read these in order before planning:

1. `docs/roadmap.md` — Phase 3 deliverables
2. `docs/specifications/model/type-system.md` — `x-graph-edges` format
3. `docs/specifications/engine/graph.md` — graph construction, edge sources, filtering, rendering
4. `docs/specifications/engine/index-management.md` — how edges affect schema_hash

Then read the per-type edge declarations:
- `docs/specifications/model/types/concept.md` — `sources` → fed-by, `concepts` → depends-on
- `docs/specifications/model/types/source.md` — `sources` → cites, `concepts` → informs
- `docs/specifications/model/types/skill.md` — `document_refs` → documented-by
- `docs/specifications/model/types/doc.md` — `sources` → informed-by
- All types: `superseded_by` → superseded-by

Then read the implementation docs:
- `docs/implementation/type-registry.md` — `EdgeDecl` struct, edge extraction
- `docs/implementation/graph-builder.md` — petgraph from index, LabeledEdge, rendering
- `docs/implementation/tantivy.md` — how edge fields are indexed
- `docs/implementation/index-manager.md` — partial rebuild when edges change

And the current source files that will be modified:
- `src/type_registry.rs` — add `x-graph-edges` parsing to RegisteredType
- `src/index_schema.rs` — edge fields already indexed as keywords (from Phase 2)
- `src/graph.rs` — replace untyped `()` edges with `LabeledEdge`, read edge declarations from registry
- `src/ingest.rs` — optionally warn when edge target has wrong type
- `src/search.rs` — no changes expected (edge fields already indexed)
- `src/mcp/tools.rs` — `wiki_graph` adds `--relation` parameter

## Your Task

Update `docs/roadmap.md` Phase 3 with a detailed implementation plan.
Break it into ordered steps where each step produces a compilable,
testable increment. No step should be larger than one session of work.

## Constraints

### Build order matters

Edge support touches the type registry, graph builder, and renderers:

```
x-graph-edges parsing in type_registry.rs
  -> EdgeDecl stored in RegisteredType
  -> graph.rs reads EdgeDecl from registry
  -> graph.rs builds LabeledEdge instead of ()
  -> render_mermaid / render_dot include relation labels
  -> GraphFilter adds relation field
  -> ingest.rs warns on type constraint violations
  -> schema_hash includes edge declarations (already from Phase 2)
```

### Each step must

- Compile (`cargo check`)
- Have tests (`cargo test`)
- Be committable with a meaningful message

### What Phase 3 implements

- `x-graph-edges` parsing from JSON Schema files
- `EdgeDecl` struct: field, relation, direction, target_types
- At ingest: read edge declarations, index edges with relation labels
  (edge fields are already indexed as keyword slugs from Phase 2)
- At graph build: petgraph nodes get `type` label, edges get
  `relation` label from `x-graph-edges`
- `wiki_graph --relation <label>` — filter edges by relation
- Mermaid output: relation labels on edges, node titles as labels,
  type as CSS class with color coding
- DOT output: relation labels on edges, node labels and type attributes
- Warn on ingest when edge target has wrong type (per `target_types`)
- Body `[[wiki-links]]` get a generic `links-to` relation

### Default edge declarations to ship

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

### What Phase 3 does NOT implement

- Skill registry features (Phase 4)
- Persistent graph index (future)
- Graph queries beyond rendering (future)
- Hot reload / file watcher (future)

### Key design notes

- The same field name (`sources`) can have different relations depending
  on the page type. The graph builder reads `x-graph-edges` from the
  type's schema to determine the correct relation label.
- Phase 1 hardcoded relations (`sources` → "fed-by", `concepts` →
  "depends-on", `body_links` → "links-to") are replaced by
  schema-driven relations. The hardcoded fallback remains for wikis
  without `x-graph-edges` declarations.
- `target_types` is advisory — violations produce warnings at ingest
  time, not errors. The edge is still created.
- Edge declarations affect `schema_hash` — changing them triggers
  a rebuild (already handled by Phase 2 change detection).

### What MUST work at the end of Phase 3

- `wiki_graph` renders labeled edges with relation names
- `wiki_graph --relation fed-by` filters to only `fed-by` edges
- `wiki_graph --type concept --relation depends-on` composes filters
- Mermaid output includes `-->|relation|` syntax and `classDef` per type
- DOT output includes `[label="relation"]` on edges
- `wiki_ingest` warns when edge target has wrong type
- Default schemas ship with `x-graph-edges` declarations
- All existing Phase 1 and Phase 2 tests still pass
- New tests for edge parsing, labeled graph construction, relation
  filtering, type constraint warnings

## Output

Update `docs/roadmap.md` Phase 3 with numbered steps. Each step:

```
### Step N: <what>

Modules: <files created or modified>
Pulls from: <Phase 2 code reused or modified>
Tests: <what's tested>
Commit: <message>
```

Keep the existing Phase 4 and Future sections unchanged.
