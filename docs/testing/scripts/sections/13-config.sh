#!/usr/bin/env bash
section "13. Config"

run  "config list global"     ""  $CLI config list
run  "config get graph.format" "" $CLI config get graph.format
