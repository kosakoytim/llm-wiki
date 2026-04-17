---
title: "ACP Workflow Dispatch"
summary: "How ACP prompt dispatch works — slash commands, engine-executed vs skill-delegated workflows, instruction streaming."
status: draft
last_updated: "2025-07-15"
---

# ACP Workflow Dispatch

The ACP agent receives user prompts and dispatches them to workflows.
Some workflows are executed by the engine directly. Others delegate to
the IDE's LLM by streaming skill instructions.

---

## 1. The Problem

The current dispatch uses keyword matching on prompt text. This is fragile:
- "add some context" triggers ingest
- Any path-like string triggers ingest
- No way to explicitly request a workflow

The ACP agent is not an LLM — it can search and lint, but it can't write
pages or synthesize knowledge. Workflows like ingest and crystallize
require an LLM to do the actual work.

---

## 2. Prefix Command Dispatch

The user prefixes the prompt with a command:

```
llm-wiki:<workflow> [prompt text]
```

No slash — avoids IDE interception. Mirrors the Claude plugin convention
(`/llm-wiki:ingest`) without the slash, since ACP prompts are plain text
that no IDE will intercept.

### Parsing

```rust
fn parse_dispatch(prompt: &str) -> (&str, &str) {
    if let Some(rest) = prompt.strip_prefix("llm-wiki:") {
        let (cmd, text) = rest.split_once(' ').unwrap_or((rest, ""));
        (cmd.trim(), text.trim())
    } else {
        // Fallback: keyword matching
        let workflow = keyword_match(prompt);
        (workflow, prompt)
    }
}
```

Examples:
- `llm-wiki:ingest the semantic-commit skill` → workflow `ingest`, text `the semantic-commit skill`
- `llm-wiki:lint` → workflow `lint`, text empty
- `llm-wiki:research what is MoE?` → workflow `research`, text `what is MoE?`
- `what do we know about MoE?` → fallback → keyword match → `research`

### Fallback

When no `llm-wiki:` prefix is present, fall back to keyword matching as
today. This preserves backward compatibility — existing prompts still work.

---

## 3. Two Types of Workflows

### Engine-executed

The ACP agent calls engine functions directly and streams results.
No LLM needed — the engine does the work.

| Workflow | What it does |
|----------|-------------|
| `research` | `search::search()` + `markdown::read_page()`, streams results |
| `lint` | `lint::lint()`, streams report |

These are already implemented with streaming (Tasks B and C).

### Skill-delegated

The ACP agent streams skill instructions from `llm-wiki instruct <workflow>`.
The IDE's LLM reads the instructions and executes the workflow using MCP
tools (`wiki_write`, `wiki_ingest`, `wiki_commit`, etc.).

| Workflow | Instructions from |
|----------|------------------|
| `ingest` | `llm-wiki instruct ingest` |
| `crystallize` | `llm-wiki instruct crystallize` |
| `new` | `llm-wiki instruct new` |
| `commit` | `llm-wiki instruct help` (commit section) |
| `help` | `llm-wiki instruct help` |
| `frontmatter` | `llm-wiki instruct frontmatter` |

The engine provides the playbook, the LLM executes it.

### Why the split?

- **Research** and **lint** are read-only queries the engine can answer
  fully — no LLM judgment needed.
- **Ingest** and **crystallize** require an LLM to read sources, synthesize
  pages, decide structure, write frontmatter. The engine can't do this.
- **New** and **commit** are simple commands, but in ACP context the user
  is asking the IDE's LLM to orchestrate them — the instructions tell it how.

---

## 4. Streaming Sequence

### Engine-executed (research, lint)

Already implemented:

```
→ AgentMessageChunk: "Searching for: {query}..."
→ ToolCall:          wiki_search
→ ToolCallUpdate:    Completed / Failed
→ ToolCall:          wiki_read (if results)
→ ToolCallUpdate:    Completed / Failed
→ AgentMessageChunk: summary
```

### Skill-delegated (ingest, crystallize, new, commit, help, frontmatter)

```
→ AgentMessageChunk: "Here are the instructions for the {workflow} workflow:"
→ AgentMessageChunk: <contents of llm-wiki instruct {workflow}>
```

The IDE's LLM reads these instructions as context and proceeds to execute
the workflow using MCP tools.

---

## 5. Dispatch Table

| Command | Type | Action |
|---------|------|--------|
| `llm-wiki:research` | Engine-executed | `run_research()` |
| `llm-wiki:lint` | Engine-executed | `run_lint()` |
| `llm-wiki:ingest` | Skill-delegated | Stream `instruct ingest` |
| `llm-wiki:crystallize` | Skill-delegated | Stream `instruct crystallize` |
| `llm-wiki:new` | Skill-delegated | Stream `instruct new` |
| `llm-wiki:commit` | Skill-delegated | Stream commit instructions |
| `llm-wiki:help` | Skill-delegated | Stream `instruct help` |
| `llm-wiki:frontmatter` | Skill-delegated | Stream `instruct frontmatter` |
| (no prefix) | Fallback | Keyword match → engine-executed or research default |

---

## 6. Implementation Sketch

```rust
impl WikiAgent {
    async fn dispatch(
        &self,
        session_id: &acp::SessionId,
        prompt: &str,
        wiki_entry: Option<&WikiEntry>,
        wiki_name: &str,
    ) -> Result<acp::PromptResponse, acp::Error> {
        let (workflow, text) = Self::parse_dispatch(prompt);

        match workflow {
            // Engine-executed
            "research" => self.run_research(session_id, text, wiki_entry, wiki_name).await,
            "lint" => self.run_lint(session_id, wiki_entry, wiki_name).await,

            // Skill-delegated
            "ingest" | "crystallize" | "new" | "help" | "frontmatter" | "commit" => {
                self.run_skill(session_id, workflow, text).await
            }

            // Unknown
            _ => {
                self.send_message(
                    session_id,
                    &format!("Unknown workflow: {workflow}. Use llm-wiki:help for available commands."),
                ).await?;
                Ok(acp::PromptResponse::new(acp::StopReason::EndTurn))
            }
        }
    }

    fn parse_dispatch(prompt: &str) -> (&str, &str) {
        if let Some(rest) = prompt.strip_prefix("llm-wiki:") {
            let (cmd, text) = rest.split_once(' ').unwrap_or((rest, ""));
            (cmd.trim(), text.trim())
        } else {
            // Fallback: keyword matching
            let workflow = Self::dispatch_workflow(prompt);
            (workflow, prompt)
        }
    }

    async fn run_skill(
        &self,
        session_id: &acp::SessionId,
        workflow: &str,
        target: &str,
    ) -> Result<acp::PromptResponse, acp::Error> {
        let instructions = crate::cli::extract_workflow(
            crate::cli::INSTRUCTIONS,
            workflow,
        );

        match instructions {
            Some(text) => {
                if !target.is_empty() {
                    self.send_message(
                        session_id,
                        &format!("Target: {target}\n\n{text}"),
                    ).await?;
                } else {
                    self.send_message(session_id, &text).await?;
                }
            }
            None => {
                self.send_message(
                    session_id,
                    &format!("No instructions found for workflow: {workflow}"),
                ).await?;
            }
        }

        self.clear_active_run(&session_id.to_string());
        Ok(acp::PromptResponse::new(acp::StopReason::EndTurn))
    }
}
```

---

## 7. Impact on Existing Code

| File | Change |
|------|--------|
| `src/acp.rs` | Replace `dispatch_workflow` + `catch_unwind` block with `parse_dispatch` + `dispatch` method. Add `run_skill`. Remove ingest/crystallize placeholder strings. |
| `tests/acp.rs` | Update ingest/crystallize tests to verify skill instructions are streamed. Add slash command parsing tests. |
| `docs/specifications/integrations/acp-transport.md` | Update §3.3 dispatch table. Update §3.4 ingest/crystallize section. Close open question #1 (workflow dispatch). |

---

## 8. Decisions

1. **No ToolCall for skill-delegated workflows.** The agent streams
   instructions as `AgentMessageChunk` only. A `ToolCall` would appear in
   the IDE tool call panel with no real tool execution behind it — misleading.

2. **Include prompt text in skill instructions.** When the user provides a
   target (e.g. `/llm-wiki:ingest the semantic-commit skill`), prepend it
   to the streamed instructions so the IDE's LLM has the full context:
   ```
   Target: the semantic-commit skill

   <instruct ingest output>
   ```

3. **Slash commands accept inline arguments.** `/llm-wiki:research what is MoE?`
   uses "what is MoE?" as the query. Already handled by `parse_dispatch`
   splitting on the first space.
