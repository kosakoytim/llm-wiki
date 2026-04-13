---
name: llm-wiki
description: llm-wiki — git-backed wiki engine. Use when ingesting sources into a wiki, researching questions, managing contradictions, or running lint. Delegates to the wiki binary for dynamic instructions.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# llm-wiki

A git-backed wiki engine. You bring the LLM analysis; the wiki stores, searches,
and surfaces structured knowledge including first-class contradiction nodes.

## Prerequisites

Ensure `wiki` is installed:

```bash
wiki --version
```

If not installed:

```bash
cargo install llm-wiki
```

## Usage

The wiki binary contains workflow instructions. To get instructions for any operation:

```bash
wiki instruct <command>
```

Where `<command>` is one of: `help`, `init`, `ingest`, `research`, `lint`, `contradiction`.

Run the appropriate instructions command, then follow the returned instructions step by step.
