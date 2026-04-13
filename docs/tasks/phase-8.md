# Phase 8 — Repository Layout + Bundle Support

Goal: the wiki supports both flat pages and bundle folders (page + co-located
assets). Slug resolution handles both forms transparently. All walkers updated.

Depends on: Phase 7 complete (incremental index update required for bundle
page indexing).
Design refs: [repository-layout.md](../design/repository-layout.md),
[asset-ingest.md](../design/asset-ingest.md).

---

## `markdown.rs`

- [ ] `slug_for(path: &Path, wiki_root: &Path) -> String`
  — if filename is `index.md` → slug = parent dir relative to wiki root
  — otherwise → slug = path without extension relative to wiki root
- [ ] `resolve_slug(wiki_root: &Path, slug: &str) -> Option<PathBuf>`
  — check `{slug}.md` first, then `{slug}/index.md`
  — return `None` if neither exists
- [ ] `promote_to_bundle(wiki_root: &Path, slug: &str) -> Result<()>`
  — move `{slug}.md` → `{slug}/index.md`, creating the directory
  — no-op if already a bundle
- [ ] `is_bundle(wiki_root: &Path, slug: &str) -> bool`
  — true if `{slug}/index.md` exists

## `integrate.rs`

- [ ] `write_asset_colocated(wiki_root, page_slug, filename, content) -> Result<()>`
  — promote page to bundle if currently flat
  — write asset to `{page_slug}/{filename}`
- [ ] `write_asset_shared(wiki_root, kind, filename, content) -> Result<()>`
  — write to `assets/{subdir}/{filename}` per kind→subdir table
  — update `assets/index.md`
- [ ] `assets_index_path(wiki_root) -> PathBuf` — `assets/index.md`
- [ ] `regenerate_assets_index(wiki_root) -> Result<()>`
  — walk `assets/` (excluding `index.md`), rebuild table, write and stage
- [ ] `IngestReport` gains `bundles_created: usize`

## `search.rs`

- [ ] Update `build_index` and `update_index` walkers — use `slug_for` instead
  of naive path stripping
- [ ] Skip non-`index.md` files inside bundle folders (they are assets, not pages)
- [ ] Skip `assets/` subtree except `assets/index.md` (index is a page, assets are not)
- [ ] Add `path: String` (absolute) field to `SearchResult`

## `graph.rs`

- [ ] Update `build_graph` walker — use `slug_for` for consistent slug derivation
- [ ] Bundle assets not treated as pages in the graph

## `context.rs`

- [ ] Update page path resolution to use `resolve_slug`

## `lint.rs`

- [ ] Update page walker to use `slug_for`
- [ ] Add orphan asset reference check: page body references `./asset` that does
  not exist in the bundle folder → report in `LintReport.orphan_asset_refs`
- [ ] `LintReport` gains `orphan_asset_refs: Vec<String>`

## `server.rs`

- [ ] Update MCP resource resolution to use `resolve_slug`
- [ ] Register bundle asset resources: `wiki://{wiki}/{slug}/{filename}`
  — read from `{wiki_root}/{slug}/{filename}`
- [ ] `wiki://{wiki}/assets/{subdir}/{stem}` — shared assets (existing)

## `cli.rs`

- [ ] `wiki read <slug>` — new subcommand: print full content of one page to stdout
  — resolves via `resolve_slug`, prints frontmatter + body
- [ ] `wiki read <slug> --body-only` — body only, no frontmatter

## Tests

**Test files:** `tests/ingest.rs` (extend), `tests/search.rs` (extend),
`tests/graph.rs` (extend)

### Unit tests

- [ ] `slug_for` — flat file `concepts/foo.md` → `"concepts/foo"`
- [ ] `slug_for` — bundle `concepts/foo/index.md` → `"concepts/foo"`
- [ ] `resolve_slug` — flat file exists → returns `.md` path
- [ ] `resolve_slug` — bundle exists → returns `index.md` path
- [ ] `resolve_slug` — neither exists → `None`
- [ ] `promote_to_bundle` — flat `.md` moved to `index.md`, directory created
- [ ] `promote_to_bundle` — already bundle → no-op, no error
- [ ] `write_asset_colocated` — flat page promoted, asset written beside `index.md`
- [ ] `write_asset_colocated` — bundle page → asset written, no promotion needed
- [ ] `write_asset_shared` — written to correct `assets/{subdir}/` path
- [ ] `regenerate_assets_index` — table contains all files under `assets/`
- [ ] `search::build_index` — bundle page indexed once (not twice)
- [ ] `search::build_index` — asset files not indexed as pages
- [ ] `lint` — orphan asset ref detected when bundle asset missing

### Integration tests

- [ ] Ingest flat page, then add co-located asset → page promoted to bundle,
  `git log` shows both changes in one commit
- [ ] `wiki read concepts/foo` — resolves bundle, prints content
- [ ] `wiki search` after bundle promotion — slug unchanged, still found
- [ ] MCP resource `wiki://default/concepts/foo/diagram.png` — returns asset content
- [ ] `wiki lint` — orphan asset ref appears in report

## Changelog

- [ ] `CHANGELOG.md` — Phase 8: bundle layout, `slug_for`, `resolve_slug`,
  `promote_to_bundle`, co-located assets, `wiki read`, orphan asset lint

## README

- [ ] CLI reference — add `wiki read <slug>`
- [ ] **Repository layout** section — flat vs bundle, when each is used, asset
  co-location, link to `docs/design/repository-layout.md`

## Dev documentation

- [ ] `docs/dev/layout.md` — slug resolution rules, bundle promotion, asset
  placement decision (co-located vs shared), `assets/index.md` format
- [ ] Update `docs/dev/architecture.md` — mark Phase 8 modules updated
