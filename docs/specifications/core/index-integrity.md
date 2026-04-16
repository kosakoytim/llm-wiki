---
title: "Index Integrity"
summary: "Corruption detection, auto-recovery, schema versioning, and health checks for the tantivy search index."
read_when:
  - Implementing or extending index corruption handling
  - Understanding the auto-recovery behavior
  - Adding or changing indexed fields (schema migration)
  - Diagnosing search failures
status: draft
last_updated: "2025-07-17"
---

# Index Integrity

The tantivy search index is a local build artifact at
`~/.llm-wiki/indexes/<name>/`. It can become corrupt (disk failure, partial
write) or incompatible (schema change between versions). This document
specifies how the engine detects and recovers from these conditions.

---

## 1. Failure Modes

| Mode | Cause | Detection |
|------|-------|-----------|
| Stale | Git commit moved since last rebuild | `state.toml` commit ≠ HEAD |
| Corrupt | Truncated mmap files, disk error | `Index::open()` fails |
| Schema mismatch | `build_schema()` changed between versions | `state.toml` schema_version ≠ current |
| Missing | Fresh clone, first use, deleted index | `state.toml` absent |
| state.toml malformed | Crash during rebuild, manual edit | `toml::from_str` fails |

All five modes should result in a recoverable state, not an opaque error.

---

## 2. state.toml

Written to `~/.llm-wiki/indexes/<name>/state.toml` on every rebuild:

```toml
schema_version = 1
built = "2025-07-17T14:32:01Z"
pages = 142
sections = 8
commit = "a3f9c12"
```

### Fields

| Field | Type | Default | Purpose |
|-------|------|---------|--------|
| `schema_version` | u32 | `0` (absent) | Tantivy schema version. Mismatch → stale. |
| `built` | string | — | ISO 8601 datetime of last rebuild |
| `pages` | usize | — | Total pages indexed |
| `sections` | usize | — | Section pages indexed |
| `commit` | string | — | Git HEAD at time of rebuild |

### Parsing rules

- Missing file → `stale: true`, `built: None`, `pages: 0`, `sections: 0`
- Malformed file (parse error) → same as missing, no error propagated
- Missing `schema_version` field → deserializes as `0` via `#[serde(default)]`
  → mismatch with `CURRENT_SCHEMA_VERSION` → `stale: true`

---

## 3. Schema Versioning

The engine defines a `CURRENT_SCHEMA_VERSION` constant in `search.rs`.
It is bumped manually whenever `build_schema()` changes
(adding/removing/renaming indexed fields, changing tokenizers).

```rust
const CURRENT_SCHEMA_VERSION: u32 = 1;
```

### Detection

`index_status` compares the stored `schema_version` in `state.toml`
against the constant:
- Match → schema is compatible
- Mismatch → `stale: true`

This prevents silent query failures after a version upgrade that changes
the indexed fields.

### Pre-versioning state.toml

`schema_version` uses `#[serde(default)]` — if the field is absent
(state.toml written by a pre-versioning build), it deserializes as `0`.
Since `CURRENT_SCHEMA_VERSION >= 1`, this triggers a mismatch →
`stale: true` → rebuild on next search/list.

### Recovery path

Schema mismatch sets `stale: true`, which flows through the existing
paths:
- `auto_rebuild = true` → rebuild before search/list (staleness check)
- `auto_recovery = true` → rebuild if `Index::open` also fails
  (schema change may make the index structurally incompatible)
- Both false → stale warning, queries may return wrong results

### When to bump

Bump `CURRENT_SCHEMA_VERSION` when:
- A new field is added to `build_schema()` (e.g. `confidence`)
- A field is removed or renamed
- A field's indexing options change (e.g. `STRING` → tokenized text)
- The tokenizer is changed

Do NOT bump for:
- Changes to `IndexState` fields (state.toml format, not tantivy schema)
- Changes to search logic (query parsing, scoring)
- Changes to `IngestReport` or other non-index types

---

## 4. Auto-Recovery

### Staleness recovery (`index.auto_rebuild`)

When the index is stale (commit mismatch or schema mismatch):
- `auto_rebuild = true` → rebuild silently before search/list
- `auto_rebuild = false` (default) → warn, continue with stale index

### Corruption recovery (`index.auto_recovery`)

When `Index::open()` fails (corrupt mmap files, incompatible format):
- `auto_recovery = true` (default) → rebuild, retry open, continue
- `auto_recovery = false` → error propagated to caller

The recovery sequence:

```
1. Index::open(dir)
   → Success: proceed with query
   → Failure + auto_recovery = true:
     2. Log warning: "index corrupt, rebuilding"
     3. rebuild_index() — full delete + re-index
     4. Index::open(dir) again
        → Success: proceed
        → Failure: error — "index still corrupt after rebuild"
   → Failure + auto_recovery = false:
     2. Error propagated
```

Recovery is attempted once. If the rebuild itself produces a corrupt
index (e.g. disk full), the error propagates.

---

## 5. Configuration

All index configuration is **global-only**. Indexes are global engine
infrastructure (`~/.llm-wiki/indexes/`), not per-wiki state.

```toml
[index]
auto_rebuild = false    # rebuild stale index before search/list
auto_recovery = true    # rebuild corrupt index on open failure
```

| Key | Scope | Default | Description |
|-----|-------|---------|-------------|
| `index.auto_rebuild` | global only | `false` | Rebuild stale index before search/list |
| `index.auto_recovery` | global only | `true` | Rebuild corrupt index on open failure |

`llm-wiki config set index.* --wiki <name>` is rejected with
`"index.* is a global-only key — use --global"`.

### Why different defaults

- `auto_rebuild = false` — rebuilding a stale index adds latency. The
  user should opt in explicitly.
- `auto_recovery = true` — a corrupt index blocks all search/list. The
  user almost always wants automatic recovery.

---

## 6. Health Check

`llm-wiki index check` performs a read-only integrity check:

1. Parse `state.toml` — exists? valid? schema version current?
2. Open index — `Index::open()` succeeds?
3. Test query — `AllQuery` with limit 1 returns a result?

Returns a structured report:

```rust
pub struct IndexCheckReport {
    pub wiki: String,
    pub openable: bool,
    pub queryable: bool,
    pub schema_version: Option<u32>,
    pub schema_current: bool,
    pub state_valid: bool,
    pub stale: bool,
}
```

Available as CLI (`llm-wiki index check`) and MCP tool (`wiki_index_check`).
Does not modify the index or trigger any rebuild.

---

## 7. Logging

All recovery actions are logged:

| Event | Level | Fields |
|-------|-------|--------|
| Corrupt index detected | `warn` | wiki, error |
| Auto-rebuild triggered | `info` | wiki, reason (corrupt/stale/schema) |
| Rebuild succeeded | `info` | wiki, pages_indexed, duration_ms |
| Rebuild failed | `error` | wiki, error |
| state.toml parse error | `warn` | wiki, error |
| Schema version mismatch | `info` | wiki, stored, current |
| Corrupt directory delete failed | `warn` | error |

---

## 8. Limitations

| Limitation | Reason | Impact |
|------------|--------|--------|
| Partial corruption may not be detected | Tantivy can serve queries from remaining healthy segments. Detection depends on which files are damaged. | Silently wrong results possible. Use `llm-wiki index check` to verify. |
| Cross-wiki search (`--all`) does not attempt recovery | `search_all` skips broken wikis. Per-wiki `wiki_root`/`repo_root` not available in the cross-wiki path. | Broken wiki silently excluded from results. |
| ACP research workflow does not attempt recovery | ACP workflow dispatch has no access to resolved config. | User gets "Search failed" message. Manual `llm-wiki index rebuild` needed. |
| No concurrent recovery protection | If two MCP calls detect corruption simultaneously, both may attempt delete + rebuild. | Unlikely with single-threaded MCP. Second rebuild is a no-op (fresh index). |
| Recovery deletes the entire index directory | No incremental repair — full rebuild from wiki markdown. | Rebuild time proportional to wiki size. |
| `remove_dir_all` failure is non-fatal | Logged at `warn` level. Rebuild proceeds but may fail if corrupt files remain. | Manual cleanup needed if permissions prevent deletion. |
