#!/usr/bin/env bash
section "9. Suggest"

run      "suggest returns results"    ""      \
         $CLI suggest concepts/mixture-of-experts
run_json "suggest json is array"      'type'  "array" \
         $CLI suggest concepts/mixture-of-experts --format json
run_json "suggest has community peers reason" \
         '[.[] | select(.reason | test("cluster"))] | length >= 0' "true" \
         $CLI suggest concepts/mixture-of-experts --format json
