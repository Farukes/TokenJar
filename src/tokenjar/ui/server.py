"""Lightweight HTTP server for TokenJar On-Demand Dashboard.

Zero-background-RAM architecture: runs only when requested, provides a local REST API
and standalone web application, then cleanly terminates on exit.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import threading
import time
import webbrowser
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from typing import Any

from tokenjar.cache.persistent_cache import PersistentCache
from tokenjar.config import load_config
from tokenjar.hooks.manager import HookManager
from tokenjar.rules.manager import RulesManager
from tokenjar.telemetry.stats import tracker

STATIC_DIR = Path(__file__).resolve().parent / "static"


def get_system_status() -> dict[str, Any]:
    """Gather live operational metrics, IDE configurations, and engine status."""
    configs = HookManager.get_supported_cli_configs()
    ide_list = []
    any_active = False

    for name, cfg_path in configs.items():
        is_installed = HookManager.is_cli_installed(name)
        is_active = False
        if cfg_path.exists():
            try:
                with open(cfg_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    if "tokenjar" in data.get("mcpServers", {}):
                        is_active = True
                        any_active = True
            except Exception:
                pass

        ide_list.append(
            {
                "name": name,
                "installed": is_installed,
                "active": is_active,
                "path": str(cfg_path),
            }
        )

    # Telemetry
    t_data = tracker.data
    cats = {
        "lockfile": {
            "name": "Lockfile Shield",
            "saved": t_data.lockfile.saved,
            "count": t_data.lockfile.count,
            "pct": round(t_data.lockfile.savings_pct, 1),
            "unit": "shields",
            "icon": "🛡️",
        },
        "symbol_search": {
            "name": "Global Symbol Search",
            "saved": t_data.symbol_search.saved,
            "count": t_data.symbol_search.count,
            "pct": round(t_data.symbol_search.savings_pct, 1),
            "unit": "searches",
            "icon": "🔍",
        },
        "repo_map": {
            "name": "Repo Map Engine",
            "saved": t_data.repo_map.saved,
            "count": t_data.repo_map.count,
            "pct": round(t_data.repo_map.savings_pct, 1),
            "unit": "maps",
            "icon": "🗺️",
        },
        "skeleton": {
            "name": "AST Skeletonizer",
            "saved": t_data.skeleton.saved,
            "count": t_data.skeleton.count,
            "pct": round(t_data.skeleton.savings_pct, 1),
            "unit": "files",
            "icon": "🦴",
        },
        "cache": {
            "name": "Smart File Cache",
            "saved": t_data.cache.saved,
            "count": t_data.cache.count,
            "pct": round(t_data.cache.savings_pct, 1),
            "unit": "reads",
            "icon": "⚡",
        },
        "command": {
            "name": "Terminal Pruner",
            "saved": t_data.command.saved,
            "count": t_data.command.count,
            "pct": round(t_data.command.savings_pct, 1),
            "unit": "runs",
            "icon": "✂️",
        },
        "slice": {
            "name": "Smart Line Slicer",
            "saved": t_data.slice.saved,
            "count": t_data.slice.count,
            "pct": round(t_data.slice.savings_pct, 1),
            "unit": "slices",
            "icon": "🔪",
        },
    }

    # Rules status
    rules_installed = any(
        (Path(".") / f).exists() for f in ["AGENTS.md", ".cursorrules", ".windsurfrules", "CLAUDE.md"]
    )

    cfg = load_config()

    # L2 Cache metrics
    l2_bytes = tracker.get_l2_cache_disk_bytes()
    l2_mb = round(l2_bytes / (1024.0 * 1024.0), 2)
    p_cache = PersistentCache()
    l2_entries = p_cache.count_entries()

    return {
        "version": "1.0.3",
        "active": any_active or rules_installed,
        "overall_status": "ACTIVE" if (any_active or rules_installed) else "INACTIVE",
        "telemetry": {
            "total_saved": t_data.total_tokens_saved,
            "total_processed": t_data.total_original_tokens,
            "savings_pct": round(t_data.savings_pct, 1),
            "dollars_saved": round(t_data.estimated_dollars_saved, 2),
            "categories": cats,
            "l2_cache": {
                "disk_bytes": l2_bytes,
                "disk_mb": l2_mb,
                "entries": l2_entries,
                "warning": l2_mb > 50.0,
            },
        },
        "ides": ide_list,
        "rules": {
            "installed": rules_installed,
            "compact_output": cfg.compact_output,
        },
        "config": {
            "lockfile_shield": cfg.lockfile_shield,
            "compact_output": cfg.compact_output,
            "prevent_truncation": cfg.prevent_truncation,
        },
    }


class DashboardHandler(BaseHTTPRequestHandler):
    """HTTP request handler providing REST endpoints and dashboard UI."""

    def log_message(self, format: str, *args: Any) -> None:
        """Suppress standard HTTP server access logs to keep terminal quiet."""
        pass

    def _is_origin_allowed(self) -> bool:
        """Verify that the request originates from a local origin (prevent CSRF/DNS rebinding)."""
        origin = self.headers.get("Origin")
        if not origin:
            # Same-origin requests or direct curl/CLI/app-mode requests often have no Origin header
            return True
        allowed_prefixes = (
            "http://127.0.0.1",
            "http://localhost",
            "https://127.0.0.1",
            "https://localhost",
        )
        return any(origin.startswith(prefix) for prefix in allowed_prefixes)

    def _send_json(self, data: Any, status: int = 200) -> None:
        payload = json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        origin = self.headers.get("Origin")
        if origin and self._is_origin_allowed():
            self.send_header("Access-Control-Allow-Origin", origin)
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()
        self.wfile.write(payload)

    def do_OPTIONS(self) -> None:
        if not self._is_origin_allowed():
            self.send_error(403, "Forbidden: Cross-origin access disallowed")
            return
        self.send_response(200)
        origin = self.headers.get("Origin")
        if origin:
            self.send_header("Access-Control-Allow-Origin", origin)
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def do_GET(self) -> None:
        if not self._is_origin_allowed():
            self.send_error(403, "Forbidden: Cross-origin access disallowed")
            return

        if self.path in ("/", "/index.html"):
            index_path = STATIC_DIR / "index.html"
            if index_path.exists():
                html_bytes = index_path.read_bytes()
            else:
                html_bytes = b"<h1>TokenJar UI Static Assets Not Found</h1>"

            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(html_bytes)))
            self.send_header("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0")
            self.send_header("Pragma", "no-cache")
            self.send_header("Expires", "0")
            self.end_headers()
            self.wfile.write(html_bytes)
            return

        if self.path == "/api/status":
            self._send_json(get_system_status())
            return

        self.send_error(404, "Not Found")

    def do_POST(self) -> None:
        if not self._is_origin_allowed():
            self.send_error(403, "Forbidden: Cross-origin access disallowed")
            return

        content_length = int(self.headers.get("Content-Length", 0))
        post_body = self.rfile.read(content_length) if content_length > 0 else b"{}"
        try:
            body = json.loads(post_body.decode("utf-8")) if post_body else {}
        except Exception:
            body = {}

        if self.path == "/api/toggle-ide":
            name = body.get("name")
            enable = bool(body.get("enable", True))
            configs = HookManager.get_supported_cli_configs()

            if name not in configs:
                self._send_json({"ok": False, "msg": f"Unknown IDE: {name}"}, status=400)
                return

            cfg_path = configs[name]
            if enable:
                ok, msg = HookManager._apply_mcp_config_with_backup(cfg_path)
            else:
                ok, msg = HookManager._revert_mcp_config_with_backup(cfg_path)

            self._send_json({"ok": ok, "msg": msg, "status": get_system_status()})
            return

        if self.path == "/api/toggle-all":
            enable = bool(body.get("enable", True))
            if enable:
                results = HookManager.enable_all()
                msg = "Activated TokenJar across detected IDEs"
            else:
                results = HookManager.disable_all()
                msg = "Deactivated TokenJar across all IDEs"

            self._send_json({"ok": True, "msg": msg, "results": results, "status": get_system_status()})
            return

        if self.path == "/api/toggle-rules":
            enable = bool(body.get("enable", True))
            if enable:
                RulesManager.install_rules(".")
                msg = "Steering rules installed in project"
            else:
                RulesManager.remove_rules(".")
                msg = "Steering rules cleanly removed from project"

            self._send_json({"ok": True, "msg": msg, "status": get_system_status()})
            return

        if self.path == "/api/toggle-output":
            compact = bool(body.get("compact", True))
            RulesManager.set_output_mode(".", enabled=compact)
            msg = f"Output mode set to: {'Compact Surgical' if compact else 'Standard Verbose'}"
            self._send_json({"ok": True, "msg": msg, "status": get_system_status()})
            return

        if self.path == "/api/prune-cache":
            try:
                p_cache = PersistentCache()
                pruned = p_cache.prune(max_entries=1000)
                msg = f"L2 Cache pruned ({pruned} entries cleared)"
                ok = True
            except Exception as e:
                msg = f"Cache prune failed: {e}"
                ok = False
            self._send_json({"ok": ok, "msg": msg, "status": get_system_status()})
            return

        if self.path in ("/api/clear-cache", "/api/reset-cache"):
            try:
                p_cache = PersistentCache()
                before = p_cache.count_entries()
                p_cache.clear()
                msg = f"L2 Cache completely cleared ({before} entries removed)"
                ok = True
            except Exception as e:
                msg = f"Cache clear failed: {e}"
                ok = False
            self._send_json({"ok": ok, "msg": msg, "status": get_system_status()})
            return

        if self.path == "/api/reset-stats":
            tracker.reset()
            self._send_json(
                {"ok": True, "msg": "Cumulative telemetry statistics reset to zero", "status": get_system_status()}
            )
            return

        if self.path == "/api/shutdown":
            self._send_json({"ok": True, "msg": "TokenJar dashboard shutting down..."})

            def _shutdown():
                time.sleep(0.3)
                self.server.shutdown()

            threading.Thread(target=_shutdown, daemon=True).start()
            return

        self.send_error(404, "Endpoint not found")


def open_app_window(url: str) -> None:
    """Open the dashboard URL in user's default browser."""
    if os.name == "nt" and hasattr(os, "startfile"):
        try:
            os.startfile(url)
            return
        except Exception:
            pass

    try:
        if webbrowser.open(url, new=2, autoraise=True):
            return
    except Exception:
        pass

    if os.name == "nt":
        try:
            subprocess.Popen(f'start "" "{url}"', shell=True)
            return
        except Exception:
            pass


def start_ui_server(port: int = 4141, host: str = "127.0.0.1", open_browser: bool = True) -> None:
    """Start the lightweight On-Demand Dashboard server."""
    # Find free port if 4141 is busy
    actual_port = port
    server = None
    for attempt in range(5):
        try:
            server = HTTPServer((host, actual_port), DashboardHandler)
            break
        except OSError:
            actual_port += 1

    if not server:
        print(f"Error: Could not bind local dashboard server on ports {port}-{actual_port}", file=sys.stderr)
        sys.exit(1)

    url = f"http://{host}:{actual_port}"
    print("┌────────────────────────────────────────────────────────────────────────┐")
    print("│ 🔋 TOKENJAR ON-DEMAND CONTROL DASHBOARD                             │")
    print("├────────────────────────────────────────────────────────────────────────┤")
    print(f"│  Dashboard URL : {url:<53} │")
    print("│  Architecture  : Standalone On-Demand (Zero Background RAM)            │")
    print("│  Exit          : Click 'Quit Dashboard' in UI or press Ctrl+C          │")
    print("└────────────────────────────────────────────────────────────────────────┘")

    sys.stdout.flush()

    if open_browser:
        threading.Thread(target=lambda: (time.sleep(0.5), open_app_window(url)), daemon=True).start()

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
        print("\n[TokenJar UI] Dashboard closed. Zero background processes active.")


if __name__ == "__main__":
    start_ui_server()
