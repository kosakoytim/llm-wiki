async def test_search_returns_results(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_search", {"query": "mixture of experts"})
    assert "mixture-of-experts" in text


async def test_search_json_results_not_empty(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_search", {"query": "mixture of experts", "format": "json"})
    assert len(data["results"]) > 0


async def test_search_type_filter(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_search", {"query": "attention", "type": "concept"})
    assert "concept" in text


async def test_search_llms_format(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_search", {"query": "transformer", "format": "llms"})
    assert "wiki://" in text


async def test_list_json_total_gt_0(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_list", {"format": "json"})
    assert data["total"] > 0


async def test_list_json_pages_is_array(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_list", {"format": "json"})
    assert isinstance(data["pages"], list)


async def test_list_type_filter_returns_concept(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_list", {"type": "concept"})
    assert "concept" in text


async def test_list_json_type_filter_all_concepts(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_list", {"type": "concept", "format": "json"})
    assert all(p["type"] == "concept" for p in data["pages"])
