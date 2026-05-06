from conftest import SLUG_ORPHAN, SPACE_NAME


async def test_lint_returns_findings(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_lint")
    combined = text.lower()
    assert "error" in combined or "warning" in combined


async def test_lint_broken_link_rule(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_lint", {"rules": ["broken-link"]})
    assert "broken-link" in text


async def test_lint_orphan_rule(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_lint", {"rules": ["orphan"]})
    assert "orphan" in text


async def test_lint_json_findings_array(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"format": "json"})
    assert isinstance(data["findings"], list)


async def test_lint_broken_link_finds_dead_ref(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"rules": ["broken-link"], "format": "json"})
    bl = [f for f in data["findings"] if f["rule"] == "broken-link"]
    assert len(bl) > 0


async def test_lint_broken_link_detects_also_does_not_exist(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"rules": ["broken-link"], "format": "json"})
    msgs = [f["message"] for f in data["findings"] if f["rule"] == "broken-link"]
    assert any("also-does-not-exist" in m for m in msgs)


async def test_lint_orphan_finds_orphan_concept(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"rules": ["orphan"], "format": "json"})
    slugs = [f["slug"] for f in data["findings"]]
    assert SLUG_ORPHAN in slugs


async def test_lint_with_wiki_param(mcp_env):
    await mcp_env.rebuild()
    text = await mcp_env.call("wiki_lint", {"wiki": SPACE_NAME})
    combined = text.lower()
    assert "error" in combined or "warning" in combined


async def test_lint_findings_have_md_path(mcp_env):
    await mcp_env.rebuild()
    data = await mcp_env.json("wiki_lint", {"rules": ["broken-link"], "format": "json"})
    for f in data["findings"]:
        assert f.get("path", "").endswith(".md"), f"finding path not .md: {f.get('path')}"
