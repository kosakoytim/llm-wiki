#!/usr/bin/env bash
section "8. Stats (MCP)"

run_mcp      "stats returns wiki name"               "research" \
             wiki_stats

run_mcp_json "stats json pages > 0"                  \
             '.pages > 0' "true" \
             wiki_stats '{"format":"json"}'

run_mcp_json "stats json orphans >= 0"               \
             '.orphans >= 0' "true" \
             wiki_stats '{"format":"json"}'

run_mcp_json "stats communities present (threshold=5)" \
             '.communities != null' "true" \
             wiki_stats '{"format":"json"}'
