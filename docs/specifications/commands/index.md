---
title: "Index"
summary: "Index maintenance commands — rebuild the tantivy index from committed Markdown and inspect index health."
read_when:
  - Implementing or extending index management
  - Understanding when and how to rebuild the tantivy index
  - Diagnosing search issues after a fresh clone or failed ingest
status: draft
last_updated: "2025-07-15"
---

# Index

The tantivy search index is a local build artifact stored in
`~/.llm-wiki/indexes/<name>/`, outside the wiki repository. It is never committed
to git. `llm-wiki index` provides commands to rebuild and inspect it.

```
~/.llm-wiki/indexes/<name>/
├── search-index/           ← tantivy files
└── state.toml              ← indexed commit, page count, built date
```

---

## 1. `state.toml`

Written to `~/.llm-wiki/indexes/<name>/state.toml` on every `llm-wiki index rebuild`:

```toml
built    = "2025-07-15T14:32:01Z"   # ISO datetime of last rebuild
pages    = 142
sections = 8
commit   = "a3f9c12"               # git HEAD at time of rebuild
```

Staleness is determined by comparing `commit` against the current `git HEAD`
of the wiki repo. If HEAD has moved since the last rebuild, the index is
stale. This is reliable across clones and filesystems — mtime is not used.

---

## 2. Subcommands

### `llm-wiki index rebuild`

Walks all committed Markdown files, indexes all frontmatter fields and body
content, writes the tantivy index to `~/.llm-wiki/indexes/<name>/search-index/`,
and writes `state.toml`. Required after:

- Fresh clone
- Manual file edits outside of `llm-wiki ingest` or `llm-wiki new`
- Index corruption

```bash
llm-wiki index rebuild
llm-wiki index rebuild --wiki research
```

### `llm-wiki index status`

Reports the current state of the index without modifying it.

```bash
llm-wiki index status
```

Output:

```
wiki:     research
path:     ~/.llm-wiki/indexes/research/search-index/
built:    2025-07-15T14:32:01Z
commit:   a3f9c12
pages:    142
sections: 8
stale:    no
```

`stale: yes` means `commit` in `state.toml` does not match the current
`git HEAD` — a rebuild is recommended.

### `llm-wiki index check`

Read-only integrity check. Tests whether the index is openable, queryable,
schema-current, and state-valid — without modifying anything.

```bash
llm-wiki index check
llm-wiki index check --wiki research
```

Output:

```
wiki:            research
openable:        true
queryable:       true
schema_version:  2
schema_current:  true
state_valid:     true
stale:           no
```

Use this to diagnose search failures before deciding whether to rebuild.
See [Index Integrity](../core/index-integrity.md) for the full recovery flow.

---

## 3. Return Types

```rust
pub struct IndexStatus {
    pub wiki:     String,
    pub path:     String,
    pub built:    Option<String>,   // ISO datetime, None if index does not exist
    pub pages:    usize,
    pub sections: usize,
    pub stale:    bool,
}

pub struct IndexReport {
    pub wiki:          String,
    pub pages_indexed: usize,
    pub duration_ms:   u64,
}

pub struct IndexCheckReport {
    pub wiki:           String,
    pub openable:       bool,
    pub queryable:      bool,
    pub schema_version: Option<u32>,
    pub schema_current: bool,
    pub state_valid:    bool,
    pub stale:          bool,
}
```

---

## 4. CLI Interface

```
llm-wiki index rebuild              # rebuild index from committed Markdown
              [--wiki <name>]
              [--dry-run]       # walk and count pages, no write

llm-wiki index status               # inspect index health
              [--wiki <name>]

llm-wiki index check                # read-only integrity check
              [--wiki <name>]
```

---

## 5. Staleness Detection

Staleness is determined by comparing the `commit` field in
`~/.llm-wiki/indexes/<name>/state.toml` against the current `git HEAD`:

- `commit == HEAD` → index is fresh
- `commit != HEAD` → index is stale, rebuild recommended
- `state.toml` missing → index has never been built

This is reliable across clones and filesystems. mtime is not used.

---

## 6. Automatic Rebuild

`llm-wiki search` and `llm-wiki list` check index staleness at startup via
`state.toml`. Behavior depends on the `index.auto_rebuild` config flag:

- `auto_rebuild = false` (default) — print a warning, continue with stale index
- `auto_rebuild = true` — rebuild silently before executing the command

All `index.*` config keys are **global-only** — indexes are global engine
infrastructure, not per-wiki state.

See [Index Integrity](../core/index-integrity.md) for corruption detection,
auto-recovery, and schema versioning.

---

## 7. MCP Tools

```rust
#[tool(description = "Rebuild the tantivy search index from committed Markdown")]
async fn wiki_index_rebuild(
    &self,
    #[tool(param)] wiki: Option<String>,
) -> IndexReport { ... }

#[tool(description = "Inspect the current state of the search index")]
async fn wiki_index_status(
    &self,
    #[tool(param)] wiki: Option<String>,
) -> IndexStatus { ... }

#[tool(description = "Run read-only integrity check on the search index")]
async fn wiki_index_check(
    &self,
    #[tool(param)] wiki: Option<String>,
) -> IndexCheckReport { ... }
```
