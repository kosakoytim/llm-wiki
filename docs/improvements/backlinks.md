---
title: "Backlinks"
summary: "Expose incoming links on wiki_content_read via an index query, no file writes."
status: proposed
last_updated: "2026-04-27"
---

# Backlinks

## Problem

The tantivy index already stores `body_links` (all `[[slug]]` references extracted
from each page's body). Incoming links are queryable in aggregate — but they are
invisible to an LLM reading a specific page.

When an agent calls `wiki_content_read` on a page, it sees the page's outgoing links
embedded in the markdown body. It has no way to know what links *to* this page without
issuing a separate `wiki_search` with a `body_links` filter — a non-obvious,
multi-step lookup that no skill currently performs. The agent cannot discover related
pages without additional queries.

## Goal

Make incoming links available as part of a `wiki_content_read` response, without
requiring a separate query and without any file writes.

## Solution

Add a `backlinks: bool` parameter to `wiki_content_read`. When `true`, the response
includes an additional `backlinks` field: an array of `{ slug, title }` objects for
all pages whose `body_links` index field contains the target slug.

```
wiki_content_read(slug: "my-page", backlinks: true)
→ { content: "...", frontmatter: {...}, backlinks: [{ slug: "other", title: "Other Page" }] }
```

Implementation: a targeted tantivy term query on the `body_links` keyword field.
No file writes, no git commits, no index mutation. The index is the source of truth.

**Why not persist backlinks into the `.md` file:**
- Git churn: any link change anywhere causes unrelated page diffs
- Stale data: the injected section is only current as of the last write; the index
  is always current
- Responsibility leak: Hugo and other renderers compute backlinks at build time from
  their own link graph — duplicating this in source files splits ownership across layers
- `[[slug]]` syntax in an injected section is unsafe: `extract_body_wikilinks` scans
  raw body text with no sentinel awareness and would index backlink sources as outgoing
  links, corrupting `body_links` for the page

The index query gives the right answer at the right time with no state to go stale.

## Tasks

- [ ] Add `backlinks_for(slug: &str, searcher: &Searcher, is: &IndexSchema) -> Vec<BacklinkRef>` in `src/ops/content.rs`; term query on `body_links` keyword field; return `Vec<{ slug, title }>`.
- [ ] Add `BacklinkRef { slug: String, title: String }` struct to the content ops return types; derive `Serialize`.
- [ ] Add `backlinks: bool` parameter to `wiki_content_read` MCP tool definition in `src/tools.rs`.
- [ ] In `handlers::handle_content_read`, when `backlinks: true`, call `backlinks_for` and include the result in the JSON response.
- [ ] Update `wiki_content_read` response schema in `docs/specifications/tools/content-operations.md`.
- [ ] Unit test: two pages link to a third; `backlinks_for` returns both; a page with no incoming links returns an empty vec.
- [ ] Unit test: `backlinks: false` (default) returns no `backlinks` field in the response (no overhead for the common case).
