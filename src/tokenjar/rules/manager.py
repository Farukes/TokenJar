"""Agent steering rules manager for TokenJar.

Generates and injects non-intrusive instructions into AGENTS.md, .cursorrules,
.windsurfrules, and CLAUDE.md to guarantee AI models actively prioritize
TokenJar MCP tools over native, unoptimized tools.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

RULES_MARKER_START = "# >>> tokenjar-rules >>>"
RULES_MARKER_END = "# <<< tokenjar-rules <<<"


def generate_rules(compact_output: bool = True, prevent_truncation: bool = True) -> str:
    """Generate non-intrusive agent steering instructions with optional output optimization."""
    lines = [
        RULES_MARKER_START,
        "## 🔋 TokenJar AI Optimization Guidelines (STRICT ENFORCEMENT)",
        "CRITICAL MANDATE: Do NOT use native file viewers (view_file, cat) or raw terminal commands when TokenJar MCP tools are available. Bypassing TokenJar wastes hundreds of thousands of tokens and triggers conversation context window compaction.",
        "",
        "1. **File Reading & Inspections (MANDATORY):**",
        "   - ALWAYS use `read_file_smart` instead of native file viewers. It caches file contents, returns compact diffs on edits, and supports `symbol`, `start_line`, and `end_line` parameters for targeted function and line range slicing.",
        "2. **Codebase Exploration & Symbol Search (MANDATORY):**",
        "   - ALWAYS use `find_symbol_global` to locate functions, classes, or methods instantly across the codebase.",
        "   - ALWAYS use `find_symbol_references` before editing or refactoring code to check blast radius (all callers, usages, and imports).",
        "   - ALWAYS use `tool_get_code_skeleton` to inspect classes, signatures, and docstrings before reading full file implementations.",
        "   - ALWAYS use `get_repo_map_tool` to explore repository architecture instead of listing and reading multiple files.",
        "3. **Terminal & Test Execution (MANDATORY):**",
        "   - Use `run_command_smart` or `filter_output` for test runners (`pytest`, `npm test`, `cargo test`, `jest`) to prune repetitive passing logs.",
    ]
    if compact_output:
        lines.extend(
            [
                "4. **Output Optimization & Code Quality Mandate (STRICT):**",
                '   - Targeted File Slices: When inspecting specific functions or line ranges, pass `symbol="function_name"` or `start_line`/`end_line` to `read_file_smart` to avoid dumping whole files into context.',
                "   - Surgical File Edits: When modifying code, use surgical replacement blocks targeting precise line ranges instead of rewriting entire unchanged files.",
            ]
        )
        if prevent_truncation:
            lines.append(
                "   - ZERO TRUNCATION MANDATE (Anti-Lazy Coder): NEVER use placeholder comments (e.g. '// ... rest of code unchanged ...' or 'TODO: keep existing logic') or omit required logic. Every generated or replaced code block must be complete, functional, and syntactically valid."
            )
        lines.append(
            "   - High-Density Rationale: Omit conversational pleasantries, introductory filler, and restating line-by-line code changes. Prioritize direct, rigorous technical justification, architectural context, and concrete solutions."
        )
    lines.append(RULES_MARKER_END)
    return "\n".join(lines)


TOKENJAR_AGENT_RULES = generate_rules(compact_output=True, prevent_truncation=True)


class RulesManager:
    """Manages injection and removal of agent steering rules across projects."""

    RULES_CONTENT = TOKENJAR_AGENT_RULES

    SUPPORTED_RULE_FILES = [
        "AGENTS.md",
        ".cursorrules",
        ".windsurfrules",
        "CLAUDE.md",
    ]

    RULE_FILE_MAPPING = {
        "AGENTS.md": "Antigravity (AGY)",
        ".cursorrules": "Cursor",
        ".windsurfrules": "Windsurf",
        "CLAUDE.md": "Claude Code",
    }

    @classmethod
    def record_project(cls, project_path: Path | str) -> None:
        """Record an initialized project directory in ~/.tokenjar/projects.json."""
        try:
            p_dir = Path.home() / ".tokenjar"
            p_dir.mkdir(parents=True, exist_ok=True)
            p_file = p_dir / "projects.json"
            resolved = str(Path(project_path).resolve())
            projects = set()
            if p_file.exists():
                try:
                    projects = set(json.loads(p_file.read_text(encoding="utf-8")))
                except Exception:
                    pass
            projects.add(resolved)
            p_file.write_text(json.dumps(sorted(projects), indent=2), encoding="utf-8")
        except Exception:
            pass

    @classmethod
    def get_known_project_roots(cls) -> list[Path]:
        """Return all project roots ever initialized or indexed by TokenJar."""
        roots = set()
        # 1. From projects.json
        p_file = Path.home() / ".tokenjar" / "projects.json"
        if p_file.exists():
            try:
                for p in json.loads(p_file.read_text(encoding="utf-8")):
                    p_path = Path(p)
                    if p_path.exists():
                        roots.add(p_path)
            except Exception:
                pass

        # 2. From SQLite PersistentCache
        try:
            from tokenjar.cache.persistent_cache import PersistentCache

            cache = PersistentCache()
            with cache._get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute(
                    "SELECT DISTINCT project_root FROM symbol_index UNION SELECT DISTINCT project_root FROM file_index_meta"
                )
                for row in cursor.fetchall():
                    if row and row[0]:
                        p_path = Path(row[0])
                        if p_path.exists():
                            roots.add(p_path)
        except Exception:
            pass

        # 3. Current working directory
        roots.add(Path.cwd())
        return sorted(roots)

    @classmethod
    def install_rules(
        cls,
        target_dir: Path | str = ".",
        compact_output: bool | None = None,
        prevent_truncation: bool | None = None,
        only_installed: bool = False,
    ) -> list[tuple[str, bool, str]]:
        """Install or update TokenJar steering rules in the specified project directory."""
        results = []
        project_path = Path(target_dir).resolve()
        cls.record_project(project_path)

        if compact_output is None or prevent_truncation is None:
            from tokenjar.config import load_config

            cfg = load_config(project_path)
            if compact_output is None:
                compact_output = cfg.compact_output
            if prevent_truncation is None:
                prevent_truncation = cfg.prevent_truncation

        rules_to_inject = generate_rules(compact_output=compact_output, prevent_truncation=prevent_truncation)

        from tokenjar.hooks.manager import HookManager

        # Identify installed AI coding assistants if only_installed is True
        installed_clis = set()
        if only_installed:
            for fname, cli_name in cls.RULE_FILE_MAPPING.items():
                if HookManager.is_cli_installed(cli_name):
                    installed_clis.add(cli_name)
            # If no assistant is detected on host, default to universal AGENTS.md
            if not installed_clis:
                installed_clis.add("Antigravity (AGY)")

        for filename in cls.SUPPORTED_RULE_FILES:
            file_path = project_path / filename
            cli_name = cls.RULE_FILE_MAPPING.get(filename, "")

            # If only_installed is enabled, skip creating rule files for CLIs not on this system
            if only_installed and not file_path.exists() and cli_name not in installed_clis:
                results.append((filename, False, f"Skipped ({cli_name} is not installed)"))
                continue
            try:
                if file_path.exists():
                    current_content = file_path.read_text(encoding="utf-8")
                    if RULES_MARKER_START in current_content:
                        # Update existing block
                        pattern = rf"{re.escape(RULES_MARKER_START)}.*?{re.escape(RULES_MARKER_END)}"
                        updated = re.sub(pattern, rules_to_inject.strip(), current_content, flags=re.DOTALL)
                        file_path.write_text(updated, encoding="utf-8")
                        results.append((filename, True, f"Updated existing rules in {file_path.name}"))
                    else:
                        updated = current_content.rstrip() + "\n\n" + rules_to_inject.strip() + "\n"
                        file_path.write_text(updated, encoding="utf-8")
                        results.append((filename, True, f"Appended rules to {file_path.name}"))
                else:
                    file_path.write_text(rules_to_inject.strip() + "\n", encoding="utf-8")
                    results.append((filename, True, f"Created {file_path.name} with rules"))
            except Exception as e:
                results.append((filename, False, f"Failed updating {filename}: {e}"))

        return results

    @classmethod
    def remove_rules(cls, target_dir: Path | str = ".") -> list[tuple[str, bool, str]]:
        """Safely remove TokenJar steering rules from all project rule files."""
        results = []
        project_path = Path(target_dir).resolve()

        for filename in cls.SUPPORTED_RULE_FILES:
            file_path = project_path / filename
            if not file_path.exists():
                continue
            try:
                content = file_path.read_text(encoding="utf-8")
                if RULES_MARKER_START in content:
                    pattern = rf"{re.escape(RULES_MARKER_START)}.*?{re.escape(RULES_MARKER_END)}\s*"
                    clean = re.sub(pattern, "", content, flags=re.DOTALL).strip()
                    if not clean:
                        file_path.unlink()
                        results.append((filename, True, f"Removed empty {file_path.name}"))
                    else:
                        file_path.write_text(clean + "\n", encoding="utf-8")
                        results.append((filename, True, f"Removed rules block from {file_path.name}"))
            except Exception as e:
                results.append((filename, False, f"Failed cleaning {filename}: {e}"))

        # Also remove project-level config if present
        cfg_toml = project_path / "tokenjar.toml"
        if cfg_toml.exists():
            try:
                cfg_toml.unlink()
                results.append(("tokenjar.toml", True, "Removed tokenjar.toml configuration"))
            except Exception:
                pass

        return results

    @classmethod
    def set_output_mode(
        cls,
        target_dir: Path | str = ".",
        enabled: bool = True,
    ) -> tuple[bool, str, list[str]]:
        """Configure output optimization mode in tokenjar.toml and refresh rule files."""
        project_path = Path(target_dir).resolve()
        updated_files = []

        # 1. Update or create tokenjar.toml
        toml_path = project_path / "tokenjar.toml"
        val_str = "true" if enabled else "false"
        try:
            if toml_path.exists():
                content = toml_path.read_text(encoding="utf-8")
                if "[output]" in content:
                    if re.search(r"compact_mode\s*=", content):
                        new_content = re.sub(
                            r"(compact_mode\s*=\s*)(true|false)",
                            rf"\g<1>{val_str}",
                            content,
                            flags=re.IGNORECASE,
                        )
                    else:
                        new_content = re.sub(
                            r"(\[output\]\s*)",
                            rf"\1compact_mode = {val_str}\n",
                            content,
                            count=1,
                        )
                else:
                    new_content = (
                        content.rstrip() + f"\n\n[output]\ncompact_mode = {val_str}\nprevent_truncation = true\n"
                    )
                toml_path.write_text(new_content, encoding="utf-8")
            else:
                new_content = (
                    f"# TokenJar Project Configuration\n[output]\ncompact_mode = {val_str}\nprevent_truncation = true\n"
                )
                toml_path.write_text(new_content, encoding="utf-8")
            updated_files.append("tokenjar.toml")
        except Exception as e:
            return False, f"Failed updating tokenjar.toml: {e}", updated_files

        # 2. Update agent steering rule files
        rule_results = cls.install_rules(
            project_path,
            compact_output=enabled,
            prevent_truncation=True,
        )
        for fname, ok, _ in rule_results:
            if ok:
                updated_files.append(fname)

        mode_name = "COMPACT (Optimized)" if enabled else "DEFAULT (Normal/Verbose)"
        msg = f"Output optimization set to {mode_name}."
        return True, msg, updated_files

    @classmethod
    def get_output_mode(cls, target_dir: Path | str = ".") -> bool:
        """Get the current output optimization state from project configuration."""
        from tokenjar.config import load_config

        cfg = load_config(Path(target_dir).resolve())
        return cfg.compact_output
