---
title: "Review Skill"
summary: "Human review queue assembled from existing fields — wiki_lint + wiki_list(status: draft) + confidence; skill drives the review loop, engine unchanged."
status: proposed
last_updated: "2026-04-27"
depends_on: confidence, lint
---

# Review Skill

## Problem

Knowledge quality degrades silently between sessions. Pages get created as
drafts and never promoted. Sources are ingested with low confidence and never
verified. Contradictions flagged during ingest are noted as open questions
and then forgotten.

ref-1 ships an async review queue — a UI panel where pages flagged for human
attention accumulate until a reviewer processes them. The mechanism is a
dedicated data structure maintained by the engine.

In llm-wiki the data is already there:

- `status: draft` — page was created but never verified
- `confidence < threshold` — page was written with low certainty
- `wiki_lint(rules: "stale")` — page is old and low-confidence
- Open questions in page body — contradictions surfaced by ingest

What is missing is a skill that assembles these signals into a prioritized
review queue and guides an agent through it systematically.

## Goal

A `review` skill that:

1. Builds a prioritized review queue from existing fields and lint findings
2. Presents items one at a time with the context needed to review them
3. Guides the reviewer through a consistent decision for each page
4. Records the outcome (promote, update, defer, flag for deletion)
5. Reports a summary at the end

No engine changes. No new data structure. The queue is a view over fields
that already exist.

## Solution

### Queue assembly

The queue is built from three sources, merged and deduplicated:

**Source 1 — lint errors and warnings:**
```
wiki_lint()
```
Pages with `Error` findings (broken links, missing fields, unknown type)
rank highest — they are structurally broken, not just unreviewed.

**Source 2 — draft pages:**
```
wiki_list(status: "draft")
```
Every page with `status: draft` is pending promotion or deletion.

**Source 3 — low-confidence active pages:**
```
wiki_list(status: "active")
```
Filter to pages where `confidence < 0.4` — active but not yet verified.

**Priority order:**

| Priority | Condition |
|---|---|
| 1 — Error | `wiki_lint` `Error` finding on this page |
| 2 — Warning | `wiki_lint` `Warning` finding on this page |
| 3 — Draft | `status: draft` with no lint finding |
| 4 — Low confidence | `status: active`, `confidence < 0.4` |

Within each priority tier, order by `confidence` ascending (least certain
first), then `last_updated` ascending (oldest first).

### Review loop

Process items one at a time. For each page:

**1. Read the page:**
```
wiki_content_read(slug: "<slug>", backlinks: true)
```

**2. Show context:**
- What triggered the flag (lint finding, draft status, low confidence)
- Who links to this page (backlinks — helps assess whether it's orphaned
  or load-bearing)
- When it was last updated (`wiki_history(slug: "<slug>", limit: 3)`)

**3. Guide the review decision:**

| Decision | When | Action |
|---|---|---|
| **Promote** | Content is correct and complete | Set `status: active`, raise `confidence` to 0.7–0.9, commit |
| **Update** | Content is partially correct | Edit claims, update `confidence`, keep `status: draft` if still incomplete, commit |
| **Resolve contradiction** | Page has open question from ingest | Verify the conflicting claims, update `claims[]`, remove open question, adjust `confidence`, commit |
| **Defer** | Cannot verify now | Leave as-is, note reason in body as a dated comment; remove from queue for this session |
| **Flag for deletion** | Page is redundant or wrong | Set `status: archived`, lower `confidence` to 0.1, add `superseded_by` if applicable, commit |

**4. Write and commit:**
```
wiki_content_write(slug: "<slug>", content: "<updated content>")
wiki_ingest(path: "<path>")
```

**5. Move to the next item.**

### Session scope

The review skill processes as many items as the session allows. It does
not need to finish the queue. The queue state is implicit in the page
fields — every session starts fresh by rebuilding it from the current
index.

A page deferred in one session reappears in the next if its fields
haven't changed. A page promoted in one session drops off the queue
permanently.

### Report

After the session ends (or the user stops), report:

```
Review session complete.

Processed: 8 pages
  Promoted (draft → active): 3
  Updated (confidence raised): 2
  Contradictions resolved: 1
  Deferred: 1
  Flagged for deletion: 1

Remaining in queue: 14 pages
  Errors: 2
  Warnings: 5
  Drafts: 4
  Low confidence: 3

Suggested next session: start with the 2 Error findings.
```

## Values

| Value | Mechanism |
|---|---|
| No engine changes | Queue is a view over `status`, `confidence`, `wiki_lint` — all existing |
| Async by design | Queue persists in page fields between sessions; no dedicated state |
| Prioritized | Structural errors before quality gaps before staleness |
| Claim provenance | Contradictions resolved in `claims[]`, not silently overwritten |
| Discoverable | Any session can run the review skill; no setup or migration needed |

## Tasks

### Skill — `llm-wiki-skills/skills/review/SKILL.md`

- [ ] Create `skills/review/SKILL.md` with frontmatter:
  `name: review`, `type: skill`, `status: active`,
  `tags: [review, quality, audit]`,
  `when_to_use: Reviewing draft pages, verifying low-confidence knowledge,
  resolving flagged contradictions, or doing a periodic wiki quality pass.`
- [ ] Write `## Build the review queue` section: three sources (`wiki_lint`,
  `wiki_list(status: "draft")`, `wiki_list(status: "active")` filtered by
  confidence), merge and deduplicate, priority table.
- [ ] Write `## Review loop` section: read page + backlinks + history, show
  context, five decision options with actions for each.
- [ ] Write `## Resolve contradictions` sub-section: read existing claims,
  verify source vs. wiki, update `claims[]`, remove open question, adjust
  confidence.
- [ ] Write `## Report` section: processed counts by outcome, remaining
  queue counts by priority tier, suggested next session focus.
- [ ] Add `## Key rules` section:
  1. Never delete — use `status: archived` + `superseded_by`
  2. Preserve claim history — add to `claims[]`, do not replace
  3. Defer is valid — note reason, move on; queue rebuilds next session
  4. One page at a time — finish each before starting the next
  5. Confirm promotions with the user when confidence is borderline

### Skill registry — `llm-wiki-skills/plugin.json`

- [ ] Register `review` in the skill list alongside `crystallize`, `ingest`,
  `lint`, etc.

### Cross-skill updates

- [ ] `skills/ingest/SKILL.md` — in step 2d (analysis pass), add a note
  that pages with unresolved contradictions should be left as `status: draft`
  so the review skill picks them up.
- [ ] `skills/lint/SKILL.md` — add a note that `wiki_lint` findings feed the
  review queue; running lint before a review session produces a fresher queue.
