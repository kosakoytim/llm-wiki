//! Structural lint pass — orphan pages, missing concept stubs, active contradictions.
//!
//! `wiki lint` writes a `LINT.md` report and commits it. The external LLM reads
//! the report and re-ingests enriched analysis as needed.

use crate::analysis::Status;
use crate::contradiction::{list as list_contradictions, ContradictionSummary};
use crate::graph::{build_graph, missing_stubs, orphans};
use anyhow::{Context, Result};
use std::path::Path;

// ── Public types ──────────────────────────────────────────────────────────────

/// Output of a lint pass.
pub struct LintReport {
    /// Pages with no incoming links (petgraph in-degree = 0).
    pub orphan_pages: Vec<String>,
    /// Concept slugs referenced from other pages but not yet created on disk.
    pub missing_stubs: Vec<String>,
    /// Contradiction pages whose status is `active` or `under-analysis`.
    pub active_contradictions: Vec<ContradictionSummary>,
}

// ── write_lint_report ─────────────────────────────────────────────────────────

/// Write a `LINT.md` report to `wiki_root/LINT.md`.
///
/// The report has three sections:
/// - `## Orphans` — pages with no inbound links
/// - `## Missing Stubs` — slugs referenced but not yet created
/// - `## Active Contradictions` — table with slug, title, dimension, sources
pub fn write_lint_report(wiki_root: &Path, report: &LintReport) -> Result<()> {
    let mut md = String::new();

    md.push_str("# Lint Report\n\n");
    md.push_str(&format!(
        "_Generated: {}_\n\n",
        chrono::Utc::now().format("%Y-%m-%d")
    ));

    // ── Orphans ──────────────────────────────────────────────────────────────
    md.push_str("## Orphans\n\n");
    md.push_str("Pages with no inbound links. Consider adding cross-references or removing them.\n\n");
    if report.orphan_pages.is_empty() {
        md.push_str("_None._\n\n");
    } else {
        for slug in &report.orphan_pages {
            md.push_str(&format!("- `{slug}`\n"));
        }
        md.push('\n');
    }

    // ── Missing Stubs ────────────────────────────────────────────────────────
    md.push_str("## Missing Stubs\n\n");
    md.push_str("Slugs referenced by other pages but not yet created on disk.\n\n");
    if report.missing_stubs.is_empty() {
        md.push_str("_None._\n\n");
    } else {
        for slug in &report.missing_stubs {
            md.push_str(&format!("- `{slug}`\n"));
        }
        md.push('\n');
    }

    // ── Active Contradictions ────────────────────────────────────────────────
    md.push_str("## Active Contradictions\n\n");
    md.push_str("Contradiction pages awaiting enrichment (`active` or `under-analysis`).\n\n");
    if report.active_contradictions.is_empty() {
        md.push_str("_None._\n\n");
    } else {
        md.push_str("| Slug | Title | Dimension | Source A | Source B |\n");
        md.push_str("|------|-------|-----------|----------|----------|\n");
        for c in &report.active_contradictions {
            let dim = format!("{:?}", c.dimension).to_lowercase();
            md.push_str(&format!(
                "| `{}` | {} | {} | `{}` | `{}` |\n",
                c.slug, c.title, dim, c.source_a, c.source_b
            ));
        }
        md.push('\n');
    }

    // Summary footer.
    md.push_str(&format!(
        "---\n\n_{} orphan(s), {} missing stub(s), {} active contradiction(s)._\n",
        report.orphan_pages.len(),
        report.missing_stubs.len(),
        report.active_contradictions.len()
    ));

    std::fs::write(wiki_root.join("LINT.md"), md)
        .context("failed to write LINT.md")?;

    Ok(())
}

// ── lint ──────────────────────────────────────────────────────────────────────

/// Run a structural lint pass on the wiki at `wiki_root`.
///
/// 1. Builds the concept graph.
/// 2. Collects orphan pages, missing stubs, and active contradictions.
/// 3. Writes `LINT.md`.
/// 4. Commits `LINT.md` with a summary message.
///
/// Returns the [`LintReport`] so callers can display a summary.
pub fn lint(wiki_root: &Path) -> Result<LintReport> {
    let graph = build_graph(wiki_root).context("failed to build concept graph")?;

    let orphan_pages = orphans(&graph);
    let missing_stubs = missing_stubs(&graph, wiki_root);

    // Active = status is `active` OR `under-analysis`.
    let all = list_contradictions(wiki_root, None)
        .context("failed to list contradiction pages")?;
    let active_contradictions: Vec<ContradictionSummary> = all
        .into_iter()
        .filter(|c| c.status == Status::Active || c.status == Status::UnderAnalysis)
        .collect();

    let report = LintReport {
        orphan_pages,
        missing_stubs,
        active_contradictions,
    };

    write_lint_report(wiki_root, &report)?;

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let message = format!(
        "lint: {} \u{2014} {} orphans, {} stubs, {} active contradictions",
        date,
        report.orphan_pages.len(),
        report.missing_stubs.len(),
        report.active_contradictions.len()
    );
    crate::git::commit(wiki_root, &message).context("failed to commit LINT.md")?;

    Ok(report)
}
