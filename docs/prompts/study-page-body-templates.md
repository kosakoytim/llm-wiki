# Study: Page body templates via wiki_content_new

Extend `wiki_content_new` to scaffold a full page body (not just
frontmatter) based on the page type. Templates live alongside schemas
using a naming convention.

## Problem

`wiki_content_new` creates a page with scaffolded frontmatter but an
empty body. The author must write the body structure from scratch
every time. For common types, the body follows a predictable pattern.

## Decisions

- **Naming convention in `schemas/`** — body templates are
  `schemas/<type>.md` next to `schemas/<type>.json`. No new directory.
  Discovery: look for `.md` sibling of the schema file.
- **Wiki-owner-defined** — custom types get body templates for free.
  Drop a `.md` file next to the schema.
- **Embedded defaults** — `llm-wiki spaces create` ships default
  templates for built-in types alongside the default schemas.
- **Fallback chain**:
  1. `schemas/<type>.md` in the wiki repo (owner-defined)
  2. Embedded default template (shipped with engine)
  3. Frontmatter-only (current behavior, if no template exists)
- **Watcher ignores `.md` in schemas/** — the watcher only watches
  `schemas/*.json` for schema changes. Template `.md` files do not
  trigger index rebuilds.
- **`wiki_content_new` always uses template** — no `--template` flag.
  If a body template exists, it's included. The author overwrites it.
- **`wiki_schema show --template` unchanged** — returns frontmatter
  only (as today). Body templates are a `content_new` feature.

## Template format

A body template is a plain Markdown file with placeholder sections.
No frontmatter — the engine prepends the scaffolded frontmatter.

Example `schemas/concept.md`:

```markdown
## Overview



## Key ideas



## Related concepts



## Open questions

```

Example `schemas/paper.md`:

```markdown
## Summary



## Key claims



## Methodology



## Limitations

```

Example `schemas/section.md`:

```markdown
## Overview

```

## Repository layout change

```
<wiki>/
├── schemas/
│   ├── base.json
│   ├── concept.json
│   ├── concept.md          ← body template
│   ├── paper.json
│   ├── paper.md            ← body template
│   ├── skill.json
│   ├── doc.json
│   ├── doc.md              ← body template
│   ├── section.json
│   └── section.md          ← body template
└── wiki/
```

## Interaction with existing features

- **Watcher** — ignores `schemas/*.md`, only watches `schemas/*.json`
- **Schema validation** — `wiki_schema validate` ignores `.md` files
- **Ingest** — no change, templates are not indexed
- **`wiki_schema show --template`** — unchanged, frontmatter only
- **`wiki_content_new`** — now appends body template after frontmatter

## Open questions

- Should `wiki_schema show --template --body` exist to preview the
  full page (frontmatter + body)? Or is `content_new` sufficient?

## Tasks

### 1. Update specifications

- [ ] Update `docs/specifications/tools/content-operations.md` —
  document that `content_new` scaffolds body from template
- [ ] Update `docs/specifications/model/wiki-repository-layout.md` —
  add `.md` template files to `schemas/` layout
- [ ] Update `docs/specifications/engine/watch.md` — clarify watcher
  ignores `schemas/*.md`

### 2. Embedded default templates

- [ ] `src/default_schemas.rs` — add embedded `.md` templates for
  built-in types (concept, paper, doc, section, query-result)
- [ ] `src/spaces.rs` — write `.md` templates alongside `.json`
  schemas in `ensure_structure`

### 3. Template resolution

- [ ] `src/ops/content.rs` — add `resolve_body_template(repo_root,
  type_name) -> Option<String>` that checks `schemas/<type>.md`
  then falls back to embedded
- [ ] `src/ops/content.rs` — in `content_new`, append body template
  after frontmatter when creating a page

### 4. Watcher exclusion

- [ ] Verify `src/watch.rs` — `is_schema_path` only matches `.json`,
  confirm `.md` files in `schemas/` are ignored

### 5. Tests

- [ ] `content_new` with body template produces frontmatter + body
- [ ] Custom template in `schemas/<type>.md` overrides embedded
- [ ] Missing template falls back to frontmatter-only
- [ ] Watcher ignores `schemas/*.md` changes
- [ ] `spaces create` writes `.md` templates alongside schemas
- [ ] Existing test suite passes unchanged

### 6. Decision record

- [ ] `docs/decisions/page-body-templates.md`

### 7. Update skills

- [ ] `llm-wiki-skills/skills/content/SKILL.md` — mention body
  templates in page creation
- [ ] `llm-wiki-skills/skills/schema/SKILL.md` — mention `.md`
  template convention
- [ ] `llm-wiki-skills/skills/ingest/SKILL.md` — note that
  `content_new` now scaffolds body structure

### 8. Update guides

- [ ] `docs/guides/custom-types.md` — add section on creating body
  templates for custom types
- [ ] `docs/guides/getting-started.md` — mention body templates in
  page creation step

### 9. Finalize

- [ ] `cargo fmt && cargo clippy --all-targets -- -D warnings`
- [ ] Update `CHANGELOG.md`
- [ ] Update `docs/roadmap.md`
- [ ] Remove this prompt

## Success criteria

- `wiki_content_new(uri: "concepts/new", type: "concept")` creates a
  page with frontmatter + concept body sections
- Custom `schemas/meeting.md` is used when creating `type: meeting`
- Missing template falls back to frontmatter-only (no error)
- Watcher does not trigger on `schemas/*.md` changes
- `llm-wiki spaces create` ships default `.md` templates
