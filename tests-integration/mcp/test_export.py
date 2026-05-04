async def _rebuild(mcp_env):
    await mcp_env.call("wiki_index_rebuild", {"wiki": "research"})


async def test_export_llms_txt_pages_written(mcp_env, wiki_env):
    await _rebuild(mcp_env)
    out = str(wiki_env.tmp / "mcp-export-test.txt")
    data = await mcp_env.json(
        "wiki_export", {"path": out, "format": "llms-txt", "wiki": "research"}
    )
    assert data["pages_written"] > 0


async def test_export_llms_full_pages_written(mcp_env, wiki_env):
    await _rebuild(mcp_env)
    out = str(wiki_env.tmp / "mcp-export-full.txt")
    data = await mcp_env.json(
        "wiki_export", {"path": out, "format": "llms-full", "wiki": "research"}
    )
    assert data["pages_written"] > 0


async def test_export_json_has_path_string(mcp_env, wiki_env):
    await _rebuild(mcp_env)
    out = str(wiki_env.tmp / "mcp-export.json")
    data = await mcp_env.json(
        "wiki_export", {"path": out, "format": "json", "wiki": "research"}
    )
    assert isinstance(data["path"], str)


async def test_export_json_bytes_gt_0(mcp_env, wiki_env):
    await _rebuild(mcp_env)
    out = str(wiki_env.tmp / "mcp-export2.json")
    data = await mcp_env.json(
        "wiki_export", {"path": out, "format": "json", "wiki": "research"}
    )
    assert data["bytes"] > 0
