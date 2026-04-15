//! Schema tests — Phase 0
//!
//! Covers: [`llm_wiki::analysis::Analysis`] JSON round-trip,
//! [`llm_wiki::markdown::PageFrontmatter`] YAML round-trip, and
//! [`llm_wiki::config::WikiConfig`] TOML loading.
//!
//! Note: inline `#[cfg(test)]` modules in `analysis.rs`, `markdown.rs`, and
//! `config.rs` already cover these cases. The stubs below track the phase-0
//! checklist and serve as the integration-level reference point.

#[test]
#[ignore = "phase 0 — stub"]
fn analysis_json_round_trip() {
    todo!()
}

#[test]
#[ignore = "phase 0 — stub"]
fn page_frontmatter_yaml_round_trip() {
    todo!()
}

#[test]
#[ignore = "phase 0 — stub"]
fn wiki_config_load_from_toml() {
    todo!()
}
