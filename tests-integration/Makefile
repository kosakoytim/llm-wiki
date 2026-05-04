BINARY ?= ../target/debug/llm-wiki

.PHONY: install test test-engine test-acp test-mcp lint lint-fix

install:
	uv sync --group dev

test: install
	LLM_WIKI_BIN=$(BINARY) uv run pytest -v

test-engine: install
	LLM_WIKI_BIN=$(BINARY) uv run pytest engine/ -v

test-acp: install
	LLM_WIKI_BIN=$(BINARY) uv run pytest acp/ -v

test-mcp: install
	LLM_WIKI_BIN=$(BINARY) uv run pytest mcp/ -v

lint: install
	uv run ruff check .

lint-fix: install
	uv run ruff check --fix .
