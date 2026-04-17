---
title: "Search"
summary: "Full-text search with optional type filter."
read_when:
  - Searching the wiki
status: ready
last_updated: "2025-07-17"
---

# Search

MCP tool: `wiki_search`

```
llm-wiki search "<query>"
            [--type <type>]           # filter by page type
            [--no-excerpt]            # refs only, no excerpt
            [--top-k <n>]             # default: from config
            [--include-sections]      # include section index pages
            [--all]                   # search across all registered wikis
            [--format <fmt>]          # text | json (default: text)
            [--wiki <name>]
```

BM25 ranks across `title`, `summary`, `read_when`, `tldr`, `tags`, and
body text. `--type` adds a keyword filter on the `type` field.

### Examples

```bash
llm-wiki search "mixture of experts"
llm-wiki search --type concept "routing strategies"
llm-wiki search --type paper,article "transformer architecture"
llm-wiki search --type skill "process PDF files"
```

### Output

Text (default):

```
concepts/mixture-of-experts  0.94  Mixture of Experts
  Sparse routing of tokens to expert subnetworks, trading compute...
sources/switch-transformer-2021  0.81  Switch Transformer (2021)
  Switch Transformer scales to trillion parameters using sparse MoE...
```

JSON (`--format json`):

```json
[
  {
    "slug": "concepts/mixture-of-experts",
    "uri": "wiki://research/concepts/mixture-of-experts",
    "title": "Mixture of Experts",
    "score": 0.94,
    "excerpt": "Sparse routing of tokens to expert subnetworks, trading compute..."
  }
]
```
