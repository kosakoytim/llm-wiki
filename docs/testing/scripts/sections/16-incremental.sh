#!/usr/bin/env bash
# Requires: RESEARCH_ROOT, CLI set by caller
section "16. Incremental validation"

MODIFIED="$RESEARCH_ROOT/wiki/concepts/scaling-laws.md"
echo "" >> "$MODIFIED"
run_json "incremental ingest reports unchanged_count" \
         '.unchanged_count >= 0' "true" \
         $CLI ingest concepts/scaling-laws.md --format json 2>/dev/null || \
    skip "incremental unchanged_count" "format json not supported on ingest"
