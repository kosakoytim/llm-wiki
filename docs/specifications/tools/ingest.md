---
title: "Ingest"
summary: "Validate, index, and optionally commit."
read_when:
  - Ingesting content into the wiki
status: ready
last_updated: "2025-07-17"
---

# Ingest

MCP tool: `wiki_ingest`

```
llm-wiki ingest <slug|uri>              # file or folder
            [--dry-run]
            [--format <fmt>]            # text | json (default: from config)
            [--wiki <name>]
```

Validates frontmatter, updates the search index, and commits to git
when `ingest.auto_commit` is true. Accepts a bare slug or `wiki://`
URI. When a `wiki://` URI is used, `--wiki` is ignored.

For the full pipeline, see
[ingest-pipeline.md](../engine/ingest-pipeline.md).

### Output

Text (default):

```
ingest: concepts/mixture-of-experts.md — 1 page, 0 assets
commit: a3f9c12
```

JSON (`--format json`):

```json
{
  "pages_validated": 1,
  "assets_found": 0,
  "warnings": [],
  "commit": "a3f9c12"
}
```

`commit` is empty when `ingest.auto_commit` is false.
