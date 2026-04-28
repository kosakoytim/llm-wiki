#!/usr/bin/env bash
# validate-engine.sh — end-to-end CLI validation for llm-wiki v0.2.0
#
# Usage:
#   # Ephemeral (auto-cleaned after run):
#   LLM_WIKI_BIN=./target/release/llm-wiki ./docs/testing/scripts/validate-engine.sh
#
#   # Persistent (inspect results after run):
#   ./docs/testing/scripts/setup-test-env.sh
#   LLM_WIKI_BIN=./target/release/llm-wiki \
#   LLM_WIKI_TEST_DIR=$HOME/llm-wiki-testing \
#   ./docs/testing/scripts/validate-engine.sh
#
#   # Run a single section:
#   LLM_WIKI_BIN=./target/release/llm-wiki \
#   LLM_WIKI_TEST_DIR=$HOME/llm-wiki-testing \
#   ./docs/testing/scripts/validate-engine.sh --section 05
#
# Requires: llm-wiki binary on PATH (or set LLM_WIKI_BIN), jq, git
# Run from the repo root.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURES="$(cd "$SCRIPT_DIR/../../.." && pwd)/tests/fixtures"

BINARY="${LLM_WIKI_BIN:-llm-wiki}"
SECTION_FILTER="${1:-}"
if [ "$SECTION_FILTER" = "--section" ]; then
    SECTION_FILTER="$2"
    shift 2
fi

# ── Test directory ────────────────────────────────────────────────────────────

if [ -n "${LLM_WIKI_TEST_DIR:-}" ]; then
    TEST_DIR="$LLM_WIKI_TEST_DIR"
    EPHEMERAL=0
else
    TEST_DIR="$(mktemp -d)"
    EPHEMERAL=1
    trap 'rm -rf "$TEST_DIR"' EXIT
fi

# LLM_WIKI_CONFIG may be exported by setup-test-env.sh; fall back to TEST_DIR layout
CONFIG_FILE="${LLM_WIKI_CONFIG:-$TEST_DIR/config.toml}"
RESEARCH_ROOT="$TEST_DIR/wikis/research"

# ── Counters (exported so section scripts can update them) ────────────────────

PASS=0
FAIL=0
SKIP=0

# ── Source helpers ────────────────────────────────────────────────────────────

# shellcheck source=lib/helpers.sh
source "$SCRIPT_DIR/lib/helpers.sh"

# ── Binary check ─────────────────────────────────────────────────────────────

if ! command -v "$BINARY" &>/dev/null && [ ! -x "$BINARY" ]; then
    red "ERROR: '$BINARY' not found."
    echo "Build with 'cargo build --release' and set LLM_WIKI_BIN or add to PATH."
    exit 1
fi

echo "llm-wiki validate-engine.sh"
echo "binary:   $($BINARY --version 2>/dev/null || echo unknown)"
echo "test dir: $TEST_DIR"
echo "mode:     $([ "$EPHEMERAL" = "1" ] && echo ephemeral || echo persistent)"

# ── Setup ─────────────────────────────────────────────────────────────────────

if [ "$EPHEMERAL" = "1" ]; then
    mkdir -p "$TEST_DIR/wikis"
    # research wiki
    cp -r "$FIXTURES/wikis/research" "$RESEARCH_ROOT"
    mkdir -p "$RESEARCH_ROOT/wiki/inbox"
    git -C "$RESEARCH_ROOT" init -q
    git -C "$RESEARCH_ROOT" add .
    git -C "$RESEARCH_ROOT" -c user.name=test -c user.email=test@test.com commit -q -m "init"
    # notes wiki
    cp -r "$FIXTURES/wikis/notes" "$TEST_DIR/wikis/notes"
    git -C "$TEST_DIR/wikis/notes" init -q
    git -C "$TEST_DIR/wikis/notes" add .
    git -C "$TEST_DIR/wikis/notes" -c user.name=test -c user.email=test@test.com commit -q -m "init"
    # inbox + register
    cp "$FIXTURES"/inbox/* "$RESEARCH_ROOT/wiki/inbox/"
    "$BINARY" --config "$CONFIG_FILE" spaces create "$RESEARCH_ROOT"     --name research 2>/dev/null || true
    "$BINARY" --config "$CONFIG_FILE" spaces create "$TEST_DIR/wikis/notes" --name notes 2>/dev/null || true
    "$BINARY" --config "$CONFIG_FILE" spaces set-default research                        2>/dev/null || true
else
    if [ ! -f "$CONFIG_FILE" ]; then
        red "ERROR: $CONFIG_FILE not found — run setup-test-env.sh first."
        exit 1
    fi
    # Re-copy inbox fixtures so each run starts with a clean inbox
    cp "$FIXTURES"/inbox/* "$RESEARCH_ROOT/wiki/inbox/"
fi

CLI="$BINARY --config $CONFIG_FILE"

# ── Run sections ──────────────────────────────────────────────────────────────

for section_script in "$SCRIPT_DIR"/sections/[0-9][0-9]-*.sh; do
    num="$(basename "$section_script" | cut -c1-2)"
    if [ -n "$SECTION_FILTER" ] && [ "$num" != "$SECTION_FILTER" ]; then
        continue
    fi
    # shellcheck source=/dev/null
    source "$section_script"
done

# ── Summary ───────────────────────────────────────────────────────────────────

echo
echo "────────────────────────────────────────"
echo "Results: $(green "$PASS passed") | $([ $FAIL -gt 0 ] && red "$FAIL failed" || echo "$FAIL failed") | $SKIP skipped"
echo "────────────────────────────────────────"

[ $FAIL -eq 0 ]
