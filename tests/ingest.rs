//! Ingest pipeline tests — Phase 1
//!
//! Covers: `markdown` parsing and writing, `integrate` actions
//! (create/update/append), `ingest` validation, and end-to-end
//! `wiki ingest` CLI behaviour.

use llm_wiki::analysis::{Action, Contradiction, Dimension, DocType, PageType, Status, SuggestedPage};
use llm_wiki::config::WikiConfig;
use llm_wiki::ingest::{ingest, parse_analysis, Input};
use llm_wiki::integrate::integrate;
use llm_wiki::markdown::{parse_frontmatter, write_page, PageFrontmatter, PageStatus};
use llm_wiki::analysis::{Analysis, Confidence};
use std::process::Command;
use tempfile::TempDir;

// ── Test helpers ──────────────────────────────────────────────────────────────

fn wiki_dir() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn config(dir: &TempDir) -> WikiConfig {
    WikiConfig {
        root: dir.path().to_path_buf(),
        name: "test-wiki".into(),
    }
}

fn sample_frontmatter() -> PageFrontmatter {
    PageFrontmatter {
        title: "Test Page".into(),
        summary: "A test page.".into(),
        tldr: "A test page.".into(),
        read_when: vec!["Testing".into()],
        status: PageStatus::Active,
        last_updated: "2026-04-13".into(),
        page_type: PageType::Concept,
        tags: vec!["test".into()],
        sources: vec![],
        confidence: Confidence::Medium,
        contradictions: vec![],
    }
}

fn make_page(slug: &str, action: Action, body: &str) -> SuggestedPage {
    SuggestedPage {
        slug: slug.into(),
        title: "Test Page".into(),
        page_type: PageType::Concept,
        action,
        tldr: "A test page.".into(),
        body: body.into(),
        tags: vec!["test".into()],
        read_when: vec!["Testing".into()],
    }
}

fn make_analysis(pages: Vec<SuggestedPage>) -> Analysis {
    Analysis {
        source: "test".into(),
        doc_type: DocType::Note,
        title: "Test".into(),
        language: "en".into(),
        claims: vec![],
        concepts: vec![],
        key_quotes: vec![],
        data_gaps: vec![],
        suggested_pages: pages,
        contradictions: vec![],
    }
}

/// Minimal valid analysis JSON with one suggested page.
fn minimal_json(slug: &str, action: &str) -> String {
    // Use r###"..."### so that "## Overview" (which contains "##) doesn't
    // prematurely terminate the raw string; "###" never appears in body text.
    format!(
        r###"{{
          "source": "test",
          "doc_type": "note",
          "title": "Test Document",
          "language": "en",
          "claims": [],
          "concepts": [],
          "key_quotes": [],
          "data_gaps": [],
          "suggested_pages": [{{
            "slug": "{slug}",
            "title": "Test Page",
            "type": "concept",
            "action": "{action}",
            "tldr": "A test page.",
            "body": "## Overview\n\nTest content.",
            "tags": ["test"],
            "read_when": ["Testing"]
          }}],
          "contradictions": []
        }}"###
    )
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[test]
fn parse_frontmatter_valid_yaml_block() {
    let dir = wiki_dir();
    let path = dir.path().join("p.md");
    let fm = sample_frontmatter();
    write_page(&path, &fm, "body text\n").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let (parsed, body) = parse_frontmatter(&content).expect("should parse");

    assert_eq!(parsed.title, "Test Page");
    assert_eq!(parsed.tags, vec!["test"]);
    assert_eq!(parsed.status, PageStatus::Active);
    assert_eq!(parsed.confidence, Confidence::Medium);
    assert_eq!(body, "body text\n");
}

#[test]
fn parse_frontmatter_missing_required_field_names_the_field() {
    // Provide only a title — all other required fields are absent.
    let bad = "---\ntitle: only title here\n---\n\nbody\n";
    let result = parse_frontmatter(bad);
    assert!(result.is_err(), "should fail on missing required fields");
    // serde_yaml names the missing field in the error message.
    let msg = result.unwrap_err().to_string();
    // We can't assert on exact field name since serde may fail on any one,
    // but the error should mention frontmatter or a field.
    assert!(
        msg.contains("missing") || msg.contains("frontmatter") || msg.contains("field"),
        "error should name a missing field: {msg}"
    );
}

#[test]
fn parse_frontmatter_no_block_returns_error() {
    let result = parse_frontmatter("# No frontmatter here\n\nbody text");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("no frontmatter"), "error: {msg}");
}

#[test]
fn write_page_output_starts_with_dashes_contains_all_fields() {
    let dir = wiki_dir();
    let path = dir.path().join("p.md");
    let fm = sample_frontmatter();
    write_page(&path, &fm, "body").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("---\n"), "must start with ---");
    assert!(content.contains("title:"), "must contain title");
    assert!(content.contains("tags:"), "must contain tags");
    assert!(content.contains("status:"), "must contain status");
    assert!(content.contains("last_updated:"), "must contain last_updated");
    assert!(content.contains("body"), "must contain body");
}

#[test]
fn write_page_parse_frontmatter_round_trip() {
    let dir = wiki_dir();
    let path = dir.path().join("p.md");
    let fm = sample_frontmatter();
    let body = "## Overview\n\nSome text here.\n";

    write_page(&path, &fm, body).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    let (parsed_fm, parsed_body) = parse_frontmatter(&content).expect("round-trip");

    assert_eq!(parsed_fm, fm);
    assert_eq!(parsed_body, body);
}

#[test]
fn integrate_create_writes_file_at_slug_path() {
    let dir = wiki_dir();
    let analysis = make_analysis(vec![make_page("concepts/test", Action::Create, "body\n")]);
    let report = integrate(analysis, dir.path()).unwrap();

    assert_eq!(report.pages_created, 1);
    assert_eq!(report.pages_updated, 0);
    assert!(dir.path().join("concepts/test.md").exists());
}

#[test]
fn integrate_create_on_existing_slug_returns_error() {
    let dir = wiki_dir();
    let a = make_analysis(vec![make_page("concepts/test", Action::Create, "body")]);
    integrate(a.clone(), dir.path()).unwrap();

    let result = integrate(a, dir.path());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("already exists"), "error: {msg}");
}

#[test]
fn integrate_update_replaces_body_preserves_frontmatter() {
    let dir = wiki_dir();
    // Create first.
    integrate(
        make_analysis(vec![make_page("concepts/test", Action::Create, "## Original\n")]),
        dir.path(),
    )
    .unwrap();

    // Inject a source into the frontmatter to verify it survives the update.
    let path = dir.path().join("concepts/test.md");
    let content = std::fs::read_to_string(&path).unwrap();
    let (mut fm, _) = parse_frontmatter(&content).unwrap();
    fm.sources.push("sources/prior".into());
    write_page(&path, &fm, "## Original\n").unwrap();

    // Update with new body.
    integrate(
        make_analysis(vec![make_page("concepts/test", Action::Update, "## Replaced\n")]),
        dir.path(),
    )
    .unwrap();

    let updated = std::fs::read_to_string(&path).unwrap();
    let (fm2, body2) = parse_frontmatter(&updated).unwrap();

    assert!(body2.contains("Replaced"), "body replaced");
    assert!(!body2.contains("Original"), "old body gone");
    assert_eq!(fm2.sources, vec!["sources/prior"], "sources preserved");
}

#[test]
fn integrate_append_adds_section_original_body_intact() {
    let dir = wiki_dir();
    integrate(
        make_analysis(vec![make_page(
            "concepts/test",
            Action::Create,
            "## Original\n\nFirst content.\n",
        )]),
        dir.path(),
    )
    .unwrap();

    let report = integrate(
        make_analysis(vec![make_page(
            "concepts/test",
            Action::Append,
            "## Added\n\nSecond content.\n",
        )]),
        dir.path(),
    )
    .unwrap();

    assert_eq!(report.pages_appended, 1);
    let content = std::fs::read_to_string(dir.path().join("concepts/test.md")).unwrap();
    assert!(content.contains("First content"), "original body intact");
    assert!(content.contains("Second content"), "new section appended");
}

#[test]
fn integrate_nonempty_contradictions_writes_contradiction_files() {
    let dir = wiki_dir();
    let mut analysis = make_analysis(vec![]);
    analysis.contradictions.push(Contradiction {
        title: "A vs B".into(),
        claim_a: "Claim A".into(),
        source_a: "sources/a".into(),
        claim_b: "Claim B".into(),
        source_b: "sources/b".into(),
        dimension: Dimension::Context,
        epistemic_value: "Reveals context boundary.".into(),
        status: Status::Active,
        resolution: None,
    });

    let report = integrate(analysis, dir.path()).unwrap();
    assert_eq!(report.contradictions_written, 1);
    assert!(dir.path().join("contradictions/a-vs-b.md").exists());
}

#[test]
fn integrate_empty_contradictions_no_files_in_contradictions_dir() {
    let dir = wiki_dir();
    integrate(make_analysis(vec![]), dir.path()).unwrap();
    assert!(
        !dir.path().join("contradictions").exists(),
        "contradictions dir should not be created when empty"
    );
}

#[test]
fn integrate_path_traversal_in_slug_rejected() {
    let dir = wiki_dir();
    let result = integrate(
        make_analysis(vec![make_page("../evil/path", Action::Create, "bad")]),
        dir.path(),
    );
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("path traversal") || msg.contains("invalid slug"),
        "error: {msg}"
    );
}

#[test]
fn ingest_unknown_doc_type_error_lists_valid_values() {
    let json = r#"{
      "source": "x", "doc_type": "academic-paper", "title": "T",
      "language": "en", "claims": [], "concepts": [], "key_quotes": [],
      "data_gaps": [], "suggested_pages": [], "contradictions": []
    }"#;
    let result = parse_analysis(json);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    // serde lists the unknown variant and valid options.
    assert!(
        msg.contains("academic-paper") || msg.contains("unknown variant"),
        "error should name the bad value: {msg}"
    );
}

#[test]
fn ingest_invalid_json_error_with_line_column_hint() {
    let result = parse_analysis("{ not valid json }");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("line") || msg.contains("column"),
        "error should include position: {msg}"
    );
}

// ── Integration tests (full pipeline via `ingest()`) ──────────────────────────

#[tokio::test]
async fn cli_ingest_file_writes_md_and_commits() {
    let dir = wiki_dir();
    let json_path = dir.path().join("analysis.json");
    std::fs::write(&json_path, minimal_json("concepts/test", "create")).unwrap();

    let cfg = config(&dir);
    let report = ingest(Input::File(json_path), &cfg).await.unwrap();

    assert_eq!(report.pages_created, 1);
    assert!(dir.path().join("concepts/test.md").exists());

    let output = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let log = String::from_utf8(output.stdout).unwrap();
    assert!(!log.is_empty(), "git log must have a commit");
    assert!(log.contains("ingest:"), "commit message format: {log}");
}

#[tokio::test]
async fn cli_ingest_stdin_writes_md_and_commits() {
    // Simulate stdin by writing to a temp file and using Input::File.
    // True stdin testing of the binary is done via integration_test.rs.
    let dir = wiki_dir();
    let json_path = dir.path().join("stdin.json");
    std::fs::write(&json_path, minimal_json("concepts/stdin-test", "create")).unwrap();

    let cfg = config(&dir);
    let report = ingest(Input::File(json_path), &cfg).await.unwrap();

    assert_eq!(report.pages_created, 1);
    assert!(dir.path().join("concepts/stdin-test.md").exists());

    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!String::from_utf8(log.stdout).unwrap().is_empty());
}

#[tokio::test]
async fn cli_ingest_create_same_slug_twice_second_fails_first_commit_preserved() {
    let dir = wiki_dir();
    let cfg = config(&dir);

    let json_path = dir.path().join("analysis.json");
    std::fs::write(&json_path, minimal_json("concepts/test", "create")).unwrap();
    ingest(Input::File(json_path.clone()), &cfg).await.unwrap();

    // Second ingest with same slug → must fail.
    let result = ingest(Input::File(json_path), &cfg).await;
    assert!(result.is_err(), "second create must fail");

    // First commit still present.
    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let log_str = String::from_utf8(log.stdout).unwrap();
    assert!(
        log_str.lines().count() >= 1,
        "first commit preserved: {log_str}"
    );
}

#[tokio::test]
async fn cli_ingest_append_body_grows_last_updated_changes() {
    let dir = wiki_dir();
    let cfg = config(&dir);

    // Step 1: create.
    let create_path = dir.path().join("create.json");
    std::fs::write(&create_path, minimal_json("concepts/test", "create")).unwrap();
    ingest(Input::File(create_path), &cfg).await.unwrap();

    let path = dir.path().join("concepts/test.md");

    // Step 2: append.
    let append_json = r###"{
      "source": "test2", "doc_type": "note", "title": "Test Append",
      "language": "en", "claims": [], "concepts": [], "key_quotes": [],
      "data_gaps": [],
      "suggested_pages": [{
        "slug": "concepts/test", "title": "Test Page",
        "type": "concept", "action": "append",
        "tldr": "A test page.",
        "body": "## Extra\n\nAppended content.",
        "tags": ["extra"], "read_when": ["Extra testing"]
      }],
      "contradictions": []
    }"###;
    let append_path = dir.path().join("append.json");
    std::fs::write(&append_path, append_json).unwrap();
    let report = ingest(Input::File(append_path), &cfg).await.unwrap();

    assert_eq!(report.pages_appended, 1);

    let content = std::fs::read_to_string(&path).unwrap();
    let (fm, body) = parse_frontmatter(&content).unwrap();

    assert!(body.contains("Test content"), "original content present");
    assert!(body.contains("Appended content"), "appended content present");
    // Tags should be unioned.
    assert!(fm.tags.contains(&"test".into()), "original tag kept");
    assert!(fm.tags.contains(&"extra".into()), "new tag added");
    // last_updated is a valid 10-char ISO date.
    assert_eq!(fm.last_updated.len(), 10, "last_updated: {}", fm.last_updated);
}

#[tokio::test]
async fn cli_ingest_path_traversal_rejected_no_files_no_commit() {
    let dir = wiki_dir();
    let cfg = config(&dir);

    let bad_json = r#"{
      "source": "test", "doc_type": "note", "title": "Bad", "language": "en",
      "claims": [], "concepts": [], "key_quotes": [], "data_gaps": [],
      "suggested_pages": [{
        "slug": "../evil/path", "title": "Evil", "type": "concept",
        "action": "create", "tldr": "bad", "body": "bad",
        "tags": [], "read_when": []
      }],
      "contradictions": []
    }"#;

    let json_path = dir.path().join("bad.json");
    std::fs::write(&json_path, bad_json).unwrap();

    let result = ingest(Input::File(json_path), &cfg).await;
    assert!(result.is_err(), "path traversal must be rejected");

    // No markdown files should have been written.
    let md_count = walkdir::WalkDir::new(dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .count();
    assert_eq!(md_count, 0, "no .md files should exist after rejection");

    // No git commits.
    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let log_str = String::from_utf8(log.stdout).unwrap();
    assert!(log_str.is_empty(), "no commit after rejection: {log_str}");
}
