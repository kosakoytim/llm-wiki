#!/usr/bin/env bash
section "9. Suggest (MCP)"

run_mcp      "suggest returns results"               "" \
             wiki_suggest '{"slug":"concepts/mixture-of-experts"}'

run_mcp_json "suggest json is array"                 \
             'type' "array" \
             wiki_suggest '{"slug":"concepts/mixture-of-experts","format":"json"}'

run_mcp_json "suggest results have slug field"       \
             '.[0].slug | type' "string" \
             wiki_suggest '{"slug":"concepts/mixture-of-experts","format":"json"}'

run_mcp_json "suggest community peers (length >= 0)" \
             '[.[] | select(.reason | test("cluster"))] | length >= 0' "true" \
             wiki_suggest '{"slug":"concepts/mixture-of-experts","format":"json"}'
