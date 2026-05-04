import pytest


async def test_content_read_returns_page_body(mcp_env):
    text = await mcp_env.call("wiki_content_read", {"uri": "concepts/mixture-of-experts"})
    assert "Mixture of Experts" in text


async def test_content_read_includes_frontmatter(mcp_env):
    text = await mcp_env.call("wiki_content_read", {"uri": "concepts/mixture-of-experts"})
    assert "type:" in text


async def test_content_read_with_backlinks(mcp_env):
    text = await mcp_env.call(
        "wiki_content_read", {"uri": "concepts/mixture-of-experts", "backlinks": True}
    )
    assert "backlinks" in text


async def test_content_read_backlinks_include_scaling_laws(mcp_env):
    text = await mcp_env.call(
        "wiki_content_read", {"uri": "concepts/scaling-laws", "backlinks": True}
    )
    assert "mixture-of-experts" in text


async def test_content_read_via_wiki_uri(mcp_env):
    text = await mcp_env.call(
        "wiki_content_read", {"uri": "wiki://research/concepts/mixture-of-experts"}
    )
    assert "Mixture of Experts" in text


async def test_resolve_existing_slug_exists_true(mcp_env):
    data = await mcp_env.json("wiki_resolve", {"uri": "concepts/mixture-of-experts"})
    assert data["exists"] is True


async def test_resolve_existing_slug_has_md_path(mcp_env):
    data = await mcp_env.json("wiki_resolve", {"uri": "concepts/mixture-of-experts"})
    assert data["path"].endswith(".md")


async def test_resolve_nonexistent_slug_exists_false(mcp_env):
    data = await mcp_env.json("wiki_resolve", {"uri": "concepts/does-not-exist-xyz"})
    assert data["exists"] is False


async def test_resolve_nonexistent_slug_returns_would_be_path(mcp_env):
    data = await mcp_env.json("wiki_resolve", {"uri": "concepts/does-not-exist-xyz"})
    assert data["path"].endswith(".md")


async def test_resolve_returns_slug(mcp_env):
    data = await mcp_env.json("wiki_resolve", {"uri": "concepts/mixture-of-experts"})
    assert len(data["slug"]) > 0


@pytest.mark.skip(reason="mutates wiki state")
async def test_content_write_skipped():
    pass


@pytest.mark.skip(reason="mutates wiki state")
async def test_content_new_skipped():
    pass


@pytest.mark.skip(reason="mutates wiki state")
async def test_content_commit_skipped():
    pass
