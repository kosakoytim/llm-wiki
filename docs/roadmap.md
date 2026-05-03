---
title: "Roadmap"
summary: "Roadmap planning for llm-wiki."
status: ready
last_updated: "2026-05-03"
---

# Roadmap

## v0.4.0 (in progress)

| Area  | What |
|-------|------|
| Graph | petgraph-live Phase 1: replace bespoke `CachedGraph` with `GenerationCache` (`feat/petgraph-live-cache`) |
| Graph | petgraph-live Phase 2: snapshot warm-start via `GraphState` (`feat/petgraph-live-snapshot`) |
| Graph | petgraph-live Phase 3: `wiki_health` MCP tool + structural algorithms (`feat/petgraph-live-algorithms`) |

## Future

| Area                                            | What                                                     |
| ----------------------------------------------- | -------------------------------------------------------- |
| IDE                                             | Zed agent panel validation; Cursor MCP config validation |
| Remote Wiki Registration and Version Management | see docs/improvements/design-spaces-register-remote.md   |
| REST / OpenAPI API                              | see docs/improvements/2026-05-03-rest-api-design.md      |

