async def test_stats_returns_wiki_name(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_stats")
    assert "research" in text


async def test_stats_json_pages_gt_0(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_stats", {"format": "json"})
    assert data["pages"] > 0


async def test_stats_json_orphans_gte_0(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_stats", {"format": "json"})
    assert data["orphans"] >= 0


async def test_stats_communities_present(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_stats", {"format": "json"})
    assert data.get("communities") is not None


async def test_stats_diameter_field(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_stats", {"format": "json"})
    assert data.get("diameter") is None or isinstance(data["diameter"], (int, float))


async def test_stats_center_is_array(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_stats", {"format": "json"})
    assert isinstance(data["center"], list)
