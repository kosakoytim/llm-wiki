import asyncio
import json

import pytest
import pytest_asyncio
from mcp.client.stdio import stdio_client

from mcp import ClientSession, StdioServerParameters


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


@pytest_asyncio.fixture()
async def mcp_env(wiki_env):
    """Standard read-only MCP session. Use mutable_mcp_env for tests that write wiki state."""
    server = StdioServerParameters(
        command=wiki_env.binary,
        args=["--config", str(wiki_env.config), "serve"],
    )

    ready: asyncio.Future = asyncio.get_event_loop().create_future()
    stop: asyncio.Event = asyncio.Event()
    env_holder: list[McpEnv] = []
    exc_holder: list[BaseException] = []

    async def _run():
        try:
            async with stdio_client(server) as (read, write), ClientSession(read, write) as session:
                await session.initialize()
                env_holder.append(McpEnv(session))
                ready.set_result(None)
                await stop.wait()
        except Exception as e:
            if not ready.done():
                ready.set_exception(e)
            else:
                exc_holder.append(e)

    task = asyncio.ensure_future(_run())
    await ready

    yield env_holder[0]

    stop.set()
    await task

    if exc_holder:
        raise exc_holder[0]


@pytest_asyncio.fixture()
async def mutable_mcp_env(wiki_env):
    """Same as mcp_env but signals that the test will mutate wiki state."""
    server = StdioServerParameters(
        command=wiki_env.binary,
        args=["--config", str(wiki_env.config), "serve"],
    )

    ready: asyncio.Future = asyncio.get_event_loop().create_future()
    stop: asyncio.Event = asyncio.Event()
    env_holder: list[McpEnv] = []
    exc_holder: list[BaseException] = []

    async def _run():
        try:
            async with stdio_client(server) as (read, write), ClientSession(read, write) as session:
                await session.initialize()
                env_holder.append(McpEnv(session))
                ready.set_result(None)
                await stop.wait()
        except Exception as e:
            if not ready.done():
                ready.set_exception(e)
            else:
                exc_holder.append(e)

    task = asyncio.ensure_future(_run())
    await ready

    yield env_holder[0]

    stop.set()
    await task

    if exc_holder:
        raise exc_holder[0]
