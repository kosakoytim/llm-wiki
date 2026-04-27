use std::cmp::Reverse;
use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::Result;
use chrono::Utc;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use tantivy::Searcher;
use tantivy::collector::TopDocs;
use tantivy::query::AllQuery;
use tantivy::schema::Value;

use crate::index_schema::IndexSchema;
use crate::type_registry::SpaceTypeRegistry;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageNode {
    pub slug: String,
    pub title: String,
    pub r#type: String,
}

#[derive(Debug, Clone)]
pub struct LabeledEdge {
    pub relation: String,
}

pub type WikiGraph = DiGraph<PageNode, LabeledEdge>;

#[derive(Debug, Clone, Default)]
pub struct GraphFilter {
    pub root: Option<String>,
    pub depth: Option<usize>,
    pub types: Vec<String>,
    pub relation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphReport {
    pub nodes: usize,
    pub edges: usize,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub nodes: usize,
    pub edges: usize,
    pub orphans: usize,
    pub avg_connections: f64,
    pub density: f64,
}

/// Compute health metrics from a built graph.
pub fn compute_metrics(graph: &WikiGraph) -> GraphMetrics {
    let nodes = graph.node_count();
    let edges = graph.edge_count();

    let orphans = graph
        .node_indices()
        .filter(|&idx| {
            graph.neighbors_directed(idx, Direction::Incoming).count() == 0
                && graph.neighbors_directed(idx, Direction::Outgoing).count() == 0
        })
        .count();

    let avg_connections = if nodes > 0 {
        (edges as f64 * 2.0) / nodes as f64
    } else {
        0.0
    };

    let density = if nodes > 1 {
        edges as f64 / (nodes as f64 * (nodes as f64 - 1.0))
    } else {
        0.0
    };

    GraphMetrics {
        nodes,
        edges,
        orphans,
        avg_connections,
        density,
    }
}

// ── build_graph ───────────────────────────────────────────────────────────────

/// Build the concept graph from the tantivy index. No file I/O.
/// Edge relations are read from  declarations in the
/// type registry. Body  get a generic  relation.
pub fn build_graph(
    searcher: &Searcher,
    is: &IndexSchema,
    filter: &GraphFilter,
    registry: &SpaceTypeRegistry,
) -> Result<WikiGraph> {
    let f_slug = is.field("slug");
    let f_title = is.field("title");
    let f_type = is.field("type");
    let f_body_links = is.field("body_links");

    let top_docs = searcher.search(&AllQuery, &TopDocs::with_limit(100_000).order_by_score())?;

    let mut graph = WikiGraph::new();
    let mut slug_to_idx: HashMap<String, NodeIndex> = HashMap::new();

    struct DocInfo {
        slug: String,
        page_type: String,
        body_links: Vec<String>,
        edge_fields: Vec<(String, Vec<String>)>, // (field_name, target_slugs)
    }
    let mut all_docs: Vec<DocInfo> = Vec::new();

    // First pass: create nodes and collect edge data
    for (_score, doc_addr) in &top_docs {
        let doc: tantivy::TantivyDocument = searcher.doc(*doc_addr)?;

        let slug = doc
            .get_first(f_slug)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let title = doc
            .get_first(f_title)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let page_type = doc
            .get_first(f_type)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if !filter.types.is_empty() && !filter.types.contains(&page_type) {
            continue;
        }

        let node = PageNode {
            slug: slug.clone(),
            title,
            r#type: page_type.clone(),
        };
        let idx = graph.add_node(node);
        slug_to_idx.insert(slug.clone(), idx);

        // Read body wiki-links
        let body_links: Vec<String> = doc
            .get_all(f_body_links)
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        // Read declared edge fields from the index
        let mut edge_fields = Vec::new();
        for decl in registry.edges(&page_type) {
            if let Some(field_handle) = is.try_field(&decl.field) {
                let targets: Vec<String> = doc
                    .get_all(field_handle)
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if !targets.is_empty() {
                    edge_fields.push((decl.field.clone(), targets));
                }
            }
        }

        all_docs.push(DocInfo {
            slug,
            page_type,
            body_links,
            edge_fields,
        });
    }

    // Second pass: add edges
    for doc_info in &all_docs {
        let from_idx = match slug_to_idx.get(&doc_info.slug) {
            Some(idx) => *idx,
            None => continue,
        };

        // Declared edges (from x-graph-edges)
        let edge_decls = registry.edges(&doc_info.page_type);
        for (field_name, targets) in &doc_info.edge_fields {
            let relation = edge_decls
                .iter()
                .find(|d| d.field == *field_name)
                .map(|d| d.relation.as_str())
                .unwrap_or("links-to");

            if filter.relation.is_some() && filter.relation.as_deref() != Some(relation) {
                continue;
            }

            for target in targets {
                if let Some(&to_idx) = slug_to_idx.get(target)
                    && from_idx != to_idx
                {
                    graph.add_edge(
                        from_idx,
                        to_idx,
                        LabeledEdge {
                            relation: relation.to_string(),
                        },
                    );
                }
            }
        }

        // Body wiki-links → "links-to"
        if filter.relation.is_none() || filter.relation.as_deref() == Some("links-to") {
            for target in &doc_info.body_links {
                if let Some(&to_idx) = slug_to_idx.get(target)
                    && from_idx != to_idx
                {
                    graph.add_edge(
                        from_idx,
                        to_idx,
                        LabeledEdge {
                            relation: "links-to".into(),
                        },
                    );
                }
            }
        }
    }

    // Apply root + depth filter
    if let Some(ref root_slug) = filter.root {
        return Ok(subgraph(&graph, root_slug, filter.depth.unwrap_or(3)));
    }

    Ok(graph)
}

// ── render_llms ───────────────────────────────────────────────────────────────

/// Natural language description of graph structure for direct LLM consumption.
pub fn render_llms(graph: &WikiGraph) -> String {
    let nodes = graph.node_count();
    let edges = graph.edge_count();

    // Group nodes by type
    let mut by_type: HashMap<String, Vec<String>> = HashMap::new();
    for idx in graph.node_indices() {
        let node = &graph[idx];
        by_type
            .entry(node.r#type.clone())
            .or_default()
            .push(node.title.clone());
    }

    // Sort type groups by count descending
    let mut type_groups: Vec<(String, Vec<String>)> = by_type.into_iter().collect();
    type_groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(&b.0)));

    // Count edge relations
    let mut relation_counts: HashMap<String, usize> = HashMap::new();
    for edge in graph.edge_indices() {
        *relation_counts
            .entry(graph[edge].relation.clone())
            .or_default() += 1;
    }
    let mut relations: Vec<(String, usize)> = relation_counts.into_iter().collect();
    relations.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    // Compute per-node total degree for hub detection
    let mut degree: Vec<(usize, String)> = graph
        .node_indices()
        .map(|idx| {
            let d = graph.neighbors_directed(idx, Direction::Incoming).count()
                + graph.neighbors_directed(idx, Direction::Outgoing).count();
            (d, graph[idx].title.clone())
        })
        .collect();
    degree.sort_by_key(|a| Reverse(a.0));
    let top_hubs: Vec<String> = degree
        .iter()
        .take(5)
        .filter(|(d, _)| *d > 0)
        .map(|(d, t)| format!("{t} ({d} edges)"))
        .collect();

    // Isolated nodes (no edges at all)
    let isolated: Vec<String> = graph
        .node_indices()
        .filter(|&idx| {
            graph.neighbors_directed(idx, Direction::Incoming).count() == 0
                && graph.neighbors_directed(idx, Direction::Outgoing).count() == 0
        })
        .map(|idx| graph[idx].title.clone())
        .collect();

    let cluster_count = type_groups.len();

    let mut out = String::new();
    out.push_str(&format!(
        "The wiki graph has {nodes} nodes and {edges} edges across {cluster_count} type groups.\n\n"
    ));

    for (type_name, mut titles) in type_groups {
        titles.sort();
        let count = titles.len();
        let sample = if titles.len() > 8 {
            format!("{}, ...", titles[..8].join(", "))
        } else {
            titles.join(", ")
        };
        out.push_str(&format!("**{type_name}** ({count} nodes): {sample}\n"));
    }

    if !top_hubs.is_empty() {
        out.push_str(&format!("\nKey hubs: {}\n", top_hubs.join(", ")));
    }

    if !relations.is_empty() {
        out.push_str("\n**Edges by relation:**\n");
        for (rel, count) in &relations {
            out.push_str(&format!("- `{rel}` ({count})\n"));
        }
    }

    if !isolated.is_empty() {
        out.push_str(&format!(
            "\n**Isolated nodes ({}):** {}\n",
            isolated.len(),
            isolated.join(", ")
        ));
    }

    out
}

// ── render_mermaid ────────────────────────────────────────────────────────────

pub fn render_mermaid(graph: &WikiGraph) -> String {
    let mut out = String::from("graph LR\n");

    // Collect unique types for classDef
    let mut types_seen: HashSet<String> = HashSet::new();

    // Declare nodes with titles and type classes
    for idx in graph.node_indices() {
        let node = &graph[idx];
        let safe_slug = mermaid_id(&node.slug);
        out.push_str(&format!(
            "  {safe_slug}[\"{}\"]:::{}\n",
            node.title, node.r#type
        ));
        types_seen.insert(node.r#type.clone());
    }

    out.push('\n');

    // Edges with relation labels
    for edge in graph.edge_indices() {
        let (from, to) = graph.edge_endpoints(edge).unwrap();
        let from_id = mermaid_id(&graph[from].slug);
        let to_id = mermaid_id(&graph[to].slug);
        let relation = &graph[edge].relation;
        out.push_str(&format!("  {from_id} -->|{relation}| {to_id}\n"));
    }

    // classDef for known types
    out.push('\n');
    let type_colors = [
        ("concept", "#cce5ff"),
        ("query-result", "#cce5ff"),
        ("paper", "#d4edda"),
        ("article", "#d4edda"),
        ("documentation", "#d4edda"),
        ("skill", "#ffeeba"),
        ("doc", "#e2e3e5"),
        ("section", "#f8f9fa"),
    ];
    for (t, color) in &type_colors {
        if types_seen.contains(*t) {
            out.push_str(&format!("  classDef {t} fill:{color}\n"));
        }
    }

    out
}

fn mermaid_id(slug: &str) -> String {
    slug.replace(['/', '-'], "_")
}

// ── render_dot ────────────────────────────────────────────────────────────────

pub fn render_dot(graph: &WikiGraph) -> String {
    let mut out = String::from("digraph wiki {\n");

    for idx in graph.node_indices() {
        let node = &graph[idx];
        out.push_str(&format!(
            "  \"{}\" [label=\"{}\" type=\"{}\"];\n",
            node.slug, node.title, node.r#type
        ));
    }

    for edge in graph.edge_indices() {
        let (from, to) = graph.edge_endpoints(edge).unwrap();
        let relation = &graph[edge].relation;
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            graph[from].slug, graph[to].slug, relation
        ));
    }

    out.push_str("}\n");
    out
}

// ── wrap_graph_md ─────────────────────────────────────────────────────────────

pub fn wrap_graph_md(rendered: &str, format: &str, filter: &GraphFilter) -> String {
    let now = Utc::now().to_rfc3339();
    let root = filter.root.as_deref().unwrap_or("");
    let depth = filter.depth.unwrap_or(0);
    let types = if filter.types.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", filter.types.join(", "))
    };

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("title: \"Wiki Graph\"\n");
    out.push_str(&format!("generated: \"{now}\"\n"));
    out.push_str(&format!("format: {format}\n"));
    out.push_str(&format!("root: {root}\n"));
    out.push_str(&format!("depth: {depth}\n"));
    out.push_str(&format!("types: {types}\n"));
    out.push_str("status: generated\n");
    out.push_str("---\n\n");
    out.push_str(&format!("```{format}\n"));
    out.push_str(rendered);
    out.push_str("```\n");
    out
}

// ── subgraph ──────────────────────────────────────────────────────────────────

pub fn subgraph(graph: &WikiGraph, root_slug: &str, depth: usize) -> WikiGraph {
    let root_idx = match graph
        .node_indices()
        .find(|&idx| graph[idx].slug == root_slug)
    {
        Some(idx) => idx,
        None => return WikiGraph::new(),
    };

    let mut visited: HashSet<NodeIndex> = HashSet::new();
    let mut queue: VecDeque<(NodeIndex, usize)> = VecDeque::new();
    queue.push_back((root_idx, 0));
    visited.insert(root_idx);

    while let Some((node, d)) = queue.pop_front() {
        if d >= depth {
            continue;
        }
        for neighbor in graph.neighbors_directed(node, Direction::Outgoing) {
            if visited.insert(neighbor) {
                queue.push_back((neighbor, d + 1));
            }
        }
        for neighbor in graph.neighbors_directed(node, Direction::Incoming) {
            if visited.insert(neighbor) {
                queue.push_back((neighbor, d + 1));
            }
        }
    }

    let mut new_graph = WikiGraph::new();
    let mut old_to_new: HashMap<NodeIndex, NodeIndex> = HashMap::new();

    for &old_idx in &visited {
        let new_idx = new_graph.add_node(graph[old_idx].clone());
        old_to_new.insert(old_idx, new_idx);
    }

    for edge in graph.edge_indices() {
        let (from, to) = graph.edge_endpoints(edge).unwrap();
        if let (Some(&new_from), Some(&new_to)) = (old_to_new.get(&from), old_to_new.get(&to)) {
            new_graph.add_edge(new_from, new_to, graph[edge].clone());
        }
    }

    new_graph
}
