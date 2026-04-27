---
title: "Crystallize Skill Improvements"
summary: "Two-step extraction pass, confidence calibration table, post-ingest lint step. Skill-only changes — no engine work required."
status: proposed
last_updated: "2026-04-27"
depends_on: lint, confidence
---

# Crystallize Skill Improvements

## Problem

The `crystallize` skill is single-pass: read the session → decide page
structure → write. This produces adequate results for short sessions but
has three gaps that compound on longer, richer sessions:

1. **No analysis pass before writing.** Structure decisions (what type,
   how many pages, what slug hierarchy) are made while writing, without
   a prior enumeration of what the session actually produced. The result
   is pages that omit important decisions or conflate distinct concepts.

2. **No confidence calibration.** The skill mentions `confidence` in the
   accumulation contract but provides no guidance on what value to assign
   for session-derived knowledge. Every LLM using the skill invents its
   own calibration, producing inconsistent confidence scores across pages
   and wikis.

3. **No structural quality check after writing.** After ingesting new
   pages, the skill calls `wiki_suggest` and `wiki_graph` but has no
   deterministic check for broken links or orphans. Once `wiki_lint`
   lands (improvement #4), sessions that introduce dead references will
   go undetected without a lint step.

## Goals

- Add an explicit **analysis step** before any writes: enumerate
  durable knowledge items, assign types, decide create vs. update.
- Add a **confidence calibration table** so all callers produce
  consistent scores for session-derived content.
- Add a **post-ingest lint step** that calls `wiki_lint` for the pages
  just written. Forward-compatible: describes the step now, activated
  when the engine ships the tool.

## Solution

### 1. Two-step extraction

Replace the current single-pass flow with an explicit analysis step
before writes.

**Section to add after "Search for an existing home":**

```markdown
## Analyse before writing

Before creating or updating any page, produce a concise extraction plan:

For each item of durable knowledge in the session, note:
- **What** — one sentence describing the knowledge
- **Type** — `decision`, `finding`, `pattern`, `open-question`
- **Action** — `create <type> <slug>`, `update <slug>`, or `discard`
- **Confidence** — estimated score (see calibration table below)

Example plan output:
```
1. tantivy fast fields require f64 not f32 (finding) → create concept tantivy-fast-field-types [0.85]
2. tweak_score replaces post-retrieval sort (decision) → create query-result search-ranking-tweak-score [0.9]
3. design-03 needs to depend on design-04 (decision) → update query-result improvements-ordering [0.8]
4. explored renaming ops module (dead end) → discard
```

Present the plan to the user and confirm before writing. This keeps the
user in control and prevents the session from producing redundant or
misclassified pages.
```

### 2. Confidence calibration table

**Section to add before or inside the "Create a new page" section:**

```markdown
## Confidence calibration for session knowledge

| Knowledge type | Suggested `confidence` |
|---|---|
| Decision explicitly reached, agreed by all parties | 0.85–0.95 |
| Pattern observed, confirmed by evidence in session | 0.70–0.85 |
| Finding confirmed and cross-referenced with existing pages | 0.80–0.95 |
| Pattern observed, not yet tested broadly | 0.50–0.65 |
| Hypothesis raised, plausible but unvalidated | 0.30–0.50 |
| Open question noted, no resolution | 0.20–0.35 |
| Speculation or brainstorm output | 0.10–0.25 |

These ranges are starting points. Adjust up when the session provided
strong evidence (code confirmed to work, multiple independent sources);
adjust down when the conclusion was tentative or context-specific.

The default when no calibration applies is `0.5` (neutral — present but
unreviewed). Never leave `confidence` absent on session-derived pages:
the lint rule will flag them as stale if they stay at default for long.
```

### 3. Post-ingest lint step

**Section to add inside "Validate and index", after the real ingest call:**

```markdown
### Check structural quality

After ingest, run the engine lint for the pages just written:

```
wiki_lint(rules: "broken-link,orphan")
```

This catches:
- Dead references in `sources`, `concepts`, or body wikilinks introduced
  by the new pages
- Orphan pages if the new page was not linked from anywhere

Fix any `Error` findings before closing the session. `Warning` findings
(orphan on a newly created standalone page) can be deferred.

> **Note:** `wiki_lint` ships with engine v0.2.0. If the command is not
> recognized, skip this step and follow the **lint** skill manually for
> broken-link and orphan checks.
```

## Tasks

### Skill — `llm-wiki-skills/skills/crystallize/SKILL.md`

- [ ] Add `## Analyse before writing` section after `## Search for an
  existing home`; describe the extraction plan format and require user
  confirmation before writes.
- [ ] Add `## Confidence calibration for session knowledge` table before
  `## Create a new page`; include the 7-row calibration table and
  guidance on adjusting up/down.
- [ ] In `## Validate and index`, add the post-ingest lint call
  `wiki_lint(rules: "broken-link,orphan")` with the forward-compat note
  about engine v0.2.0.
- [ ] Update `metadata.version` from `0.2.0` to `0.3.0`.
- [ ] Update `last_updated` to today's date.

### Tests / validation

- [ ] Manually run crystallize on a session transcript after the skill
  update; verify the plan is presented before any write.
- [ ] Verify `confidence` is set on all output pages and matches the
  calibration table.
- [ ] After engine v0.2.0 ships `wiki_lint`, verify the lint step runs
  cleanly on crystallize output.
