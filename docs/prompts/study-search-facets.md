# Study: wiki_search facets

Explore adding faceted search to `wiki_search` — return type, status,
and tag distributions alongside search results.

## Problem

Today, `wiki_search` returns a flat ranked list. To understand the
shape of results — how many concepts vs papers, which tags dominate,
how many drafts — you need to paginate through everything or run
multiple filtered queries.

Facets solve this by returning distribution counts in a single query.

## What facets would look like

### CLI

```
llm-wiki search "mixture of experts" --facets
llm-wiki search "mixture of experts" --facets type,status,tags
```

### MCP

```json
{
  "query": "mixture of experts",
  "facets": ["type", "status", "tags"]
}
```

### Response (JSON)

```json
{
  "results": [ ... ],
  "facets": {
    "type": {
      "concept": 12,
      "paper": 8,
      "article": 3
    },
    "status": {
      "active": 20,
      "draft": 3
    },
    "tags": {
      "mixture-of-experts": 15,
      "scaling": 9,
      "transformers": 7,
      "routing": 5
    }
  }
}
```

### Response (text)

Appended after results:

```
--- facets ---
type:    concept(12)  paper(8)  article(3)
status:  active(20)  draft(3)
tags:    mixture-of-experts(15)  scaling(9)  transformers(7)  routing(5)
```

## Tantivy support

Tantivy supports faceted search natively via `FAST` fields. The
current index schema already has `type` and `status` as keyword
fields. Questions to investigate:

- Are `type` and `status` already `FAST` fields? If not, they need
  to be — facet counting requires `FAST` on keyword fields.
- `tags` is a multi-valued keyword field — does tantivy support
  facet counting on multi-valued `FAST` fields?
- `slug` is already `FAST` (used for sorted pagination). Same
  pattern applies.
- What is the performance cost of facet collection on top of BM25
  ranking? Is it negligible for typical wiki sizes (<10K pages)?

## Tantivy implementation sketch

```rust
// At index build time: ensure type, status, tags are FAST keyword fields
// (may already be the case)

// At query time:
let facet_collector = FacetCollector::for_field("type");
let (top_docs, facet_counts) = searcher.search(
    &query,
    &(TopDocs::with_limit(top_k), facet_collector)
)?;
```

Tantivy's `FacetCollector` may not work directly on keyword fields —
it's designed for hierarchical facet fields. Alternative: use
`TermAggregation` or manually collect term frequencies from `FAST`
fields via `column_values()`.

Investigate which approach is simpler and more performant.

## Interaction with existing filters

Facets should reflect the full result set, not the filtered subset.
Or should they? Two options:

1. **Unfiltered facets** — facets show the full distribution, filters
   narrow results only. Useful for "what else is there?"
2. **Filtered facets** — facets reflect the current filter. Useful
   for drill-down ("of the concepts, how many are draft?")

Most search UIs use filtered facets (option 2). The user applies
`--type concept` and facets update to show only concept distributions.

## Interaction with wiki_list

`wiki_list` could also benefit from facets — "how many concepts vs
papers in the wiki?" without paginating through everything. Same
implementation, different query (match-all instead of BM25).

Consider adding `--facets` to `wiki_list` as well.

## Impact on skills

- **bootstrap** — use facets to report wiki composition (page count
  per type) without paginating
- **research** — show result distribution to help narrow queries
- **lint** — use facets to find status distribution (how many drafts?)

## Open questions

- Should facets be opt-in (`--facets`) or always returned?
- Top-N tags or all tags? For wikis with hundreds of tags, returning
  all is noisy. Default to top 10?
- Should `wiki_list` also support facets?
- Performance: is facet collection cheap enough to always include,
  or should it be opt-in for large wikis?

## Success criteria

- `wiki_search("moe", facets: ["type"])` returns type distribution
  alongside results
- Facets work with existing filters (`--type`, `--status`)
- No measurable performance regression for non-faceted queries
- At least `type`, `status`, and `tags` supported as facet fields
