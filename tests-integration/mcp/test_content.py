import pytest
from pathlib import Path


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


async def test_content_write_and_read_back(mutable_mcp_env):
    # Create a page first so we have a target to write to
    new_data = await mutable_mcp_env.json(
        "wiki_content_new",
        {"uri": "concepts/test-write-target", "wiki": "research"},
    )
    slug = new_data["slug"]

    # Write custom content to it
    content = "---\ntitle: Write Test\ntype: page\nstatus: draft\n---\n\nHello write test.\n"
    await mutable_mcp_env.call(
        "wiki_content_write",
        {"uri": slug, "content": content, "wiki": "research"},
    )

    # Read back and verify
    text = await mutable_mcp_env.call("wiki_content_read", {"uri": slug, "wiki": "research"})
    assert "Hello write test." in text


async def test_content_new_creates_page(mutable_mcp_env):
    data = await mutable_mcp_env.json(
        "wiki_content_new",
        {"uri": "concepts/test-new-page", "wiki": "research"},
    )
    assert data["slug"] == "concepts/test-new-page"
    assert data["path"].endswith(".md")
    assert Path(data["path"]).exists()


async def test_content_commit_after_write(mutable_mcp_env):
    # Create + write a page
    new_data = await mutable_mcp_env.json(
        "wiki_content_new",
        {"uri": "concepts/test-commit-target", "wiki": "research"},
    )
    slug = new_data["slug"]
    content = "---\ntitle: Commit Test\ntype: page\nstatus: draft\n---\n\nCommit me.\n"
    await mutable_mcp_env.call(
        "wiki_content_write",
        {"uri": slug, "content": content, "wiki": "research"},
    )

    # Commit the page
    result_text = await mutable_mcp_env.call(
        "wiki_content_commit",
        {"slugs": [slug], "message": "test: commit test page", "wiki": "research"},
    )
    assert result_text  # non-empty response
