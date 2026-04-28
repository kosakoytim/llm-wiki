#!/usr/bin/env bash
section "7. Graph"

# Rebuild both wikis — cross-wiki graph needs notes index current too
$CLI index rebuild --wiki research > /dev/null 2>&1
$CLI index rebuild --wiki notes    > /dev/null 2>&1

run  "graph mermaid output"   "graph"       $CLI graph
run  "graph dot output"       "digraph"     $CLI graph --format dot
run  "graph llms output"      "type groups" $CLI graph --format llms
run  "graph type filter"      ""            $CLI graph --type concept
run  "graph root + depth"     ""            $CLI graph \
     --root concepts/mixture-of-experts --depth 2
run  "graph cross-wiki includes notes wiki"  "Attention Mechanism" \
     $CLI graph --cross-wiki
