#!/usr/bin/env bash
section "18. Logs"

# Seed a log file so tail/list work without a running server.
# logs_path = parent(config_file)/logs = TEST_DIR/logs
_LOGS_DIR="$(dirname "$CONFIG_FILE")/logs"
mkdir -p "$_LOGS_DIR"
printf 'line1\nline2\nline3\nline4\nline5\n' > "$_LOGS_DIR/2000-01-01.log"

run      "logs list returns log files"    "2000-01-01"  $CLI logs list
run      "logs tail returns output"       "line"        $CLI logs tail
run      "logs tail 3 lines"              "line3"       $CLI logs tail --lines 3
run      "logs clear removes files"       "removed 1"   $CLI logs clear
run      "logs list empty after clear"    "no log"      $CLI logs list
