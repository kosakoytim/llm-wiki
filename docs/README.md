---
title: "Documentation"
summary: "Index of all llm-wiki documentation."
status: active
last_updated: "2025-07-17"
---

# Documentation

## Design Documents

| Document                                                     | Type   | Description                                             |
| ------------------------------------------------------------ | ------ | ------------------------------------------------------- |
| [overview.md](overview.md)                                   | design | Project introduction, architecture, core concepts       |
| [focused-llm-wiki-design.md](focused-llm-wiki-design.md)     | design | Focused engine: 16 MCP tools, skills in separate repo   |
| [type-specific-frontmatter.md](type-specific-frontmatter.md) | design | JSON Schema type profiles, wiki.toml registry, aliases  |
| [roadmap.md](roadmap.md)                                     | design | Phased roadmap from spec cleanup through skill registry |
| [skills-architecture.md](skills-architecture.md)             | design | Skills separation: engine vs plugin vs wiki skills      |

## Reference

| Document                         | Type      | Description                    |
| -------------------------------- | --------- | ------------------------------ |
| [features.md](features.md)       | reference | What llm-wiki can do           |
| [diagrams.md](diagrams.md)       | reference | Architecture and flow diagrams |
| [release.md](release.md)         | reference | Release notes                  |
| [improvments.md](improvments.md) | reference | Improvement ideas              |

## Directories

| Directory                                   | Type           | Description                                                |
| ------------------------------------------- | -------------- | ---------------------------------------------------------- |
| [specifications/](specifications/README.md) | spec           | Formal specifications (model, tools, engine, integrations) |
| [implementation/](implementation/)          | implementation | Implementation notes (Rust, ACP SDK)                       |
| [decisions/](decisions/)                    | decision       | Architectural decisions and rationale                      |
| [prompts/](prompts/)                        | prompt         | Session prompts for LLM-driven work                        |
| [archive/](archive/)                        | archive        | Historical session logs and early ideas                    |
