#!/usr/bin/env bash
# Requires: RESEARCH_ROOT, CLI, FIXTURES set by caller
section "5. Ingest"

run "ingest dry-run inbox/"       "dry run"  $CLI ingest inbox/ --dry-run
run "ingest single file dry-run"  "dry run"  \
    $CLI ingest "inbox/01-paper-switch-transformer.md" --dry-run

# Real ingest of a copy (paths relative to wiki_root = repo/wiki/)
cp "$RESEARCH_ROOT/wiki/inbox/01-paper-switch-transformer.md" \
   "$RESEARCH_ROOT/wiki/inbox/test-ingest.md"
run "ingest real file"       "Ingested"  $CLI ingest "inbox/test-ingest.md"
run "ingest dry-run incremental"  ""     $CLI ingest inbox/ --dry-run

# Redaction
cp "$RESEARCH_ROOT/wiki/inbox/03-note-with-secrets.md" \
   "$RESEARCH_ROOT/wiki/inbox/secrets-test.md"
run "ingest with redact flag"  "redacted"  $CLI ingest "inbox/secrets-test.md" --redact

# Verify redaction worked — check only the redacted copy, not the source fixture
SECRETS_FILE="$RESEARCH_ROOT/wiki/inbox/secrets-test.md"
if grep -q "sk-ant-api03" "$SECRETS_FILE" 2>/dev/null; then
    fail "redaction: Anthropic key was NOT redacted in secrets-test.md"
else
    pass "redaction: Anthropic key was redacted"
fi
if grep -q "REDACTED" "$SECRETS_FILE" 2>/dev/null; then
    pass "redaction: REDACTED placeholder present in output"
else
    fail "redaction: REDACTED placeholder not found"
fi
