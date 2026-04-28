#!/usr/bin/env bash
section "8. Stats"

run      "stats returns output"    "research"  $CLI stats
run_json "stats json has pages"    '.pages > 0' "true" \
         $CLI stats --format json
run_json "stats communities present (threshold=5)" '.communities != null' "true" \
         $CLI stats --format json
