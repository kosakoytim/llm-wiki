#!/usr/bin/env bash
section "15. Backlinks"

# Backlinks are only available via the MCP wiki_content_read tool (backlinks: true).
# The CLI content read command does not expose a --backlinks flag.
skip "backlinks via CLI" "backlinks not exposed as a CLI flag (MCP only — see validate-skills.md)"
