async def test_schema_list_returns_array(mcp_env):
    data = await mcp_env.json("wiki_schema", {"action": "list", "wiki": "research"})
    assert isinstance(data, list)


async def test_schema_list_contains_concept(mcp_env):
    data = await mcp_env.json("wiki_schema", {"action": "list", "wiki": "research"})
    assert any(s["name"] == "concept" for s in data)


async def test_schema_show_concept(mcp_env):
    text = await mcp_env.call("wiki_schema", {"action": "show", "type": "concept", "wiki": "research"})
    lower = text.lower()
    assert "title" in lower or "summary" in lower or "confidence" in lower


async def test_history_json_entries_array(mcp_env):
    data = await mcp_env.json(
        "wiki_history",
        {"slug": "concepts/mixture-of-experts", "wiki": "research", "format": "json"},
    )
    assert isinstance(data["entries"], list)


async def test_history_has_at_least_one_commit(mcp_env):
    data = await mcp_env.json(
        "wiki_history",
        {"slug": "concepts/mixture-of-experts", "wiki": "research", "format": "json"},
    )
    assert len(data["entries"]) > 0
