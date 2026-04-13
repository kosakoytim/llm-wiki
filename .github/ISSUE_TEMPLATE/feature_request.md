---
name: Feature request
about: Propose a new capability for the wiki engine
labels: enhancement
---

## Phase / component affected

<!-- Which phase and module does this touch? e.g. Phase 2 / search.rs -->

## Problem it solves

<!-- Describe the gap or friction — not just the solution. -->

## Proposed CLI or MCP tool signature

```bash
# CLI example
wiki <new-command> [options]
```

```rust
// MCP tool signature (if applicable)
async fn wiki_new_tool(&self, #[tool(param)] input: String) -> String { ... }
```

## Alternatives considered

<!-- What else did you consider and why did you rule it out? -->
