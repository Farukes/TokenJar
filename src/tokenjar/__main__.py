"""TokenJar CLI & MCP Entry Point.

Supports running both as an MCP server for AI coding assistants
and as a standalone CLI tool for developers (run, stats, hook, unhook).
"""

from __future__ import annotations

import argparse
import sys

# Configure stdout and stderr to handle UTF-8 cleanly on Windows/legacy terminals
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


def main() -> None:
    """Main CLI entry point for TokenJar."""
    from tokenjar import __version__

    parser = argparse.ArgumentParser(
        prog="tokenjar",
        description="TokenJar: Zero-cost token optimization engine for AI coding assistants and developers.",
    )
    parser.add_argument(
        "-v",
        "--version",
        action="version",
        version=f"tokenjar {__version__}",
        help="Show program's version number and exit",
    )
    subparsers = parser.add_subparsers(dest="subcommand", metavar="<command>", help="Available subcommands")

    # Subcommand: version
    subparsers.add_parser("version", help="Show program's version number and exit")

    # Subcommand: server (default if no args)
    subparsers.add_parser("server")

    # Subcommand: stats
    subparsers.add_parser("stats", help="Display cumulative token and financial savings dashboard")

    # Subcommand: status
    status_parser = subparsers.add_parser(
        "status",
        help="Check comprehensive live operational status of TokenJar across all AI CLIs and project rules",
    )
    status_parser.add_argument(
        "--path",
        default=".",
        help="Target project directory to check rules for (default: current directory)",
    )

    # Subcommand: reset-stats (legacy alias -> tokenjar clean --stats)
    subparsers.add_parser("reset-stats")

    # Subcommand: run
    run_parser = subparsers.add_parser("run", help="Execute a shell command with intelligent output filtering")
    run_parser.add_argument("command", nargs=argparse.REMAINDER, help="The command to execute (e.g. pytest, npm test)")

    # Subcommand: hook (internal)
    hook_parser = subparsers.add_parser("hook")
    hook_parser.add_argument(
        "--shell",
        choices=["auto", "powershell", "bash"],
        default="auto",
        help="Target shell environment (default: auto)",
    )

    # Subcommand: unhook (internal)
    subparsers.add_parser("unhook")

    # Subcommand: install
    install_parser = subparsers.add_parser(
        "install",
        help="One-time system install: adds to PATH, enables MCP across detected IDEs, sets up slash commands",
    )
    install_parser.add_argument(
        "--all",
        action="store_true",
        help="Configure for all supported IDEs even if not currently detected on system",
    )

    # Subcommand: on
    on_parser = subparsers.add_parser(
        "on", help="Activate TokenJar in current project (creates/updates AGENTS.md rules)"
    )
    on_parser.add_argument(
        "-g",
        "--global",
        dest="global_scope",
        action="store_true",
        help="Configure MCP server globally in all detected IDEs without modifying project files",
    )

    # Subcommand: off
    off_parser = subparsers.add_parser("off", help="Deactivate TokenJar in current project (cleans AGENTS.md rules)")
    off_parser.add_argument(
        "-g",
        "--global",
        dest="global_scope",
        action="store_true",
        help="Uninstall TokenJar MCP configuration globally from all IDEs",
    )

    # Subcommand: enable
    enable_parser = subparsers.add_parser(
        "enable",
        aliases=["enable-mcp", "install-mcp"],
        help="Enable TokenJar MCP server in all detected AI assistants globally",
    )
    enable_parser.add_argument(
        "--all",
        action="store_true",
        help="Configure for all supported IDEs even if not currently detected on system",
    )

    # Subcommand: disable
    subparsers.add_parser(
        "disable",
        aliases=["disable-mcp", "uninstall-mcp"],
        help="Disable TokenJar MCP server from all AI assistants globally",
    )

    # Subcommand: clean / clear
    clean_parser = subparsers.add_parser(
        "clean",
        aliases=["clear"],
        help="Reset telemetry metrics and clear L2 SQLite cache",
    )
    clean_parser.add_argument(
        "--cache",
        action="store_true",
        help="Only clear L2 SQLite cache",
    )
    clean_parser.add_argument(
        "--stats",
        action="store_true",
        help="Only reset telemetry statistics",
    )

    # Subcommand: ui
    ui_parser = subparsers.add_parser(
        "ui",
        help="Launch the lightweight On-Demand Settings & Dashboard UI (Zero Background RAM)",
    )
    ui_parser.add_argument(
        "--port",
        type=int,
        default=4141,
        help="Port to bind the local dashboard server (default: 4141)",
    )
    ui_parser.add_argument(
        "--no-open",
        action="store_true",
        help="Do not automatically open the browser or native app window",
    )

    # Subcommand: setup-commands (legacy alias -> tokenjar install)
    subparsers.add_parser("setup-commands")

    # Subcommand: output (legacy alias -> tokenjar on / off)
    output_parser = subparsers.add_parser("output")
    output_parser.add_argument(
        "state",
        nargs="?",
        choices=["on", "off", "status"],
        default="status",
        help="Output mode action: 'on' (compact surgical diffs), 'off' (default output), or 'status'",
    )
    output_parser.add_argument(
        "--path",
        default=".",
        help="Target project directory (default: current directory)",
    )

    # Subcommand: init / init-rules (legacy alias -> tokenjar on / off)
    rules_parser = subparsers.add_parser(
        "init",
        aliases=["init-rules"],
    )
    rules_parser.add_argument(
        "--path",
        default=".",
        help="Target project directory (default: current directory)",
    )
    rules_parser.add_argument(
        "--compact",
        dest="compact_output",
        action="store_true",
        default=None,
        help="Enforce compact surgical output rules",
    )
    rules_parser.add_argument(
        "--no-compact",
        dest="compact_output",
        action="store_false",
        help="Disable compact output restrictions in agent rules",
    )
    rules_parser.add_argument(
        "--all",
        action="store_true",
        help="Generate rule files for all AI coding assistants (default: auto-detect installed assistants)",
    )
    rules_parser.add_argument(
        "--clean",
        action="store_true",
        help="Clean steering rules from target project directory instead of installing",
    )

    # Subcommand: cache-prune / cache-clear / cache-reset (legacy alias -> tokenjar clean)
    prune_parser = subparsers.add_parser(
        "cache-prune",
        aliases=["cache-clear", "cache-reset"],
    )
    prune_parser.add_argument(
        "--all",
        action="store_true",
        help="Completely clear all cached files and symbols",
    )
    prune_parser.add_argument(
        "--max-entries",
        type=int,
        default=5000,
        help="Maximum cache entries to retain (default: 5000)",
    )
    prune_parser.add_argument(
        "--ttl-days",
        type=int,
        default=30,
        help="Evict entries older than N days (default: 30)",
    )

    # Subcommand: update / upgrade
    update_parser = subparsers.add_parser(
        "update",
        aliases=["upgrade"],
        help="Check for updates and automatically upgrade TokenJar to the latest release",
    )
    update_parser.add_argument(
        "--force",
        action="store_true",
        help="Force reinstallation even if already on the latest version",
    )

    # Subcommand: uninstall / purge
    uninstall_parser = subparsers.add_parser(
        "uninstall",
        aliases=["purge", "self-destruct"],
        help="Completely uninstall TokenJar: revert IDE configs, remove project rules, hooks, cache, and PATH",
    )
    uninstall_parser.add_argument(
        "-y",
        "--yes",
        action="store_true",
        help="Skip confirmation prompt and immediately purge all TokenJar traces",
    )

    # If called with no arguments:
    # If piped by AI assistant (Cursor / Claude Desktop / Windsurf) -> run Stdio MCP Server!
    # If user runs interactively in terminal -> open Web Dashboard UI in browser (matching Rust binary)!
    if len(sys.argv) == 1:
        if sys.stdin.isatty():
            print("🚀 Launching TokenJar Web Dashboard in your browser...")
            from tokenjar.ui.server import start_ui_server

            start_ui_server(port=4141, open_browser=True)
            return
        else:
            from tokenjar.server import mcp

            mcp.run(transport="stdio")
            return

    args = parser.parse_args()

    if args.subcommand == "version":
        print(f"tokenjar {__version__}")
        return

    if args.subcommand in (None, "server"):
        from tokenjar.server import mcp

        mcp.run(transport="stdio")

    elif args.subcommand == "stats":
        from tokenjar.telemetry.stats import tracker

        print(tracker.render_dashboard())

    elif args.subcommand == "status":
        import json
        from pathlib import Path

        from tokenjar.cache.persistent_cache import PersistentCache
        from tokenjar.hooks.manager import HookManager
        from tokenjar.rules.manager import RULES_MARKER_START, RulesManager

        project_path = Path(args.path).resolve()
        configs = HookManager.get_supported_cli_configs()

        print("=" * 60)
        print("🔋 TOKENJAR SYSTEM STATUS REPORT")
        print("=" * 60)

        # 1. Check CLI MCP Integrations
        active_clis = []
        inactive_clis = []
        for name, cfg_path in configs.items():
            is_inst = HookManager.is_cli_installed(name)
            is_active = False
            if cfg_path.exists():
                try:
                    with open(cfg_path, "r", encoding="utf-8") as f:
                        data = json.load(f)
                        if "tokenjar" in data.get("mcpServers", {}):
                            is_active = True
                except Exception:
                    pass
            if is_active:
                active_clis.append(f"🟢 {name} (Active in {cfg_path.name})")
            elif is_inst:
                inactive_clis.append(f"🔴 {name} (Installed, but TokenJar disabled)")
            else:
                inactive_clis.append(f"⚪ {name} (Not installed)")

        overall_active = len(active_clis) > 0
        overall_badge = "🟢 ACTIVE (Operational)" if overall_active else "🔴 INACTIVE (Turn on with 'tokenjar on')"
        print(f"Overall Engine Status : {overall_badge}\n")

        print("AI Assistant Integrations:")
        for line in active_clis + inactive_clis:
            print(f"  • {line}")

        # 2. Check Output Optimization Status
        output_active = RulesManager.get_output_mode(project_path)
        out_badge = (
            "🟢 ON (Compact surgical diffs & zero-truncation)" if output_active else "⚪ OFF (Default full output)"
        )
        print(f"\nOutput Optimization   : {out_badge}")

        # 3. Check Project Steering Rules
        rule_files_found = []
        for r_name in RulesManager.SUPPORTED_RULE_FILES:
            r_path = project_path / r_name
            if r_path.exists():
                try:
                    content = r_path.read_text(encoding="utf-8")
                    if RULES_MARKER_START in content:
                        rule_files_found.append(r_name)
                except Exception:
                    pass
        if rule_files_found:
            print(f"Project Steering Rules: 🟢 INSTALLED ({', '.join(rule_files_found)})")
        else:
            print("Project Steering Rules: 🔴 NOT INSTALLED (Run 'tokenjar init-rules')")

        # 4. Check L2 Persistent Cache
        try:
            cache = PersistentCache()
            entries = cache.count_entries()
            print(f"L2 Persistent Cache   : 🟢 ONLINE ({entries} cached entries in ~/.tokenjar/cache.db)")
        except Exception as e:
            print(f"L2 Persistent Cache   : ⚠️ Error accessing DB: {e}")

        print("=" * 60)
        print("Useful Commands:")
        print("  tokenjar install      -> One-time setup: adds to PATH & enables IDE MCP")
        print("  tokenjar on           -> Activate TokenJar & generate AGENTS.md in project")
        print("  tokenjar off          -> Deactivate TokenJar & clean AGENTS.md from project")
        print("  tokenjar enable       -> Enable MCP server in all detected IDEs globally")
        print("  tokenjar disable      -> Disable MCP server from all IDEs globally")
        print("  tokenjar stats        -> View live token and financial savings")
        print("  tokenjar status       -> Check operational status")
        print("  tokenjar clean        -> Reset telemetry metrics and clear L2 cache")
        print("  tokenjar ui           -> Open Web Dashboard in browser")
        print("=" * 60)
        if not overall_active:
            sys.exit(1)

    elif args.subcommand == "reset-stats":
        from tokenjar.telemetry.stats import tracker

        tracker.reset()
        print("Telemetry metrics have been successfully reset.")

    elif args.subcommand == "run":
        if not args.command:
            print("Error: No command provided to run. Example: tokenjar run pytest")
            sys.exit(1)
        from tokenjar.hooks.manager import execute_filtered_command

        exit_code = execute_filtered_command(args.command)
        sys.exit(exit_code)

    elif args.subcommand == "hook":
        from tokenjar.hooks.manager import HookManager

        success, msg = HookManager.install_shell_hook(target_shell=args.shell)
        print(msg)
        if not success:
            sys.exit(1)

    elif args.subcommand == "unhook":
        from tokenjar.hooks.manager import HookManager

        success, msg = HookManager.uninstall_shell_hook()
        print(msg)
        if not success:
            sys.exit(1)

    elif args.subcommand == "install":
        from tokenjar.hooks.manager import HookManager

        print("=" * 60)
        print("📦 TOKENJAR AUTOMATIC SYSTEM INSTALLER")
        print("=" * 60)
        path_ok, path_msg = HookManager.ensure_in_user_path()
        if path_ok:
            print(f"  🟢 System PATH: {path_msg}")
        else:
            print(f"  ⚠️ System PATH: {path_msg}")

        print("\n🔌 Activating TokenJar MCP across AI assistants...")
        results = HookManager.enable_all(only_installed=not getattr(args, "all", False))
        for name, ok, msg in results:
            status = "⚪" if "skipped" in msg.lower() else ("🟢" if ok else "❌")
            print(f"  {status} {name}: {msg}")

        HookManager.install_all_slash_commands(only_installed=True)
        print("  🟢 Slash Commands: Configured /tokenjar in Antigravity and Claude Code")

        from tokenjar.rules.manager import RulesManager

        known = RulesManager.get_known_project_roots()
        updated_count = 0
        for root in known:
            if root.exists():
                res = RulesManager.install_rules(root)
                if any(r.success for r in res):
                    updated_count += 1
        if updated_count > 0:
            print(f"  🟢 Project Rules: Synchronized latest AGENTS.md across {updated_count} active project(s).")

        print("\n✨ TokenJar has been successfully installed & activated globally!")
        print("💡 To activate in any project and generate AGENTS.md, run:")
        print("     tokenjar on")
        print("=" * 60)

    elif args.subcommand in ("enable", "enable-mcp", "install-mcp"):
        from tokenjar.hooks.manager import HookManager

        print("🔌 Activating TokenJar MCP across AI assistants...")
        results = HookManager.enable_all(only_installed=not getattr(args, "all", False))
        for name, ok, msg in results:
            status = "⚪" if "skipped" in msg.lower() else ("🟢" if ok else "❌")
            print(f"  {status} {name}: {msg}")
        print("\n✨ TokenJar MCP is now ACTIVE across detected IDEs!")
        print("💡 Run 'tokenjar on' in your project to inject AGENTS.md rules.")

    elif args.subcommand in ("disable", "disable-mcp", "uninstall-mcp"):
        from tokenjar.hooks.manager import HookManager

        print("🔌 Deactivating TokenJar MCP globally from all AI assistants...")
        results = HookManager.disable_all()
        for name, ok, msg in results:
            status = "⚪" if "skipped" in msg.lower() else ("🔴" if ok else "❌")
            print(f"  {status} {name}: {msg}")
        print("\n⚪ TokenJar MCP has been deactivated globally.")

    elif args.subcommand in ("clean", "clear"):
        clear_all = not getattr(args, "cache", False) and not getattr(args, "stats", False)
        print("🧹 Cleaning TokenJar cache and metrics...")
        if clear_all or getattr(args, "stats", False):
            from tokenjar.telemetry.stats import tracker

            tracker.reset()
            print("  🟢 Telemetry: Statistics and savings counters reset to zero.")
        if clear_all or getattr(args, "cache", False):
            from tokenjar.cache.persistent_cache import PersistentCache

            p = PersistentCache()
            before = p.count_entries()
            p.clear()
            print(f"  🟢 Cache: L2 SQLite cache completely cleared ({before} entries removed).")
        print("✨ Clean complete.")

    elif args.subcommand == "on":
        from tokenjar.hooks.manager import HookManager
        from tokenjar.rules.manager import RulesManager

        if getattr(args, "global_scope", False):
            results = HookManager.enable_all()
            for name, ok, msg in results:
                status = "⚪" if "skipped" in msg.lower() else ("🟢" if ok else "❌")
                print(f"  {status} {name}: {msg}")
            print("\n✨ TokenJar MCP is now GLOBALLY ACTIVE across detected IDEs!")
            print("💡 To enable for a specific project, run: tokenjar on")
        else:
            HookManager.enable_all()
            print("📝 Generating TokenJar rules (AGENTS.md) in current project...")
            rule_results = RulesManager.install_rules(".")
            for name, ok, msg in rule_results:
                icon = "🟢" if ok else "❌"
                print(f"  {icon} {name}: {msg}")
            print("\n✨ TokenJar is now ACTIVE for this project!")
            print("🤖 AGENTS.md rule file is ready for AI coding assistants.")

    elif args.subcommand == "off":
        from tokenjar.hooks.manager import HookManager
        from tokenjar.rules.manager import RulesManager

        if getattr(args, "global_scope", False):
            results = HookManager.disable_all()
            for name, ok, msg in results:
                status = "⚪" if "skipped" in msg.lower() else ("🔴" if ok else "❌")
                print(f"  {status} {name}: {msg}")
            print("\n⚪ TokenJar MCP has been deactivated globally across all IDEs.")
        else:
            print("📝 Cleaning TokenJar rules from current project...")
            rule_results = RulesManager.remove_rules(".")
            for name, ok, msg in rule_results:
                print(f"  🔴 {name}: {msg}")
            print("\n⚪ TokenJar has been deactivated for THIS project.")
            print("💡 Global MCP and other projects remain active and unaffected.")
            print("   (To remove globally from all IDEs, run: tokenjar disable)")

    elif args.subcommand in ("setup-commands", "install-commands"):
        from tokenjar.hooks.manager import HookManager

        results = HookManager.install_all_slash_commands(only_installed=True)
        all_ok = True
        for name, ok, msg in results:
            if "skipped" in msg.lower():
                status = "⚪"
            else:
                status = "✅" if ok else "❌"
                if not ok:
                    all_ok = False
            print(f"{status} {name}: {msg}")
        if not all_ok:
            sys.exit(1)

    elif args.subcommand in ("init", "init-rules"):
        from tokenjar.rules.manager import RulesManager

        if getattr(args, "clean", False):
            results = RulesManager.remove_rules(args.path)
            for name, ok, msg in results:
                print(f"🔴 Rules: {msg}")
        else:
            results = RulesManager.install_rules(
                args.path,
                compact_output=args.compact_output,
                only_installed=not args.all,
            )
            all_ok = True
            for name, ok, msg in results:
                if "skipped" in msg.lower():
                    status = "⚪"
                else:
                    status = "✅" if ok else "❌"
                    if not ok:
                        all_ok = False
                print(f"{status} {name}: {msg}")
            if not all_ok:
                sys.exit(1)

    elif args.subcommand == "output":
        from tokenjar.rules.manager import RulesManager

        if args.state == "on":
            ok, msg, files = RulesManager.set_output_mode(args.path, enabled=True)
            if ok:
                print("🟢 Output Optimization: ON (Compact Mode Active)")
                print("   • Enforces surgical diffs and targeted block replacements.")
                print("   • ZERO TRUNCATION MANDATE active (no lazy comments).")
                print(f"   • Updated files: {', '.join(files)}")
            else:
                print(f"❌ Error: {msg}")
                sys.exit(1)
        elif args.state == "off":
            ok, msg, files = RulesManager.set_output_mode(args.path, enabled=False)
            if ok:
                print("⚪ Output Optimization: OFF (Default Output Restored)")
                print("   • AI assistant will use standard, unrestricted output.")
                print("   • Output format returned to default.")
                print(f"   • Updated files: {', '.join(files)}")
            else:
                print(f"❌ Error: {msg}")
                sys.exit(1)
        else:
            active = RulesManager.get_output_mode(args.path)
            state_str = "🟢 ON (Compact Mode Active)" if active else "⚪ OFF (Default Output)"
            print(f"Output Optimization Status: {state_str}")
            print("\nUsage:")
            print("  tokenjar output on   -> Activate compact surgical diffs & zero-truncation")
            print("  tokenjar output off  -> Revert to default normal/verbose output")

    elif args.subcommand in ("cache-prune", "cache-clear", "cache-reset"):
        from tokenjar.cache.persistent_cache import PersistentCache

        p = PersistentCache()
        before = p.count_entries()
        if getattr(args, "all", False) or args.subcommand in ("cache-clear", "cache-reset"):
            p.clear()
            after = p.count_entries()
            print(f"🧹 L2 Cache Reset: {before} entries cleared. ({after} remaining in SQLite)")
        else:
            deleted = p.prune(max_entries=args.max_entries, max_age_days=args.ttl_days)
            after = p.count_entries()
            print(f"L2 Cache Pruned: {deleted} entries removed. ({before} -> {after} entries remaining)")

    elif args.subcommand == "ui":
        from tokenjar.hooks.manager import HookManager

        HookManager.ensure_in_user_path()
        from tokenjar.ui.server import start_ui_server

        start_ui_server(port=args.port, open_browser=not args.no_open)

    elif args.subcommand in ("uninstall", "purge", "self-destruct"):
        if not getattr(args, "yes", False):
            try:
                confirm = input("⚠️  Are you sure you want to completely uninstall TokenJar from this computer? (y/N): ")
                if confirm.strip().lower() not in ("y", "yes"):
                    print("Aborted.")
                    sys.exit(0)
            except (KeyboardInterrupt, EOFError):
                print("\nAborted.")
                sys.exit(0)

        from tokenjar.hooks.manager import HookManager

        HookManager.full_uninstall()

    elif args.subcommand in ("update", "upgrade"):
        import json
        import subprocess
        import urllib.request

        from tokenjar import __version__

        print("=" * 60)
        print("🔄 TOKENJAR AUTOMATIC UPDATE MANAGER")
        print("=" * 60)
        print(f"Current Engine Version : v{__version__}")
        print("Checking for latest release on PyPI...")

        latest_version = None
        try:
            req = urllib.request.Request(
                "https://pypi.org/pypi/tokenjar-engine/json",
                headers={"User-Agent": f"tokenjar/{__version__}"},
            )
            with urllib.request.urlopen(req, timeout=5) as response:
                data = json.loads(response.read().decode("utf-8"))
                latest_version = data.get("info", {}).get("version")
        except Exception as e:
            print(f"Note: Could not reach PyPI index directly ({e}). Proceeding to pip upgrade check...")

        if latest_version:
            print(f"Latest PyPI Release    : v{latest_version}")

        if latest_version and latest_version == __version__ and not getattr(args, "force", False):
            print("\n✨ TokenJar is already on the latest version!")
            print(f"   (No action needed. Current: v{__version__})")
            return

        if latest_version and latest_version != __version__:
            print(f"\n🚀 New version detected: v{latest_version} (installed: v{__version__})")
        print("\n📦 Upgrading tokenjar-engine via pip...")

        try:
            cmd = [sys.executable, "-m", "pip", "install", "--upgrade", "tokenjar-engine"]
            if getattr(args, "force", False):
                cmd.append("--force-reinstall")
            res = subprocess.run(cmd, capture_output=True, text=True)
            if res.returncode == 0:
                target_v = latest_version or "latest"
                print(f"\n🎉 TokenJar has been successfully updated to v{target_v}!")

                from tokenjar.hooks.manager import HookManager
                from tokenjar.rules.manager import RulesManager

                print("🔄 Synchronizing IDE configurations, slash commands, and project rules...")
                HookManager.enable_all(only_installed=True)
                HookManager.install_all_slash_commands(only_installed=True)

                known = RulesManager.get_known_project_roots()
                updated_count = 0
                for root in known:
                    if root.exists():
                        res = RulesManager.install_rules(root)
                        if any(r.success for r in res):
                            updated_count += 1
                if updated_count > 0:
                    print(f"  🟢 Project Rules: Synchronized latest AGENTS.md across {updated_count} active project(s).")

                print("💡 Tip: Restart any open AI coding sessions or IDE windows to load updated middleware.")
            else:
                print(f"\n❌ Pip update command returned error:\n{res.stderr or res.stdout}")
                sys.exit(res.returncode)
        except Exception as e:
            print(f"\n❌ Failed to execute pip updater: {e}")
            sys.exit(1)


if __name__ == "__main__":
    main()
