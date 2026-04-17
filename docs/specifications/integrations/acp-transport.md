---
title: "ACP Transport"
summary: "ACP server for Zed and VS Code agent panels — session-oriented, streaming."
read_when:
  - Integrating llm-wiki with Zed or VS Code agent panel
status: ready
last_updated: "2025-07-17"
---

# ACP Transport

ACP (Agent Client Protocol) is a session-oriented, streaming protocol
over stdio/NDJSON. `llm-wiki serve --acp` makes llm-wiki a first-class
IDE agent with zero MCP configuration required.


## Why ACP

MCP is request/response — the IDE calls a tool, gets a result, no
streaming. ACP is session-oriented and streaming — every step of a
multi-turn workflow streams back as events visible to the user.

| Concern                  | MCP stdio           | ACP stdio            |
| ------------------------ | ------------------- | -------------------- |
| Zed agent panel          | requires MCP config | native — zero config |
| Streaming workflow steps | not visible         | streams as events    |
| Session continuity       | stateless           | named sessions       |
| Cancel mid-workflow      | not supported       | `cancel` message     |


## Protocol

ACP is NDJSON over stdio. Key messages:

| Message      | Direction     | Purpose                                    |
| ------------ | ------------- | ------------------------------------------ |
| `initialize` | client → wiki | Start session, wiki sends capabilities     |
| `newSession` | client → wiki | Create named session                       |
| `prompt`     | client → wiki | Submit user message                        |
| `cancel`     | client → wiki | Cancel active run                          |
| `message`    | wiki → client | Streaming assistant text                   |
| `tool_call`  | wiki → client | Streaming tool invocation (visible in IDE) |
| `done`       | wiki → client | Run complete                               |


## Session Model

Sessions are transient conversation threads stored in memory for the
process lifetime. A session targets a specific wiki from the spaces
config (default wiki if not specified).


## Streaming

Each workflow streams intermediate events:

```
prompt: "what do we know about MoE scaling?"

→ message: "Searching for: MoE scaling..."
→ tool_call: wiki_search("MoE scaling")
→ tool_call: wiki_content_read("concepts/moe")
→ message: "Based on 2 pages: MoE reduces compute 8x..."
→ done
```


## Zed Configuration

```json
{
  "agent_servers": {
    "llm-wiki": {
      "type": "custom",
      "command": "llm-wiki",
      "args": ["serve", "--acp"],
      "env": {}
    }
  }
}
```


## What ACP Does Not Replace

- **MCP stdio** — agent pipelines, Claude Code tool calls, batch ingest
- **MCP SSE** — remote multi-client access (ACP is stdio-only)

See [server.md](../engine/server.md) for transport configuration and
resilience.
