#!/usr/bin/env bash
section "7. Graph (MCP)"

run_mcp      "graph mermaid output"                  "graph LR\|graph TD\|flowchart" \
             wiki_graph

run_mcp      "graph dot output"                      "digraph" \
             wiki_graph '{"format":"dot"}'

run_mcp      "graph llms output"                     "nodes\|edges\|type groups" \
             wiki_graph '{"format":"llms"}'

run_mcp      "graph type filter"                     "" \
             wiki_graph '{"type":"concept"}'

run_mcp      "graph root and depth"                  "" \
             wiki_graph '{"root":"concepts/mixture-of-experts","depth":2}'

run_mcp      "graph cross-wiki includes notes wiki"  "" \
             wiki_graph '{"cross_wiki":true}'
