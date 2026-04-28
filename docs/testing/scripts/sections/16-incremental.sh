#!/usr/bin/env bash
# Requires: RESEARCH_ROOT, CLI set by caller
section "16. Incremental validation"

MODIFIED="$RESEARCH_ROOT/wiki/concepts/scaling-laws.md"
echo "" >> "$MODIFIED"
# Suppress stderr so log lines don't corrupt the JSON output fed to jq
run_json "incremental ingest reports unchanged_count" \
         '.unchanged_count >= 0' "true" \
         bash -c "$CLI ingest concepts/scaling-laws.md --format json 2>/dev/null"
