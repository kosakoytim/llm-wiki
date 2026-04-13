---
title: "Epistemic Model"
summary: "Why the five default wiki categories exist — the epistemic roles of concepts, sources, contradictions, queries, and raw, and why separating them matters."
read_when:
  - Understanding why the wiki has these specific default directories
  - Deciding which category a new page belongs in
  - Explaining the design philosophy to a new contributor or LLM
status: active
last_updated: "2025-07-15"
---

# Epistemic Model

The five default wiki categories are not arbitrary. Each has a distinct
epistemic role. Mixing them collapses distinctions that matter for knowledge
quality, contradiction detection, and provenance tracking.

## Origin

The category structure is grounded in Karpathy's LLM Wiki concept (April 2026
gist). Karpathy's core ideas:

- Process sources at **ingest time**, not query time — build a persistent wiki
- The LLM reads each source and integrates it, updating existing pages
- It detects and flags **contradictions** between new and existing claims
- Save valuable Q&A as **query-result** pages
- Run **lint passes** to audit for contradictions, orphans, and obsolete content

What this project adds beyond Karpathy:

- Contradictions as **first-class knowledge nodes** with their own pages,
  `epistemic_value` field, `dimension` taxonomy, and permanent status —
  Karpathy flags them during ingestion; this design preserves them as
  structured artifacts that are richer than either source alone
- The explicit **epistemic layer model** below — Karpathy describes the
  workflow but does not name the layers this way
- `raw/` as a dedicated directory — implied by Karpathy's workflow but not
  formalized as a category

---

## The Layers

```
raw/              → what we received        (unprocessed input)
sources/          → what each source claims (provenance)
concepts/         → what we know            (synthesized knowledge)
contradictions/   → where sources disagree  (knowledge structure)
queries/          → what we concluded       (reasoning output)
```

Each layer answers a different question. None can substitute for another.

---

## `raw/`

**What we received — unprocessed input.**

Original source files: PDFs, transcripts, Markdown files, HTML exports.
Never modified after ingestion. Excluded from tantivy indexing, orphan
detection, and graph traversal.

**Why it exists separately:**

The wiki is self-contained. You can always re-analyze a source from scratch
without fetching it again. `raw/` is an archive, not a knowledge base. Keeping
it separate prevents raw content from polluting search results and concept
pages.

---

## `sources/`

**What each source claims — provenance.**

One page per source document. Records what a specific paper, blog post, or
transcript claims, with what confidence, and where the gaps are.

**Why it exists separately from `concepts/`:**

The same concept can be claimed by many sources with different confidence
levels, different methodologies, and different scopes. Contradiction detection
requires knowing *which source made which claim*. If source summaries are
merged directly into concept pages, that provenance is lost.

```
sources/switch-transformer-2021.md  → "sparse MoE reduces compute 8x (high confidence)"
sources/moe-survey-2023.md          → "MoE gains diminish beyond 100B params (medium confidence)"
```

Without `sources/`, you cannot ask "which sources support this claim?" or
"does this source contradict that one?".

---

## `concepts/`

**What we know — synthesized knowledge.**

One page per concept, continuously enriched across multiple sources. The
canonical answer to "what do we know about X?". Pages accumulate claims,
tags, confidence levels, and contradiction links over time.

**Why it exists separately from `sources/`:**

A concept page represents the *current state of knowledge* about a topic,
synthesized from all sources. A source page represents what *one document*
said. These are different things:

- `concepts/mixture-of-experts.md` — everything we know about MoE, from all sources
- `sources/switch-transformer-2021.md` — what this one paper said about MoE

Concept pages are the primary retrieval target for `wiki context`. They are
what an LLM reads to answer a question. Source pages are what an LLM reads
to check provenance or detect contradictions.

---

## `contradictions/`

**Where sources disagree — knowledge structure.**

One page per detected contradiction between sources. Each page carries
`claim_a`, `claim_b`, `dimension` (context/time/scale/methodology/open-dispute),
`epistemic_value`, and `status` (active/resolved/under-analysis).

**Karpathy's framing:** the LLM detects and flags contradictions during
ingestion. They are surfaced in lint passes for review.

**This project's extension:** contradictions are not just flags — they are
persistent structured pages that are never deleted. A contradiction page is
*richer than either source alone* because it encodes information neither
source captures individually:

- **Context-dependence** — claim A holds in domain X, claim B in domain Y
- **Time-dependence** — A was true in 2021, superseded by B in 2024
- **Scale-dependence** — A works at small scale, B at large scale
- **Methodology divergence** — different measurement approaches yield different results
- **Genuine open dispute** — the field has not resolved this yet

A "resolved" contradiction still carries the analysis that explains *why*
the two sources disagreed. That explanation is the knowledge. `git log`
preserves the full history of how the understanding evolved.

---

## `queries/`

**What we concluded — reasoning output.**

Saved Q&A results. When an LLM synthesizes an answer from wiki context,
that synthesis is itself knowledge worth preserving — especially when it
draws on multiple concept pages and surfaces contradiction pages.

**Why it exists separately from `concepts/`:**

A query result is not a concept. It is a *conclusion* drawn from concepts
at a specific point in time, for a specific question. Keeping them separate
prevents conflating:

- "What does the wiki know about MoE?" (concept page)
- "What is the answer to: does MoE scale efficiently?" (query result)

Query results also carry their source slugs — which concept and source pages
were used to produce the answer. This makes them auditable and re-derivable.

---

## Why Separation Matters

The failure mode of naive RAG is collapsing these layers:

| Collapsed | Problem |
|-----------|---------|
| `sources/` merged into `concepts/` | Cannot ask "which source claims this?" — provenance lost |
| `contradictions/` deleted or merged | Knowledge structure lost — domain boundaries invisible |
| `queries/` merged into `concepts/` | Conclusions presented as facts — reasoning not auditable |
| `raw/` indexed alongside pages | Unprocessed content pollutes search — noise in retrieval |

Each separation is a deliberate choice to preserve a distinction that matters
for knowledge quality.

---

## Relationship to `validate_slug`

The fixed category prefixes enforced by `validate_slug_analysis` in
`integrate.rs` are exactly these five directories (minus `raw/`, which is
never a slug prefix). Analysis-only ingest is restricted to these categories
because the enrichment contract (`enrichments[]`, `query_results[]`,
`contradictions[]`) maps directly onto them.

Direct ingest (`wiki ingest <path> --prefix <name>`) relaxes this — user-defined
prefixes like `skills/` or `guides/` are valid because they represent
structured content that does not fit the epistemic model but is still worth
indexing and searching. See [repository-layout.md](repository-layout.md) for
the full distinction between fixed categories and user-defined prefixes.
