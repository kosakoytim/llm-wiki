#!/usr/bin/env bash
section "7. Graph"

run  "graph mermaid output"   "graph"    $CLI graph
run  "graph dot output"       "digraph"  $CLI graph --format dot
run  "graph llms output"      "cluster"  $CLI graph --format llms
run  "graph type filter"      ""         $CLI graph --type concept
run  "graph root + depth"     ""         $CLI graph \
     --root concepts/mixture-of-experts --depth 2
run  "graph cross-wiki"       "external\|notes\|attention" \
     $CLI graph --cross-wiki
