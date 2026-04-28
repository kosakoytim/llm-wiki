#!/usr/bin/env bash
section "1. Space management"

run      "spaces list returns both wikis"       "research"    $CLI spaces list
run      "spaces list shows default marker"     '\* research' $CLI spaces list
run      "spaces list json has research entry"  "" \
         bash -c "$CLI spaces list --format json | jq -e '.[] | select(.name==\"research\")' > /dev/null"
run      "spaces set-default notes"             ""            $CLI spaces set-default notes
run      "spaces set-default back to research"  ""            $CLI spaces set-default research
