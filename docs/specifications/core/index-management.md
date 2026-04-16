---
title: "Index Management"
summary: "How documents enter and leave the tantivy search index — full rebuild, incremental update via git diffs, delete+insert pattern, state tracking."
read_when:
  - Implementing or extending the index update pipeline
  - Understanding when and how the index is updated
  - Understanding the delete+insert constraint from tantivy
  - Deciding between full rebuild and incremental update
status: draft
last_updated: "2025-07-15"
---

# Index Management

The search index is a tantivy BM25 index stored outside the wiki repo at
`~/.llm-wiki/indexes/<name>/search-index/`. It is a local build artifact —
never committed, never shared. Rebuilding from the wiki tree is always safe.

For corruption detection and auto-recovery, see
[index-integrity.md](index-integrity.md).

---

## 1. The Constraint

Tantivy does not support in-place document updates. To update a document:

1. Delete the old document by term (unique key)
2. Insert the new document
3. Commit the writer

The `slug` field (`STRING | STORED`) is the unique key — exact match, no
tokenization. `delete_term(Term::from_field_text(f_slug, slug))` matches
exactly one document.

---

## 2. Full Rebuild

Drops all documents and re-indexes the entire wiki tree from disk.

```
delete_all_documents()
walk wiki/ → parse each .md → add_document()
writer.commit()
update state.toml
```

Triggered by:
- `llm-wiki index rebuild` (explicit)
- First index creation (no `state.toml`)
- Index corruption (auto-recovery)
- Schema migration (`schema_version` mismatch)
- Fallback when incremental update fails

Cost: O(n) where n = total pages.

---

## 3. Incremental Update

Collects changed `.md` files from two git diffs, merges them into one set,
then does a single delete+insert pass.

```
A = working tree vs HEAD           (uncommitted changes on disk)
B = state.toml.commit vs HEAD      (commits since last index update)

changed = A ∪ B, deduplicated by path

for each changed path:
    slug = slug_for(path)
    writer.delete_term(slug)
    if file still exists on disk:
        parse frontmatter + body
        writer.add_document(doc)
writer.commit()
```

**Why two diffs:**
- **A** catches uncommitted changes — ingest writes files to disk before
  committing to git. At index time, the working tree has changes that HEAD
  does not.
- **B** catches committed changes since the last index update — someone
  committed outside llm-wiki, or previous ingests with `auto_commit`
  moved HEAD past the last indexed commit.
- The union covers both. No separate code paths.

**Why delete+insert always:** simpler than branching on git status. If the
slug existed before, the delete removes it. If it didn't, the delete is a
no-op. Then insert the current version from disk.

Cost: O(k) where k = number of changed pages.

---

## 4. Change Detection

| Diff | Catches | git2 call |
|------|---------|-----------|
| Working tree vs HEAD | Uncommitted edits, new files, deletions | `diff_tree_to_workdir_with_index` |
| `state.toml.commit` vs HEAD | Commits since last index update | `diff_tree_to_tree` |

Both diffs are filtered to `.md` files under `wiki/`. The results are
merged into a single `HashMap<PathBuf, Delta>` — later entries (working
tree) win on duplicates, since they reflect the most recent state.

### Edge cases

| Condition | Behavior |
|-----------|----------|
| No HEAD (fresh repo) | Both diffs impossible → full rebuild |
| No `state.toml.commit` (first run) | Skip diff B, use diff A only |
| `state.toml.commit` not in history (rebase/force-push) | Diff B fails → full rebuild |
| Both diffs empty | No-op — index is up to date |
| Renamed file | Delete old slug, insert new slug |
| Non-`.md` file changed (bundle asset) | Ignored — only `.md` triggers re-index |

---

## 5. State Tracking

```toml
# ~/.llm-wiki/indexes/<name>/state.toml
schema_version = 1
commit         = "a3f9c12..."
pages          = 142
sections       = 8
built          = "2025-07-17T14:32:01Z"
```

The `commit` field records git HEAD at the time of the last index update
that can guarantee completeness:

| Operation | `commit` updated? |
|-----------|-------------------|
| Full rebuild | Yes — set to HEAD |
| Incremental update with changes | No — HEAD may not have moved yet |
| Incremental update with no changes | No |

After an incremental update during ingest, `commit` still points to the
old HEAD. The index is fresh (reflects disk), but `state.toml` appears
stale. This is cosmetic — on the next call, diff B
(`state.toml.commit` vs HEAD) will capture the gap.

---

## 6. Who Calls What

| Trigger | Operation |
|---------|-----------|
| `llm-wiki ingest` | Incremental update |
| `llm-wiki search` (stale + `auto_rebuild`) | Incremental update |
| `llm-wiki list` (stale + `auto_rebuild`) | Incremental update |
| `llm-wiki index rebuild` | Full rebuild |
| Index corruption (`auto_recovery`) | Full rebuild |
| Incremental update failure | Full rebuild (fallback) |

---

## 7. Fallback

If the incremental update fails for any reason, fall back to full rebuild.
The full rebuild is always correct — incremental is an optimization.

```rust
let state = load_state(index_path);
let last_commit = state.as_ref().map(|s| s.commit.as_str());

match update_index(wiki_root, index_path, repo_root, last_commit) {
    Ok(report) => report,
    Err(_) => rebuild_index(wiki_root, index_path, wiki_name, repo_root)?,
}
```

---

## 8. Pipeline Position

In the ingest pipeline, the index update runs after validation and before
the optional git commit:

```
validate → write frontmatter → update_index → commit (if auto_commit)
```

This ordering is essential: the working tree diff (A) only works because
the changes are on disk but not yet committed.
