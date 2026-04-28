#!/usr/bin/env bash
section "4. Content"

run  "read page by slug"          "Mixture of Experts" \
     $CLI content read concepts/mixture-of-experts
run  "read cross-wiki page via uri" "Attention" \
     $CLI content read "wiki://notes/concepts/attention-mechanism"

# backlinks are MCP-only (no --backlinks CLI flag)
skip "read page with backlinks" "backlinks not exposed as a CLI flag (MCP only)"
