"""TokenJar persistent telemetry and statistics tracking.

Stores cumulative token and financial savings in ~/.tokenjar/telemetry.json
so metrics persist across all sessions, commands, and MCP calls.
Provides granular category breakdowns (AST, Cache, RepoMap, Commands, Symbols).
"""

from __future__ import annotations

import atexit
import json
import threading
import time
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path


def _get_storage_path() -> Path:
    """Return the cross-platform path to the telemetry storage file."""
    home = Path.home()
    data_dir = home / ".tokenjar"
    data_dir.mkdir(parents=True, exist_ok=True)
    return data_dir / "telemetry.json"


@dataclass
class CategoryStats:
    """Statistics for an individual TokenJar optimization category."""

    original: int = 0
    optimized: int = 0
    saved: int = 0
    count: int = 0

    @property
    def savings_pct(self) -> float:
        if self.original <= 0:
            return 0.0
        return max(0.0, (self.saved / self.original) * 100)


@dataclass
class TelemetryData:
    """Cumulative metrics storage schema with category breakdowns."""

    total_original_tokens: int = 0
    total_optimized_tokens: int = 0
    total_tokens_saved: int = 0
    total_commands_filtered: int = 0
    total_files_cached: int = 0
    total_skeletons_generated: int = 0
    total_repo_maps_generated: int = 0
    first_used_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    last_used_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    # Granular categories
    skeleton: CategoryStats = field(default_factory=CategoryStats)
    slice: CategoryStats = field(default_factory=CategoryStats)
    cache: CategoryStats = field(default_factory=CategoryStats)
    repo_map: CategoryStats = field(default_factory=CategoryStats)
    command: CategoryStats = field(default_factory=CategoryStats)
    symbol_search: CategoryStats = field(default_factory=CategoryStats)
    lockfile: CategoryStats = field(default_factory=CategoryStats)

    @property
    def savings_pct(self) -> float:
        # Calculate savings across operations that TokenJar actually optimized
        effective_original = (
            self.skeleton.original
            + self.slice.original
            + self.cache.original
            + self.repo_map.original
            + self.command.original
            + self.symbol_search.original
            + self.lockfile.original
        )
        effective_saved = (
            self.skeleton.saved
            + self.slice.saved
            + self.cache.saved
            + self.repo_map.saved
            + self.command.saved
            + self.symbol_search.saved
            + self.lockfile.saved
        )
        if effective_original > 0:
            return (effective_saved / effective_original) * 100

        if self.total_original_tokens == 0:
            return 0.0
        return (self.total_tokens_saved / self.total_original_tokens) * 100

    @property
    def estimated_dollars_saved(self) -> float:
        # Standard blended input rate of $3.00 per 1M tokens (Claude 3.5 Sonnet / GPT-4o)
        return (self.total_tokens_saved / 1_000_000) * 3.00


class TelemetryTracker:
    """Singleton tracker for recording and querying token savings with cross-process sync."""

    def __init__(self) -> None:
        self.file_path = _get_storage_path()
        self._lock = threading.Lock()
        self._last_stat: tuple[int, int] = (0, 0)
        self._last_save_time: float = 0.0
        self._dirty: bool = False
        self._data: TelemetryData = self._load()
        atexit.register(self.flush)

    @property
    def data(self) -> TelemetryData:
        """Always return the latest telemetry data synchronized with disk."""
        with self._lock:
            if self._dirty:
                self._save()
                self._dirty = False
                self._last_save_time = time.time()
            self._refresh_if_needed()
            return self._data

    def flush(self) -> None:
        """Explicitly write any pending dirty telemetry to disk."""
        with self._lock:
            if self._dirty:
                self._save()
                self._dirty = False
                self._last_save_time = time.time()

    def _refresh_if_needed(self) -> None:
        try:
            if not self.file_path.exists():
                self._data = TelemetryData()
                self._last_stat = (0, 0)
                return
            st = self.file_path.stat()
            current_stat = (st.st_mtime_ns, st.st_size)
            if current_stat != self._last_stat:
                self._data = self._load()
                self._last_stat = current_stat
        except Exception:
            pass

    def _load(self) -> TelemetryData:
        if not self.file_path.exists():
            return TelemetryData()
        try:
            with open(self.file_path, "r", encoding="utf-8") as f:
                content = json.load(f)

            # Reconstruct CategoryStats objects
            cats = {}
            for cat in ("skeleton", "slice", "cache", "repo_map", "command", "symbol_search", "lockfile"):
                if cat in content and isinstance(content[cat], dict):
                    cats[cat] = CategoryStats(**content.pop(cat))

            # Remove obsolete fields if present
            content.pop("skeleton", None)
            content.pop("slice", None)
            content.pop("cache", None)
            content.pop("repo_map", None)
            content.pop("command", None)
            content.pop("symbol_search", None)
            content.pop("lockfile", None)

            base = TelemetryData(**content)
            for cat, stat in cats.items():
                setattr(base, cat, stat)
            return base
        except Exception:
            return TelemetryData()

    def _save(self) -> None:
        try:
            temp_path = self.file_path.with_suffix(".tmp")
            with open(temp_path, "w", encoding="utf-8") as f:
                json.dump(asdict(self._data), f, indent=2)
            temp_path.replace(self.file_path)
            st = self.file_path.stat()
            self._last_stat = (st.st_mtime_ns, st.st_size)
        except Exception:
            try:
                with open(self.file_path, "w", encoding="utf-8") as f:
                    json.dump(asdict(self._data), f, indent=2)
                st = self.file_path.stat()
                self._last_stat = (st.st_mtime_ns, st.st_size)
            except Exception:
                pass

    def record_savings(
        self,
        category: str,
        original_tokens: int,
        optimized_tokens: int,
    ) -> None:
        """Record token savings for a specific operation with multi-process consistency."""
        if original_tokens <= 0:
            return

        with self._lock:
            # Re-read fresh state from disk to prevent clobbering external resets or concurrent processes
            self._refresh_if_needed()

            saved = max(0, original_tokens - optimized_tokens)
            self._data.total_original_tokens += original_tokens
            self._data.total_optimized_tokens += optimized_tokens
            self._data.total_tokens_saved += saved
            self._data.last_used_at = datetime.now(timezone.utc).isoformat()

            # Update legacy counters
            if category == "command":
                self._data.total_commands_filtered += 1
            elif category == "cache":
                self._data.total_files_cached += 1
            elif category == "skeleton":
                self._data.total_skeletons_generated += 1
            elif category == "repo_map":
                self._data.total_repo_maps_generated += 1

            # Update category stats
            if hasattr(self._data, category):
                cat_stat: CategoryStats = getattr(self._data, category)
                cat_stat.original += original_tokens
                cat_stat.optimized += optimized_tokens
                cat_stat.saved += saved
                cat_stat.count += 1

            self._dirty = True
            now = time.time()
            if now - self._last_save_time >= 0.5:
                self._save()
                self._dirty = False
                self._last_save_time = now

    def reset(self) -> None:
        """Reset all telemetry metrics."""
        with self._lock:
            self._data = TelemetryData()
            self._dirty = False
            self._save()
            self._last_save_time = time.time()

    def get_l2_cache_disk_bytes(self) -> int:
        """Calculate total physical disk usage of L2 SQLite cache files (db, wal, shm)."""
        cache_dir = self.file_path.parent
        total = 0
        for name in ("cache.db", "cache.db-wal", "cache.db-shm"):
            f = cache_dir / name
            try:
                if f.is_file():
                    total += f.stat().st_size
            except OSError:
                pass
        return total

    def render_dashboard(self) -> str:
        """Format a detailed categorical terminal dashboard of metrics."""
        d = self.data
        dollars = f"${d.estimated_dollars_saved:.2f}"
        pct = f"%{d.savings_pct:.1f}"

        orig_str = f"{d.total_original_tokens:,}"
        saved_str = f"{d.total_tokens_saved:,}"

        cache_bytes = self.get_l2_cache_disk_bytes()
        cache_mb = cache_bytes / (1024.0 * 1024.0)
        cache_str = f"{cache_mb:.2f} MB"

        def fmt_cat(name: str, stat: CategoryStats, unit: str) -> str:
            sav_str = f"{stat.saved:,} tokens saved"
            pct_str = f"(%{stat.savings_pct:.1f})"
            count_str = f"{stat.count} {unit}"
            return f"│  • {name:<22} {sav_str:<21} {pct_str:<7} │ {count_str:<12} │"

        lines = [
            "┌────────────────────────────────────────────────────────────────────────┐",
            "│ 🔋 TOKENJAR DETAILED PERFORMANCE & SAVINGS DASHBOARD                │",
            "├────────────────────────────────────────────────────────────────────────┤",
            fmt_cat("AST Skeletonizer:", d.skeleton, "files"),
            fmt_cat("Smart Line Slicer:", d.slice, "slices"),
            fmt_cat("Smart File Cache:", d.cache, "reads"),
            fmt_cat("Lockfile Shield:", d.lockfile, "shields"),
            fmt_cat("Repo Map Engine:", d.repo_map, "maps"),
            fmt_cat("Global Symbol Search:", d.symbol_search, "searches"),
            fmt_cat("Terminal Pruner:", d.command, "runs"),
            "├────────────────────────────────────────────────────────────────────────┤",
            f"│  TOTAL TOKENS SAVED:       {saved_str:<16} ({pct} optimized reduction)│",
            f"│  ESTIMATED MONEY SAVED:    {dollars:<16} (at $3.00/1M blended rate) │",
            f"│  RAW CONTEXT PROCESSED:    {orig_str:<16} tokens total                  │",
            f"│  L2 CACHE DISK USAGE:      {cache_str:<16} (SQLite WAL storage)       │",
            "└────────────────────────────────────────────────────────────────────────┘",
        ]
        return "\n".join(lines)


# Global singleton instance
tracker = TelemetryTracker()
