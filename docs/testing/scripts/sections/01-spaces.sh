#!/usr/bin/env bash
section "1. Space management"

run      "spaces list returns both wikis"       "research"    $CLI spaces list
run      "spaces list shows default marker"     '\* research' $CLI spaces list
run      "spaces list json has research entry"  "" \
         bash -c "$CLI spaces list --format json | jq -e '.[] | select(.name==\"research\")' > /dev/null"
run      "spaces set-default notes"             ""            $CLI spaces set-default notes
run      "spaces set-default back to research"  ""            $CLI spaces set-default research

# ── register ──────────────────────────────────────────────────────────────────
REGISTER_DIR="$TEST_DIR/wikis/register-test"
mkdir -p "$REGISTER_DIR/content"

run      "spaces register creates entry" \
         "register-test" \
         $CLI spaces register --name "register-test" \
              --wiki-root "content" \
              --description "integration test wiki" \
              "$REGISTER_DIR"

run      "spaces register wiki.toml has name" \
         "register-test" \
         bash -c "cat '$REGISTER_DIR/wiki.toml'"

run      "spaces register wiki.toml has wiki_root" \
         "wiki_root" \
         bash -c "cat '$REGISTER_DIR/wiki.toml'"

run      "spaces register creates inbox dir" \
         "" \
         bash -c "[ -d '$REGISTER_DIR/inbox' ] && echo ok"

run      "spaces register creates schemas dir" \
         "" \
         bash -c "[ -d '$REGISTER_DIR/schemas' ] && echo ok"

# ── remove ────────────────────────────────────────────────────────────────────
run      "spaces remove unregisters wiki" \
         "Removed" \
         $CLI spaces remove register-test

run_nocheck "spaces list no longer shows register-test" \
         "" \
         bash -c "$CLI spaces list 2>&1 | grep -v 'register-test'"
