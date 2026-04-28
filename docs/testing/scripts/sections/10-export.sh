#!/usr/bin/env bash
# Requires: TEST_DIR set by caller
section "10. Export"

EXPORT_OUT="$TEST_DIR/export-llms.txt"
run  "export llms-txt"   ""  $CLI export --path "$EXPORT_OUT" --wiki research
[ -f "$EXPORT_OUT" ] && pass "export: file created" || fail "export: file not created"
grep -q "Mixture of Experts" "$EXPORT_OUT" 2>/dev/null && \
    pass "export: content contains expected page" || \
    fail "export: content missing expected page"

EXPORT_JSON="$TEST_DIR/export.json"
run  "export json format"  ""  $CLI export --path "$EXPORT_JSON" \
     --format json --wiki research
[ -f "$EXPORT_JSON" ] && pass "export json: file created" || fail "export json: file not created"
