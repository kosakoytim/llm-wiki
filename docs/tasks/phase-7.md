# Phase 7 — Search Index — Incremental Update

Goal: the search index is no longer rebuilt on every `wiki search` call.
It is built on first use, updated incrementally after each ingest, and
rebuilt explicitly via `--rebuild-index`. Search results always reflect
the current wiki state without full-rebuild cost on every query.

Depends on: Phase 6 complete.
Design ref: [dev/search.md](../dev/search.md)

---

## `search.rs`

- [ ] `build_index(wiki_root: &Path, index_dir: &Path) -> Result<Index>`
  — unchanged: walk all `.md` files, parse frontmatter + body, index each
  — called only when index does not exist
- [ ] `update_index(wiki_root: &Path, index_dir: &Path, changed_slugs: &[String]) -> Result<()>`
  — open existing index
  — for each slug in `changed_slugs`: delete existing document if present,
    re-index from current file on disk
  — for each slug that no longer exists on disk: delete from index
  — commit writer
- [ ] `open_or_build_index(wiki_root: &Path, index_dir: &Path) -> Result<Index>`
  — if `.wiki/search-index/` exists and is valid → open
  — if missing or corrupt → call `build_index`, return result
  — replaces the current always-rebuild logic in `search()`
- [ ] `search(wiki_root, query, limit)` — call `open_or_build_index` instead
  of `build_index`
- [ ] `search_all(registry, query, limit)` — same: `open_or_build_index` per wiki

## `integrate.rs`

- [ ] All integrate functions (`integrate_direct_file`, `integrate_direct_folder`,
  `integrate_analysis`, `integrate_enrichment`, `integrate_query_result`) collect
  the slugs of pages written or deleted during the operation
- [ ] After git commit: call `search::update_index(wiki_root, index_dir, &changed_slugs)`
- [ ] `IngestReport` gains `index_updated: bool` — true if index was updated,
  false if index did not exist (will be built on next search)

## `git.rs`

No changes. Index update happens after commit, not inside git operations.

## `cli.rs`

- [ ] `wiki search --rebuild-index` — call `build_index` (wipe + rebuild),
  exit 0. Behaviour unchanged.
- [ ] `wiki search "<query>"` — call `open_or_build_index` (not `build_index`)

## Tests

**Test file:** `tests/search.rs` (extend)

### Unit tests

- [ ] `open_or_build_index` — missing index dir → builds and returns valid index
- [ ] `open_or_build_index` — existing index → opens without rebuilding
  (verify by checking mtime of index files does not change)
- [ ] `update_index` — new page added → appears in subsequent search
- [ ] `update_index` — existing page modified → updated content appears in search
- [ ] `update_index` — page deleted → no longer appears in search
- [ ] `update_index` — empty `changed_slugs` → no-op, index unchanged

### Integration tests

- [ ] `wiki ingest` → index updated incrementally, new page searchable immediately
- [ ] `wiki search` on existing index → does not rebuild (fast path)
- [ ] `wiki search` with missing index → builds automatically, returns results
- [ ] `wiki search --rebuild-index` → wipes and rebuilds, exits 0
- [ ] Two consecutive `wiki search` calls → index files not modified on second call

## Changelog

- [ ] `CHANGELOG.md` — Phase 7: incremental search index update, `update_index`,
  `open_or_build_index`, fix always-rebuild behavior

## Dev documentation

- [ ] `docs/dev/search.md` — already updated with correct rebuild policy.
  Add `update_index` function signature and `open_or_build_index` description.
