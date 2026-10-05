"""Session-level file cache for TokenJar.

Stores SHA-256 hashes and content of previously read files.
Enables differential reads (returning only diffs on re-reads)
and cache-hit detection (returning a compact reference for unchanged files).
"""

from __future__ import annotations

import difflib
import hashlib
import os
from dataclasses import dataclass
from pathlib import Path

from tokenjar.cache.persistent_cache import PersistentCache


@dataclass
class CacheEntry:
    """A cached file entry with content and hash."""

    content: str
    hash: str
    read_count: int = 0


class SessionCache:
    """Session cache with L1 in-memory and L2 persistent SQLite backing.

    Tracks file content by SHA-256 hash. When a file is re-read:
    - If unchanged: returns a compact '[CACHED] unchanged' message (~3 tokens)
    - If changed: returns a unified diff if smaller than full file (~50-200 tokens)
    - If tiny file: returns full content to avoid diff header overhead (guardrail)

    Replaces sending the full file content repeatedly across sessions.
    """

    def __init__(self, persistent: bool = True, db_path: str | Path | None = None) -> None:
        self._cache: dict[str, CacheEntry] = {}
        self._stats = CacheStats()
        self._persistent = persistent
        self._persistent_cache = PersistentCache(db_path) if persistent else None

    def get(self, file_path: str, current_content: str) -> CacheResult:
        """Check cache and return appropriate result.

        Args:
            file_path: Absolute or relative path to the file.
            current_content: The current content of the file.

        Returns:
            CacheResult with status and optimized content.
        """
        current_hash = self._compute_hash(current_content)
        normalized_path = self._normalize_path(file_path)

        # Check L2 persistent cache on memory miss
        if normalized_path not in self._cache and self._persistent_cache:
            persisted = self._persistent_cache.get_entry(normalized_path)
            if persisted is not None:
                p_hash, p_content, p_reads = persisted
                self._cache[normalized_path] = CacheEntry(
                    content=p_content,
                    hash=p_hash,
                    read_count=p_reads,
                )

        if normalized_path not in self._cache:
            # First read — cache it and return full content
            entry = CacheEntry(
                content=current_content,
                hash=current_hash,
                read_count=1,
            )
            self._cache[normalized_path] = entry
            if self._persistent_cache:
                self._persistent_cache.set_entry(normalized_path, current_hash, current_content, 1)

            self._stats.total_reads += 1
            self._stats.cache_misses += 1
            return CacheResult(
                status=CacheStatus.FIRST_READ,
                content=current_content,
                original_tokens=len(current_content) // 4,
                optimized_tokens=len(current_content) // 4,
            )

        entry = self._cache[normalized_path]
        entry.read_count += 1
        self._stats.total_reads += 1

        if entry.hash == current_hash:
            # Unchanged — return compact reference (pure L1 in-memory hit)
            self._stats.cache_hits += 1
            if "#" in file_path:
                base_part, frag = file_path.split("#", 1)
                base_name = os.path.basename(base_part)
                compact = f"[CACHED] {base_name} ({frag}) — unchanged since last read (read #{entry.read_count})"
            else:
                base_name = os.path.basename(file_path)
                compact = f"[CACHED] {base_name} — unchanged since last read (read #{entry.read_count})"
            return CacheResult(
                status=CacheStatus.UNCHANGED,
                content=compact,
                original_tokens=len(current_content) // 4,
                optimized_tokens=len(compact) // 4,
            )

        # Changed — compute diff
        self._stats.cache_diffs += 1
        diff = self._compute_diff(entry.content, current_content, file_path)

        # Update cache with new content
        entry.content = current_content
        entry.hash = current_hash
        if self._persistent_cache:
            self._persistent_cache.set_entry(normalized_path, current_hash, current_content, entry.read_count)

        # Guardrail (Tiny File Anomaly Guard):
        # If the diff (including headers) is larger than or equal to the file itself,
        # return full content to guarantee TokenJar never inflates token cost.
        if len(diff) >= len(current_content) and len(current_content) > 0:
            return CacheResult(
                status=CacheStatus.CHANGED,
                content=current_content,
                original_tokens=len(current_content) // 4,
                optimized_tokens=len(current_content) // 4,
            )

        return CacheResult(
            status=CacheStatus.CHANGED,
            content=diff,
            original_tokens=len(current_content) // 4,
            optimized_tokens=len(diff) // 4,
        )

    def invalidate(self, file_path: str) -> None:
        """Remove a file from both memory and persistent cache."""
        normalized = self._normalize_path(file_path)
        self._cache.pop(normalized, None)
        if self._persistent_cache:
            self._persistent_cache.invalidate(normalized)

    def clear(self) -> None:
        """Clear both memory and persistent cache."""
        self._cache.clear()
        self._stats = CacheStats()
        if self._persistent_cache:
            self._persistent_cache.clear()

    def get_stats(self) -> CacheStats:
        """Get cache statistics."""
        return self._stats

    @staticmethod
    def _compute_hash(content: str) -> str:
        """Compute SHA-256 hash of content."""
        return hashlib.sha256(content.encode("utf-8")).hexdigest()

    @staticmethod
    def _normalize_path(file_path: str) -> str:
        """Normalize file path for consistent cache keys."""
        import os

        if "#" in file_path:
            base_p, frag = file_path.split("#", 1)
            return f"{os.path.normpath(os.path.abspath(base_p))}#{frag}"
        return os.path.normpath(os.path.abspath(file_path))

    @classmethod
    def _compute_diff(cls, old_content: str, new_content: str, file_path: str) -> str:
        """Compute an AST-aware semantic diff between old and new content."""
        import os

        base_name = os.path.basename(file_path)
        old_lines = old_content.splitlines(keepends=True)
        new_lines = new_content.splitlines(keepends=True)

        diff = difflib.unified_diff(
            old_lines,
            new_lines,
            fromfile=f"a/{base_name}",
            tofile=f"b/{base_name}",
            n=2,
        )

        diff_text = "".join(diff)
        if not diff_text:
            return f"[CACHED] {base_name} — no visible changes"

        # AST Semantic Awareness: identify which symbols were altered
        semantic_header = ""
        try:
            from tokenjar.tools.symbol_index import SymbolIndexer
            from tokenjar.utils.file_utils import detect_language

            lang = detect_language(file_path)
            if lang:
                old_syms = {
                    s.name: s.content_hash
                    for s in SymbolIndexer.extract_symbols_from_code(old_content, lang, base_name)
                }
                new_syms = {
                    s.name: s.content_hash
                    for s in SymbolIndexer.extract_symbols_from_code(new_content, lang, base_name)
                }
                changed_syms = [
                    name for name, chash in new_syms.items() if name not in old_syms or old_syms[name] != chash
                ]
                if changed_syms:
                    semantic_header = f"[SEMANTIC FOCUS: {', '.join(changed_syms[:5])}]\n"
        except Exception:
            pass

        return f"[DIFF] Changes in {base_name}:\n{semantic_header}{diff_text}"


class CacheStatus:
    """Cache lookup result status."""

    FIRST_READ = "first_read"
    UNCHANGED = "unchanged"
    CHANGED = "changed"


@dataclass
class CacheResult:
    """Result of a cache lookup."""

    status: str
    content: str
    original_tokens: int
    optimized_tokens: int

    @property
    def savings_pct(self) -> float:
        """Calculate percentage of tokens saved."""
        if self.original_tokens == 0:
            return 0.0
        return (1 - self.optimized_tokens / self.original_tokens) * 100


@dataclass
class CacheStats:
    """Cumulative cache statistics."""

    total_reads: int = 0
    cache_hits: int = 0
    cache_diffs: int = 0
    cache_misses: int = 0

    @property
    def hit_rate(self) -> float:
        """Calculate cache hit rate percentage."""
        if self.total_reads == 0:
            return 0.0
        return (self.cache_hits / self.total_reads) * 100

    def summary(self) -> str:
        """Human-readable summary of cache performance."""
        return (
            f"Cache: {self.total_reads} reads, "
            f"{self.cache_hits} hits ({self.hit_rate:.0f}%), "
            f"{self.cache_diffs} diffs, "
            f"{self.cache_misses} misses"
        )
