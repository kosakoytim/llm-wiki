# Decisions

Architectural decisions and their rationale.

| Decision                                            | Summary                                                                    |
| --------------------------------------------------- | -------------------------------------------------------------------------- |
| [engine-vs-skills](engine-vs-skills.md)             | Engine is a stateless tool provider, workflow intelligence lives in skills |
| [tool-surface](tool-surface.md)                     | 15 tools, stateful access criterion, CLI consistency                       |
| [wiki-as-skill-registry](wiki-as-skill-registry.md) | No separate skill protocol, the wiki is the registry                       |
| [schema-md-eliminated](schema-md-eliminated.md)     | Type registry to wiki.toml, conventions to skills                          |
| [three-repositories](three-repositories.md)         | Engine, skills, hugo-cms as independent repos                              |
| [json-schema-validation](json-schema-validation.md) | JSON Schema for per-type validation, x- extensions for engine behavior     |
| [typed-graph-edges](typed-graph-edges.md)           | x-graph-edges in JSON Schema for labeled directed edges                    |
| [rationalize-specs](rationalize-specs.md)           | How the specifications were rationalized — layout, decisions, rules        |
