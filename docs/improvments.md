# Improvements

## config

parameter global only should not be updated locally

## ACP streaming

→ [acp-tasks.md](acp-tasks.md) · [ACP SDK reference](implementation/acp-sdk.md) · [ACP transport spec](specifications/integrations/acp-transport.md)


## ingest

→ [ingest-auto-commit.md](ingest-auto-commit.md)

## release

- [ ] Fix `docs/release.md` — currently a verbatim copy of agentctl's; still says "agentctl" in title and post-release sections
- [ ] Replace agentctl-specific post-release steps (homebrew formula `agentctl.rb`, `agent-skills/agentctl`, `asdf-agentctl`, `chocolatey-agentctl`) with llm-wiki equivalents or remove if not applicable yet
- [ ] Decide which distribution channels apply to llm-wiki (homebrew, asdf, chocolatey, cargo-binstall) and document only those


### Distribution channels (new repos/configs needed)
- [ ] Homebrew formula in `homebrew-agent` tap (`Formula/llm-wiki.rb` or `Formula/wiki.rb`)
- [ ] asdf plugin (`asdf-llm-wiki`)
- [ ] Chocolatey package (`chocolatey-llm-wiki`)
- [ ] Verify `cargo-binstall` metadata works (already has `[package.metadata.binstall]`)



## Graph

 graph.type — documented but not yet implemented in set_global_config_value. A