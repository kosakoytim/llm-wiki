#!/usr/bin/env bash
section "11. Schema & History (MCP)"

run_mcp_json "schema list returns array"             \
             'type' "array" \
             wiki_schema '{"action":"list","wiki":"research"}'

run_mcp_json "schema list contains concept type"     \
             '[.[] | select(.name=="concept")] | length > 0' "true" \
             wiki_schema '{"action":"list","wiki":"research"}'

run_mcp      "schema show concept type"              "title\|summary\|confidence" \
             wiki_schema '{"action":"show","type":"concept","wiki":"research"}'

run_mcp_json "history json entries array"            \
             '.entries | type' "array" \
             wiki_history '{"slug":"concepts/mixture-of-experts","wiki":"research","format":"json"}'

run_mcp_json "history has at least one commit"       \
             '.entries | length > 0' "true" \
             wiki_history '{"slug":"concepts/mixture-of-experts","wiki":"research","format":"json"}'
