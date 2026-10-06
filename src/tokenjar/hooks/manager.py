"""Transparent hooking manager for TokenJar.

Provides non-intrusive, transparent interception for terminal commands
across PowerShell, Bash, Zsh, and Claude Code environments.

Allows safe installation (hook) and clean, 100% reversible uninstallation (unhook).
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass
if hasattr(sys.stderr, "reconfigure"):
    try:
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

HOOK_MARKER_START = "# >>> tokenjar-hook >>>"
HOOK_MARKER_END = "# <<< tokenjar-hook <<<"

# Commands that benefit most from output filtering
DEFAULT_WRAPPED_COMMANDS = ["pytest", "jest", "vitest", "npm", "cargo"]


def is_bypass_active() -> bool:
    """Check if transparent filtering should be bypassed."""
    return os.environ.get("RAW") == "1" or os.environ.get("TOKENJAR_BYPASS") == "1" or "--raw" in sys.argv


def execute_filtered_command(command: str | list[str], cwd: str = ".") -> int:
    """Execute a command and print filtered output to stdout.

    Respects RAW=1 or --raw bypass to return raw output without modification.
    """
    if isinstance(command, list):
        if os.name == "nt":
            cmd_str = subprocess.list2cmdline(command)
        else:
            import shlex
            cmd_str = shlex.join(command)
    else:
        cmd_str = command

    # Check for --raw flag and strip it
    if "--raw" in cmd_str:
        cmd_str = cmd_str.replace("--raw", "").strip()
        bypass = True
    else:
        bypass = is_bypass_active()

    if os.name == "nt":
        exec_cmd = f'cmd.exe /s /c "{cmd_str}"'
        use_shell = False
    else:
        exec_cmd = cmd_str
        use_shell = True

    try:
        res = subprocess.run(
            exec_cmd,
            cwd=cwd,
            shell=use_shell,
            capture_output=True,
        )
        stdout_str = (res.stdout or b"").decode("utf-8", errors="replace")
        stderr_str = (res.stderr or b"").decode("utf-8", errors="replace")
        combined_output = f"{stdout_str}\n{stderr_str}".strip() if stderr_str else stdout_str
        exit_code = res.returncode

        if bypass:
            sys.stdout.write(combined_output)
            sys.stdout.flush()
            return exit_code

        from tokenjar.tools.output_pruner import filter_output_logic

        filtered = filter_output_logic(combined_output, output_type="auto", exit_code=exit_code)
        sys.stdout.write(filtered + "\n")
        sys.stdout.flush()
        return exit_code
    except Exception as e:
        sys.stderr.write(f"[tokenjar] Error executing '{cmd_str}': {e}\n")
        return 1


class HookManager:
    """Manages transparent shell and assistant hooks."""

    @staticmethod
    def get_powershell_profile_path() -> Path | None:
        """Find the PowerShell profile path on Windows or cross-platform."""
        home = Path.home()
        # Windows PowerShell standard profile location
        if os.name == "nt":
            docs = home / "Documents" / "WindowsPowerShell"
            if not docs.exists():
                docs = home / "OneDrive" / "Documents" / "WindowsPowerShell"
            profile = docs / "Microsoft.PowerShell_profile.ps1"
            return profile
        else:
            profile = home / ".config" / "powershell" / "Microsoft.PowerShell_profile.ps1"
            return profile

    @staticmethod
    def get_bash_profile_path() -> Path:
        """Find bashrc or zshrc path."""
        home = Path.home()
        zshrc = home / ".zshrc"
        if zshrc.exists():
            return zshrc
        return home / ".bashrc"

    @classmethod
    def install_shell_hook(cls, target_shell: str = "auto") -> tuple[bool, str]:
        """Install transparent shell wrappers into user's profile."""
        if target_shell == "auto":
            is_win = os.name == "nt"
            target_shell = "powershell" if is_win else "bash"

        if target_shell == "powershell":
            profile_path = cls.get_powershell_profile_path()
            if not profile_path:
                return False, "Could not determine PowerShell profile directory."

            hook_code = f"""
{HOOK_MARKER_START}
# TokenJar transparent output filters
function pytest {{ tokenjar run "pytest $args" }}
function npm {{ if ($args[0] -eq "test") {{ tokenjar run "npm $args" }} else {{ & (Get-Command -CommandType Application npm) @args }} }}
{HOOK_MARKER_END}
"""
        else:
            profile_path = cls.get_bash_profile_path()
            hook_code = f"""
{HOOK_MARKER_START}
# TokenJar transparent output filters
pytest() {{ tokenjar run "pytest $@" ; }}
npm() {{ if [ "$1" = "test" ]; then tokenjar run "npm $@"; else command npm "$@"; fi ; }}
{HOOK_MARKER_END}
"""

        try:
            profile_path.parent.mkdir(parents=True, exist_ok=True)
            current_content = profile_path.read_text(encoding="utf-8") if profile_path.exists() else ""

            if HOOK_MARKER_START in current_content:
                return True, f"Hook is already installed in {profile_path}"

            updated_content = current_content + "\n" + hook_code.strip() + "\n"
            profile_path.write_text(updated_content, encoding="utf-8")
            return True, f"Successfully installed transparent hook into {profile_path}"
        except Exception as e:
            return False, f"Failed to install shell hook: {e}"

    @classmethod
    def uninstall_shell_hook(cls) -> tuple[bool, str]:
        """Safely remove the transparent hook from all detected shell profiles."""
        removed_from = []

        # Check PowerShell
        ps_path = cls.get_powershell_profile_path()
        if ps_path and ps_path.exists():
            try:
                content = ps_path.read_text(encoding="utf-8")
                if HOOK_MARKER_START in content:
                    pattern = rf"{re.escape(HOOK_MARKER_START)}.*?{re.escape(HOOK_MARKER_END)}\s*"
                    clean = re.sub(pattern, "", content, flags=re.DOTALL)
                    ps_path.write_text(clean, encoding="utf-8")
                    removed_from.append(str(ps_path))
            except Exception:
                pass

        # Check Bash/Zsh
        bash_path = cls.get_bash_profile_path()
        if bash_path and bash_path.exists():
            try:
                content = bash_path.read_text(encoding="utf-8")
                if HOOK_MARKER_START in content:
                    pattern = rf"{re.escape(HOOK_MARKER_START)}.*?{re.escape(HOOK_MARKER_END)}\s*"
                    clean = re.sub(pattern, "", content, flags=re.DOTALL)
                    bash_path.write_text(clean, encoding="utf-8")
                    removed_from.append(str(bash_path))
            except Exception:
                pass

        if removed_from:
            return True, f"Successfully removed hooks from: {', '.join(removed_from)}"
        return True, "No active hooks were found to remove."

    @staticmethod
    def _apply_mcp_config_with_backup(config_path: Path) -> tuple[bool, str]:
        """Apply TokenJar MCP configuration while safely creating a backup of original state."""
        import json

        config_path.parent.mkdir(parents=True, exist_ok=True)
        bak_path = config_path.with_name(config_path.name + ".ts_bak")

        # Save a backup of the original file if not already backed up
        if config_path.exists() and not bak_path.exists():
            try:
                content = config_path.read_text(encoding="utf-8")
                bak_path.write_text(content, encoding="utf-8")
            except Exception:
                pass
        elif not config_path.exists() and not bak_path.exists():
            # Mark that the file did not exist originally
            try:
                bak_path.write_text("__NON_EXISTENT__", encoding="utf-8")
            except Exception:
                pass

        config: dict = {}
        if config_path.exists() and config_path.stat().st_size > 0:
            try:
                with open(config_path, "r", encoding="utf-8") as f:
                    config = json.load(f)
            except Exception:
                config = {}

        if "mcpServers" not in config:
            config["mcpServers"] = {}

        python_cmd = sys.executable if sys.executable else "python"
        config["mcpServers"]["tokenjar"] = {
            "command": python_cmd,
            "args": ["-m", "tokenjar"],
            "env": {"PYTHONUNBUFFERED": "1"},
        }

        try:
            with open(config_path, "w", encoding="utf-8") as f:
                json.dump(config, f, indent=2)
            return True, f"Activated in {config_path}"
        except Exception as e:
            return False, f"Failed writing {config_path}: {e}"

    @staticmethod
    def _revert_mcp_config_with_backup(config_path: Path) -> tuple[bool, str]:
        """Revert MCP configuration back to pre-activation state, safely preserving any newly added user configurations."""
        import json

        bak_path = config_path.with_name(config_path.name + ".ts_bak")

        # 1. Parse current config to inspect live state
        current_config: dict = {}
        has_valid_current = False
        if config_path.exists() and config_path.stat().st_size > 0:
            try:
                with open(config_path, "r", encoding="utf-8") as f:
                    current_config = json.load(f)
                has_valid_current = True
            except Exception:
                has_valid_current = False

        bak_content = None
        if bak_path.exists():
            try:
                bak_content = bak_path.read_text(encoding="utf-8")
            except Exception:
                pass

        # 2. Check if user modified or added other things while TokenJar was installed
        user_modified = False
        mcp_servers = current_config.get("mcpServers", {}) if has_valid_current else {}
        other_servers = {k: v for k, v in mcp_servers.items() if k not in ("tokenjar", "token-saver", "token_saver")}
        other_keys = {k: v for k, v in current_config.items() if k != "mcpServers"} if has_valid_current else {}

        if bak_content and bak_content != "__NON_EXISTENT__":
            try:
                bak_json = json.loads(bak_content)
                temp_current = json.loads(json.dumps(current_config))
                if "mcpServers" in temp_current:
                    for key in ("tokenjar", "token-saver", "token_saver"):
                        temp_current["mcpServers"].pop(key, None)
                if temp_current != bak_json:
                    user_modified = True
            except Exception:
                user_modified = True
        elif bak_content == "__NON_EXISTENT__":
            if other_servers or other_keys:
                user_modified = True

        # Case A: User modified the config (added new servers/keys) while TokenJar was active
        # MUST NEVER overwrite or delete user additions!
        if user_modified and has_valid_current:
            for key in ("tokenjar", "token-saver", "token_saver"):
                mcp_servers.pop(key, None)
            if not mcp_servers and other_keys:
                del current_config["mcpServers"]
            else:
                current_config["mcpServers"] = mcp_servers

            try:
                with open(config_path, "w", encoding="utf-8") as f:
                    json.dump(current_config, f, indent=2)
                bak_path.unlink(missing_ok=True)
                server_count = len(other_servers)
                return (
                    True,
                    f"Deactivated tokenjar from {config_path.name} (preserved {server_count} user servers/settings)",
                )
            except Exception as e:
                return False, f"Failed updating {config_path}: {e}"

        # Case B: User did not add or modify anything -> restore exact original state
        if bak_content:
            try:
                if bak_content == "__NON_EXISTENT__":
                    if config_path.exists():
                        config_path.unlink()
                else:
                    config_path.write_text(bak_content, encoding="utf-8")
                bak_path.unlink(missing_ok=True)

                # Clean up empty parent directories if created by tokenjar
                parent = config_path.parent
                try:
                    if parent.exists() and not any(parent.iterdir()):
                        parent.rmdir()
                        grandparent = parent.parent
                        if grandparent.exists() and not any(grandparent.iterdir()):
                            grandparent.rmdir()
                except Exception:
                    pass

                return True, f"Restored original config for {config_path.name}"
            except Exception as e:
                return False, f"Failed restoring backup for {config_path}: {e}"

        # Fallback if no backup file
        if config_path.exists() and has_valid_current:
            if "mcpServers" in current_config and "tokenjar" in current_config["mcpServers"]:
                del current_config["mcpServers"]["tokenjar"]
                try:
                    with open(config_path, "w", encoding="utf-8") as f:
                        json.dump(current_config, f, indent=2)
                    return True, f"Deactivated tokenjar from {config_path.name}"
                except Exception as e:
                    return False, f"Failed modifying {config_path}: {e}"

        return True, f"TokenJar was already inactive in {config_path.name}"

    @classmethod
    def is_cli_installed(cls, name: str) -> bool:
        """Check if a specific AI coding CLI or assistant is installed on the host system."""
        import shutil

        home = Path.home()

        if name == "Antigravity (AGY)":
            return bool(shutil.which("agy") or shutil.which("antigravity") or (home / ".gemini").is_dir())

        if name == "Claude Code":
            if shutil.which("claude") or shutil.which("claude.cmd"):
                return True
            claude_dir = home / ".claude"
            if claude_dir.is_dir():
                files = [p.name for p in claude_dir.iterdir() if p.name != "commands"]
                if files:
                    return True
            claude_cfg = home / ".claude.json"
            claude_bak = home / ".claude.json.ts_bak"
            if claude_cfg.exists():
                if claude_bak.exists():
                    try:
                        if claude_bak.read_text(encoding="utf-8").strip() == "__NON_EXISTENT__":
                            return False
                    except Exception:
                        pass
                return True
            return False

        if name == "Cursor":
            if shutil.which("cursor") or shutil.which("cursor.cmd"):
                return True
            if os.name == "nt":
                if (home / "AppData" / "Roaming" / "Cursor").is_dir() or (
                    home / "AppData" / "Local" / "Programs" / "cursor"
                ).is_dir():
                    return True
            elif sys.platform == "darwin":
                if (
                    Path("/Applications/Cursor.app").is_dir()
                    or (home / "Library" / "Application Support" / "Cursor").is_dir()
                ):
                    return True
            else:
                if (home / ".config" / "Cursor").is_dir():
                    return True
            cursor_cfg = home / ".cursor" / "mcp.json"
            cursor_bak = home / ".cursor" / "mcp.json.ts_bak"
            if cursor_cfg.exists():
                if cursor_bak.exists():
                    try:
                        if cursor_bak.read_text(encoding="utf-8").strip() == "__NON_EXISTENT__":
                            return False
                    except Exception:
                        pass
                return True
            return False

        if name == "Windsurf":
            if shutil.which("windsurf") or shutil.which("windsurf.cmd"):
                return True
            if os.name == "nt":
                if (
                    (home / "AppData" / "Roaming" / "Windsurf").is_dir()
                    or (home / "AppData" / "Local" / "Programs" / "Windsurf").is_dir()
                    or (home / "AppData" / "Roaming" / "Codeium").is_dir()
                ):
                    return True
            elif sys.platform == "darwin":
                if (
                    Path("/Applications/Windsurf.app").is_dir()
                    or (home / "Library" / "Application Support" / "Windsurf").is_dir()
                ):
                    return True
            else:
                if (home / ".config" / "Windsurf").is_dir():
                    return True
            windsurf_cfg = home / ".codeium" / "windsurf" / "mcp_config.json"
            windsurf_bak = home / ".codeium" / "windsurf" / "mcp_config.json.ts_bak"
            if windsurf_cfg.exists():
                if windsurf_bak.exists():
                    try:
                        if windsurf_bak.read_text(encoding="utf-8").strip() == "__NON_EXISTENT__":
                            return False
                    except Exception:
                        pass
                return True
            return False

        if name == "Claude Desktop":
            if os.name == "nt":
                appdata = os.environ.get("APPDATA")
                roaming = Path(appdata) if appdata else home / "AppData" / "Roaming"
                if (roaming / "Claude").is_dir() or (home / "AppData" / "Local" / "Programs" / "Claude").is_dir():
                    return True
            elif sys.platform == "darwin":
                if (
                    Path("/Applications/Claude.app").is_dir()
                    or (home / "Library" / "Application Support" / "Claude").is_dir()
                ):
                    return True
            else:
                if (home / ".config" / "Claude").is_dir():
                    return True
            cfg = cls.get_supported_cli_configs().get("Claude Desktop")
            if cfg and cfg.exists():
                return True
            return False

        if name == "VS Code (Cline)":
            cfg = cls.get_supported_cli_configs().get("VS Code (Cline)")
            if cfg and (cfg.parent.is_dir() or cfg.exists()):
                return True
            return False

        if name == "VS Code (Roo)":
            cfg = cls.get_supported_cli_configs().get("VS Code (Roo)")
            if cfg and (cfg.parent.is_dir() or cfg.exists()):
                return True
            return False

        return False

    @classmethod
    def get_supported_cli_configs(cls) -> dict[str, Path]:
        """Return paths to all supported AI coding assistant configuration files."""
        home = Path.home()
        configs = {
            "Antigravity (AGY)": home / ".gemini" / "config" / "mcp_config.json",
            "Claude Code": home / ".claude.json",
            "Cursor": home / ".cursor" / "mcp.json",
            "Windsurf": home / ".codeium" / "windsurf" / "mcp_config.json",
        }

        # Claude Desktop
        if os.name == "nt":
            appdata = os.environ.get("APPDATA")
            roaming = Path(appdata) if appdata else home / "AppData" / "Roaming"
            configs["Claude Desktop"] = roaming / "Claude" / "claude_desktop_config.json"
        elif sys.platform == "darwin":
            configs["Claude Desktop"] = (
                home / "Library" / "Application Support" / "Claude" / "claude_desktop_config.json"
            )
        else:
            configs["Claude Desktop"] = home / ".config" / "Claude" / "claude_desktop_config.json"

        # VS Code (Cline / Roo)
        if os.name == "nt":
            appdata = os.environ.get("APPDATA")
            roaming = Path(appdata) if appdata else home / "AppData" / "Roaming"
            code_storage = roaming / "Code" / "User" / "globalStorage"
        elif sys.platform == "darwin":
            code_storage = home / "Library" / "Application Support" / "Code" / "User" / "globalStorage"
        else:
            code_storage = home / ".config" / "Code" / "User" / "globalStorage"

        configs["VS Code (Cline)"] = code_storage / "saoudrizwan.claude-dev" / "settings" / "cline_mcp_settings.json"
        configs["VS Code (Roo)"] = code_storage / "rooveterinaryinc.roo-cline" / "settings" / "cline_mcp_settings.json"

        return configs

    @classmethod
    def enable_all(cls, only_installed: bool = True) -> list[tuple[str, bool, str]]:
        """Universally activate TokenJar across detected AI coding CLIs with backups."""
        results = []
        configs = cls.get_supported_cli_configs()

        for name, path in configs.items():
            if only_installed and not cls.is_cli_installed(name):
                results.append((name, False, "Not installed (skipped — no config created)"))
                continue
            ok, msg = cls._apply_mcp_config_with_backup(path)
            results.append((name, ok, msg))

        # Also install slash commands
        cls.install_all_slash_commands(only_installed=only_installed)

        # Ensure CLI binary directory is in user's PATH
        path_ok, path_msg = cls.ensure_in_user_path()
        if path_ok and "Added" in path_msg:
            results.append(("System PATH", True, path_msg))

        return results

    @classmethod
    def ensure_in_user_path(cls) -> tuple[bool, str]:
        """Ensure the directory containing tokenjar CLI is in Windows User PATH or shell PATH."""
        try:
            exe_path = Path(sys.argv[0]).resolve()
            exe_dir = exe_path.parent
            exe_dir_str = str(exe_dir)

            if exe_path.name.lower() in ("python.exe", "python3.exe", "__main__.py", "pythonw.exe"):
                scripts_dir = Path(sys.executable).parent / ("Scripts" if os.name == "nt" else "bin")
                if scripts_dir.exists():
                    exe_dir_str = str(scripts_dir)

            current_paths = [os.path.normpath(p).lower() for p in os.environ.get("PATH", "").split(os.pathsep) if p]
            if os.path.normpath(exe_dir_str).lower() in current_paths:
                return True, f"Already in PATH: {exe_dir_str}"

            if os.name == "nt":
                import winreg

                with winreg.OpenKey(
                    winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_READ | winreg.KEY_WRITE
                ) as key:
                    try:
                        val, reg_type = winreg.QueryValueEx(key, "Path")
                    except FileNotFoundError:
                        val, reg_type = "", winreg.REG_EXPAND_SZ

                    parts = [p.strip() for p in val.split(";") if p.strip()]
                    if os.path.normpath(exe_dir_str).lower() not in [os.path.normpath(p).lower() for p in parts]:
                        new_val = (val.rstrip(";") + ";" + exe_dir_str) if val else exe_dir_str
                        winreg.SetValueEx(key, "Path", 0, reg_type, new_val)
                        os.environ["PATH"] = f"{exe_dir_str};{os.environ.get('PATH', '')}"
                        return True, f"Added {exe_dir_str} to Windows User PATH"
                return True, f"Already in User PATH: {exe_dir_str}"
            else:
                home = Path.home()
                updated = False
                for rc_name in (".bashrc", ".zshrc"):
                    rc_file = home / rc_name
                    if rc_file.exists():
                        content = rc_file.read_text(encoding="utf-8", errors="replace")
                        if exe_dir_str not in content:
                            with rc_file.open("a", encoding="utf-8") as f:
                                f.write(f'\nexport PATH="{exe_dir_str}:$PATH"\n')
                            updated = True
                if updated:
                    return True, f"Added {exe_dir_str} to shell profile PATH"
                return True, f"PATH verified: {exe_dir_str}"
        except Exception as e:
            return False, f"Could not update PATH: {e}"

    @classmethod
    def remove_from_user_path(cls) -> tuple[bool, str]:
        """Remove tokenjar binary directory from Windows User PATH or shell profile."""
        try:
            exe_path = Path(sys.argv[0]).resolve()
            exe_dir = exe_path.parent
            exe_dir_str = str(exe_dir)
            if exe_path.name.lower() in ("python.exe", "python3.exe", "__main__.py", "pythonw.exe"):
                scripts_dir = Path(sys.executable).parent / ("Scripts" if os.name == "nt" else "bin")
                if scripts_dir.exists():
                    exe_dir_str = str(scripts_dir)

            if os.name == "nt":
                import winreg

                with winreg.OpenKey(
                    winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_READ | winreg.KEY_WRITE
                ) as key:
                    try:
                        val, reg_type = winreg.QueryValueEx(key, "Path")
                    except FileNotFoundError:
                        return True, "No User PATH entry found"

                    parts = [p.strip() for p in val.split(";") if p.strip()]
                    norm_target = os.path.normpath(exe_dir_str).lower()
                    new_parts = [p for p in parts if os.path.normpath(p).lower() != norm_target]
                    if len(new_parts) != len(parts):
                        new_val = ";".join(new_parts)
                        winreg.SetValueEx(key, "Path", 0, reg_type, new_val)
                        return True, f"Removed {exe_dir_str} from Windows User PATH"
                return True, "Not present in Windows User PATH"
            else:
                home = Path.home()
                updated = False
                for rc_name in (".bashrc", ".zshrc"):
                    rc_file = home / rc_name
                    if rc_file.exists():
                        content = rc_file.read_text(encoding="utf-8", errors="replace")
                        if exe_dir_str in content:
                            lines = [line for line in content.splitlines() if exe_dir_str not in line]
                            rc_file.write_text("\n".join(lines) + "\n", encoding="utf-8")
                            updated = True
                if updated:
                    return True, f"Removed {exe_dir_str} from shell profile PATH"
                return True, "Not present in shell profile PATH"
        except Exception as e:
            return False, f"Could not clean PATH: {e}"

    @classmethod
    def uninstall_all_slash_commands(cls) -> list[tuple[str, bool, str]]:
        """Remove slash commands installed for AGY and Claude Code."""
        results = []
        home = Path.home()

        # 1. AGY CLI skill
        agy_skill = home / ".gemini" / "config" / "skills" / "tokenjar"
        if agy_skill.exists():
            try:
                import shutil

                shutil.rmtree(agy_skill)
                results.append(("Antigravity (AGY)", True, f"Removed slash command skill at {agy_skill}"))
            except Exception as e:
                results.append(("Antigravity (AGY)", False, f"Failed removing skill: {e}"))

        # 2. Antigravity MCP schemas & skills (current + legacy across all variations)
        for sub in ("antigravity", "antigravity-cli", "antigravity-ide"):
            for name in ("tokenjar", "token-saver", "token_saver"):
                schema_dir = home / ".gemini" / sub / "mcp" / name
                if schema_dir.exists():
                    try:
                        import shutil
                        shutil.rmtree(schema_dir, ignore_errors=True)
                    except Exception:
                        pass
        for name in ("token-saver", "token_saver"):
            legacy_skill = home / ".gemini" / "config" / "skills" / name
            if legacy_skill.exists():
                try:
                    import shutil
                    shutil.rmtree(legacy_skill, ignore_errors=True)
                except Exception:
                    pass

        # 3. Claude Code command
        claude_cmd = home / ".claude" / "commands" / "tokenjar.md"
        if claude_cmd.exists():
            try:
                claude_cmd.unlink()
                results.append(("Claude Code", True, f"Removed slash command at {claude_cmd}"))
            except Exception as e:
                results.append(("Claude Code", False, f"Failed removing command: {e}"))

        return results

    @classmethod
    def full_uninstall(cls) -> None:
        """Completely purge TokenJar from host: IDE configs, project rules, hooks, cache, and PATH."""
        from tokenjar.rules.manager import RulesManager

        print("=" * 65)
        print("⚠️  TOKENJAR COMPLETE UNINSTALL & PURGE")
        print("=" * 65)

        # 1. Revert all AI coding CLIs
        print("\n🔌 Reverting MCP server configurations in AI coding assistants...")
        ide_results = cls.disable_all()
        for name, ok, msg in ide_results:
            icon = "⚪" if ok else "❌"
            print(f"  {icon} {name}: {msg}")

        # 2. Remove shell hooks
        print("\n🪝 Removing shell hooks from terminal profiles...")
        ok_hook, hook_msg = cls.uninstall_shell_hook()
        print(f"  {'⚪' if ok_hook else '❌'} {hook_msg}")

        # 3. Clean project rules across all known projects
        print("\n📝 Cleaning TokenJar steering rules from all known projects...")
        project_roots = RulesManager.get_known_project_roots()
        cleaned_rules_count = 0
        for root in project_roots:
            rule_results = RulesManager.remove_rules(root)
            for fname, ok, msg in rule_results:
                if ok and "removed" in msg.lower():
                    cleaned_rules_count += 1
                    print(f"  ⚪ {root.name}/{fname}: {msg}")
        if cleaned_rules_count == 0:
            print("  ⚪ No active project steering rules found.")

        # 4. Remove slash commands
        print("\n⚡ Removing slash command definitions...")
        cmd_results = cls.uninstall_all_slash_commands()
        for name, ok, msg in cmd_results:
            print(f"  {'⚪' if ok else '❌'} {name}: {msg}")

        # 5. Clean User PATH
        print("\n🌐 Removing TokenJar from User PATH...")
        ok_path, path_msg = cls.remove_from_user_path()
        print(f"  {'⚪' if ok_path else '❌'} {path_msg}")

        # 6. Delete ~/.tokenjar data directory
        data_dir = Path.home() / ".tokenjar"
        print(f"\n💾 Purging {data_dir} (L2 SQLite cache, telemetry, settings)...")
        if data_dir.exists():
            try:
                shutil.rmtree(data_dir)
                print(f"  ⚪ Deleted {data_dir} successfully.")
            except Exception as e:
                print(f"  ❌ Could not delete {data_dir}: {e}")
        else:
            print("  ⚪ Data directory already clean.")

        print("\n" + "=" * 65)
        print("✨ TokenJar has been completely uninstalled from your computer!")
        print("   Zero background processes, zero configs, and zero traces remain.")
        print("   You can now delete this executable file if desired.")
        print("=" * 65)

    @classmethod
    def disable_all(cls) -> list[tuple[str, bool, str]]:
        """Universally revert all AI coding CLIs back to their exact pre-activation settings."""
        results = []
        configs = cls.get_supported_cli_configs()

        for name, path in configs.items():
            bak_path = path.with_name(path.name + ".ts_bak")
            if not path.exists() and not bak_path.exists():
                continue
            ok, msg = cls._revert_mcp_config_with_backup(path)
            results.append((name, ok, msg))

        # Also clean up uninstalled Claude slash command if present
        home = Path.home()
        claude_cmd = home / ".claude" / "commands" / "tokenjar.md"
        if claude_cmd.exists() and not cls.is_cli_installed("Claude Code"):
            try:
                claude_cmd.unlink()
                cmd_dir = claude_cmd.parent
                if cmd_dir.exists() and not any(cmd_dir.iterdir()):
                    cmd_dir.rmdir()
                claude_dir = home / ".claude"
                if claude_dir.exists() and not any(claude_dir.iterdir()):
                    claude_dir.rmdir()
            except Exception:
                pass

        return results

    @classmethod
    def get_agy_config_path(cls) -> Path:
        """Return the path to AGY CLI global mcp_config.json."""
        return cls.get_supported_cli_configs()["Antigravity (AGY)"]

    @classmethod
    def enable_agy(cls) -> tuple[bool, str]:
        """Activate TokenJar in AGY CLI global configuration."""
        return cls._apply_mcp_config_with_backup(cls.get_agy_config_path())

    @classmethod
    def disable_agy(cls) -> tuple[bool, str]:
        """Deactivate TokenJar from AGY CLI global configuration."""
        return cls._revert_mcp_config_with_backup(cls.get_agy_config_path())

    @classmethod
    def install_agy_slash_command(cls) -> tuple[bool, str]:
        """Install global slash command skill for Antigravity (AGY) CLI."""
        home = Path.home()
        skill_dir = home / ".gemini" / "config" / "skills" / "tokenjar"
        skill_dir.mkdir(parents=True, exist_ok=True)
        skill_file = skill_dir / "SKILL.md"

        content = """---
name: tokenjar
description: >-
  Instant slash command controller for the TokenJar token optimization engine.
  Use immediately when user types /tokenjar, /tokenjar on, /tokenjar off,
  /tokenjar output on, /tokenjar output off, /tokenjar stats, /tokenjar ui,
  /tokenjar cache-clear, /tokenjar reset-stats, or requests to toggle tokenjar state.
---

# TokenJar Slash Command Controller

When this command is invoked with an argument:

1. **If argument is 'on' or 'enable':**
   Execute shell command: `tokenjar on`
   Report confirmation that TokenJar is active.

2. **If argument is 'off' or 'disable':**
   Execute shell command: `tokenjar off`
   Report confirmation that TokenJar is deactivated.

3. **If argument starts with 'output':**
   Execute shell command: `tokenjar output <arg>` (e.g. `tokenjar output on` or `tokenjar output off`)
   Report confirmation of the output mode change.

4. **If argument is 'status':**
   Execute shell command: `tokenjar status`
   Display the overall operational status report.

5. **If argument is 'stats' or 'telemetry':**
   Execute shell command: `tokenjar stats`
   Display the savings dashboard.

6. **If argument is 'ui' or 'dashboard':**
   Execute shell command: `tokenjar ui`
   Report confirmation that the Web Dashboard has been launched.

7. **If argument is 'cache-clear' or 'cache-reset':**
   Execute shell command: `tokenjar cache-clear`
   Report confirmation that L2 SQLite cache has been cleared.

8. **If argument is 'reset-stats':**
   Execute shell command: `tokenjar reset-stats`
   Report confirmation that cumulative telemetry metrics have been reset.

9. **If no argument or 'help':**
   Show options: `/tokenjar status`, `/tokenjar on`, `/tokenjar off`, `/tokenjar output on/off`, `/tokenjar stats`, `/tokenjar ui`, `/tokenjar cache-clear`, `/tokenjar reset-stats`.
"""
        try:
            skill_file.write_text(content, encoding="utf-8")
            return True, f"Installed /tokenjar command for AGY CLI at {skill_file}"
        except Exception as e:
            return False, f"Failed to install AGY slash command: {e}"

    @classmethod
    def install_claude_code_slash_command(cls) -> tuple[bool, str]:
        """Install native slash command definition for Claude Code."""
        home = Path.home()
        claude_dir = home / ".claude" / "commands"
        claude_dir.mkdir(parents=True, exist_ok=True)
        command_file = claude_dir / "tokenjar.md"

        content = """---
description: Manage TokenJar token optimization engine (on, off, output on/off, stats, ui, cache-clear, reset-stats)
---

Execute the requested TokenJar operation:
$ARGUMENTS

Instructions:
1. If argument is "on" or "enable", run `tokenjar on` and confirm activation.
2. If argument is "off" or "disable", run `tokenjar off` and confirm deactivation.
3. If argument starts with "output", run `tokenjar output <args>` and report status.
4. If argument is "stats", run `tokenjar stats` and show the telemetry dashboard.
5. If argument is "ui" or "dashboard", run `tokenjar ui` and confirm dashboard launch.
6. If argument is "cache-clear" or "cache-reset", run `tokenjar cache-clear` and report cache reset.
7. If argument is "reset-stats", run `tokenjar reset-stats` and report telemetry reset.
8. If empty or help, show usage instructions.
"""
        try:
            command_file.write_text(content, encoding="utf-8")
            return True, f"Installed /tokenjar command for Claude Code at {command_file}"
        except Exception as e:
            return False, f"Failed to install Claude Code slash command: {e}"

    @classmethod
    def install_all_slash_commands(cls, only_installed: bool = False) -> list[tuple[str, bool, str]]:
        """Install slash command definitions across supported AI coding CLIs."""
        results = []
        if not only_installed or cls.is_cli_installed("Antigravity (AGY)"):
            ok1, msg1 = cls.install_agy_slash_command()
            results.append(("Antigravity (AGY)", ok1, msg1))

        if not only_installed or cls.is_cli_installed("Claude Code"):
            ok2, msg2 = cls.install_claude_code_slash_command()
            results.append(("Claude Code", ok2, msg2))
        else:
            results.append(("Claude Code", False, "Not installed (skipped)"))

        return results
