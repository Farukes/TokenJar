# TokenJar Project Guidelines for Claude Code

This project is **TokenJar**, a zero-cost MCP server that cuts 70-95% of tokens for AI coding assistants.

## Commands

- Run all tests: `python -m pytest tests/ -v`
- Run specific test: `python -m pytest tests/test_skeleton.py -v`
- Start MCP server: `python -m tokenjar`
- Lint code: `ruff check src/ tests/`

## Coding Conventions

- Python 3.10+ with `from __future__ import annotations` at the top of every file.
- All MCP tools must register via FastMCP and provide concise, clear docstrings that help LLMs select the tool.
- Always output debug logs to `sys.stderr`, NEVER `sys.stdout` (as `stdout` is reserved for JSON-RPC MCP messages).
- All file reads, parsers, and filters must be deterministic and preserve critical information (types, signatures, error tracebacks, exit codes).

# >>> tokenjar-rules >>>
## 🔋 TokenJar AI Optimization Guidelines (STRICT ENFORCEMENT)
CRITICAL MANDATE: Do NOT use native file viewers (view_file, cat) or raw terminal commands when TokenJar MCP tools are available. Bypassing TokenJar wastes hundreds of thousands of tokens and triggers conversation context window compaction.

1. **File Reading & Inspections (MANDATORY):**
   - ALWAYS use `read_file_smart` instead of native file viewers. It caches file contents, returns compact diffs on edits, and supports `symbol`, `start_line`, and `end_line` parameters for targeted function and line range slicing.
2. **Codebase Exploration & Symbol Search (MANDATORY):**
   - ALWAYS use `find_symbol_global` to locate functions, classes, or methods instantly across the codebase.
   - ALWAYS use `find_symbol_references` before editing or refactoring code to check blast radius (all callers, usages, and imports).
   - ALWAYS use `tool_get_code_skeleton` to inspect classes, signatures, and docstrings before reading full file implementations.
   - ALWAYS use `get_repo_map_tool` to explore repository architecture instead of listing and reading multiple files.
3. **Terminal & Test Execution (MANDATORY):**
   - For fast test suites and scripts (`pytest`, `npm test`, targeted unit tests): Use `run_command_smart` directly.
   - For long-running servers, bots, streamers, or watchers: ALWAYS set `background=true` in `run_command_smart` to avoid blocking or timing out the agent.
   - For heavy compilation or workspace-wide builds (e.g. `cargo test --workspace`, `cargo build --release`): Run via native terminal, then pass output through `filter_output` to prune logs without blocking the MCP connection.
4. **Output Optimization & Code Quality Mandate (STRICT):**
   - Targeted File Slices: When inspecting specific functions or line ranges, pass `symbol="function_name"` or `start_line`/`end_line` to `read_file_smart` to avoid dumping whole files into context.
   - Surgical File Edits: When modifying code, use surgical replacement blocks targeting precise line ranges instead of rewriting entire unchanged files.
   - ZERO TRUNCATION MANDATE (Anti-Lazy Coder): NEVER use placeholder comments (e.g. '// ... rest of code unchanged ...' or 'TODO: keep existing logic') or omit required logic. Every generated or replaced code block must be complete, functional, and syntactically valid.
   - High-Density Rationale: Omit conversational pleasantries, introductory filler, and restating line-by-line code changes. Prioritize direct, rigorous technical justification, architectural context, and concrete solutions.
# <<< tokenjar-rules <<<
