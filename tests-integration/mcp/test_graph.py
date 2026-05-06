async def test_graph_mermaid_output(mcp_env):
    await mcp_env.rebuild("research")
    await mcp_env.rebuild("notes")
    text = await mcp_env.call("wiki_graph")
    lower = text.lower()
    assert "graph lr" in lower or "graph td" in lower or "flowchart" in lower


async def test_graph_dot_output(mcp_env):
    await mcp_env.rebuild("research")
    await mcp_env.rebuild("notes")
    text = await mcp_env.call("wiki_graph", {"format": "dot"})
    assert "digraph" in text


async def test_graph_llms_output(mcp_env):
    await mcp_env.rebuild("research")
    await mcp_env.rebuild("notes")
    text = await mcp_env.call("wiki_graph", {"format": "llms"})
    lower = text.lower()
    assert "nodes" in lower or "edges" in lower or "type groups" in lower


async def test_graph_type_filter(mcp_env):
    await mcp_env.rebuild("research")
    await mcp_env.rebuild("notes")
    result = await mcp_env.call("wiki_graph", {"type": "concept"})
    assert result is not None


async def test_graph_root_depth(mcp_env):
    await mcp_env.rebuild("research")
    await mcp_env.rebuild("notes")
    result = await mcp_env.call(
        "wiki_graph", {"root": "concepts/mixture-of-experts", "depth": 2}
    )
    assert result is not None


async def test_graph_cross_wiki(mcp_env):
    await mcp_env.rebuild("research")
    await mcp_env.rebuild("notes")
    result = await mcp_env.call("wiki_graph", {"cross_wiki": True})
    assert result is not None
