# Improvements

## Roadmap

# Indexation
blacklist:
  - Repositories\
  - 
index_excludes:
  - raw\
  - archive\
  - ingested\

# Skills

Skill	What it does
wiki-configure
Updates the wiki configuration (config.toml) and restarts the server if needed; can be used to add new tools, update tool specs, or change global settings

wiki-ingest	Processes files from raw/ into wiki pages; classifies, synthesises, cross-references, and moves each file to ingested/ as an atomic commit

wiki-query	Answers questions by reading the wiki index and relevant pages; cites sources; can file valuable answers as new wiki pages

wiki-lint	Health-checks the wiki: broken links, orphaned pages, missing index entries, unreferenced sources in ingested/

wiki-integrate	Weaves a new or updated page into the knowledge graph by adding backlinks and index entries

wiki-crystallize	Distils a working session or accumulated conversation into a structured wiki page (biased toward updating existing hubs and the overview.md, as the top level summary of everything that is known). You can teach your LLM to adopt a structure that serves your workflow.

wiki-writer edit ?

wiki-graph 


https://github.com/anthropics/claude-plugins-official/tree/main/plugins/example-plugin

-> skill format that replace command
-> 

https://modelcontextprotocol.io/specification/2025-11-25/server/resources ?
https://modelcontextprotocol.io/specification/2025-11-25/server/prompts


### 2. ACP streaming

Implement streaming tool calls in ACP workflows (tasks D → A → B → C → E).

→ [acp-tasks.md](acp-tasks.md) · [ACP SDK reference](implementation/acp-sdk.md) · [ACP transport spec](specifications/integrations/acp-transport.md)

### 3. Distribution channels

Decide which channels to support. Chocolatey is too heavy to maintain —
drop it. Evaluate: cargo install, cargo-binstall, homebrew, asdf.

- [ ] Fix `docs/release.md` — currently a verbatim copy of agentctl's; still says "agentctl"
- [ ] Replace agentctl-specific post-release steps with llm-wiki equivalents
- [ ] Decide final channel list and document only those

---

## Other improvements

### config

Parameter global-only should not be updated locally.

### graph

`graph.type` — documented but not yet implemented in `set_global_config_value`.

### init wiki

- [ ] Generate a `.gitignore`
- [ ] Prepare a `ci.yaml` template

### Documentation

**Implementation docs** — architecture overview, config system, server internals, logging:
- [ ] Architecture overview (module map, data flow, key abstractions)
- [ ] Config system (two-level resolution, adding new keys, serde patterns)
- [ ] Server internals (MCP stdio/SSE, ACP, transport lifecycle)
- [ ] Logging (rotation, format, file vs stderr, serve mode)

**User-facing docs** — installation and integration guides:
- [ ] Installation guide (cargo install, pre-built binaries, platform notes)
- [ ] Windows installation and usage notes
- [ ] IDE integration (VS Code, Cursor, Windsurf — beyond MCP config snippets)
- [ ] CI/CD integration (using llm-wiki in automated pipelines)

**Publishing** — use the wiki as a content source for static sites:
- [ ] Hugo integration guide (wiki pages as Hugo content, frontmatter mapping, example site)
- [ ] GitHub Pages deployment (CI workflow to build and publish from wiki repo)
