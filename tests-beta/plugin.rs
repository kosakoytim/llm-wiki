//! Claude plugin and `wiki init` tests — Phase 5
//!
//! Covers: `wiki instruct` output completeness for all six workflows and
//! `wiki init` directory structure creation.

use llm_wiki::init::init_wiki;
use tempfile::TempDir;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extract a named `## {name}-workflow` section from `instructions.md`.
fn extract_section(name: &str) -> String {
    let instructions = include_str!("../src/instructions.md");
    let header = format!("## {name}-workflow");
    let start = instructions
        .find(header.as_str())
        .unwrap_or_else(|| panic!("section '{}' not found in instructions.md", name));
    let section = &instructions[start..];
    let end = section[header.len()..]
        .find("\n## ")
        .map(|pos| header.len() + pos)
        .unwrap_or(section.len());
    section[..end].to_string()
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[test]
fn wiki_instruct_help_contains_all_six_command_names() {
    let section = extract_section("help");
    for cmd in &["help", "init", "ingest", "research", "lint", "contradiction"] {
        assert!(
            section.contains(cmd),
            "help-workflow should mention command '{}': {}",
            cmd,
            section
        );
    }
}

#[test]
fn wiki_instruct_ingest_contains_analysis_json() {
    let section = extract_section("ingest");
    assert!(
        section.contains("analysis.json"),
        "ingest-workflow should mention analysis.json: {section}"
    );
}

#[test]
fn wiki_instruct_ingest_contains_two_step_workflow() {
    let section = extract_section("ingest");
    assert!(
        section.contains("wiki_context"),
        "ingest-workflow should mention wiki_context (step 1): {section}"
    );
    assert!(
        section.contains("wiki_ingest"),
        "ingest-workflow should mention wiki_ingest (step 2): {section}"
    );
}

#[test]
fn wiki_instruct_research_contains_wiki_context() {
    let section = extract_section("research");
    assert!(
        section.contains("wiki_context"),
        "research-workflow should mention wiki_context: {section}"
    );
}

#[test]
fn wiki_instruct_lint_contains_lint_md() {
    let section = extract_section("lint");
    assert!(
        section.contains("LINT.md"),
        "lint-workflow should mention LINT.md: {section}"
    );
}

#[test]
fn wiki_instruct_contradiction_contains_epistemic_value() {
    let section = extract_section("contradiction");
    assert!(
        section.contains("epistemic_value"),
        "contradiction-workflow should mention epistemic_value: {section}"
    );
}

#[test]
fn wiki_init_creates_required_directory_structure() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    init_wiki(root).expect("init_wiki should succeed");

    for subdir in &["concepts", "sources", "contradictions", "queries", "raw"] {
        assert!(
            root.join(subdir).is_dir(),
            "directory '{}' should exist after wiki init",
            subdir
        );
    }
    let config_path = root.join(".wiki").join("config.toml");
    assert!(
        config_path.exists(),
        ".wiki/config.toml should exist after wiki init"
    );
}

#[test]
fn wiki_init_existing_git_repo_no_error_creates_missing_dirs() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    // Pre-create .git to simulate an existing git repo (init_if_needed skips it).
    std::fs::create_dir(root.join(".git")).expect("create .git");

    // init_wiki must succeed even when .git/ already exists.
    init_wiki(root).expect("init_wiki should succeed on existing git repo");

    for subdir in &["concepts", "sources", "contradictions", "queries", "raw"] {
        assert!(
            root.join(subdir).is_dir(),
            "directory '{}' should exist after wiki init on existing repo",
            subdir
        );
    }
}
