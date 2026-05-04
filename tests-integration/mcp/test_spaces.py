async def test_spaces_list_returns_research(mcp_env):
    text = await mcp_env.call("wiki_spaces_list")
    assert "research" in text


async def test_spaces_list_json_contains_research(mcp_env):
    data = await mcp_env.json("wiki_spaces_list")
    assert any(w["name"] == "research" for w in data)


async def test_spaces_set_default_research(mcp_env):
    text = await mcp_env.call("wiki_spaces_set_default", {"name": "research"})
    assert "research" in text
