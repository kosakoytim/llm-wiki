#!/usr/bin/env bash
section "2. Index"

run "index rebuild research"  "Indexed"   $CLI index rebuild --wiki research
run "index status research"   "research"  $CLI index status  --wiki research
run "index rebuild notes"     "Indexed"   $CLI index rebuild --wiki notes
