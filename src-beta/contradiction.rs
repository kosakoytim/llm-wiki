//! Contradiction page management — list, filter, and summarise contradiction nodes.

use crate::analysis::{Dimension, Status};
use crate::graph::WikiGraph;
use anyhow::{anyhow, Result};
use petgraph::Direction;
use serde::Deserialize;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;

// ── Frontmatter schema ────────────────────────────────────────────────────────

/// YAML frontmatter for contradiction pages as written by `integrate.rs`.
///
/// Mirrors [`crate::integrate::ContradictionFrontmatter`] (which is private).
/// `related_concepts` and `resolution` use `#[serde(default)]` because they
/// may be absent in pages written before those fields were introduced.
#[derive(Debug, Deserialize)]
struct ContradictionPageFrontmatter {
    title: String,
    source_a: String,
    source_b: String,
    dimension: Dimension,
    status: Status,
    #[serde(default)]
    related_concepts: Vec<String>,
    // remaining fields are present but not needed for summaries
    #[allow(dead_code)]
    #[serde(default)]
    resolution: Option<String>,
}

/// Extract the raw YAML block from a frontmatter-delimited file.
fn yaml_block(content: &str) -> Result<&str> {
    let after_open = content
        .strip_prefix("---\n")
        .ok_or_else(|| anyhow!("no frontmatter block"))?;
    let end = after_open
        .find("\n---\n")
        .ok_or_else(|| anyhow!("frontmatter not closed"))?;
    Ok(&after_open[..end])
}

// ── Public types ──────────────────────────────────────────────────────────────

/// A lightweight summary of a contradiction page for listing and lint reports.
#[derive(Debug, Clone, PartialEq)]
pub struct ContradictionSummary {
    /// Page slug (e.g. `contradictions/moe-scaling-efficiency`).
    pub slug: String,
    /// Contradiction title from frontmatter.
    pub title: String,
    /// Current lifecycle status.
    pub status: Status,
    /// The dimension along which the claims contradict.
    pub dimension: Dimension,
    /// Slug of the source page for claim A.
    pub source_a: String,
    /// Slug of the source page for claim B.
    pub source_b: String,
}

// ── Public functions ──────────────────────────────────────────────────────────

/// List all contradiction pages under `wiki_root/contradictions/`, optionally
/// filtered by `status`.
///
/// Pages with unparseable frontmatter are skipped with a warning.
pub fn list(wiki_root: &Path, status: Option<Status>) -> Result<Vec<ContradictionSummary>> {
    let dir = wiki_root.join("contradictions");
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();

    for entry in WalkDir::new(&dir).min_depth(1).max_depth(1) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension() != Some(OsStr::new("md")) {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        let yaml = match yaml_block(&content) {
            Ok(y) => y,
            Err(e) => {
                tracing::warn!("skipping {}: {e}", path.display());
                continue;
            }
        };
        let fm: ContradictionPageFrontmatter = match serde_yaml::from_str(yaml) {
            Ok(fm) => fm,
            Err(e) => {
                tracing::warn!("skipping {}: {e}", path.display());
                continue;
            }
        };

        // Apply status filter.
        if let Some(ref filter_status) = status {
            if &fm.status != filter_status {
                continue;
            }
        }

        let slug = path
            .strip_prefix(wiki_root)
            .map_err(|e| anyhow!("path strip error: {e}"))?
            .with_extension("")
            .to_string_lossy()
            .into_owned();

        results.push(ContradictionSummary {
            slug,
            title: fm.title,
            status: fm.status,
            dimension: fm.dimension,
            source_a: fm.source_a,
            source_b: fm.source_b,
        });
    }

    // Deterministic output order.
    results.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(results)
}

/// Return the slugs of concept pages that are graph-adjacent to the given
/// contradiction `slugs` (i.e. connected by any edge in either direction).
///
/// Contradiction pages themselves are excluded from the result.
pub fn cluster(graph: &WikiGraph, slugs: &[String]) -> Vec<String> {
    let mut connected: HashSet<String> = HashSet::new();

    for slug in slugs {
        if let Some(&idx) = graph.node_map.get(slug.as_str()) {
            // Neighbours via outgoing edges.
            for nb in graph.inner.neighbors_directed(idx, Direction::Outgoing) {
                let nb_slug = &graph.inner[nb];
                if !nb_slug.starts_with("contradictions/") {
                    connected.insert(nb_slug.clone());
                }
            }
            // Neighbours via incoming edges.
            for nb in graph.inner.neighbors_directed(idx, Direction::Incoming) {
                let nb_slug = &graph.inner[nb];
                if !nb_slug.starts_with("contradictions/") {
                    connected.insert(nb_slug.clone());
                }
            }
        }
    }

    let mut result: Vec<String> = connected.into_iter().collect();
    result.sort();
    result
}
