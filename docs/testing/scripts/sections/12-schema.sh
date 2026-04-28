#!/usr/bin/env bash
section "12. Schema"

run  "schema list"          "concept"  $CLI schema list
run  "schema show concept"  "title"    $CLI schema show concept
run  "schema validate"      ""         $CLI schema validate
