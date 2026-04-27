---
title: "Lint System"
summary: "wiki_lint MCP tool: deterministic index-based rules. Skill lint layer calls wiki_lint and adds judgment-based rules on top."
status: proposed
last_updated: "2026-04-27"
---

# Lint System

## Problem

Wiki quality degrades silently. Broken links, orphaned pages, missing required
frontmatter, and stale entries accumulate without any feedback loop. An LLM writing
to the wiki has no machine-readable signal to detect these problems; a human running
CI has no automated gate.

`llm-wiki-skills/skills/lint/SKILL.md` already covers structural auditing, but it
does so by composing multiple tool calls (`wiki_graph`, `wiki_list`,
`wiki_content_read`) and interpreting results manually. This is slow, non-deterministic
in output format, and not usable as a CI check. Some of its checks — orphan detection,
broken links — are pure index operations that should not require an LLM at all.

## Goal

Separate the rule set into two layers with a clear ownership boundary:

- **Engine (`wiki_lint`)**: deterministic, index-only rules. Fast, machine-readable,
  usable in CI. No LLM involvement.
- **Skill (`lint/SKILL.md`)**: judgment-based rules requiring reasoning or multi-step
  tool composition. Calls `wiki_lint` for the deterministic layer, adds its own logic
  on top.

## Rule ownership

### Engine rules (`wiki_lint`)

Pure index operations. Always deterministic, always fast.

| Rule ID | Severity | Description |
|---|---|---|
| `orphan` | Warning | Page has no incoming links and is not a root/index page |
| `broken-link` | Error | A slug in `body_links` or frontmatter slug fields is not in the index |
| `missing-fields` | Error | Required frontmatter fields (per type schema) are absent |
| `stale` | Warning | `last_updated` older than threshold AND `confidence` below threshold (if set) |
| `unknown-type` | Error | `type` field value is not registered in the type registry |

### Skill rules (`lint/SKILL.md`)

Judgment-based. The skill calls `wiki_lint()` first to get the deterministic
findings, then runs its own checks for issues that require reasoning:

| Rule | Method | Why it stays in the skill |
|---|---|---|
| Untyped sources | Inspect `type` field semantics | Requires judgment: is this page *acting* as a source? |
| Under-linked pages | `wiki_suggest` per page | Requires relevance judgment |
| Missing stubs | Interpret dead references | Requires decision: create stub or remove reference? |
| Empty sections | `wiki_list(type: section)` + child count | Simple but fix requires content decisions |
| Draft audit | `wiki_stats()` + interpret | Requires review of what draft means per page |
| Edge type mismatches | `wiki_ingest(dry_run: true)` | Already implemented in engine; skill surfaces fixes |

The skill should be updated to call `wiki_lint()` for orphan and broken-link checks
instead of reimplementing them via `wiki_graph` and `wiki_list` + `wiki_content_read`.

## Solution

### Engine: `wiki_lint` tool

```
wiki_lint()                       — all rules, default wiki
wiki_lint(wiki: "name")           — target a specific wiki
wiki_lint(rules: "orphan,stale")  — subset of rules
wiki_lint(severity: "error")      — filter output to errors only
```

**Finding structure:**
```rust
struct LintFinding {
    slug:     String,
    rule:     &'static str,
    severity: Severity,     // Error | Warning
    message:  String,
}
```

Output: JSON array of `LintFinding`. Empty array = clean. CLI exits non-zero on
any `Error` finding.

**`stale` rule and `confidence`:** when the confidence field (improvement #1) is
present in the index, the `stale` rule combines both signals:
`last_updated` older than threshold **and** `confidence < stale_confidence_threshold`.
A page that is old but `confidence: 0.9` is not stale. A page that is recent but
`confidence: 0.1` is flagged. When confidence is absent the rule falls back to
date-only.

**Configuration** (overridable in `config.toml` and `wiki.toml`):
```toml
[lint]
stale_days                = 90
stale_confidence_threshold = 0.4   # ignored if confidence field not indexed
```

### Skill: updated `lint/SKILL.md`

Replace the manual orphan and broken-link checks with:
```
wiki_lint()
```

Retain all judgment-based rules. Present `wiki_lint` findings alongside skill
findings in the same grouped report.

## Tasks

### Engine — `src/ops/lint.rs`
- [ ] Add `src/ops/lint.rs`; define `LintFinding`, `Severity`, `run_lint()` skeleton.
- [ ] Implement `orphan` rule: reverse `body_links` term query across all pages; flag slugs with zero incoming links; exclude `type: section` pages.
- [ ] Implement `broken-link` rule: for each page, check every slug in `body_links` and frontmatter slug-list fields (`sources`, `concepts`, `superseded_by`) exists in the index.
- [ ] Implement `missing-fields` rule: for each page, load its type schema; validate required fields against parsed frontmatter.
- [ ] Implement `stale` rule: parse `last_updated`; compare to `now - stale_days`; if confidence field is indexed, also require `confidence < stale_confidence_threshold`; both conditions must hold.
- [ ] Implement `unknown-type` rule: check `type` field against `TypeRegistry::known_types()`.

### Engine — config
- [ ] Add `LintConfig` to `src/config.rs` with `stale_days: u32` (default 90) and `stale_confidence_threshold: f32` (default 0.4).
- [ ] Wire into `WikiConfig` under `[lint]`; expose via `ResolvedConfig`.

### Engine — MCP + CLI
- [ ] Add `wiki_lint` to `src/tools.rs` with parameters `wiki`, `rules`, `severity`.
- [ ] Add `lint` subcommand to `src/cli.rs`; wire `--format json|text`.
- [ ] CLI exits non-zero when any `Error` findings exist.

### Skill — `llm-wiki-skills/skills/lint/SKILL.md`
- [ ] Replace manual orphan detection (`wiki_graph` walk) with `wiki_lint(rules: "orphan")`.
- [ ] Replace manual broken-link detection (`wiki_list` + `wiki_content_read` per page) with `wiki_lint(rules: "broken-link")`.
- [ ] Add `wiki_lint()` as the first step in the audit workflow; merge findings into the grouped report.

### Config spec docs
- [ ] Update `docs/specifications/model/global-config.md`: add `[lint]` to overridable defaults table.
- [ ] Update `docs/specifications/model/wiki-toml.md`: add `[lint]` to per-wiki overridable settings.

### Tool spec docs
- [ ] Create `docs/specifications/tools/lint.md`.

### Tests
- [ ] Unit test per rule: pass and fail case each.
- [ ] Integration test: create wiki with known issues; run `wiki_lint`; assert expected findings.
- [ ] `stale` rule: page old + low confidence → stale; page old + high confidence → not stale; page recent + low confidence → stale.
