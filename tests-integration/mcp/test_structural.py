async def test_lint_articulation_point_findings_array(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"rules": "articulation-point"})
    assert isinstance(data["findings"], list)


async def test_lint_bridge_findings_array(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"rules": "bridge"})
    assert isinstance(data["findings"], list)


async def test_lint_periphery_findings_array(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"rules": "periphery"})
    assert isinstance(data["findings"], list)


async def test_lint_all_rules_includes_structural(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {})
    rules_seen = {f["rule"] for f in data["findings"]}
    structural = {"articulation-point", "bridge", "periphery"}
    assert structural & rules_seen, f"no structural rules in: {rules_seen}"


async def test_lint_articulation_point_finding_has_slug(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"rules": "articulation-point"})
    for f in data["findings"]:
        assert len(f["slug"]) > 0
