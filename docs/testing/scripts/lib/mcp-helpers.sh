#!/usr/bin/env bash
# mcp-helpers.sh — shared helpers for MCP validation scripts
# Do not execute directly. Source after helpers.sh.
#
# Requires: MCP_SERVER (array) — the command to launch the MCP server via stdio.
# Set by validate-mcp.sh:
#   MCP_SERVER=("$CLI" --config "$CONFIG" serve)
#
# mcptools (mcp) handles the stdio session internally — one subprocess per call.
# Response shape: {"content":[{"text":"...","type":"text"}]}

# Extract the text field from an mcptools JSON response.
_mcp_text() {
    echo "$1" | jq -r '.content[0].text // empty' 2>/dev/null
}

# run_mcp <desc> <pattern> <tool> [params_json]
# Calls tool via mcptools stdio, checks text content matches pattern (grep -q).
run_mcp() {
    local desc="$1" pattern="$2" tool="$3" params="${4:-EMPTY}"
    [ "$params" = "EMPTY" ] && params="{}"
    local raw text
    if raw=$(mcp call "$tool" -p "$params" -f json "${MCP_SERVER[@]}" 2>&1); then
        text=$(_mcp_text "$raw")
        if [ -z "$pattern" ] || echo "$text" | grep -q "$pattern"; then
            pass "$desc"
        else
            fail "$desc" "output did not match: $pattern"
            echo "    output: $(echo "$text" | head -3)"
        fi
    else
        fail "$desc" "mcp call failed"
        echo "    output: $(echo "$raw" | head -3)"
    fi
}

# run_mcp_json <desc> <jq_filter> <expected> <tool> [params_json]
# Calls tool, parses text content as JSON, applies jq filter, checks result.
run_mcp_json() {
    local desc="$1" filter="$2" expected="$3" tool="$4" params="${5:-EMPTY}"
    [ "$params" = "EMPTY" ] && params="{}"
    local raw text actual
    if raw=$(mcp call "$tool" -p "$params" -f json "${MCP_SERVER[@]}" 2>&1); then
        text=$(_mcp_text "$raw")
        actual=$(echo "$text" | jq -r "$filter" 2>/dev/null || echo "jq-error")
        if [ "$actual" = "$expected" ]; then
            pass "$desc"
        else
            fail "$desc" "expected '$expected', got '$actual'"
            echo "    text: $(echo "$text" | head -3)"
        fi
    else
        fail "$desc" "mcp call failed"
        echo "    output: $(echo "$raw" | head -3)"
    fi
}
