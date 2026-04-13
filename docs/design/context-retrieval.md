---
title: "Context Retrieval"
summary: "wiki context always returns ranked references (slug, URI, path, title, score) — never full page bodies. wiki read fetches a single page on demand."
read_when:
  - Implementing or changing wiki context or wiki read
  - Understanding why wiki context returns references not content
  - Integrating wiki context into an agent workflow
status: draft
last_updated: "2025-07-15"
---

# Context Retrieval

`wiki context` returns a ranked reference list — slug, URI, path, title, score.
It never returns page bodies. The caller fetches only the pages it needs via
`wiki read` or MCP resource.

---

## 1. The Problem with the Current Design

`wiki context "question"` currently returns the full Markdown body of the top-K
pages concatenated. This is the wrong default:

- Floods the caller's context window with content it may not need
- The caller cannot selectively fetch only the pages it wants
- Returns no paths or URIs — the caller cannot reference, open, or re-fetch results

---

## 2. Correct Design

`wiki context` always returns `Vec<ContextRef>` — a ranked list of references.
No bodies, no concatenation, ever.

The caller reviews the list and fetches only what it needs:
- `wiki read <slug>` — CLI
- `wiki_read` MCP tool
- MCP resource fetch (`wiki://<name>/<slug>`)
- Direct file read (absolute path on disk)

---

## 3. ContextRef

```rust
pub struct ContextRef {
    pub slug:  String,
    pub uri:   String,   // wiki://<wiki-name>/<slug>
    pub path:  String,   // absolute file path on disk
    pub title: String,
    pub score: f32,      // BM25 relevance score
}
```

---

## 4. CLI

```
$ wiki context "MoE scaling efficiency" --top-k 3

slug: concepts/mixture-of-experts
uri:  wiki://research/concepts/mixture-of-experts
path: /Users/geronimo/wikis/research/concepts/mixture-of-experts.md
title: Mixture of Experts
score: 0.94

slug: sources/switch-transformer-2021
uri:  wiki://research/sources/switch-transformer-2021
path: /Users/geronimo/wikis/research/sources/switch-transformer-2021.md
title: Switch Transformer (2021)
score: 0.87

slug: contradictions/moe-scaling-efficiency
uri:  wiki://research/contradictions/moe-scaling-efficiency
path: /Users/geronimo/wikis/research/contradictions/moe-scaling-efficiency.md
title: MoE scaling efficiency: contradictory views
score: 0.81
```

```
wiki context "<question>"
             [--top-k <n>]     # default: 5
             [--wiki <name>]

wiki read <slug>               # fetch full content of one page
          [--wiki <name>]
```

---

## 5. MCP Tools

```rust
// wiki_context: always returns Vec<ContextRef>, never full bodies
#[tool(description = "Return ranked page references for a question — slug, URI, path, title, score")]
async fn wiki_context(
    &self,
    #[tool(param)] question: String,
    #[tool(param)] wiki: Option<String>,
    #[tool(param)] top_k: Option<u32>,
) -> Vec<ContextRef> { ... }

// wiki_read: fetch full content of one page by slug
#[tool(description = "Read the full Markdown content of a wiki page by slug")]
async fn wiki_read(
    &self,
    #[tool(param)] slug: String,
    #[tool(param)] wiki: Option<String>,
) -> String { ... }
```

---

## 6. Usage Example

```bash
# Find relevant pages
wiki context "semantic commit conventions" --top-k 3

# slug: skills/semantic-commit/skill
# uri:  wiki://research/skills/semantic-commit/skill
# path: /wikis/research/skills/semantic-commit/skill.md
# title: Semantic Commit Skill
# score: 0.96

# Read only the page needed
wiki read skills/semantic-commit/skill
# → full SKILL.md content

# Use the slug in a new analysis
# "source_a": "skills/semantic-commit/skill"
```

---

## 7. Rust Module Changes

| Module | Change |
|--------|--------|
| `context.rs` | Replace `String` return with `Vec<ContextRef>`; remove body assembly |
| `search.rs` | Add `score: f32` to `SearchResult` |
| `server.rs` | Update `wiki_context` return type; add `wiki_read` tool |
| `cli.rs` | Update `context` output format; add `read` subcommand |

---

## 8. Implementation Status

| Feature | Status |
|---------|--------|
| `wiki context` returning full bodies | implemented (to be replaced) |
| `wiki context` returning `Vec<ContextRef>` | **not implemented** |
| `wiki read <slug>` | **not implemented** |
| `wiki_read` MCP tool | **not implemented** |
| `wiki_context` returning `Vec<ContextRef>` | **not implemented** |
