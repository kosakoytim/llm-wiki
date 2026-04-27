---
title: "Ingest Two-Step: Analysis Before Write"
summary: "Add an explicit analysis pass to the ingest skill before any pages are written; enumerate entities, claims, and contradictions first, then generate page structure."
status: proposed
last_updated: "2026-04-27"
---

# Ingest Two-Step: Analysis Before Write

## Problem

The `ingest` skill processes source files in a single pass: read the source,
classify it, search for integration points, then write pages. Structure
decisions — how many pages, what types, what slug hierarchy, which existing
pages to update — are made *while writing*, not before.

This produces lower-quality wiki structure than a two-pass approach for
three reasons:

1. **Missed entities.** A single-pass write stops when the main topic is
   covered. A dedicated analysis pass forces enumeration of all concepts,
   claims, and entities in the source before any commitment to structure.

2. **Undetected contradictions.** The skill searches for integration points
   (step 2c) but does not explicitly compare source claims against existing
   wiki knowledge. Contradictions surface only if the LLM happens to notice
   them while writing — not systematically.

3. **Premature page structure.** The number of pages and their type
   assignments are decided at the same time as the writing. After a dedicated
   analysis pass, the structure decision is better informed: the LLM knows
   the full shape of the source before committing to one page vs. three, or
   to `concept` vs. `query-result`.

ref-1's ingest pipeline explicitly separates analysis (LLM reads source,
enumerates entities, concepts, contradictions) from generation (LLM writes
wiki structure from the analysis output). The analysis output becomes the
context for the generation step, producing richer, better-integrated pages.

## Goal

Add an explicit **analysis step** between "find integration points" (step 2c)
and "decide: create or update" (step 2d). The analysis step:

- Enumerates all durable knowledge items in the source
- Detects contradictions with existing wiki pages found in step 2c
- Assigns a type and confidence estimate to each item
- Produces an **ingest plan** — what to create, what to update, what to skip
- Requires user confirmation before any write

The engine tools and write path are unchanged. This is a skill-only change.

## Solution

### New step 2d — Analysis pass

Insert between current steps 2c (find integration points) and 2d (decide):

```markdown
#### 2d. Analyse the source

Before writing anything, produce an analysis of the source:

**Enumerate durable knowledge items:**
For each concept, claim, finding, or entity in the source, note:
- **What** — one sentence
- **Type** — `concept`, `paper`, `article`, `documentation`, etc.
- **Novelty** — `new` (not in wiki), `extends` (adds to existing page),
  `contradicts` (conflicts with existing page), `duplicate` (already covered)
- **Confidence** — estimated score (see calibration table in the
  **frontmatter** skill)

**Detect contradictions:**
For each page found in step 2c, compare its claims against the source.
Flag explicit contradictions — do not infer or speculate. Report:
- Which source claim contradicts which wiki page
- The slug of the page to update
- Whether the source or the wiki appears more authoritative

**Produce an ingest plan:**

Example:
```
Source: "Mixtral of Experts" (paper, 2024)

1. Mixture of Experts — concept — extends concepts/mixture-of-experts [0.9]
2. Mixtral architecture — concept — new: concepts/mixtral-architecture [0.85]
3. Sparse MoE routing efficiency — claim — extends concepts/sparse-routing [0.8]
4. Contradicts: concepts/moe-compute-cost claims O(n); source claims O(k/n) — update needed
5. Switch Transformer comparison — extends sources/switch-transformer-2021 [0.75]
```

Present the plan to the user and confirm before writing. If the user
redirects (e.g. "skip item 2", "merge 1 and 2"), update the plan before
proceeding.
```

### Updated step 2e — Write from plan

The existing write step becomes plan-driven:

- Work through the plan items in order
- Each `new` item → `wiki_content_new` + `wiki_content_write`
- Each `extends` item → `wiki_content_read` first, then update
- Each `contradicts` item → read the existing page, flag the contradiction
  in the `claims` array with `confidence` adjusted, update the page
- Each `duplicate` item → skip, note in the outcome

### Confidence on contradiction updates

When a source contradicts an existing page, do not silently overwrite. The
correct behavior:

1. Read the existing page
2. Add a new entry to `claims[]` with the source's version and a reference
   to the source page
3. Lower the page-level `confidence` if the contradiction is unresolved
4. Add an open question in the body flagging the discrepancy for human review

This preserves both versions of the claim in the graph and surfaces the
conflict rather than resolving it unilaterally.

## Values

| Value | Mechanism |
|---|---|
| More complete coverage | Enumeration pass forces all entities, not just the main topic |
| Contradiction surfacing | Explicit comparison step, not incidental |
| Better page structure | Structure decided from a complete picture, not mid-write |
| User control | Plan presented and confirmed before any write |
| Claim provenance | Contradictions preserved in `claims[]`, not overwritten |

## Tasks

### Skill — `llm-wiki-skills/skills/ingest/SKILL.md`

- [ ] Insert new `#### 2d. Analyse the source` step between current steps
  2c and 2d (which becomes 2e); include the enumeration format, contradiction
  detection instructions, and ingest plan format with example.
- [ ] Update `#### 2e. Write pages` (renumbered from 2d) to reference the
  ingest plan: work through items in order, handle `new` / `extends` /
  `contradicts` / `duplicate` cases explicitly.
- [ ] Add contradiction handling instructions: read existing page, add to
  `claims[]` with source reference, lower page `confidence`, add open
  question to body.
- [ ] Update step 3 summary to include "Contradictions flagged" as a
  reportable outcome alongside pages created and updated.
- [ ] Update `metadata.version` to `0.4.0` and `last_updated`.
