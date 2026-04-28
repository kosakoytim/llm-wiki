#!/usr/bin/env bash
section "2. Index (MCP)"

run_mcp_json "index_rebuild returns pages_indexed"   \
             '.pages_indexed > 0' "true" \
             wiki_index_rebuild '{"wiki":"research"}'

run_mcp_json "index_status has built timestamp"      \
             '.built | type' "string" \
             wiki_index_status '{"wiki":"research"}'

run_mcp_json "index_status queryable true"           \
             '.queryable' "true" \
             wiki_index_status '{"wiki":"research"}'
