BINARY ?= ../target/debug/llm-wiki

.PHONY: install test test-engine test-acp test-mcp

install:
	uv sync

test: install
	LLM_WIKI_BIN=$(BINARY) uv run pytest -v

test-engine: install
	LLM_WIKI_BIN=$(BINARY) uv run pytest engine/ -v

test-acp: install
	LLM_WIKI_BIN=$(BINARY) uv run pytest acp/ -v

test-mcp: install
	LLM_WIKI_BIN=$(BINARY) uv run pytest mcp/ -v
