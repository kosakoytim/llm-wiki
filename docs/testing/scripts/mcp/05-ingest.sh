#!/usr/bin/env bash
section "5. Ingest (MCP)"

run_mcp_json "ingest dry run pages_validated >= 0"   \
             '.pages_validated >= 0' "true" \
             wiki_ingest '{"path":"inbox/01-paper-switch-transformer.md","dry_run":true}'

run_mcp_json "ingest dry run has warnings array"     \
             '.warnings | type' "array" \
             wiki_ingest '{"path":"inbox/01-paper-switch-transformer.md","dry_run":true}'

run_mcp_json "ingest dry run unchanged_count >= 0"   \
             '.unchanged_count >= 0' "true" \
             wiki_ingest '{"path":"inbox/01-paper-switch-transformer.md","dry_run":true}'

run_mcp_json "ingest with redact dry run succeeds"   \
             '.pages_validated >= 0' "true" \
             wiki_ingest '{"path":"inbox/03-note-with-secrets.md","dry_run":true,"redact":true}'
