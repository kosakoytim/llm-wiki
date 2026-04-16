# llm-wiki Claude Code Plugin

A Claude Code plugin for `llm-wiki` — a git-backed wiki engine that turns a
folder of Markdown files into a searchable, structured knowledge base.

## Prerequisites

- `llm-wiki` binary installed: `cargo install llm-wiki`

## Installation

### Claude Code Marketplace

```bash
claude plugin marketplace add geronimo-iia/llm-wiki
claude plugin install --scope user llm-wiki
```

### Local Clone

```bash
claude plugin add /path/to/llm-wiki
```

## Post-Install Setup

After installing, run the init command to verify your install and initialize a wiki:

```
/llm-wiki:init
```

## Available Slash Commands

| Command | Description |
|---|---|
| `/llm-wiki:help` | Show available tools, slash commands, and workflows |
| `/llm-wiki:init` | Verify install, initialize a wiki repo, configure MCP |
| `/llm-wiki:new` | Create a new page or section in the wiki |
| `/llm-wiki:ingest` | Ingest a file or folder into the wiki |
| `/llm-wiki:research` | Answer a question using the wiki as context |
| `/llm-wiki:lint` | Structural audit — orphans, missing stubs, empty sections |
| `/llm-wiki:crystallize` | Distil session insights into wiki pages |

## How It Works

The plugin wires up `llm-wiki serve` as an MCP server. Slash commands delegate to
the `llm-wiki` skill, which calls `llm-wiki instruct <command>` to get workflow
instructions from the binary. The binary is the source of truth — no stale docs.

```
/llm-wiki:ingest
  → skill calls: llm-wiki instruct ingest
  → binary returns: step-by-step ingest workflow
  → Claude follows: search → read → write → ingest
```

## MCP Tools (available after install)

| Tool | Description |
|---|---|
| `wiki_search` | Full-text BM25 search across wiki pages |
| `wiki_read` | Read a page by slug |
| `wiki_write` | Write a file into the wiki tree |
| `wiki_list` | List pages with optional type/status filters |
| `wiki_ingest` | Validate, commit, and index files |
| `wiki_new_page` | Create a scaffolded page |
| `wiki_new_section` | Create a section index |
| `wiki_lint` | Structural audit report |
| `wiki_graph` | Render the knowledge graph |
| `wiki_config` | Get/set configuration |
| `wiki_index_rebuild` | Rebuild the search index |
| `wiki_index_status` | Check if the index is stale |
| `wiki_index_check` | Run read-only integrity check |

## Full Documentation

See the [specifications](../docs/specifications/) for architecture, the epistemic
model, and the full command reference.
