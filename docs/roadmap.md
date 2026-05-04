---
title: "Roadmap"
summary: "Roadmap planning for llm-wiki."
status: ready
last_updated: "2026-05-03"
---

# Roadmap

## v0.4.1 — Bug Fix Release

| Area | What |
|------|------|
| Bug fix | `spaces register` now calls `ensure_structure` — creates `wiki.toml` and directory scaffold (`inbox/`, `raw/`, `schemas/`, content dir) matching `spaces create` behaviour (issue #62) |
| Tests | Unit tests for `schema remove`, `logs tail/list/clear`, and missing assertions on `spaces register` |
| Tests | Integration scripts for `spaces register`, `spaces remove`, `list`, `schema add/remove`, `logs` |

## Future

| Area                                            | What                                                     |
| ----------------------------------------------- | -------------------------------------------------------- |
| IDE                                             | Zed agent panel validation; Cursor MCP config validation |
| Remote Wiki Registration and Version Management | see docs/improvements/design-spaces-register-remote.md   |
| REST / OpenAPI API                              | see docs/improvements/2026-05-03-rest-api-design.md      |
