---
title: "Graph Community Detection"
summary: "Louvain clustering on the existing petgraph DiGraph; community stats in wiki_stats; community-aware secondary signal in wiki_suggest."
status: proposed
last_updated: "2026-04-27"
---

# Graph Community Detection

## Problem

The knowledge graph grows monotonically. Beyond ~50 nodes, global structure
becomes unreadable — every node is connected to many others with no visible
grouping. The current `wiki_stats` reports aggregate metrics (orphan count,
density) but nothing about topology. `wiki_suggest` finds pages by tag overlap
and graph distance but has no awareness of thematic clusters.

Two concrete gaps:

1. **Stats give count, not names.** `orphans: 8` tells you something is wrong
   but not which 8 pages. A list of isolated slugs is immediately actionable.

2. **Suggest misses cross-cluster links.** Strategies 1–3 in `suggest.rs`
   (tag overlap, 2-hop neighborhood, BM25 similarity) all work within the
   local neighborhood. Two concept clusters that share a domain but have no
   explicit links are invisible to them. Community membership finds these
   structurally distant but thematically related pages.

## Goals

- Compute knowledge clusters automatically from graph structure, with no
  manual taxonomy required.
- Add `communities` to `wiki_stats` output: count, size range, and a named
  list of isolated slugs.
- Add a community-aware strategy 4 to `wiki_suggest`: pages in the same
  community not already in the result set.
- No new external dependencies: Louvain on the existing `petgraph::DiGraph`
  is ~200 lines of pure Rust.
- Feature is suppressed (returns `None`) below a configurable node threshold
  (default: 30) where the algorithm produces noise, not signal.

## Solution

### Algorithm: Louvain on DiGraph

Louvain maximizes modularity — the fraction of intra-community edges minus
what would be expected in a random graph with the same degree sequence.
It runs in O(n log n) for sparse graphs, which covers realistic wiki sizes.

Louvain is defined for undirected graphs. For a directed `WikiGraph`:
- Symmetrize for the community pass: treat every directed edge `A → B` as
  undirected `A — B` by summing both directions in the adjacency weight.
- Directed edge labels and the original graph structure are preserved; only
  community assignment uses the symmetrized view.

Two-phase iteration:
1. **Phase 1 — node moves**: for each node, compute the modularity gain of
   moving it into each neighboring community. Move to the best gain > 0.
   Repeat until no moves improve modularity.
2. **Phase 2 — contraction**: collapse each community into a supernode, using
   intra-community edge count as the self-loop weight. Build a new graph.
   Repeat from phase 1 on the contracted graph.
3. Stop when a full pass produces no improvement.

For determinism (tests, reproducibility): process nodes in sorted-slug order.
Louvain is sensitive to order — sorted order is not optimal but is stable
across runs on the same graph.

### New types in `src/graph.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityStats {
    pub count: usize,
    pub largest: usize,
    pub smallest: usize,
    pub isolated: Vec<String>,   // slugs in communities of size <= 2
}

/// Assign each node a community id. Returns None when graph has
/// fewer than `min_nodes` nodes (signal unreliable at small scale).
pub fn compute_communities(
    graph: &WikiGraph,
    min_nodes: usize,
) -> Option<CommunityStats> { ... }
```

`isolated` contains slugs from communities of size ≤ 2. This is a superset
of full orphans (size 1): a pair of mutually-linked pages that are connected
to nothing else is also isolated in the community sense.

### `wiki_stats` change

`WikiStats` gains an optional field:

```rust
pub communities: Option<CommunityStats>,
```

`None` when `graph.node_count() < min_nodes_for_communities`. The `stats()`
function already builds the `WikiGraph` — pass it to `compute_communities()`
directly, no extra graph build.

JSON output example:
```json
{
  "nodes": 120,
  "edges": 340,
  "orphans": 8,
  "density": 0.047,
  "communities": {
    "count": 7,
    "largest": 34,
    "smallest": 3,
    "isolated": ["tangent-thought-xyz", "draft-stub-abc"]
  }
}
```

### `wiki_suggest` change

`suggest.rs` already builds the full `WikiGraph` for strategy 2. Add
strategy 4 after the existing three strategies:

```rust
// Strategy 4: Community peers (same Louvain community, not already linked)
if let Some(community_map) = graph::node_community_map(&wiki_graph, min_nodes) {
    if let Some(&my_community) = community_map.get(slug.as_str()) {
        for (node_slug, &community_id) in &community_map {
            if community_id != my_community { continue; }
            if node_slug == slug.as_str() { continue; }
            if existing_links.contains(node_slug.as_str()) { continue; }
            if candidates.contains_key(node_slug.as_str()) { continue; }
            // Add up to community_suggestions_limit extra results
            candidates.insert(node_slug.to_string(), CandidateScore {
                slug: node_slug.to_string(),
                title: ...,
                page_type: ...,
                score: 0.4,
                reason: "same knowledge cluster".to_string(),
            });
        }
    }
}
```

`node_community_map` is a helper that runs `compute_communities` and returns
a `HashMap<String, usize>` mapping slug → community id. Returns `None` below
`min_nodes`.

The community strategy score (0.4) intentionally sits between 2-hop
neighborhood (0.5) and tag overlap (variable). Community peers are less
certain than direct neighbors but more specific than BM25 similarity.

### Configuration

```toml
# config.toml (global default) or wiki.toml (per-wiki override)
[graph]
min_nodes_for_communities   = 30   # suppress below this threshold
community_suggestions_limit = 2    # extra results from community strategy
```

Resolution chain: `CLI flag → wiki.toml [graph] → config.toml [graph] → built-in default`.

These fields extend the existing `[graph]` section (if one exists) or create
it. They are read once per `wiki_stats` / `wiki_suggest` call from
`ResolvedConfig`.

### Determinism note

The sorted-slug processing order makes community assignments deterministic
for tests. Real-world assignments will shift as content is added (Louvain is
sensitive to graph structure). This is intentional: clusters reflect current
knowledge topology, not a frozen taxonomy.

## Values

| Value | Mechanism |
|---|---|
| Named isolation list | `isolated` in `CommunityStats` surfaces slugs, not just a count |
| Organic taxonomy | Clusters emerge from link structure, shift as content evolves |
| Better suggestions | Strategy 4 finds thematically related pages across structural distance |
| No new dependencies | Louvain implemented over existing `petgraph` |
| Graceful degradation | `None` below threshold — small wikis see no change |

## Tasks

### `src/graph.rs`

- [ ] Add `CommunityStats { count, largest, smallest, isolated }` struct
  (derives `Serialize`, `Deserialize`, `Debug`, `Clone`).
- [ ] Add `fn build_undirected_adjacency(graph: &WikiGraph) -> HashMap<NodeIndex, HashSet<NodeIndex>>`
  symmetrizing the directed graph for the community pass.
- [ ] Implement Louvain phase 1: `fn louvain_phase1(adj: &mut ..., community: &mut Vec<usize>, degrees: &[usize], m: usize) -> bool` — returns true if any node moved.
- [ ] Implement Louvain phase 2: contract communities into supernodes.
- [ ] Add `pub fn compute_communities(graph: &WikiGraph, min_nodes: usize) -> Option<CommunityStats>` — returns `None` below threshold; processes nodes in sorted-slug order for determinism.
- [ ] Add `pub fn node_community_map(graph: &WikiGraph, min_nodes: usize) -> Option<HashMap<String, usize>>` — helper for `suggest.rs`; maps slug → community id.

### `src/ops/stats.rs`

- [ ] Add `pub communities: Option<CommunityStats>` to `WikiStats`.
- [ ] After `graph::compute_metrics(&wiki_graph)`, call
  `graph::compute_communities(&wiki_graph, resolved.graph.min_nodes_for_communities)`
  and assign to `WikiStats::communities`.

### `src/ops/suggest.rs`

- [ ] After strategy 3, add strategy 4: call `graph::node_community_map()`
  on the already-built `wiki_graph`; add community peers not already in
  `candidates` up to `resolved.graph.community_suggestions_limit` extra entries
  with `score: 0.4` and `reason: "same knowledge cluster"`.

### `src/config.rs`

- [ ] Add `min_nodes_for_communities: usize` (default `30`) and
  `community_suggestions_limit: usize` (default `2`) to the `GraphConfig`
  struct (or create `GraphConfig` if it doesn't exist).
- [ ] Wire into `WikiConfig` under `[graph]`; expose via `ResolvedConfig`.

### Spec docs

- [ ] `docs/specifications/tools/stats.md`: document `communities` field
  (`null` below threshold, else `{ count, largest, smallest, isolated }`).
- [ ] `docs/specifications/tools/suggest.md`: document strategy 4 (community
  peers), score 0.4, reason string, and the `community_suggestions_limit`
  config key.
- [ ] `docs/specifications/model/global-config.md`: add `[graph]` section with
  `min_nodes_for_communities` and `community_suggestions_limit`.
- [ ] `docs/specifications/model/wiki-toml.md`: add `[graph]` to per-wiki
  overridable settings.

### Skill — `llm-wiki-skills/skills/graph/SKILL.md`

- [ ] In `## Interpret the graph`, replace the manual "Clusters" and
  "Isolated nodes" bullet points with guidance to call `wiki_stats()` first
  and read the `communities` field: `count`, `largest`, `smallest`, and the
  named `isolated` list. Manual graph inspection for clusters is only needed
  when `communities` is `null` (wiki below threshold).
- [ ] Add a `## Community insights` section after `## Interpret the graph`:
  ```
  ## Community insights

  When the wiki has enough pages (≥ 30), `wiki_stats()` includes a
  `communities` field:

  - `count` — number of distinct knowledge clusters
  - `largest` / `smallest` — size range of clusters
  - `isolated` — slugs in communities of size ≤ 2; candidates for
    consolidation or new links

  Use the isolated list as a prioritized review queue — these pages are
  structurally disconnected from the main knowledge body.

  Cross-cluster suggestions (pages in the same Louvain community that are
  not directly linked) appear in `wiki_suggest` with
  `reason: "same knowledge cluster"`. These are the highest-value link
  candidates: thematically related but not yet connected.
  ```
- [ ] Update `metadata.version` to `0.3.0` and `last_updated` to the release date.

### Tests

- [ ] Unit test `compute_communities`: graph with 3 clear clusters of 10 nodes
  each (dense intra, sparse inter) → `count: 3`, no isolated.
- [ ] Unit test `isolated`: 2 nodes connected only to each other, rest of graph
  dense → those 2 appear in `isolated`.
- [ ] Unit test `min_nodes` threshold: graph with 29 nodes → `compute_communities`
  returns `None`.
- [ ] `wiki_suggest` test: input page in community A; community B has a page
  with no path to input within 2 hops; assert community B page appears in
  results with `reason: "same knowledge cluster"`.
- [ ] Determinism test: run `compute_communities` twice on same graph → identical
  `isolated` list and `count`.
