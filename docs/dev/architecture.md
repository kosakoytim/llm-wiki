# Architecture

## Design principles

1. **No LLM calls.** The `wiki` binary has zero LLM dependency. It manages Markdown
   files, git history, and tantivy search indexes. All intelligence is external.
2. **analysis.json is the boundary.** External LLMs produce `analysis.json`;
   the wiki engine consumes it. The contract is documented in
   [`docs/design/design.md`](../design/design.md).
3. **Git is the backend.** Every ingest session is a commit. The full history of
   how the knowledge base evolved is in `git log`.
4. **Contradictions are knowledge.** Contradiction pages are first-class nodes,
   never deleted, only enriched.

## Module map

Modules marked ✅ are fully implemented. Others are stubs for future phases.

```
src/
├── main.rs          ✅ Entry point — parse CLI, dispatch to modules
├── cli.rs           ✅ clap Command enum (no logic)
│
├── analysis.rs      ✅ Analysis JSON schema — DocType, Claim, SuggestedPage,
│                       Contradiction, Action, Status, Dimension, PageType
├── markdown.rs      ✅ PageFrontmatter, parse_frontmatter, write_page,
│                       frontmatter_from_page, today_iso8601
├── config.rs        ✅ WikiConfig — per-wiki .wiki/config.toml
│
├── ingest.rs        ✅ Deserialise analysis.json → validate → call integrate → commit
├── integrate.rs     ✅ Write pages (create/update/append) + contradictions
├── git.rs           ✅ init_if_needed, stage_all, commit via git2
├── init.rs          ✅ Phase 5 — init_wiki: git init + create dirs + config.toml
│
├── search.rs        ✅ tantivy index build + BM25 query + search_all (Phase 6)
├── context.rs       ✅ top-K pages as Markdown context for an external LLM
│
├── lint.rs          ✅ structural audit: orphans, missing stubs, active contradictions
├── graph.rs         ✅ petgraph concept graph → DOT / Mermaid output
├── contradiction.rs ✅ contradiction page list + filter by status
│
├── server.rs        ✅ Phase 4+6 — rmcp WikiServer — MCP tools + prompts + resources
│                       Phase 6: registry field, new_with_registry, multi-wiki tools,
│                       namespaced wiki:// URIs, SSE via SseServer::with_service
├── instructions.md  ✅ Phase 5 — embedded LLM guide (all 6 workflow sections complete)
└── registry.rs      ✅ Phase 6 — WikiRegistry, WikiEntry, load, resolve,
                        global_config_path, register_wiki
```

## Dependency graph

```
main ──▶ cli
     ──▶ ingest ──▶ config
                ──▶ analysis
                ──▶ integrate ──▶ analysis
                               ──▶ git
                               ──▶ markdown
     ──▶ search
     ──▶ context ──▶ search
     ──▶ lint    ──▶ graph
                 ──▶ contradiction
                 ──▶ git
     ──▶ graph
     ──▶ contradiction ──▶ analysis
     ──▶ git
     ──▶ init    ──▶ git
                 ──▶ config
     ──▶ server  ──▶ ingest
                 ──▶ context
                 ──▶ search
                 ──▶ lint
     ──▶ registry ──▶ config
```

No cycles. `analysis`, `config`, and `markdown` are leaf modules with no internal
dependencies.

## Implementation phases

| Phase | Status | Key module(s) |
|-------|--------|---------------|
| 0 | ✅ done | All — typed skeletons, no logic |
| 1 | ✅ done | `ingest`, `integrate`, `git`, `markdown` — `wiki ingest` works end-to-end |
| 2 | ✅ done | `search`, `context` — `wiki search` + `wiki context` work end-to-end |
| 3 | ✅ done | `lint`, `graph`, `contradiction` — `wiki lint`, `wiki contradict`, `wiki graph`, `wiki list`, `wiki diff` |
| 4 | ✅ done | `server` (rmcp MCP server — `wiki serve`, `wiki instruct`) |
| 5 | ✅ done | `init` (`wiki init`), `.claude-plugin/` commands + `plugin.json`, complete `instructions.md` |
| 6 | ✅ done | `registry` (multi-wiki registry, `--wiki` flag, `search_all`, SSE transport) |
