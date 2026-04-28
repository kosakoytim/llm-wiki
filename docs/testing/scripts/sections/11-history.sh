#!/usr/bin/env bash
section "11. History"

run      "history returns commits"  ""          \
         $CLI history concepts/mixture-of-experts
run_json "history json has entries" 'length > 0' "true" \
         $CLI history concepts/mixture-of-experts --format json
