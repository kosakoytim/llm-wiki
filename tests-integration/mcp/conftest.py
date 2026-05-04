import json

import pytest
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


class McpEnv:
    def __init__(self, session: ClientSession):
        self._session = session

    async def call(self, tool: str, args: dict | None = None) -> str:
        result = await self._session.call_tool(tool, args or {})
        if not result.content:
            raise AssertionError(f"call_tool({tool!r}) returned empty content")
        if getattr(result, "isError", False):
            raise AssertionError(f"call_tool({tool!r}) returned error: {result.content[0].text}")
        return result.content[0].text

    async def json(self, tool: str, args: dict | None = None) -> dict | list:
        text = await self.call(tool, args)
        return json.loads(text)


@pytest.fixture()
async def mcp_env(wiki_env):
    server = StdioServerParameters(
        command=wiki_env.binary,
        args=["--config", str(wiki_env.config), "serve", "--stdio"],
    )
    async with stdio_client(server) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            yield McpEnv(session)
