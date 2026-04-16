---
title: "Init"
summary: "Initialize a new wiki — create directory structure, git repo, and register in ~/.llm-wiki/config.toml. Safe to re-run."
read_when:
  - Implementing or extending the init command
  - Understanding what llm-wiki init creates and registers
  - Setting up a new wiki from scratch
status: draft
last_updated: "2025-07-15"
---

# Init

`llm-wiki init` creates a new wiki at a given path, initializes a git repository,
creates the default directory structure, writes an initial commit, and
registers the wiki in `~/.llm-wiki/config.toml`. Safe to re-run.

---

## 1. What Init Creates

```
<path>/
├── README.md       ← for humans (wiki name, description, usage)
├── wiki.toml       ← per-wiki config with name and description
├── schema.md       ← default wiki conventions
├── inbox/
├── raw/
└── wiki/
```

No hidden directories. No `.gitignore` needed for engine artifacts — indexes
live in `~/.llm-wiki/indexes/<name>/`, logs in `~/.llm-wiki/logs/`.

### Global engine directories

On the first `llm-wiki init`, the engine also ensures the global `~/.llm-wiki/`
infrastructure exists:

```
~/.llm-wiki/
├── config.toml     ← global config (created by spaces::register)
├── indexes/        ← search indexes, one per wiki
└── logs/           ← rotating log files for llm-wiki serve
```

`~/.llm-wiki/logs/` is created alongside `config.toml` so that `llm-wiki serve`
can write log files without additional setup.

Initial git commit: `init: <name>`

---

## 2. CLI Interface

```
llm-wiki init <path>
          --name <name>              # wiki name — required, used in wiki:// URIs
          [--description <text>]     # optional one-line description
          [--force]                  # update space entry if name differs from existing
          [--set-default]            # set as default_wiki in ~/.llm-wiki/config.toml
```

### Examples

```bash
llm-wiki init ~/wikis/research --name research
llm-wiki init ~/wikis/research --name research --description "ML research knowledge base"
llm-wiki init ~/wikis/research --name research --set-default
llm-wiki init ~/wikis/research --name research-v2 --force   # rename in spaces
```

---

## 3. Re-run Behavior

| Condition | Behavior |
|-----------|----------|
| Path does not exist | Create directory, git repo, structure, commit, register |
| Path exists, not a git repo | `git init`, create missing dirs, commit, register |
| Path exists, git repo, not registered | Register in `~/.llm-wiki/config.toml` |
| Path exists, registered, same name | Skip silently — already initialized |
| Path exists, registered, different name | Error: `wiki already registered as "<old-name>". Use --force to rename.` |
| `--force` with different name | Update space entry with new name |
| Description changed | Always update silently |

---

## 4. Space Entry Written

Appended to `~/.llm-wiki/config.toml`:

```toml
[[wikis]]
name        = "research"
path        = "/Users/geronimo/wikis/research"
description = "ML research knowledge base"   # omitted if not provided
```

If `--set-default`:

```toml
[global]
default_wiki = "research"
```

If `[global]` already exists, only `default_wiki` is updated.

---

## 5. Generated Files

### `README.md`

```markdown
# research

ML research knowledge base

Managed by [llm-wiki](https://github.com/…). Run `llm-wiki serve` to start the MCP server.
```

Title from `--name`, description from `--description` (omitted if not
provided). The owner customizes it freely.

### `wiki.toml`

```toml
name        = "research"
description = "ML research knowledge base"
```

### `schema.md`

Default template with suggested conventions (`concepts/`, `sources/`,
`queries/`). The owner customizes it to match their domain.

---

## 6. MCP Tool

```rust
#[tool(description = "Initialize a new wiki and register it")]
async fn wiki_init(
    &self,
    #[tool(param)] path: String,
    #[tool(param)] name: String,
    #[tool(param)] description: Option<String>,
    #[tool(param)] force: Option<bool>,
    #[tool(param)] set_default: Option<bool>,
) -> InitReport { ... }

pub struct InitReport {
    pub path:       String,
    pub name:       String,
    pub created:    bool,    // false if already existed
    pub registered: bool,    // false if already registered with same name
    pub committed:  bool,
}
```
