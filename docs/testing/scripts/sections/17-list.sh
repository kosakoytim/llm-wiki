#!/usr/bin/env bash
section "17. List (page enumeration)"

run      "list returns pages"       "concept"  $CLI list
run      "list json format"         "" \
         bash -c "$CLI list --format json | jq -e '.pages | length > 0' > /dev/null"
run      "list filters by type"     "concept"  $CLI list --type concept
run      "list json type filter"    "concept" \
         bash -c "$CLI list --type concept --format json | jq -r '.pages[0].type // empty'"
run      "list pagination page 1"   "Page 1"   $CLI list --page 1 --page-size 2
