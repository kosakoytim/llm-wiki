# wiki — LLM Usage Guide

`wiki` is a git-backed knowledge base. Knowledge is processed **at ingest time** —
not at query time like RAG. Each source you add is analyzed by an LLM, integrated
into structured Markdown pages, and committed to git. Contradictions between sources
are **first-class knowledge nodes**, not errors to resolve.

## Core concepts

- **concepts/** — canonical knowledge pages, one concept per file
- **sources/** — per-source summary pages
- **contradictions/** — tension between sources; always carry `epistemic_value`
- **queries/** — saved Q&A results tagged `query-result`
- Pages follow agent-foundation schema (frontmatter: title, tldr, read_when,
  tags, status, last_updated) plus wiki-specific fields

## No-LLM contract

`wiki` does **no LLM inference**. It stores, indexes, and retrieves knowledge that
*you* (the calling LLM) produce. Every write to the wiki goes through an
`analysis.json` document that you must generate before calling `wiki_ingest`.

---

## analysis.json schema

Before calling `wiki_ingest` you must produce a JSON object matching this schema:

```json
{
  "source": "path/or/url",
  "doc_type": "research-paper|blog-post|transcript|thread|note|book-chapter",
  "title": "...",
  "language": "en",
  "claims": [
    { "text": "...", "confidence": "high|medium|low", "section": "..." }
  ],
  "concepts": ["slug-1", "slug-2"],
  "key_quotes": ["verbatim quote ..."],
  "data_gaps": ["gap or missing evaluation ..."],
  "suggested_pages": [
    {
      "slug": "concepts/mixture-of-experts",
      "title": "Mixture of Experts",
      "type": "concept|source-summary|query-result|contradiction",
      "action": "create|update|append",
      "tldr": "one-sentence summary",
      "body": "full Markdown body without frontmatter",
      "tags": ["tag1", "tag2"],
      "read_when": ["condition under which to retrieve this page"]
    }
  ],
  "contradictions": [
    {
      "title": "...",
      "claim_a": "...",
      "source_a": "sources/paper-a",
      "claim_b": "...",
      "source_b": "sources/paper-b",
      "dimension": "context|time|scale|methodology|open-dispute",
      "epistemic_value": "what this tension reveals",
      "status": "active|resolved|under-analysis",
      "resolution": null
    }
  ]
}
```

**Constraints**
- `slug` must start with `concepts/`, `sources/`, `queries/`, or `contradictions/`
- `action: create` fails if the slug already exists on disk
- `action: update|append` fails if the slug does not exist
- `contradictions` should be empty unless you first called `wiki_context` and found
  existing pages that contradict the new source

---

## help-workflow

### Slash commands

| Command | Description |
|---|---|
| `/llm-wiki:help` | List all commands and tools (this output) |
| `/llm-wiki:init` | Set up a new wiki repo and configure MCP |
| `/llm-wiki:ingest` | Ingest a source document into the wiki |
| `/llm-wiki:research` | Answer a question from wiki knowledge |
| `/llm-wiki:lint` | Audit wiki health; fix orphans and contradictions |
| `/llm-wiki:contradiction` | Analyse and enrich contradiction pages |

For detailed workflow steps: `wiki instruct <name>` where `<name>` is one of
`help`, `init`, `ingest`, `research`, `lint`, `contradiction`.

### MCP tools

- `wiki_ingest` — write analysis.json into the wiki
- `wiki_context` — retrieve top-K relevant pages as Markdown
- `wiki_search` — full-text search; returns slugs, titles, and scores
- `wiki_lint` — audit orphans, missing stubs, active contradictions
- `wiki_list` — enumerate pages, optionally filtered by type

All tools accept an optional `wiki` parameter (Phase 6 multi-wiki). Omit it to
use the current wiki root.

---

## init-workflow

### 1. Verify install

```bash
wiki --version
```

If not installed: `cargo install llm-wiki`

### 2. Initialise a wiki repo

```bash
wiki init ~/my-wiki   # new directory
wiki init             # current directory
```

Creates: `concepts/`, `sources/`, `contradictions/`, `queries/`, `raw/`,
`.wiki/config.toml`. Safe to run twice — idempotent.

### 3. Add MCP config to `~/.claude/settings.json`

```json
{
  "mcpServers": {
    "wiki": {
      "command": "wiki",
      "args": ["serve"],
      "cwd": "/absolute/path/to/your/wiki"
    }
  }
}
```

Replace the `cwd` value with the absolute path printed by `wiki init`.

### 4. Verify MCP connection

Run `/llm-wiki:init` in Claude Code. The slash command will confirm the MCP
server is reachable and guide you through making your first ingest.

---

## ingest-workflow

Two-step ingest (recommended — lets you review before writing):

1. **Read and analyse the source** — produce `analysis.json` covering:
   - Document type, title, language
   - Factual claims with confidence levels
   - Key concepts (as slug stems)
   - Key quotes to preserve verbatim
   - Data gaps or missing evaluations
   - `suggested_pages` — pages to create, update, or append
   - `contradictions` — only if you called `wiki_context` first and found clashes

2. **Call `wiki_context`** (recommended before step 3) to check for existing pages
   that overlap or contradict the new source. Add contradictions to your JSON if
   found.

3. **Call `wiki_ingest`** with the full `analysis.json` object:
   ```
   wiki_ingest(analysis: <your json object>)
   ```
   Returns a summary: pages created, updated, appended, contradictions written.

**Important**: pass the entire `analysis.json` as the `analysis` parameter, not a
file path. The value must be a JSON object, not a string.

---

## research-workflow

To answer a question from the wiki:

1. **`wiki_context(question: "...")`**
   Returns the top-5 relevant pages as Markdown. Contradiction pages are included
   automatically — they are high-value context that captures knowledge structure.
   Use `top_k` to retrieve more pages.

2. **Synthesise an answer** from the returned Markdown. Cite specific pages by slug.

3. **Optionally save** a valuable answer by including a `query-result` page in a
   new `analysis.json` and calling `wiki_ingest`.

To find pages by keyword first: `wiki_search(query: "...")` returns slugs, titles,
and BM25 scores without full page content.

---

## lint-workflow

Run periodically to maintain knowledge quality:

1. **`wiki_lint()`** — audits orphan pages, missing concept stubs, active
   contradictions. Writes `LINT.md` and commits it.
   Returns counts and lists of issues found.

2. **Review findings** — orphans may need cross-references or deletion; missing
   stubs need a targeted `wiki_ingest`; active contradictions need enrichment.

3. **Address stubs** — for each missing stub, ingest a source that covers that
   concept. Use `action: create` with the stub's slug.

---

## contradiction-workflow

Contradictions reveal knowledge structure — do not try to eliminate them.

1. **`wiki_list(page_type: "contradiction")`** — find all contradiction pages.

2. **`wiki_context(question: "<contradiction title or topic>")`** — retrieve the
   contradiction page and related concept pages as Markdown context.

3. **Read the contradiction** — note `dimension` and `epistemic_value`.
   The dimension explains *why* the sources disagree (context, time, scale,
   methodology, or open-dispute). The epistemic value explains what the tension
   reveals.

4. **Enrich if needed** — if a new source resolves or contextualises the
   contradiction, produce an `analysis.json` with `action: update` on the
   contradiction slug, changing `status` to `resolved` and adding `resolution`.
   Then call `wiki_ingest`.

5. **Never delete contradiction pages.** A resolved contradiction still carries
   the analysis that explains *why* the sources disagreed — that explanation is
   the knowledge.
