# Plan Phase 2 Implementation

## Context

Phase 1 is complete. The engine has 15 working tools, hardcoded
field-to-index mapping, and base frontmatter validation (`title`
required, `type` non-empty). The codebase lives in `src/` with
reference material in `code-ref/`.

Phase 2 replaces the hardcoded type registry with a dynamic one
driven by JSON Schema files and `wiki.toml` `[types.*]` entries.

## Read first

Read these in order before planning:

1. `docs/roadmap.md` — Phase 2 deliverables
2. `docs/specifications/model/type-system.md` — full type system spec
3. `docs/specifications/model/wiki-toml.md` — `[types.*]` registry format
4. `docs/specifications/model/types/base.md` — base schema, default fallback
5. `docs/specifications/model/types/concept.md` — concept + query-result
6. `docs/specifications/model/types/source.md` — paper, article, etc.
7. `docs/specifications/model/types/skill.md` — skill with field aliasing
8. `docs/specifications/model/types/doc.md` — doc type
9. `docs/specifications/model/types/section.md` — section type
10. `docs/specifications/engine/index-management.md` — schema change detection, partial rebuild
11. `docs/specifications/engine/ingest-pipeline.md` — validate → alias → index → commit

Then read the implementation docs:
- `docs/implementation/type-registry.md` — SpaceTypeRegistry, validators, caching
- `docs/implementation/tantivy.md` — dynamic schema building, field classification
- `docs/implementation/index-manager.md` — SpaceIndexManager, rebuild, staleness
- `docs/implementation/manager-pattern.md` — detect, refresh, cascade

And the current source files that will be modified:
- `src/type_registry.rs` — currently hardcoded, becomes dynamic
- `src/index_schema.rs` — currently hardcoded fields, becomes derived from registry
- `src/search.rs` — rebuild/update use IndexSchema
- `src/ingest.rs` — validation pipeline
- `src/spaces.rs` — `create()` generates schemas/ and wiki.toml types
- `src/engine.rs` — startup loads type registry, change detection
- `src/config.rs` — WikiConfig needs no `[types.*]` (loaded separately)

## Your Task

Update `docs/roadmap.md` Phase 2 with a detailed implementation plan.
Break it into ordered steps where each step produces a compilable,
testable increment. No step should be larger than one session of work.

## Constraints

### Build order matters

The type system touches multiple modules. Build bottom-up:

```
JSON Schema files (schemas/*.json)
  -> type_registry.rs (load schemas, compile validators, extract aliases/edges)
  -> index_schema.rs (derive tantivy schema from type registry)
  -> search.rs (use dynamic IndexSchema for rebuild/update)
  -> ingest.rs (validate against JSON Schema, apply aliases)
  -> spaces.rs (create() ships default schemas + wiki.toml types)
  -> engine.rs (schema_hash change detection, partial rebuild)
```

### Each step must

- Compile (`cargo check`)
- Have tests (`cargo test`)
- Be committable with a meaningful message

### What Phase 2 implements

- `[types.*]` section in `wiki.toml`
- `schemas/` directory with JSON Schema files per type
- Ship 6 default schemas: `base.json`, `concept.json`, `paper.json`,
  `skill.json`, `doc.json`, `section.json`
- JSON Schema validation on `wiki_ingest` via `jsonschema` crate
- `x-index-aliases` — resolve field aliases at ingest time
  (e.g. skill's `name` → `title`, `description` → `summary`)
- `llm-wiki spaces create` generates default `wiki.toml` with
  `[types.*]` entries and populates `schemas/` directory
- `wiki_config list` returns type names + descriptions
- Schema change detection via `schema_hash` in `state.toml`
- Per-type hashes for partial rebuild decisions

### What Phase 2 does NOT implement

- `x-graph-edges` typed edges (Phase 3)
- Skill registry features (Phase 4)
- Hot reload / file watcher (future)
- Custom tokenizer registration (future)

### Backward compatibility

- Pages without a `type` field default to `type: page`, validated
  against `[types.default]`
- Pages with an unregistered type are validated against `[types.default]`
- Wikis with no `schemas/` directory use a built-in base schema
- No frontmatter rewriting — existing files are untouched
- Existing Phase 1 wikis continue to work (no schemas/ = base validation)

### What MUST work at the end of Phase 2

- `llm-wiki spaces create` generates `schemas/` with 6 JSON Schema files
  and `wiki.toml` with `[types.*]` entries
- `wiki_ingest` validates frontmatter against the type's JSON Schema
- `wiki_ingest` resolves `x-index-aliases` before indexing
  (skill pages with `name`/`description` index as `title`/`summary`)
- `wiki_config list` shows registered types
- Schema change detection triggers rebuild when `wiki.toml` or
  `schemas/` change
- Custom types addable via `wiki.toml` + schema file
- All existing Phase 1 tests still pass
- New tests for schema validation, aliasing, change detection

## Output

Update `docs/roadmap.md` Phase 2 with numbered steps. Each step:

```
### Step N: <what>

Modules: <files created or modified>
Pulls from: <Phase 1 code reused or modified>
Tests: <what's tested>
Commit: <message>
```

Keep the existing Phase 3-4 and Future sections unchanged.
