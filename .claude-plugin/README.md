# llm-wiki Claude Code Plugin

A Claude Code plugin for `llm-wiki` — a git-backed wiki engine where you bring your
own LLM to analyze sources, and the wiki stores, searches, and surfaces structured
knowledge including first-class contradiction nodes.

## Prerequisites

- `wiki` binary installed: `cargo install llm-wiki`

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
| `/llm-wiki:help` | Show available tools, skills, and workflows |
| `/llm-wiki:init` | Verify install, initialize a wiki repo, configure MCP |
| `/llm-wiki:ingest` | Analyze a source and ingest it into the wiki |
| `/llm-wiki:research` | Answer a question using the wiki as context |
| `/llm-wiki:lint` | Structural lint — orphans, stubs, active contradictions |
| `/llm-wiki:contradiction` | Deep analysis of a contradiction page |

## How It Works

The plugin wires up `wiki serve` as an MCP server. Slash commands delegate to the
`llm-wiki` skill, which calls `wiki instruct <command>` to get workflow instructions
from the binary. The binary is the source of truth — no stale docs.

```
/llm-wiki:ingest
  → skill calls: wiki instruct ingest
  → binary returns: step-by-step ingest workflow
  → Claude follows: wiki_context → analysis.json → wiki_ingest
```

## MCP Tools (available after install)

| Tool | Description |
|---|---|
| `wiki_ingest` | Integrate an `analysis.json` into the wiki |
| `wiki_context` | Return top-K relevant pages as Markdown context |
| `wiki_search` | Full-text search across wiki pages |
| `wiki_lint` | Structural audit report |
| `wiki_list` | List pages filtered by type |

## Full Documentation

See [docs/design/](../docs/design/) for architecture, the `analysis.json` contract,
and the implementation roadmap.
