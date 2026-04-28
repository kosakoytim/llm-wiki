#!/usr/bin/env bash
section "1. Spaces (MCP)"

run_mcp      "spaces_list returns wikis"              "research" \
             wiki_spaces_list

run_mcp_json "spaces_list contains research entry"   \
             '[.[] | select(.name=="research")] | length > 0' "true" \
             wiki_spaces_list

run_mcp      "spaces_set_default research"            "research" \
             wiki_spaces_set_default '{"name":"research"}'
