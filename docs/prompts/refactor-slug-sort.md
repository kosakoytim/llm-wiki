# Refactor: Replace _slug_ord with order_by_string_fast_field

## Context

tantivy 0.26 added `TopDocs::order_by_string_fast_field` — native
lexicographic sorting on text FAST fields. This makes our `_slug_ord`
u64 hack (8-byte prefix encoding) unnecessary.

## Current State

- `_slug_ord`: u64 FAST field in `index_schema.rs`
- `slug_ordinal()`: encodes first 8 bytes of slug as big-endian u64
- `index_page()`: writes `_slug_ord` for every document
- `list()` in `search.rs`: uses `order_by_fast_field::<u64>("_slug_ord", Order::Asc)`
- Ties (same 8-byte prefix) broken by in-window slug sort

## Target State

- `slug` field: add FAST flag (`STRING | STORED | FAST`)
- `list()`: use `order_by_string_fast_field("slug", Order::Asc)`
- Remove `_slug_ord` field, `slug_ordinal()` function
- No tie-breaking needed — native lexicographic sort is exact

## Changes

### src/index_schema.rs

- Add FAST to slug field: `STRING | STORED | FAST`
- Remove `_slug_ord` u64 field from `add_fixed_fields`

### src/index_manager.rs

- Remove `slug_ordinal()` function
- Remove `doc.add_u64(is.field("_slug_ord"), slug_ordinal(slug))` from `index_page`

### src/search.rs

- Replace `order_by_fast_field::<u64>("_slug_ord", Order::Asc)` with
  `order_by_string_fast_field("slug", Order::Asc)`
- Remove the in-window tie-breaking sort (`summaries.sort_by(...)`)
- Adjust the return type handling (string fast field returns
  `Vec<(String, DocAddress)>` instead of `Vec<(u64, DocAddress)>`)

### Specs and docs

- Update `docs/specifications/engine/index-management.md` — remove
  `_slug_ord` description, update slug field description
- Update `docs/specifications/tools/list.md` — simplify pagination
  description
- Update `docs/implementation/tantivy.md` — replace `_slug_ord`
  section with native string sort
- Update `docs/decisions/list-pagination.md` — note the simplification

### Tests

- Existing list tests should pass unchanged (behavior is the same)
- Remove any test that specifically tests `_slug_ord` ordering

## Notes

- Requires index rebuild (schema change) — not a concern pre-release
- The `_slug_ord` approach was correct for tantivy 0.25 which lacked
  string fast field sorting
- This is a simplification, not a behavior change
