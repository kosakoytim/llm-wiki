---
title: "Diagrams"
summary: "Mermaid diagrams for llm-wiki architecture and flows."
status: active
last_updated: "2025-07-17"
---

# Diagrams

Mermaid sources for llm-wiki diagrams.

## 1. Architecture Overview

How the engine sits between humans, LLMs, and the wiki repository.

```mermaid
graph LR
    Human([Human])
    LLM([LLM])

    subgraph Engine["wiki engine"]
        CLI[CLI]
        MCP[MCP server]
        ACP[ACP server]
    end

    subgraph Repo["git repository"]
        inbox[inbox/]
        raw[raw/]
        wiki[wiki/]
    end

    Git[(git)]
    Index[(tantivy index)]

    Human -->|drops files| inbox
    Human -->|commands| CLI
    LLM -->|tools| MCP
    LLM -->|prompts| ACP

    CLI --> wiki
    MCP --> wiki
    ACP --> wiki

    wiki --> Git
    wiki --> Index
    raw --> Git
```

References:
- [overview.md](overview.md)
- [server.md](specifications/engine/server.md)

## 2. Repository Layers

The structure of a wiki repository.

```mermaid
graph TD
    Root["my-wiki/"]

    Root --> Config["wiki.toml — config + type registry"]
    Root --> Schemas["schemas/ — JSON Schema per type"]
    Root --> Inbox["inbox/ — drop zone"]
    Root --> Raw["raw/ — immutable archive"]
    Root --> Wiki["wiki/ — compiled knowledge"]

    Inbox -..->|"human drops files"| Inbox
    Raw -..->|"originals preserved"| Raw
    Wiki -..->|"authors write here"| Wiki

    style Inbox fill:#ffeeba
    style Raw fill:#d4edda
    style Wiki fill:#cce5ff
```

References:
- [wiki-repository-layout.md](specifications/model/wiki-repository-layout.md)
- [wiki-toml.md](specifications/model/wiki-toml.md)

## 3. Ingest Pipeline

How content enters the wiki.

```mermaid
flowchart LR
    A[Author writes file\ninto wiki/ tree] --> B{wiki ingest}
    B --> C[Validate frontmatter]
    C -->|valid| D[Update tantivy index]
    C -->|invalid| E[Error — file rejected]
    D --> F{auto_commit?}
    F -->|yes| G[git add + commit]
    F -->|no| H[IngestReport returned]
    G --> H

    style E fill:#f8d7da
    style H fill:#d4edda
```

References:
- [ingest-pipeline.md](specifications/engine/ingest-pipeline.md)

## 4. LLM Ingest Workflow

The full LLM-driven ingest loop via MCP tools.

```mermaid
sequenceDiagram
    participant LLM
    participant Engine as wiki engine
    participant Repo as git repo

    LLM->>Engine: wiki_search("topic")
    Engine-->>LLM: related pages

    LLM->>Engine: wiki_content_read(hub page)
    Engine-->>LLM: current knowledge

    Note over LLM: reads wiki.toml<br/>reads inbox file<br/>synthesizes pages

    LLM->>Engine: wiki_content_write("concepts/topic.md", content)
    Engine-->>LLM: ok

    LLM->>Engine: wiki_ingest("concepts/topic.md")
    Engine->>Repo: validate → index → commit (if auto_commit)
    Engine-->>LLM: IngestReport
```

References:
- [ingest-pipeline.md](specifications/engine/ingest-pipeline.md)
- [content-operations.md](specifications/tools/content-operations.md)

## 5. Epistemic Model

The three epistemic roles and how they relate.

```mermaid
graph TD
    C["concept\nwhat we know"]
    S1["paper / article / docs\nwhat sources claim"]
    Q["query-result\nwhat we concluded"]

    S1 -->|"feeds into"| C
    C -->|"used by"| Q
    S1 -->|"cited by"| Q

    C -.-|"provenance"| S1
    Q -.-|"auditable"| S1

    style C fill:#cce5ff
    style S1 fill:#d4edda
    style Q fill:#ffeeba
```

References:
- [epistemic-model.md](specifications/model/epistemic-model.md)

## 6. RAG vs DKR

Side-by-side comparison of the two approaches.

```mermaid
flowchart LR
    subgraph RAG["Traditional RAG"]
        direction TB
        RQ[Query] --> RR[Retrieve chunks]
        RR --> RG[Generate answer]
        RG --> RA[Answer — ephemeral]
    end

    subgraph DKR["llm-wiki DKR"]
        direction TB
        DS[Source arrives] --> DI[LLM processes at ingest]
        DI --> DW[Wiki pages updated]
        DW --> DC[Knowledge compounds]
        DC -->|"next source"| DI
    end

    style RA fill:#f8d7da
    style DC fill:#d4edda
```

References:
- [overview.md](overview.md)
