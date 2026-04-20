---
title: "Planned Improvements"
summary: "Known engineering improvements — not bugs, not features, just better code."
status: ready
last_updated: "2025-07-18"
---

# Planned Improvements

Engineering improvements that don't change behavior but improve
performance, maintainability, or correctness. Not tracked in the
roadmap (those are features).

## Distribution channels

**Problem:** `docs/release.md` is a verbatim copy of agentctl's
release process — still says "agentctl" in places. Distribution
channels haven't been decided for llm-wiki.

**Fix:**

- [ ] Fix `docs/release.md` — replace agentctl references with
  llm-wiki equivalents
- [ ] Decide final channel list. Candidates:
  - `cargo install llm-wiki` — always supported (source build)
  - `cargo-binstall` — pre-built binaries via GitHub releases
    (already configured in `Cargo.toml` `[package.metadata.binstall]`)
  - Homebrew tap — macOS/Linux, low maintenance with a formula repo
  - asdf plugin — version manager integration
  - ~~Chocolatey~~ — too heavy to maintain, drop
- [ ] Document only the supported channels in README and release.md
- [ ] Verify `cargo-binstall` works with current `pkg-url` config

**Impact:** Users can't install easily without `cargo install` today.

**Blocked by:** First stable release (need binaries to distribute).

## User-facing documentation

**Problem:** The README has quick-start snippets but no detailed
guides for installation, platform-specific issues, or integration
beyond MCP config.

**Fix:**

- [ ] Installation guide (cargo install, pre-built binaries, platform
  notes, prerequisites)
- [ ] Windows installation and usage notes (path separators, git
  config, shell differences)
- [ ] IDE integration guides (VS Code, Cursor, Windsurf — beyond the
  MCP config snippets in README, covering workflow examples)
- [ ] CI/CD integration (using llm-wiki in automated pipelines —
  ingest on PR merge, index rebuild in CI, schema validation as
  a pre-commit check)

**Impact:** Adoption barrier — users who aren't Rust developers or
MCP experts can't get started easily.

**Blocked by:** Distribution channels (need installable binaries
before writing installation guides for non-Rust users).

## Implementation documentation gaps

**Problem:** Some implementation areas lack dedicated docs. New
contributors need to read source code to understand the config
system, server lifecycle, and logging.

**Fix:**

- [ ] Architecture overview (module map, data flow diagram, key
  abstractions and their relationships)
- [ ] Config system (two-level resolution, how to add a new config
  key, serde patterns, global-only vs per-wiki keys)
- [ ] Server internals (MCP stdio/SSE transport lifecycle, ACP
  agent session management, shutdown/restart behavior)
- [ ] Logging (rotation config, format options, file vs stderr,
  serve mode vs CLI mode, tracing spans)

**Impact:** Onboarding time for contributors. Currently need to
read `config.rs`, `server.rs`, `mcp/mod.rs` to understand these.

**Blocked by:** Nothing — can be written anytime from existing code.
