#!/usr/bin/env bash
# helpers.sh — shared functions sourced by all validate-*.sh section scripts
# Do not execute directly.

green() { printf '\033[32m%s\033[0m\n' "$*"; }
red()   { printf '\033[31m%s\033[0m\n' "$*"; }
yellow(){ printf '\033[33m%s\033[0m\n' "$*"; }

pass() { PASS=$((PASS+1)); green "  ✓ $1"; }
fail() { FAIL=$((FAIL+1)); red   "  ✗ $1"; [ -n "${2:-}" ] && red "    $2"; }
skip() { SKIP=$((SKIP+1)); yellow "  - $1 (skipped: $2)"; }

section() { echo; echo "── $1 ──────────────────────────────────────"; }

run() {
    local desc="$1" pattern="$2"; shift 2
    local out
    if out=$("$@" 2>&1); then
        if [ -z "$pattern" ] || echo "$out" | grep -q "$pattern"; then
            pass "$desc"
        else
            fail "$desc" "output did not match: $pattern"
            echo "    output: $(echo "$out" | head -3)"
        fi
    else
        fail "$desc" "command failed (exit $?)"
        echo "    output: $(echo "$out" | head -3)"
    fi
}

run_nocheck() {
    # like run but does not fail on non-zero exit (use when command exits 1 by design)
    local desc="$1" pattern="$2"; shift 2
    local out
    out=$("$@" 2>&1) || true
    if [ -z "$pattern" ] || echo "$out" | grep -q "$pattern"; then
        pass "$desc"
    else
        fail "$desc" "output did not match: $pattern"
        echo "    output: $(echo "$out" | head -3)"
    fi
}

run_json() {
    local desc="$1" filter="$2" expected="$3"; shift 3
    local out actual
    if out=$("$@" 2>&1); then
        actual=$(echo "$out" | jq -r "$filter" 2>/dev/null || echo "jq-error")
        if [ "$actual" = "$expected" ]; then
            pass "$desc"
        else
            fail "$desc" "expected '$expected', got '$actual'"
        fi
    else
        fail "$desc" "command failed"
        echo "    output: $(echo "$out" | head -3)"
    fi
}

run_json_nocheck() {
    # like run_json but does not fail on non-zero exit
    local desc="$1" filter="$2" expected="$3"; shift 3
    local out actual
    out=$("$@" 2>&1) || true
    actual=$(echo "$out" | jq -r "$filter" 2>/dev/null || echo "jq-error")
    if [ "$actual" = "$expected" ]; then
        pass "$desc"
    else
        fail "$desc" "expected '$expected', got '$actual'"
    fi
}
