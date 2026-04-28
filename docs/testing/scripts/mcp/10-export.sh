#!/usr/bin/env bash
section "10. Export (MCP)"

run_mcp_json "export llms-txt pages_written > 0"    \
             '.pages_written > 0' "true" \
             wiki_export '{"path":"mcp-export-test.txt","format":"llms-txt","wiki":"research"}'

run_mcp_json "export llms-full pages_written > 0"   \
             '.pages_written > 0' "true" \
             wiki_export '{"path":"mcp-export-full.txt","format":"llms-full","wiki":"research"}'

run_mcp_json "export json has path string"           \
             '.path | type' "string" \
             wiki_export '{"path":"mcp-export.json","format":"json","wiki":"research"}'

run_mcp_json "export json bytes > 0"                 \
             '.bytes > 0' "true" \
             wiki_export '{"path":"mcp-export.json","format":"json","wiki":"research"}'
