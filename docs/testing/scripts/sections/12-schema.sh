#!/usr/bin/env bash
section "12. Schema"

run  "schema list"          "concept"  $CLI schema list
run  "schema show concept"  "title"    $CLI schema show concept
run  "schema validate"      ""         $CLI schema validate

# ── schema add / remove ───────────────────────────────────────────────────────

# Write a minimal valid JSON Schema 2020-12 file declaring a custom type
cat > /tmp/llm-wiki-test-custom.json << 'JSON'
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "test-custom",
  "type": "object",
  "x-wiki-types": {
    "test-custom": { "label": "Test Custom", "fields": [] }
  }
}
JSON

run      "schema add custom type" \
         "copied" \
         $CLI schema add test-custom /tmp/llm-wiki-test-custom.json

run      "schema list shows added type" \
         "test-custom" \
         $CLI schema list

run      "schema remove custom type" \
         "schema file deleted" \
         $CLI schema remove test-custom --delete

run_nocheck "schema list no longer shows removed type" \
         "" \
         bash -c "$CLI schema list 2>&1 | grep -v 'test-custom'"
