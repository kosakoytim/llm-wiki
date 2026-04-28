#!/usr/bin/env bash
section "3. Search & List (MCP)"

run_mcp      "search returns results"                "mixture-of-experts" \
             wiki_search '{"query":"mixture of experts"}'

run_mcp_json "search json results array not empty"  \
             '.results | length > 0' "true" \
             wiki_search '{"query":"mixture of experts","format":"json"}'

run_mcp      "search with type filter"              "concept" \
             wiki_search '{"query":"attention","type":"concept"}'

run_mcp      "search llms format"                   "wiki://" \
             wiki_search '{"query":"transformer","format":"llms"}'

run_mcp_json "list json total > 0"                  \
             '.total > 0' "true" \
             wiki_list '{"format":"json"}'

run_mcp_json "list json pages array"                \
             '.pages | type' "array" \
             wiki_list '{"format":"json"}'

run_mcp      "list with type filter returns concept" "concept" \
             wiki_list '{"type":"concept"}'

run_mcp_json "list type filter json all concepts"   \
             '.pages | map(select(.type != "concept")) | length == 0' "true" \
             wiki_list '{"type":"concept","format":"json"}'
