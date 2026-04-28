#!/usr/bin/env bash
section "6. Lint"

run      "lint all rules"         "findings"  $CLI lint
run      "lint broken-link rule"  "broken"    $CLI lint --rules broken-link
run      "lint orphan rule"       "orphan"    $CLI lint --rules orphan
run_json "lint json has findings array" '.findings | type' "array" \
         $CLI lint --format json
run_json "broken-link finds concepts/does-not-exist" \
         '[.findings[] | select(.rule=="broken-link")] | length > 0' "true" \
         $CLI lint --rules broken-link --format json
run_json "orphan finds orphan-concept" \
         '[.findings[] | select(.slug=="concepts/orphan-concept")] | length > 0' "true" \
         $CLI lint --rules orphan --format json
run      "lint with --wiki flag"  "findings"  $CLI lint --wiki research
