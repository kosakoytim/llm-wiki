---
title: "Features"
summary: "What llm-wiki can do — organized by what you'd want to accomplish."
read_when:
  - Getting a full picture of what llm-wiki supports
  - Checking whether a specific capability exists
  - Onboarding a new contributor
status: proposal
last_updated: "2025-07-17"
---

# Features

Everything the engine supports, in plain language. Each section links to
the specs that define the details.

## Manage Wikis

- Create a new wiki with one command
- Register, list, and remove wikis
- Run multiple wikis from a single process
- Set a default wiki for quick access
- Read and write settings per wiki or globally

References:
- [space-management](tools/space-management.md)
- [config-management](tools/config-management.md)
- [server](engine/server.md)

## Write and Organize Content

- Read any page by slug or `wiki://` URI
- Write pages directly into the wiki tree
- Create pages and sections with scaffolded frontmatter
- Bundle pages with co-located assets (images, configs)
- Commit changes by page, by section, or all at once
- Superseded pages show a redirect notice

References:
- [content-operations](tools/content-operations.md)

## Search and Discover

- Full-text search across one wiki or all of them
- Filter search results by page type
- List pages with type and status filters
- Visualize the concept graph in Mermaid or DOT
- Filter the graph by type, relation, root node, or depth

References:
- [search](tools/search.md)
- [list](tools/list.md)
- [graph](tools/graph.md)

## Ingest and Validate

- Validate pages against their type's schema on ingest
- Index pages automatically when ingested
- Commit to git automatically or on demand
- Rebuild the search index from committed files
- Detect and recover from index problems automatically

References:
- [ingest-pipeline](engine/ingest-pipeline.md)
- [index-management](engine/index-management.md)

## Knowledge Structure

- Separate what you know, what sources claim, and what you concluded
- Rich frontmatter: title, summary, tags, sources, confidence, claims
- Define custom page types with their own schemas
- Typed edges between pages (fed-by, depends-on, superseded-by)

References:
- [epistemic-model](model/epistemic-model.md)
- [type-system](model/type-system.md)

## Connect

- MCP server over stdio or SSE
- ACP server for Zed and VS Code agent panels
- Ready-made config for Cursor, VS Code, and Windsurf
- Transport crash recovery and supervision

References:
- [server](engine/server.md)
- [mcp-clients](integrations/mcp-clients.md)
- [acp-transport](integrations/acp-transport.md)

## Store and Discover Skills

- Store agent skills as wiki pages
- Find skills by searching or listing with `--type skill`

References:
- [type-system](model/type-system.md)
- [search](tools/search.md)
