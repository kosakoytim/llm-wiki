---
title: "Index Management"
summary: "How the tantivy search index is built, updated incrementally, and rebuilt — delete+insert pattern, git-based change detection, unique slug key."
read_when:
  - Implementing or extending the search index pipeline
  - Understanding how incremental index updates work
  - Understanding the relationship between ingest, git, and the index
  - Debugging stale or corrupt index issues
status: draft
last_updated: "2025-07-15"
---

# Index Management

The search index is a tantivy BM25 index stored in
`~/.llm-wiki/indexes/<name>/search-index/`. It is a local artifact — never
committed to git, never shared between machines. Rebuilding is always safe.

---

## 1. Tantivy Constraint

Tantivy does not support in-place document updates. To update a document:

1. Delete the old document by term (unique key)
2. Insert the new document
3. Commit the writer

The `slug` field (`STRING | STORED`) is the unique key. Every page has
exactly one slug, and slugs are stable across edits.

---

## 2. Two Index Operations

### Full Rebuild — `llm-wiki index rebuild`

Drops all documents and re-indexes the entire wiki tree from disk.

```
delete_all_documents()
walk wiki/ → parse each .md → add_document()
writer.commit()
update state.toml (commit hash, page count, built date)
```

Used when:
- First index creation (no `state.toml`, no HEAD)
- Index corruption (auto-recovery)
- Schema migration (new fields added)
- Manual rebuild (`llm-wiki index rebuild`)
- Fallback when incremental update fails

Cost: O(n) where n = total pages.

### Incremental Update — `update_index`

Collects all changed `.md` files from two git diffs, merges them into a
single set, then does one delete+insert pass.

```
A = working tree vs HEAD           (uncommitted changes on disk)
B = state.toml.commit vs HEAD      (committed changes since last index update)

changed = A ∪ B, deduplicated by slug

for each changed slug:
    writer.delete_term(slug)
    if file still exists on disk:
        parse frontmatter + body
        writer.add_document(doc)
writer.commit()
```

This single pass covers everything:
- **A** catches uncommitted changes (ingest just wrote them, commit hasn't
  happened yet)
- **B** catches committed changes since the last index update (someone
  committed outside llm-wiki, or previous `auto_commit` ingests)
- The union covers both — no separate code paths needed

Cost: O(k) where k = changed pages. Typically 1–10 pages per ingest.

---

## 3. Change Detection via Git

### Two diffs, one union

| Diff | What it catches | git2 call |
|------|----------------|-----------|
| Working tree vs HEAD | Uncommitted edits, new files, deletions | `diff_tree_to_workdir_with_index` |
| `state.toml.commit` vs HEAD | Commits since last index update | `diff_tree_to_tree` |

Both diffs produce a list of changed `.md` files under `wiki/`. The union
(deduplicated by path) is the complete set of documents to update.

### Per-file action

| Git status | Index action |
|-----------|-------------|
| New / Added | `add_document` |
| Modified | `delete_term(slug)` + `add_document` |
| Deleted | `delete_term(slug)` |
| Renamed | `delete_term(old_slug)` + `add_document(new_slug)` |
| Unchanged | Skip |

In practice, we always `delete_term` first then `add_document` if the file
exists — simpler than branching on status.

### Implementation sketch

```rust
pub fn collect_changed_files(
    repo_root: &Path,
    wiki_root: &Path,
    last_indexed_commit: Option<&str>,
) -> Result<HashMap<PathBuf, Delta>> {
    let repo = Repository::open(repo_root)?;
    let mut changes = HashMap::new();

    // A: working tree vs HEAD
    if let Ok(head) = repo.head().and_then(|h| h.peel_to_tree()) {
        let diff = repo.diff_tree_to_workdir_with_index(Some(&head), None)?;
        collect_from_diff(&diff, wiki_root, &mut changes);
    }

    // B: state.toml.commit vs HEAD
    if let Some(from_hash) = last_indexed_commit {
        if let (Ok(from_oid), Ok(head)) = (
            git2::Oid::from_str(from_hash),
            repo.head().and_then(|h| h.peel_to_tree()),
        ) {
            if let Ok(from_tree) = repo.find_commit(from_oid).and_then(|c| c.tree()) {
                let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&head), None)?;
                collect_from_diff(&diff, wiki_root, &mut changes);
            }
        }
    }

    Ok(changes)
}

fn collect_from_diff(
    diff: &git2::Diff,
    wiki_root: &Path,
    changes: &mut HashMap<PathBuf, Delta>,
) {
    diff.foreach(&mut |delta, _| {
        let path = delta.new_file().path()
            .or_else(|| delta.old_file().path());
        if let Some(p) = path {
            if p.starts_with("wiki/") && p.extension() == Some("md".as_ref()) {
                // Later entry wins (working tree is more recent than commit diff)
                changes.insert(p.to_path_buf(), delta.status());
            }
        }
        true
    }, None, None, None).ok();
}
```

### Edge cases

- **No HEAD** (fresh repo, no commits yet) — skip diff A, diff B also
  impossible. Fall back to full rebuild.
- **No `state.toml.commit`** (first run) — skip diff B, use diff A only.
  If A is also empty, full rebuild.
- **`state.toml.commit` not in history** (force-push, rebase) — diff B
  fails. Fall back to full rebuild.
- **Both diffs empty** — index is already up to date. No-op.
- **Renamed file** — git reports old and new paths. Delete old slug,
  insert new slug.
- **Bundle assets changed** — non-`.md` files are ignored by the index.
  Only `index.md` changes trigger a re-index.

---

## 4. Schema

Current schema (unchanged):

```rust
fn build_schema() -> Schema {
    let mut builder = Schema::builder();

    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("default")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_opts = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();

    builder.add_text_field("slug", STRING | STORED);   // unique key for delete
    builder.add_text_field("title", text_opts.clone());
    builder.add_text_field("summary", text_opts.clone());
    builder.add_text_field("body", text_opts.clone());
    builder.add_text_field("type", STRING | STORED);
    builder.add_text_field("status", STRING | STORED);
    builder.add_text_field("tags", text_opts);
    builder.build()
}
```

The `slug` field is `STRING | STORED` — exact match, no tokenization. This
is what makes `delete_term(Term::from_field_text(f_slug, slug))` work: it
matches exactly one document.

---

## 5. State Tracking — `state.toml`

```toml
# ~/.llm-wiki/indexes/<name>/state.toml
commit      = "abc123..."    # git HEAD at last full rebuild or catch-up
pages       = 142            # total indexed pages
built       = "2025-07-15T10:30:00Z"
```

### When `commit` is updated

| Operation | `state.toml.commit` updated? |
|-----------|------------------------------|
| Full rebuild | Yes — set to current HEAD |
| Incremental update (changes found) | No — HEAD hasn't moved yet if called during ingest |
| Incremental update (no changes) | No — nothing to do |

After an incremental update during ingest, the index reflects disk but
`state.toml.commit` still points to the old HEAD. This is cosmetic —
search works correctly. The commit hash catches up when the git commit
happens and a subsequent operation runs the incremental path again.

### Staleness detection

```
state.toml.commit != git HEAD  →  stale
```

When `index.auto_rebuild = true`, a stale index triggers an incremental
update (not a full rebuild) before search or list. Full rebuild only
happens as a fallback.

---

## 6. Pipeline Integration

### Ingest pipeline

```
validate → write frontmatter → update_index → commit (if auto_commit)
```

`update_index` replaces the current full rebuild in the ingest path.

### Who calls what

| Trigger | Index operation |
|---------|----------------|
| `llm-wiki ingest` | `update_index` (incremental) |
| `llm-wiki search` (stale + auto_rebuild) | `update_index` (incremental) |
| `llm-wiki list` (stale + auto_rebuild) | `update_index` (incremental) |
| `llm-wiki index rebuild` | Full rebuild |
| Index corruption (auto_recovery) | Full rebuild |
| Any `update_index` failure | Full rebuild (fallback) |

---

## 7. `update_index` Function

```rust
pub fn update_index(
    wiki_root: &Path,
    index_path: &Path,
    repo_root: &Path,
    last_indexed_commit: Option<&str>,
) -> Result<UpdateReport> {
    let changes = collect_changed_files(repo_root, wiki_root, last_indexed_commit)?;
    if changes.is_empty() {
        return Ok(UpdateReport { updated: 0, deleted: 0 });
    }

    let schema = build_schema();
    let search_dir = index_path.join("search-index");
    let dir = MmapDirectory::open(&search_dir)?;
    let index = Index::open(dir)?;
    let mut writer: IndexWriter = index.writer(50_000_000)?;

    let f_slug = schema.get_field("slug").unwrap();
    let mut updated = 0;
    let mut deleted = 0;

    for (path, status) in &changes {
        let slug = slug_for(path, wiki_root);

        // Always delete the old document
        writer.delete_term(Term::from_field_text(f_slug, &slug));

        if *status == Delta::Deleted {
            deleted += 1;
        } else {
            // Added, Modified, Renamed — insert new version from disk
            let full_path = repo_root.join(path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                if let Ok((fm, body)) = parse_frontmatter(&content) {
                    let doc = build_document(&schema, &slug, &fm, &body);
                    writer.add_document(doc)?;
                    updated += 1;
                }
            }
        }
    }

    writer.commit()?;
    Ok(UpdateReport { updated, deleted })
}

pub struct UpdateReport {
    pub updated: usize,
    pub deleted: usize,
}
```

---

## 8. Fallback Strategy

If `update_index` fails for any reason (no git repo, no HEAD, missing
commit in history, index corrupt), fall back to full rebuild. The full
rebuild is always correct — incremental is an optimization.

```rust
let state = load_state(index_path);
let last_commit = state.as_ref().map(|s| s.commit.as_str());

match update_index(wiki_root, index_path, repo_root, last_commit) {
    Ok(report) => report,
    Err(_) => rebuild_index(wiki_root, index_path, wiki_name, repo_root)?,
}
```

---

## 9. Implementation Tasks

Ordered tasks to implement incremental index updates. Each task has code
changes and exit criteria.

→ [Specification](specifications/core/index-management.md)

---

### Task 1 — `git::changed_wiki_files` and `git::changed_since_commit`

**Goal:** Add git functions to detect changed `.md` files.

#### Code changes

- `src/git.rs`:
  - Add `ChangedFile { path: PathBuf, status: git2::Delta }`
  - Add `changed_wiki_files(repo_root, wiki_root) -> Result<Vec<ChangedFile>>`
    — diff working tree vs HEAD, filter to `wiki/*.md`
  - Add `changed_since_commit(repo_root, wiki_root, from_commit) -> Result<Vec<ChangedFile>>`
    — diff `from_commit` tree vs HEAD tree, filter to `wiki/*.md`

#### Tests

- `tests/git.rs`:
  - `changed_wiki_files_detects_new_file` — write a `.md` in `wiki/`, assert
    it appears as `Added`
  - `changed_wiki_files_detects_modified_file` — commit a file, modify it,
    assert `Modified`
  - `changed_wiki_files_detects_deleted_file` — commit a file, delete it,
    assert `Deleted`
  - `changed_wiki_files_ignores_non_md` — write a `.png`, assert not in
    results
  - `changed_wiki_files_ignores_files_outside_wiki` — write to repo root,
    assert not in results
  - `changed_since_commit_detects_gap` — make two commits, pass first
    commit hash, assert second commit's files appear

#### Exit criteria

- Both functions return correct `ChangedFile` lists
- `cargo test --test git` passes

---

### Task 2 — `search::collect_changed_files`

**Goal:** Merge both diffs into a single deduplicated set.

#### Code changes

- `src/search.rs`:
  - Add `collect_changed_files(repo_root, wiki_root, last_indexed_commit: Option<&str>) -> Result<HashMap<PathBuf, Delta>>`
  - Calls `git::changed_wiki_files` for diff A
  - Calls `git::changed_since_commit` for diff B (if `last_indexed_commit`
    is Some and valid)
  - Merges into `HashMap<PathBuf, Delta>` — working tree entries win on
    duplicates

#### Tests

- `tests/search.rs` (or `tests/index.rs`):
  - `collect_changed_files_merges_both_diffs` — commit a file (moves HEAD
    past state.toml.commit), then modify another file in working tree.
    Assert both appear.
  - `collect_changed_files_deduplicates` — same file changed in both diffs,
    assert only one entry
  - `collect_changed_files_skips_diff_b_when_no_commit` — pass `None` as
    `last_indexed_commit`, assert only working tree changes
  - `collect_changed_files_falls_back_on_missing_commit` — pass a
    nonexistent commit hash, assert no error (diff B skipped gracefully)

#### Exit criteria

- Union of both diffs, deduplicated
- Graceful when either diff is unavailable
- `cargo test` passes

---

### Task 3 — `search::build_document` helper

**Goal:** Extract document construction into a reusable function, shared
between `rebuild_index` and `update_index`.

#### Code changes

- `src/search.rs`:
  - Extract the `TantivyDocument` construction from `rebuild_index` into:
    ```rust
    fn build_document(schema: &Schema, slug: &str, fm: &Frontmatter, body: &str) -> TantivyDocument
    ```
  - Refactor `rebuild_index` to call `build_document`

#### Tests

- No new tests needed — existing `rebuild_index` tests cover this. Run
  `cargo test --test search` and `cargo test --test ingest` to confirm no
  regression.

#### Exit criteria

- `rebuild_index` uses `build_document`
- All existing tests pass
- No behavior change

---

### Task 4 — `search::update_index`

**Goal:** Implement the incremental update function.

#### Code changes

- `src/search.rs`:
  - Add `UpdateReport { updated: usize, deleted: usize }`
  - Add `update_index(wiki_root, index_path, repo_root, last_indexed_commit: Option<&str>) -> Result<UpdateReport>`
    - Calls `collect_changed_files`
    - Returns early if empty (no-op)
    - Opens existing index
    - For each changed path: `delete_term(slug)`, then `add_document` if
      file exists on disk
    - `writer.commit()`

#### Tests

- `tests/search.rs` (or `tests/index.rs`):
  - `update_index_adds_new_page` — rebuild empty index, write a new page,
    call `update_index`, search for it, assert found
  - `update_index_updates_modified_page` — rebuild with page A (title "Old"),
    modify title to "New", call `update_index`, search for "New", assert
    found; search for "Old", assert not found
  - `update_index_deletes_removed_page` — rebuild with page A, delete file,
    call `update_index`, search for it, assert not found
  - `update_index_noop_when_no_changes` — rebuild, call `update_index`
    with no changes, assert `UpdateReport { updated: 0, deleted: 0 }`
  - `update_index_handles_multiple_changes` — add, modify, delete in one
    pass, assert report counts

#### Exit criteria

- Incremental update correctly adds, updates, and deletes documents
- Search reflects the changes immediately after `update_index`
- `cargo test` passes

---

### Task 5 — Wire `update_index` into ingest

**Goal:** Replace `rebuild_index` with `update_index` in the ingest path,
with full rebuild as fallback.

#### Code changes

- `src/main.rs` — ingest handler:
  - Load `state.toml` to get `last_indexed_commit`
  - Call `update_index` instead of `rebuild_index`
  - On failure, fall back to `rebuild_index`
- `src/mcp/tools.rs` — `handle_ingest`:
  - Same: `update_index` with fallback to `rebuild_index`

#### Tests

- `tests/ingest.rs`:
  - `ingest_updates_index_incrementally` — ingest a page, verify searchable,
    modify it, ingest again, verify updated content is searchable
  - `ingest_falls_back_to_rebuild_on_update_failure` — hard to test
    directly, but verify that a full rebuild after ingest still works
  - Existing `ingest_rebuilds_index_when_auto_rebuild_enabled` — should
    still pass (now uses incremental path)

#### Exit criteria

- Ingest uses incremental update by default
- Falls back to full rebuild on failure
- `cargo test` passes

---

### Task 6 — Wire `update_index` into stale index catch-up

**Goal:** Replace `rebuild_index` with `update_index` in the search/list
stale-index auto-rebuild path.

#### Code changes

- `src/main.rs` — search and list handlers:
  - When `status.stale && auto_rebuild`: call `update_index` with
    `state.commit` as `last_indexed_commit`
  - On failure, fall back to `rebuild_index`
- `src/mcp/tools.rs` — `handle_search` and `handle_list`:
  - Same pattern

#### Tests

- `tests/search.rs`:
  - `stale_index_catches_up_incrementally` — rebuild index, commit a new
    page outside ingest (manual `git::commit`), search with `auto_rebuild`,
    assert new page is found
  - Existing `ingest_leaves_index_stale_when_auto_rebuild_disabled` —
    should still pass

#### Exit criteria

- Stale index triggers incremental catch-up, not full rebuild
- Falls back to full rebuild on failure
- `cargo test` passes

---

### Task 7 — `llm-wiki index rebuild` stays full rebuild

**Goal:** Confirm explicit rebuild still does a full rebuild, not
incremental.

#### Code changes

- None — `llm-wiki index rebuild` already calls `rebuild_index` directly.
  Verify it is not affected by the incremental changes.

#### Tests

- Existing `tests/search.rs` rebuild tests should pass unchanged.

#### Exit criteria

- `llm-wiki index rebuild` still does `delete_all_documents` + full walk
- `cargo test` passes

---

### Execution order

| Order | Task | Dependencies |
|-------|------|-------------|
| 1 | Task 1 — Git change detection | None |
| 2 | Task 2 — Collect + merge diffs | Task 1 |
| 3 | Task 3 — Extract `build_document` | None |
| 4 | Task 4 — `update_index` | Tasks 2, 3 |
| 5 | Task 5 — Wire into ingest | Task 4 |
| 6 | Task 6 — Wire into stale catch-up | Task 4 |
| 7 | Task 7 — Verify full rebuild | Tasks 5, 6 |

Tasks 1 and 3 can run in parallel. Task 7 is verification only.
