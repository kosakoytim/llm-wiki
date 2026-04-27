---
title: "Privacy Redaction"
summary: "Opt-in redaction pass on wiki_ingest: built-in secret patterns with per-wiki disable/extend via wiki.toml."
status: proposed
last_updated: "2026-04-27"
---

# Privacy Redaction

## Problem

When content is ingested into a wiki from external sources — web clips, session
transcripts, raw notes — it may contain secrets that should never be committed to
a git repository: API keys, personal access tokens, email addresses, cloud
credentials. Once committed, these are in git history permanently unless the
history is rewritten, which is destructive and error-prone.

The ingest pipeline currently performs no content inspection. It normalizes line
endings and validates frontmatter but passes the body through to the git commit
unchanged. A user or LLM agent ingesting a session transcript that incidentally
contains an API key will silently commit that key.

## Goals

- Detect and redact known secret patterns from page bodies before git commit.
- Redaction is opt-in (a flag), not applied silently to all ingests.
- Redaction is lossy by design: the original value is replaced by a placeholder;
  the redacted content is what gets committed. There is no recovery path.
- Report what was redacted so the author knows what was removed.
- Built-in patterns cover universal secrets; per-wiki config handles context-specific
  cases without replacing the defaults.

## Solution

Add a `redact: bool` parameter to `wiki_ingest`. When `true`, run the body of each
file through a redaction pass before validation and commit.

### Built-in patterns

Universal secrets that should never appear in any wiki. Always active when
`redact: true`. Defined as a static slice in `src/ops/redact.rs`.

| Pattern name | Regex (simplified) | Replacement |
|---|---|---|
| `github-pat` | `ghp_[A-Za-z0-9]{36}` | `[REDACTED:github-pat]` |
| `openai-key` | `sk-[A-Za-z0-9]{48}` | `[REDACTED:openai-key]` |
| `anthropic-key` | `sk-ant-[A-Za-z0-9\-]{90,}` | `[REDACTED:anthropic-key]` |
| `aws-access-key` | `AKIA[0-9A-Z]{16}` | `[REDACTED:aws-access-key]` |
| `bearer-token` | `Bearer [A-Za-z0-9\-._~+/]{20,}` | `[REDACTED:bearer-token]` |
| `email` | standard RFC 5322 simplified | `[REDACTED:email]` |

### Per-wiki configuration (`wiki.toml`)

Not all patterns are universal. Email addresses belong in a wiki about people;
internal hostnames belong in an infrastructure wiki. Wikis can tune the pattern set
via `wiki.toml`, which is committed to git — changes are versioned and auditable.

```toml
[redact]
# Disable specific built-in patterns for this wiki
disable = ["email"]

# Add custom patterns
[[redact.patterns]]
name        = "internal-hostname"
pattern     = "corp\\.internal\\.[a-z]+"
replacement = "[REDACTED:internal-hostname]"

[[redact.patterns]]
name        = "employee-id"
pattern     = "EMP-[0-9]{6}"
replacement = "[REDACTED:employee-id]"
```

Effective pattern set = built-ins minus `disable` plus `[[redact.patterns]]`.
An empty `[redact]` section changes nothing — built-ins remain active.

### Report

Each match produces a `RedactionMatch { pattern_name, line_number }`.
The report lists pattern names and line numbers but never the original values.

```rust
struct IngestReport {
    // existing fields ...
    redacted: Vec<RedactionReport>,
}

struct RedactionReport {
    slug: String,
    matches: Vec<RedactionMatch>,
}
```

**Scope**: body only, not frontmatter. Frontmatter is structured YAML; redacting
it would likely corrupt the document. Frontmatter redaction is a future extension.

## Tasks

- [ ] Add `src/ops/redact.rs`; define `RedactPattern { name, regex, replacement }` and `BUILTIN_PATTERNS` static slice with the 6 initial patterns.
- [ ] Add `fn build_patterns(config: &RedactConfig) -> Vec<RedactPattern>` — merges built-ins minus disabled plus custom patterns from `wiki.toml`.
- [ ] Add `fn redact_body(body: &str, patterns: &[RedactPattern]) -> (String, Vec<RedactionMatch>)`.
- [ ] Add `RedactConfig` struct to `src/config.rs` with `disable: Vec<String>` and `patterns: Vec<CustomPattern>`; wire into `WikiConfig` under `[redact]`.
- [ ] Update `wiki.toml` spec (`docs/specifications/model/wiki-toml.md`): add `[redact]` section with `disable` and `[[redact.patterns]]` fields.
- [ ] Update ingest pipeline spec (`docs/specifications/engine/ingest-pipeline.md`): add redaction as step 0 (before frontmatter parse); add `redact: bool` to the tool parameter table; note that "the file on disk is never modified" now has an exception when `redact: true`.
- [ ] Update ingest tool spec (`docs/specifications/tools/ingest.md`): document `--redact` flag; add `redacted` field to the JSON output example.
- [ ] Add `redact: bool` parameter to `wiki_ingest` MCP tool definition in `src/tools.rs`.
- [ ] In `src/ingest.rs`, when `options.redact` is true, call `redact_body` on each file's content before `validate_file`; accumulate `RedactionReport` per file.
- [ ] Add `redacted: Vec<RedactionReport>` to `IngestReport`; `#[serde(default)]` for backwards compatibility.
- [ ] Update CLI text output to report redaction count: `Ingested: 3 pages, 2 redactions`.
- [ ] Unit test: each built-in pattern matches its canonical example and is replaced correctly.
- [ ] Unit test: `disable = ["email"]` removes the email pattern from the effective set.
- [ ] Unit test: custom pattern in config is applied alongside built-ins.
- [ ] Unit test: `redact: false` skips the redaction pass entirely (no performance cost).
