---
title: "Spaces"
summary: "Manage llm-wiki spaces — list registered wikis, remove entries, and set the default wiki."
read_when:
  - Implementing or extending the spaces command
  - Listing, removing, or changing the default wiki
  - Understanding how spaces relate to ~/.llm-wiki/config.toml
status: draft
last_updated: "2025-07-15"
---

# Spaces

`llm-wiki spaces` manages the llm-wiki spaces registered in `~/.llm-wiki/config.toml`.
It provides subcommands to list spaces, remove entries, and set the default.

---

## 1. Subcommands

### `llm-wiki spaces list`

Prints all registered wikis with their name, path, description, and whether
they are the current default.

```bash
llm-wiki spaces list
```

Output:

```
  name        path                              description
* research    /Users/geronimo/wikis/research    ML research knowledge base
  work        /Users/geronimo/wikis/work        —
  sp-theory   /Users/geronimo/build/sp_theory   SP theory knowledge base
```

`*` marks the current default wiki.

---

### `llm-wiki spaces remove <name>`

Removes a wiki entry from `~/.llm-wiki/config.toml`. Refuses if the wiki is the
current default — set a new default first with `llm-wiki spaces set-default`.

```bash
llm-wiki spaces remove work
llm-wiki spaces remove work --delete   # also delete the local directory
```

Flags:

```
llm-wiki spaces remove <name>
                   [--delete]   # also delete the wiki directory from disk
```

Errors:

| Condition | Error |
|-----------|-------|
| Name not found | `error: wiki "work" is not registered` |
| Is current default | `error: "work" is the default wiki — set a new default first` |
| `--delete` but path does not exist | Warning only, entry still removed |

Git commit is not made — space changes are local only.

---

### `llm-wiki spaces set-default <name>`

Sets the default wiki. Thin alias for `llm-wiki config set global.default_wiki <name>`.

```bash
llm-wiki spaces set-default research
```

Errors:

| Condition | Error |
|-----------|-------|
| Name not found | `error: wiki "unknown" is not registered` |

---

## 2. MCP Tools

```rust
#[tool(description = "List all registered llm-wiki spaces")]
async fn wiki_spaces_list(&self) -> Vec<SpaceEntry> { ... }

#[tool(description = "Remove a wiki space")]
async fn wiki_spaces_remove(
    &self,
    #[tool(param)] name: String,
    #[tool(param)] delete: Option<bool>,
) -> String { ... }

#[tool(description = "Set the default wiki space — alias for llm-wiki config set global.default_wiki")]
async fn wiki_spaces_set_default(
    &self,
    #[tool(param)] name: String,
) -> String { ... }

pub struct SpaceEntry {
    pub name:        String,
    pub path:        String,
    pub description: Option<String>,
    pub default:     bool,
}
```
