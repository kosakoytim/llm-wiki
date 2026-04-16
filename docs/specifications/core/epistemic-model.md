---
title: "Epistemic Model"
summary: "The type taxonomy carries epistemic distinctions — what we know, what sources claim, what we concluded. Folder structure is organizational, not epistemic."
read_when:
  - Understanding why the type taxonomy exists
  - Deciding which type to assign to a new page
  - Explaining the design philosophy to a new contributor or LLM
status: active
last_updated: "2025-07-15"
---

# Epistemic Model

The `type` field is the epistemic axis. It carries the distinction between
synthesized knowledge, source provenance, and reasoning output. Folder
structure is organizational — defined by `schema.md`, not by the engine.

## Origin

The epistemic model is grounded in Karpathy's LLM Wiki concept (April 2026
gist). Karpathy's core ideas:

- Process sources at **ingest time**, not query time — build a persistent wiki
- The LLM reads each source and integrates it, updating existing pages
- Save valuable Q&A as **query-result** pages
- Run **lint passes** to audit for orphans and obsolete content

What this project adds beyond Karpathy:

- The explicit **type-based epistemic model** below — Karpathy describes the
  workflow but does not formalize the type distinctions
- `raw/` as a physical layer — implied by Karpathy's workflow but not
  formalized

---

## The Three Epistemic Roles

```
concept        → what we know            (synthesized knowledge)
source types   → what each source claims (provenance)
query-result   → what we concluded       (reasoning output)
```

Each role answers a different question. None can substitute for another.
The role is carried by the `type` field, not by the folder a page lives in.

---

## Concepts — `type: concept`

**What we know — synthesized knowledge.**

One page per concept, continuously enriched across multiple sources. The
canonical answer to "what do we know about X?". Pages accumulate claims,
tags, confidence levels, and source links over time.

**Why it's a distinct type:**

A concept page represents the *current state of knowledge* about a topic,
synthesized from all sources. A source page represents what *one document*
said. These are different things:

- `type: concept` — everything we know about MoE, from all sources
- `type: paper` — what this one paper said about MoE

Concept pages are the primary retrieval target for `llm-wiki search`. They are
what an LLM reads to answer a question. Source pages are what an LLM reads
to check provenance.

---

## Source Types — `type: paper`, `article`, `documentation`, etc.

**What each source claims — provenance.**

One page per source document. Records what a specific paper, blog post, or
transcript claims, with what confidence, and where the gaps are.

**Why sources are distinct from concepts:**

The same concept can be claimed by many sources with different confidence
levels, different methodologies, and different scopes. Provenance tracking
requires knowing *which source made which claim*. If source summaries are
merged directly into concept pages, that provenance is lost.

```
type: paper    "Switch Transformer (2021)"  → "sparse MoE reduces compute 8x (high confidence)"
type: paper    "MoE Survey (2023)"          → "MoE gains diminish beyond 100B params (medium confidence)"
```

Without source pages, you cannot ask "which sources support this claim?".

See [source-classification.md](source-classification.md) for the full
source type taxonomy.

---

## Query Results — `type: query-result`

**What we concluded — reasoning output.**

Saved Q&A results. When an LLM synthesizes an answer from llm-wiki context,
that synthesis is itself knowledge worth preserving — especially when it
draws on multiple concept pages.

**Why it's distinct from concepts:**

A query result is not a concept. It is a *conclusion* drawn from concepts
at a specific point in time, for a specific question. Keeping them separate
prevents conflating:

- "What does the wiki know about MoE?" (concept)
- "Does MoE scale efficiently?" (query-result — a conclusion)

Query results carry their source slugs — which concept and source pages
were used to produce the answer. This makes them auditable and re-derivable.

---

## Why Type, Not Folder

The previous design coupled epistemic role to folder: `concepts/` for
concepts, `sources/` for sources, `queries/` for query results. This is
redundant — the `type` field already carries the distinction.

| Approach | Problem |
|----------|---------|
| Folder = epistemic role | A cooking wiki wants `recipes/`, `techniques/` — not `concepts/` |
| Folder = epistemic role | Type and folder say the same thing — redundant |
| Folder = epistemic role | Engine must enforce folder-type coupling — rigid |

With type as the axis:

- `llm-wiki list --type concept` works regardless of folder
- `llm-wiki search --type paper` works regardless of folder
- A wiki can organize by domain (`ml/`, `systems/`, `cooking/`) and still
  have clear epistemic distinctions via type
- The default `schema.md` can *suggest* `concepts/`, `sources/`, `queries/`
  as conventions, but the engine doesn't enforce it

---

## Physical Layers vs Epistemic Types

The wiki has two physical layers that the engine *does* enforce — these are
structural, not epistemic:

```
inbox/  → waiting to be processed (human drop zone)
raw/    → what we received        (immutable archive, never indexed)
wiki/   → compiled knowledge      (authors write here, engine indexes)
```

`inbox/` and `raw/` are outside the wiki root. They are physical separation
for workflow purposes. Everything inside `wiki/` is indexed and searchable.

Within `wiki/`, the `type` field is the only epistemic axis. Folder structure
is the wiki owner's choice, defined in `schema.md`.

---

## Why Separation Matters

The failure mode of naive RAG is collapsing these distinctions:

| Collapsed | Problem |
|-----------|---------|
| Sources merged into concepts | Cannot ask "which source claims this?" — provenance lost |
| Query results merged into concepts | Conclusions presented as facts — reasoning not auditable |
| `raw/` indexed alongside pages | Unprocessed content pollutes search — noise in retrieval |

Each type distinction is a deliberate choice to preserve something that
matters for knowledge quality. The `type` field is how the engine and the
LLM maintain these distinctions — not the folder tree.
