#!/usr/bin/env bash
section "3. Search"

run      "basic search returns results"   "mixture"  $CLI search "mixture of experts"
run      "type filter: concept"           "concept"  $CLI search "routing" --type concept
run      "cross-wiki search"              "attention" $CLI search "attention" --cross-wiki
run_json "search json has results array"  '.results | length > 0' "true" \
         $CLI search "transformer" --format json
