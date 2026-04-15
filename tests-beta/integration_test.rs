//! Cross-phase smoke tests — full pipeline end-to-end.
//!
//! Each test exercises multiple phases in sequence to verify the full `wiki`
//! pipeline holds together. Phase-specific tests live in their own files:
//! `schema.rs`, `ingest.rs`, `search.rs`, `graph.rs`, `mcp.rs`, `plugin.rs`,
//! `registry.rs`.

/// Placeholder: verifies the integration test suite compiles and passes.
#[test]
fn placeholder() {
    // no-op
}

#[test]
#[ignore = "cross-phase — stub"]
fn full_pipeline_ingest_search_lint() {
    // ingest → search → lint end-to-end
    todo!()
}

#[test]
#[ignore = "cross-phase — stub"]
fn full_pipeline_ingest_context_mcp() {
    // ingest → context → mcp serve end-to-end
    todo!()
}
