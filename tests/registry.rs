//! Multi-wiki registry and SSE tests — Phase 6
//!
//! Covers: `WikiRegistry` load and resolve, cross-wiki `search_all`,
//! config validation errors, and end-to-end multi-wiki and SSE behaviour.

use llm_wiki::registry::WikiRegistry;
use llm_wiki::search::search_all;
use llm_wiki::server::WikiServer;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

// ── Test helpers ──────────────────────────────────────────────────────────────

/// Build a minimal valid `analysis.json` Value that creates one concept page.
fn one_concept_analysis(slug: &str, title: &str) -> serde_json::Value {
    serde_json::json!({
        "source": "test-source",
        "doc_type": "note",
        "title": title,
        "language": "en",
        "claims": [],
        "concepts": [slug],
        "key_quotes": [],
        "data_gaps": [],
        "suggested_pages": [{
            "slug": format!("concepts/{slug}"),
            "title": title,
            "type": "concept",
            "action": "create",
            "tldr": "A test concept page.",
            "body": format!("## Overview\n\nThis page covers {}.", title),
            "tags": ["test"],
            "read_when": []
        }],
        "contradictions": []
    })
}

/// Write a two-wiki global config to `config_path`.
///
/// `dir1` is the "work" wiki (default), `dir2` is the "research" wiki.
fn write_two_wiki_config(config_path: &std::path::Path, dir1: &std::path::Path, dir2: &std::path::Path) {
    let toml = format!(
        "[[wikis]]\nname = \"work\"\npath = \"{}\"\ndefault = true\n\n\
         [[wikis]]\nname = \"research\"\npath = \"{}\"\ndefault = false\n",
        dir1.display(),
        dir2.display()
    );
    fs::write(config_path, toml).expect("write config");
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[test]
fn registry_load_two_wikis_both_resolved_by_name() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    let registry = WikiRegistry::load(&cfg_path).expect("load registry");

    let work = registry.resolve(Some("work")).expect("resolve work");
    assert_eq!(work.name, "work");
    assert_eq!(work.root, work_dir.path());

    let research = registry.resolve(Some("research")).expect("resolve research");
    assert_eq!(research.name, "research");
    assert_eq!(research.root, research_dir.path());
}

#[test]
fn registry_resolve_none_returns_default_wiki() {
    let dir = TempDir::new().expect("dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    let toml = format!(
        "[[wikis]]\nname = \"main\"\npath = \"{}\"\ndefault = true\n",
        dir.path().display()
    );
    fs::write(&cfg_path, toml).expect("write config");

    let registry = WikiRegistry::load(&cfg_path).expect("load");
    let result = registry.resolve(None).expect("resolve default");
    assert_eq!(result.name, "main");
    assert_eq!(result.root, dir.path());
}

#[test]
fn registry_resolve_named_wiki_returns_correct_entry() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    let registry = WikiRegistry::load(&cfg_path).expect("load");

    // Resolve by name explicitly.
    let work = registry.resolve(Some("work")).expect("resolve work");
    assert_eq!(work.name, "work");
    assert_eq!(work.root, work_dir.path());

    // The root of "work" must not be the research dir.
    assert_ne!(work.root, research_dir.path());
}

#[test]
fn registry_resolve_unknown_name_error_lists_available() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    let registry = WikiRegistry::load(&cfg_path).expect("load");
    let result = registry.resolve(Some("nonexistent"));

    assert!(result.is_err(), "expected error for unknown wiki name");
    let msg = result.unwrap_err().to_string();
    // Error must mention the requested name and list available names.
    assert!(
        msg.contains("nonexistent"),
        "error should mention the bad name: {msg}"
    );
    assert!(
        msg.contains("work") && msg.contains("research"),
        "error should list available wikis: {msg}"
    );
}

#[test]
fn registry_resolve_none_no_default_configured_returns_error() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    // Both wikis have default = false.
    let toml = format!(
        "[[wikis]]\nname = \"work\"\npath = \"{}\"\ndefault = false\n\n\
         [[wikis]]\nname = \"research\"\npath = \"{}\"\ndefault = false\n",
        work_dir.path().display(),
        research_dir.path().display()
    );
    fs::write(&cfg_path, toml).expect("write config");

    let registry = WikiRegistry::load(&cfg_path).expect("load");
    let result = registry.resolve(None);
    assert!(result.is_err(), "expected error when no default is set");
}

#[test]
fn search_all_term_in_one_wiki_result_has_correct_wiki_name() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    // Ingest a unique concept only into the "work" wiki.
    let work_server = WikiServer::new(work_dir.path().to_path_buf());
    work_server
        .do_ingest(one_concept_analysis("unique-bumblebee", "Unique Bumblebee"))
        .expect("ingest into work");

    let registry = WikiRegistry::load(&cfg_path).expect("load");
    let results = search_all(&registry, "bumblebee", 20).expect("search_all");

    assert!(!results.is_empty(), "expected at least one result");
    // All results should come from the "work" wiki.
    for r in &results {
        assert_eq!(
            r.wiki_name, "work",
            "result should be from 'work' wiki: {:?}",
            r
        );
    }
    // No results from "research".
    assert!(
        results.iter().all(|r| r.wiki_name != "research"),
        "research wiki should have no matching results"
    );
}

#[test]
fn search_all_term_in_both_wikis_results_merged_by_score() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    // Ingest the same concept slug into both wikis.
    WikiServer::new(work_dir.path().to_path_buf())
        .do_ingest(one_concept_analysis("transformer-arch", "Transformer Architecture"))
        .expect("ingest work");
    WikiServer::new(research_dir.path().to_path_buf())
        .do_ingest(one_concept_analysis("transformer-arch", "Transformer Architecture"))
        .expect("ingest research");

    let registry = WikiRegistry::load(&cfg_path).expect("load");
    let results = search_all(&registry, "transformer", 20).expect("search_all");

    assert!(results.len() >= 2, "expected results from both wikis");

    let wiki_names: Vec<&str> = results.iter().map(|r| r.wiki_name.as_str()).collect();
    assert!(
        wiki_names.contains(&"work"),
        "results should include work wiki"
    );
    assert!(
        wiki_names.contains(&"research"),
        "results should include research wiki"
    );

    // Results must be ordered by descending score.
    for window in results.windows(2) {
        assert!(
            window[0].score >= window[1].score,
            "results must be sorted by descending score: {} < {}",
            window[0].score,
            window[1].score
        );
    }
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[test]
fn cli_ingest_to_named_wiki_writes_to_correct_root() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    let registry = Arc::new(WikiRegistry::load(&cfg_path).expect("load"));
    let server =
        WikiServer::new_with_registry(work_dir.path().to_path_buf(), registry);

    // Ingest targeting the "research" wiki.
    server
        .do_ingest_with_wiki(
            one_concept_analysis("attention-heads", "Attention Heads"),
            Some("research"),
        )
        .expect("ingest to research");

    // File must appear in research, NOT in work.
    let in_research = research_dir
        .path()
        .join("concepts")
        .join("attention-heads.md");
    let in_work = work_dir
        .path()
        .join("concepts")
        .join("attention-heads.md");

    assert!(
        in_research.exists(),
        "page should be written to research wiki: {}",
        in_research.display()
    );
    assert!(
        !in_work.exists(),
        "page must NOT appear in work wiki: {}",
        in_work.display()
    );
}

#[test]
fn cli_search_named_wiki_only_searches_that_index() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    // Ingest a unique page only into "research".
    WikiServer::new(research_dir.path().to_path_buf())
        .do_ingest(one_concept_analysis("neural-ode", "Neural ODE"))
        .expect("ingest");

    // Search the work index — must find nothing.
    let work_results =
        llm_wiki::search::search("neural-ode", work_dir.path(), false).unwrap_or_default();
    assert!(
        work_results.is_empty(),
        "work wiki should have no neural-ode results, got: {:?}",
        work_results
    );

    // Search the research index — must find the page.
    let research_results =
        llm_wiki::search::search("neural-ode", research_dir.path(), false).unwrap_or_default();
    assert!(
        !research_results.is_empty(),
        "research wiki should contain the neural-ode page"
    );
}

#[test]
fn cli_search_all_returns_results_from_both_wikis_with_label() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    WikiServer::new(work_dir.path().to_path_buf())
        .do_ingest(one_concept_analysis("sparse-matrix", "Sparse Matrix"))
        .expect("ingest work");
    WikiServer::new(research_dir.path().to_path_buf())
        .do_ingest(one_concept_analysis("sparse-tensor", "Sparse Tensor"))
        .expect("ingest research");

    let registry = WikiRegistry::load(&cfg_path).expect("load");
    let results = search_all(&registry, "sparse", 20).expect("search_all");

    assert!(results.len() >= 2, "expected at least 2 results across both wikis");

    let names: Vec<&str> = results.iter().map(|r| r.wiki_name.as_str()).collect();
    assert!(names.contains(&"work"), "should include work result");
    assert!(names.contains(&"research"), "should include research result");
}

#[tokio::test]
async fn cli_serve_sse_accepts_second_client_after_first() {
    use rmcp::transport::sse_server::SseServer;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let dir = TempDir::new().expect("temp dir");
    let wiki_root = dir.path().to_path_buf();

    // Pre-bind to get a free port, then release it (brief race window is
    // acceptable in tests; the port is unlikely to be taken in microseconds).
    let free_listener = std::net::TcpListener::bind("127.0.0.1:0").expect("pre-bind");
    let port = free_listener.local_addr().expect("local_addr").port();
    drop(free_listener);

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let sse_server = SseServer::serve(addr).await.expect("sse server start");

    let ct = sse_server.with_service(move || WikiServer::new(wiki_root.clone()));

    // Give the axum server a moment to accept connections.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Helper: raw HTTP GET /sse and return the first chunk.
    async fn get_sse(port: u16) -> String {
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect");
        stream
            .write_all(
                b"GET /sse HTTP/1.1\r\nHost: localhost\r\nAccept: text/event-stream\r\n\r\n",
            )
            .await
            .expect("write");
        let mut buf = vec![0u8; 512];
        let n = stream.read(&mut buf).await.expect("read");
        String::from_utf8_lossy(&buf[..n]).to_string()
    }

    // First client.
    let resp1 = get_sse(port).await;
    assert!(
        resp1.contains("200"),
        "first client should get HTTP 200, got: {resp1}"
    );
    assert!(
        resp1.to_lowercase().contains("text/event-stream"),
        "first client should get SSE content-type, got: {resp1}"
    );

    // Second client connects while the first may still be open.
    let resp2 = get_sse(port).await;
    assert!(
        resp2.contains("200"),
        "second client should get HTTP 200, got: {resp2}"
    );
    assert!(
        resp2.to_lowercase().contains("text/event-stream"),
        "second client should get SSE content-type, got: {resp2}"
    );

    ct.cancel();
}

#[test]
fn mcp_ingest_tool_wiki_param_writes_to_named_wiki() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    let registry = Arc::new(WikiRegistry::load(&cfg_path).expect("load"));
    let server =
        WikiServer::new_with_registry(work_dir.path().to_path_buf(), registry);

    // Ingest via the multi-wiki helper targeting "research".
    server
        .do_ingest_with_wiki(
            one_concept_analysis("causal-attention", "Causal Attention"),
            Some("research"),
        )
        .expect("ingest");

    let page = research_dir
        .path()
        .join("concepts")
        .join("causal-attention.md");
    assert!(
        page.exists(),
        "page should be in research wiki: {}",
        page.display()
    );

    let not_in_work = work_dir
        .path()
        .join("concepts")
        .join("causal-attention.md");
    assert!(
        !not_in_work.exists(),
        "page must NOT be in work wiki: {}",
        not_in_work.display()
    );
}

#[test]
fn mcp_resource_named_wiki_uri_reads_from_correct_root() {
    let work_dir = TempDir::new().expect("work dir");
    let research_dir = TempDir::new().expect("research dir");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    write_two_wiki_config(&cfg_path, work_dir.path(), research_dir.path());

    // Put a page only in research.
    WikiServer::new(research_dir.path().to_path_buf())
        .do_ingest(one_concept_analysis("diffusion-model", "Diffusion Model"))
        .expect("ingest");

    let registry = Arc::new(WikiRegistry::load(&cfg_path).expect("load"));
    let server =
        WikiServer::new_with_registry(work_dir.path().to_path_buf(), registry);

    // Read via multi-wiki URI.
    let content = server
        .do_read_resource("wiki://research/concepts/diffusion-model")
        .expect("resource read should succeed");
    assert!(
        content.contains("Diffusion Model"),
        "content should contain the page title: {content}"
    );

    // Reading from the wrong wiki should fail (page only in research, not work).
    let err = server.do_read_resource("wiki://work/concepts/diffusion-model");
    assert!(
        err.is_err(),
        "reading from 'work' wiki should fail when page only exists in 'research'"
    );
}

// ── Config tests ──────────────────────────────────────────────────────────────

#[test]
fn config_missing_path_field_clear_error_names_wiki() {
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    // `path` field is missing — TOML parse will fail on the missing required field
    // OR serde will return an error.
    let toml = "[[wikis]]\nname = \"broken\"\ndefault = true\n";
    fs::write(&cfg_path, toml).expect("write");

    let result = WikiRegistry::load(&cfg_path);
    assert!(
        result.is_err(),
        "expected error for missing 'path' field, got Ok"
    );
}

#[test]
fn config_nonexistent_path_clear_error() {
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    let toml =
        "[[wikis]]\nname = \"ghost\"\npath = \"/this/path/does/not/exist/xyz\"\ndefault = true\n";
    fs::write(&cfg_path, toml).expect("write");

    let result = WikiRegistry::load(&cfg_path);
    assert!(result.is_err(), "expected error for non-existent path");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("ghost"),
        "error should name the affected wiki: {msg}"
    );
}

#[test]
fn config_two_wikis_both_default_error_on_load() {
    let dir1 = TempDir::new().expect("dir1");
    let dir2 = TempDir::new().expect("dir2");
    let cfg_dir = TempDir::new().expect("cfg dir");
    let cfg_path = cfg_dir.path().join("config.toml");

    let toml = format!(
        "[[wikis]]\nname = \"alpha\"\npath = \"{}\"\ndefault = true\n\n\
         [[wikis]]\nname = \"beta\"\npath = \"{}\"\ndefault = true\n",
        dir1.path().display(),
        dir2.path().display()
    );
    fs::write(&cfg_path, toml).expect("write");

    let result = WikiRegistry::load(&cfg_path);
    assert!(
        result.is_err(),
        "expected error when two wikis are both default"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("default"),
        "error should mention 'default': {msg}"
    );
}
