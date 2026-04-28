#!/usr/bin/env bash
# clean-test-env.sh — remove the persistent test environment and unset env vars
#
# Source this script to also unset env vars in the current shell:
#
#   source ./docs/testing/scripts/clean-test-env.sh [--dir <path>] [--yes]
#
# Or run directly (env vars won't be unset in parent shell):
#
#   ./docs/testing/scripts/clean-test-env.sh [--dir <path>] [--yes]

# Detect if sourced or executed
_SOURCED=0
if [ -n "${BASH_SOURCE[0]:-}" ] && [ "${BASH_SOURCE[0]}" != "$0" ]; then
    _SOURCED=1
fi

set -euo pipefail

TEST_DIR="${LLM_WIKI_TEST_DIR:-$HOME/llm-wiki-testing}"
CONFIRMED=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dir) TEST_DIR="$2"; shift 2 ;;
        --yes) CONFIRMED=1; shift ;;
        *) echo "Unknown argument: $1"; return 1 2>/dev/null || exit 1 ;;
    esac
done

green() { printf '\033[32m%s\033[0m\n' "$*"; }

if [ ! -d "$TEST_DIR" ]; then
    echo "Nothing to clean — $TEST_DIR does not exist."
else
    if [ "$CONFIRMED" = "0" ]; then
        echo "This will permanently delete: $TEST_DIR"
        printf "Continue? [y/N] "
        read -r answer
        case "$answer" in
            [yY]*) ;;
            *) echo "Aborted."; return 0 2>/dev/null || exit 0 ;;
        esac
    fi

    rm -rf "$TEST_DIR"
    green "  ✓ Removed $TEST_DIR"
fi

# ── Unset env vars ────────────────────────────────────────────────────────────

if [ "$_SOURCED" = "1" ]; then
    unset LLM_WIKI_TEST_DIR
    unset LLM_WIKI_CONFIG
    green "  ✓ Unset LLM_WIKI_TEST_DIR and LLM_WIKI_CONFIG"
else
    echo
    echo "NOTE: env vars were NOT unset (script was executed, not sourced)."
    echo "To unset them, run:"
    echo "  source ./docs/testing/scripts/clean-test-env.sh"
fi
