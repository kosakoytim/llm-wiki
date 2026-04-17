---
title: "Type System"
summary: "What a type is, how types are managed, base schema, field aliasing, and graph edges."
read_when:
  - Understanding how per-type validation works
  - Adding a custom type to a wiki
  - Understanding field aliasing
  - Understanding typed graph edges
status: ready
last_updated: "2025-07-17"
---

# Type System

Every page has a `type` field. The type determines which JSON Schema
validates the frontmatter and how fields are indexed.

Types are defined by the wiki owner in `wiki.toml` and `schemas/`, not
hardcoded in the engine. See [wiki-toml.md](wiki-toml.md) for the type
registry format.

For the epistemic rationale behind types, see
[epistemic-model.md](epistemic-model.md).

## Built-in Types

Registered in `wiki.toml` by `llm-wiki spaces create`:

| Type                                      | Schema         | Description                                                  |
| ----------------------------------------- | -------------- | ------------------------------------------------------------ |
| [base](types/base.md)                     | `base.json`    | Default fallback for unrecognized types                      |
| [concept, query-result](types/concept.md) | `concept.json` | Synthesized knowledge and saved conclusions                  |
| [source types](types/source.md)           | `paper.json`   | What each source claims (paper, article, documentation, ...) |
| [skill](types/skill.md)                   | `skill.json`   | Agent skill with workflow instructions                       |
| [doc](types/doc.md)                       | `doc.json`     | Reference document with agent-foundation frontmatter         |
| [section](types/section.md)               | `section.json` | Section index grouping related pages                         |

## How It Works

1. `wiki.toml` registers types — each type has a description and a
   path to its JSON Schema file
2. On `wiki_ingest`, the engine validates frontmatter against the
   type's JSON Schema
3. Field aliases map type-specific names to canonical index roles

## Field Aliasing — `x-index-aliases`

Different types use different field names for the same role. The engine
maps them to canonical index fields via `x-index-aliases` in the JSON
Schema:

```json
"x-index-aliases": {
  "name": "title",
  "description": "summary",
  "when_to_use": "read_when"
}
```

- **Key** = source field name in frontmatter (e.g., `name`)
- **Value** = canonical index field (e.g., `title`)
- At ingest: if source field exists and canonical field does not, index
  source value under the canonical name
- If both exist, the canonical field wins
- Aliases affect indexing only — the file on disk is never rewritten

Fields not aliased to a canonical field are indexed as generic text.
For the full list of canonical index fields, see
[index-management.md](../engine/index-management.md).

## Typed Graph Edges — `x-graph-edges`

Each type schema declares its outgoing edges:

```json
"x-graph-edges": {
  "sources": {
    "relation": "fed-by",
    "direction": "outgoing",
    "target_types": ["paper", "article", "documentation"]
  },
  "concepts": {
    "relation": "depends-on",
    "direction": "outgoing",
    "target_types": ["concept"]
  }
}
```

| Field          | Required | Description                               |
| -------------- | -------- | ----------------------------------------- |
| `relation`     | Yes      | Edge label (e.g., `fed-by`, `depends-on`) |
| `direction`    | Yes      | `outgoing` (this page → target)           |
| `target_types` | No       | Valid target types. Omitted = any type.   |

Body `[[wiki-links]]` get a generic `links-to` relation.

See [graph.md](../engine/graph.md) for how the engine builds and renders
the graph.

> **Note:** Typed graph edges are subject to change (Phase 3).

## Custom Types

Add a schema file and register in `wiki.toml`:

```toml
[types.meeting-notes]
schema = "schemas/meeting-notes.json"
description = "Meeting notes with attendees and action items"
```

The engine doesn't need to know what "meeting-notes" means. It validates
against the schema and indexes using the alias mapping.

## Backward Compatibility

- Pages without a `type` field default to `type: page`, validated
  against `[types.default]`
- Pages with an unregistered type are validated against `[types.default]`
- Wikis with no `schemas/` directory use a built-in base schema
- No frontmatter rewriting — existing files are untouched
