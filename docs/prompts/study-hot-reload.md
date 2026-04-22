# Study: Hot reload — add/remove wikis without restart

Explore adding hot reload to `llm-wiki serve` so that wikis can be
added, removed, or re-registered without restarting the server.

## Problem

Today, `llm-wiki serve` mounts all registered wikis at startup. To
add or remove a wiki, the server must be restarted. This breaks
active MCP/ACP sessions and forces agents to re-bootstrap.

For workflows where wikis are created or removed frequently (e.g.
project-scoped wikis, temporary research wikis), this is disruptive.

## Current architecture

From `server.md`, the startup sequence:

```
1. Load ~/.llm-wiki/config.toml — spaces + global config
2. Mount all registered wikis
3. Check index staleness for each wiki
4. Start transports (stdio, HTTP, ACP)
```

Wikis are mounted once. The space registry (`[[wikis]]` in
`config.toml`) is read once at startup. Space management tools
(`wiki_spaces_create`, `wiki_spaces_remove`, `wiki_spaces_set_default`)
modify `config.toml` but the running server doesn't pick up changes.

## Proposed behavior

### Automatic reload

The server watches `~/.llm-wiki/config.toml` for changes. When the
file changes:

1. Re-read the space registry
2. Diff against currently mounted wikis
3. Mount new wikis (create index if needed)
4. Unmount removed wikis (close index handles)
5. Update default wiki if changed
6. Log: `reload: mounted <name>, unmounted <name>`

### Triggered by space management tools

Alternatively (or additionally), the space management tools trigger
reload directly when called from within the running server:

- `wiki_spaces_create` → mount the new wiki immediately
- `wiki_spaces_remove` → unmount immediately
- `wiki_spaces_set_default` → update default immediately

This is simpler than file watching and avoids race conditions.

## Implementation considerations

### Shared state

The wiki engine holds a map of mounted wikis (name → wiki handle).
This needs to be behind a `RwLock` or similar:

```rust
struct Engine {
    wikis: RwLock<HashMap<String, WikiHandle>>,
    default_wiki: RwLock<String>,
}
```

Read path (search, list, read) takes a read lock. Mount/unmount
takes a write lock. Contention should be minimal — mount/unmount
is rare.

### Index lifecycle

- Mount: open or create the tantivy index at
  `~/.llm-wiki/indexes/<name>/`
- Unmount: close the index reader/writer handles. Do not delete
  the index files (the wiki might be re-mounted later).
- `wiki_spaces_remove --delete`: also delete index files.

### In-flight requests

What happens to a request targeting a wiki that's being unmounted?

Options:
1. **Reject** — return error "wiki not found" if unmounted mid-request
2. **Complete** — hold a reference to the wiki handle until the
   request finishes (Arc-based)

Option 2 is safer. The wiki handle is `Arc<WikiHandle>`, so
in-flight requests keep it alive even after unmount removes it from
the map.

### File watching vs tool-triggered

| Approach | Pros | Cons |
|----------|------|------|
| File watching | Picks up external edits to config.toml | Needs a watcher (notify crate), debouncing, race conditions |
| Tool-triggered | Simple, no watcher, no races | Only works for changes made through the engine |
| Both | Covers all cases | More complexity |

Recommendation: start with tool-triggered (simpler), add file
watching later if needed.

### Transport impact

All transports share the same engine. A reload is transparent to
transports — they call into the engine, which resolves the wiki
name at request time from the current map.

No transport restart needed. No session interruption.

## Interaction with existing features

### Cross-wiki search

`wiki_search(cross_wiki: true)` iterates all mounted wikis. After
hot reload, it sees the updated set immediately.

### wiki:// URI resolution

`wiki://research/concepts/moe` resolves the wiki name from the
mounted map. If the wiki was unmounted, the URI fails to resolve
with a clear error.

### Index staleness

A newly mounted wiki may have a stale or missing index. Apply the
same staleness check as startup:
- `index.auto_rebuild: true` → rebuild silently
- `index.auto_rebuild: false` → warn

## Open questions

- Should unmounting a wiki that is the current default be allowed?
  Currently `wiki_spaces_remove` refuses. Same rule for hot reload?
- Should there be a `wiki_spaces_reload` tool for explicit reload,
  or is tool-triggered sufficient?
- Should the server emit an MCP notification on reload so agents
  know the wiki set changed?
- Performance: is `RwLock<HashMap>` sufficient, or does the read
  path need to be lock-free (e.g. `arc-swap`)?

## Success criteria

- `wiki_spaces_create` from a running server makes the new wiki
  immediately searchable without restart
- `wiki_spaces_remove` from a running server unmounts the wiki
  immediately
- In-flight requests to a removed wiki complete without error
- Cross-wiki search reflects the updated wiki set
- No transport restart or session interruption
- Existing tests pass unchanged
