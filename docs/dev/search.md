---
title: "Search — tantivy integration"
summary: "How the full-text index is built, queried, and managed."
read_when:
  - Debugging search results or index lifecycle
  - Extending search to new field types
status: active
last_updated: "2026-04-13"
---

# Search — tantivy integration

`wiki search` and `wiki context` are powered by [tantivy](https://github.com/quickwit-oss/tantivy) —
a Lucene-equivalent full-text search engine written in Rust.

---

## Index location

```
{wiki_root}/.wiki/search-index/
```

The index is **gitignored** and rebuilt locally from committed Markdown files.
It is never committed to git. A fresh clone runs `wiki search --rebuild-index`
to become fully functional.

---

## Schema

| Field  | Type            | Options          | Purpose                                      |
|--------|-----------------|------------------|----------------------------------------------|
| `slug` | `STRING`        | STORED           | Page slug — relative path without `.md`      |
| `title`| `TEXT`          | STORED           | Page title — tokenised for full-text search  |
| `tags` | `TEXT`          | STORED           | Space-joined tag list — tokenised            |
| `body` | `TEXT`          | STORED           | Full page body — tokenised; stored for snippet |
| `type` | `STRING`        | STORED           | Page category (exact token, not tokenised)   |

`TEXT | STORED` fields are both indexed for full-text search and stored so their
values can be retrieved after a query. `STRING | STORED` fields are stored as
exact tokens (no tokenisation) — suitable for `slug` and `type` which are never
searched with partial matches.

---

## `SearchResult` fields

| Field       | Type     | Description                                                   |
|-------------|----------|---------------------------------------------------------------|
| `slug`      | `String` | Relative path without `.md` (e.g. `concepts/mixture-of-experts`) |
| `title`     | `String` | Page title from frontmatter                                    |
| `snippet`   | `String` | First 200 characters of the page body                         |
| `score`     | `f32`    | BM25 relevance score — higher is more relevant                |
| `page_type` | `String` | Kebab-case page category: `concept`, `source-summary`, etc.   |

---

## Index lifecycle

### Rebuild policy

The index is **not rebuilt on every search**. It is managed incrementally:

- **First use** — if `.wiki/search-index/` does not exist, `search()` builds
  it automatically before querying
- **After ingest** — `wiki ingest` updates the index incrementally: new and
  modified pages are added, deleted pages are removed
- **Explicit rebuild** — `wiki search --rebuild-index` wipes and rebuilds
  from scratch

This means search results always reflect the current state of the wiki without
the cost of a full rebuild on every query.

### Skipped directories

- `raw/` — unprocessed source files, never wiki-managed pages
- `.wiki/` — contains the index itself; including it would be recursive

Files without valid frontmatter are skipped silently. They may be raw Markdown
or externally modified files that the wiki engine did not produce.

### `wiki search --rebuild-index`

Rebuilds the index and exits with code 0 — no search query is performed. Useful
for:

- Pre-warming the index on a fresh clone
- Verifying the index builds correctly in CI
- Scripts that need a fresh index before a subsequent search

### Fresh clone workflow

```bash
git clone <wiki-repo> my-wiki
cd my-wiki
wiki search --rebuild-index   # one-time local setup
wiki search "mixture of experts"
```

---

## `context` output format

`wiki context "<question>"` runs a BM25 search and returns a ranked list of
page references — never full page bodies. See [context-retrieval.md](../design/context-retrieval.md)
for the full design.

Each result:

```
slug:  concepts/mixture-of-experts
uri:   wiki://research/concepts/mixture-of-experts
path:  /Users/.../concepts/mixture-of-experts.md
title: Mixture of Experts
score: 0.94
```

Contradiction pages are **not filtered** — they are high-value context.
The caller fetches only the pages it needs via `wiki read <slug>`.

---

## Gitignore rationale

The tantivy index is a binary artefact derived entirely from the committed
Markdown files. Committing it would:

1. Add large binary diffs to every ingest commit
2. Require conflict resolution for a file that can be trivially rebuilt
3. Cause merge conflicts for collaborators working in parallel

Since `build_index` is fast (milliseconds for hundreds of pages), rebuilding
locally is always cheaper than managing the index in git.
