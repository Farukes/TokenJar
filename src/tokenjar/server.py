"""TokenJar MCP Server.

Central FastMCP server that registers all token-saving tools.
Communicates with AI coding assistants via stdio (JSON-RPC 2.0).
"""

import sys

from fastmcp import FastMCP

from tokenjar.tools.output_pruner import register_output_pruner_tools
from tokenjar.tools.repo_map import register_repo_map_tools
from tokenjar.tools.skeleton import register_skeleton_tools
from tokenjar.tools.smart_reader import register_smart_reader_tools
from tokenjar.tools.symbol_index import register_symbol_index_tools

# Create the MCP server instance
mcp = FastMCP(
    name="tokenjar",
    version="1.1.0",
)

# Register all tool modules
register_skeleton_tools(mcp)
register_smart_reader_tools(mcp)
register_output_pruner_tools(mcp)
register_repo_map_tools(mcp)
register_symbol_index_tools(mcp)


# Register MCP Resources
@mcp.resource("tokenjar://stats")
def resource_stats() -> str:
    """Live cumulative token savings dashboard and metrics."""
    try:
        from tokenjar.telemetry.stats import tracker

        return tracker.render_dashboard()
    except Exception as e:
        return f"Error retrieving stats: {e}"


@mcp.resource("tokenjar://guide")
def resource_guide() -> str:
    """Best-practice guidelines for AI agents to minimize token consumption."""
    from tokenjar.rules.manager import RulesManager

    return RulesManager.RULES_CONTENT


@mcp.resource("tokenjar://config")
def resource_config() -> str:
    """Active TokenJar configuration settings."""
    import json
    from dataclasses import asdict

    from tokenjar.config import load_config

    return json.dumps(asdict(load_config()), indent=2)


# Register MCP Prompts
@mcp.prompt("optimize_coding_task")
def prompt_optimize_task(task_description: str) -> str:
    """System prompt template directing the AI assistant to execute a coding task with minimal tokens."""
    return (
        f"You are executing the following coding task: '{task_description}'\n\n"
        "Strictly adhere to the TokenJar AI Optimization Guidelines:\n"
        "1. Start by calling 'find_symbol_global' or 'tool_get_code_skeleton' to inspect signatures and API shapes.\n"
        "2. Do NOT read entire files with standard viewers; use 'read_file_smart' to fetch compact diffs.\n"
        "3. Run all tests and terminal builds through 'run_command_smart' to prune repetitive passing output.\n"
        "4. Focus edits strictly on the necessary symbols with surgical block replacements.\n"
        "5. ZERO TRUNCATION MANDATE: Never use placeholder comments like '// ... rest unchanged'. Generated code must remain 100% complete, fully implemented, and syntactically valid.\n"
    )


# Ensure debug output goes to stderr, never stdout (MCP protocol requirement)
if not sys.stderr.isatty():
    import logging

    logging.basicConfig(stream=sys.stderr, level=logging.WARNING)
