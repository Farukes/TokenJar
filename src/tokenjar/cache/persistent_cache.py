"""Persistent SQLite storage for TokenJar file cache.

Enables cache persistence across MCP server restarts and independent CLI sessions.
Uses SQLite with WAL mode for fast, safe concurrent reads and writes.
"""

from __future__ import annotations

import sqlite3
import time
from pathlib import Path

MAX_CACHEABLE_BYTES = 5 * 1024 * 1024  # 5 MB


class PersistentCache:
    """SQLite-backed persistent store for file hash and content cache."""

    def __init__(self, db_path: Path | str | None = None) -> None:
        if db_path is None:
            cache_dir = Path.home() / ".tokenjar"
            cache_dir.mkdir(parents=True, exist_ok=True)
            self._db_path = cache_dir / "cache.db"
        else:
            self._db_path = Path(db_path)
            self._db_path.parent.mkdir(parents=True, exist_ok=True)

        self._init_db()

    def _get_connection(self) -> sqlite3.Connection:
        conn = sqlite3.connect(str(self._db_path), timeout=5.0)
        conn.execute("PRAGMA journal_mode=WAL;")
        conn.execute("PRAGMA synchronous=NORMAL;")
        return conn

    def _init_db(self) -> None:
        """Create cache table and indexes if they do not exist."""
        try:
            with self._get_connection() as conn:
                conn.execute(
                    """
                    CREATE TABLE IF NOT EXISTS file_cache (
                        path TEXT PRIMARY KEY,
                        hash TEXT NOT NULL,
                        content TEXT NOT NULL,
                        read_count INTEGER DEFAULT 1,
                        updated_at REAL NOT NULL
                    )
                    """
                )
                conn.execute("CREATE INDEX IF NOT EXISTS idx_cache_updated ON file_cache(updated_at);")
                conn.execute(
                    """
                    CREATE TABLE IF NOT EXISTS symbol_index (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        project_root TEXT NOT NULL,
                        file_path TEXT NOT NULL,
                        name TEXT NOT NULL,
                        kind TEXT NOT NULL,
                        line INTEGER NOT NULL,
                        signature TEXT NOT NULL,
                        file_hash TEXT NOT NULL,
                        mtime REAL NOT NULL DEFAULT 0.0
                    )
                    """
                )
                try:
                    conn.execute("ALTER TABLE symbol_index ADD COLUMN mtime REAL DEFAULT 0.0")
                except Exception:
                    pass
                conn.execute("CREATE INDEX IF NOT EXISTS idx_symbol_project_name ON symbol_index(project_root, name);")
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_symbol_project_file ON symbol_index(project_root, file_path);"
                )
                conn.execute(
                    """
                    CREATE TABLE IF NOT EXISTS file_index_meta (
                        project_root TEXT NOT NULL,
                        file_path TEXT NOT NULL,
                        file_hash TEXT NOT NULL,
                        mtime REAL NOT NULL,
                        PRIMARY KEY (project_root, file_path)
                    )
                    """
                )
        except Exception:
            pass

    def get_entry(self, path: str) -> tuple[str, str, int] | None:
        """Fetch cached entry (hash, content, read_count) for a given path."""
        try:
            with self._get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute(
                    "SELECT hash, content, read_count FROM file_cache WHERE path = ?",
                    (path,),
                )
                row = cursor.fetchone()
                if row:
                    return str(row[0]), str(row[1]), int(row[2])
                return None
        except Exception:
            return None

    def set_entry(self, path: str, hash_val: str, content: str, read_count: int = 1) -> None:
        """Store or update a cached file entry. Protects SQLite from giant files > 5MB."""
        try:
            stored_content = content
            if len(content) > MAX_CACHEABLE_BYTES:
                stored_content = f"__TOKENJAR_LARGE_FILE__:{len(content)}"

            with self._get_connection() as conn:
                conn.execute(
                    """
                    INSERT INTO file_cache (path, hash, content, read_count, updated_at)
                    VALUES (?, ?, ?, ?, ?)
                    ON CONFLICT(path) DO UPDATE SET
                        hash = excluded.hash,
                        content = excluded.content,
                        read_count = excluded.read_count,
                        updated_at = excluded.updated_at
                    """,
                    (path, hash_val, stored_content, read_count, time.time()),
                )
        except Exception:
            pass

    def invalidate(self, path: str) -> None:
        """Remove an entry from the persistent cache."""
        try:
            with self._get_connection() as conn:
                conn.execute("DELETE FROM file_cache WHERE path = ?", (path,))
        except Exception:
            pass

    def clear(self) -> None:
        """Clear all entries in the persistent cache."""
        try:
            with self._get_connection() as conn:
                conn.execute("DELETE FROM file_cache")
                conn.execute("DELETE FROM symbol_index")
                conn.execute("DELETE FROM file_index_meta")
                conn.execute("VACUUM")
        except Exception:
            pass

    def prune(self, max_entries: int = 5000, max_age_days: int = 7) -> int:
        """Prune old entries to prevent unbounded disk usage."""
        try:
            cutoff = time.time() - (max_age_days * 86400)
            with self._get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute("DELETE FROM file_cache WHERE updated_at < ?", (cutoff,))
                deleted = cursor.rowcount

                cursor.execute("SELECT COUNT(*) FROM file_cache")
                count = cursor.fetchone()[0]
                if count > max_entries:
                    excess = count - max_entries
                    cursor.execute(
                        """
                        DELETE FROM file_cache WHERE path IN (
                            SELECT path FROM file_cache ORDER BY updated_at ASC LIMIT ?
                        )
                        """,
                        (excess,),
                    )
                    deleted += cursor.rowcount
                return deleted
        except Exception:
            return 0

    def count_entries(self) -> int:
        """Return total number of cached entries (files and indexed symbols)."""
        try:
            with self._get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute("SELECT COUNT(*) FROM file_cache")
                files_count = cursor.fetchone()[0]
                cursor.execute("SELECT COUNT(*) FROM symbol_index")
                syms_count = cursor.fetchone()[0]
                return int(files_count + syms_count)
        except Exception:
            return 0

    def get_indexed_file_meta(self, project_root: str, file_path: str) -> tuple[str, float] | None:
        """Fetch indexed file hash and mtime for a file within a project."""
        try:
            with self._get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute(
                    "SELECT file_hash, mtime FROM file_index_meta WHERE project_root = ? AND file_path = ?",
                    (project_root, file_path),
                )
                row = cursor.fetchone()
                if row:
                    return str(row[0]), float(row[1])
                return None
        except Exception:
            return None

    def set_file_symbols(
        self,
        project_root: str,
        file_path: str,
        file_hash: str,
        mtime: float,
        symbols: list[dict],
    ) -> None:
        """Atomically update the symbol index and metadata for a specific file."""
        try:
            with self._get_connection() as conn:
                conn.execute(
                    "DELETE FROM symbol_index WHERE project_root = ? AND file_path = ?",
                    (project_root, file_path),
                )
                for s in symbols:
                    conn.execute(
                        """
                        INSERT INTO symbol_index (
                            project_root, file_path, name, kind, line, signature, file_hash, mtime
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                        """,
                        (
                            project_root,
                            file_path,
                            s.get("name", ""),
                            s.get("kind", ""),
                            s.get("line", 0),
                            s.get("signature", ""),
                            file_hash,
                            float(mtime),
                        ),
                    )
                conn.execute(
                    """
                    INSERT INTO file_index_meta (project_root, file_path, file_hash, mtime)
                    VALUES (?, ?, ?, ?)
                    ON CONFLICT(project_root, file_path) DO UPDATE SET
                        file_hash = excluded.file_hash,
                        mtime = excluded.mtime
                    """,
                    (project_root, file_path, file_hash, mtime),
                )
        except Exception:
            pass

    def get_file_symbols(self, project_root: str, file_path: str) -> list[dict]:
        """Fetch all indexed symbols for a specific file."""
        try:
            with self._get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute(
                    """
                    SELECT name, kind, file_path, line, signature, file_hash
                    FROM symbol_index
                    WHERE project_root = ? AND file_path = ?
                    ORDER BY line ASC
                    """,
                    (project_root, file_path),
                )
                rows = cursor.fetchall()
                return [
                    {
                        "name": r[0],
                        "kind": r[1],
                        "file_path": r[2],
                        "line": int(r[3]),
                        "signature": r[4],
                        "file_hash": r[5],
                    }
                    for r in rows
                ]
        except Exception:
            return []

    def search_symbols(
        self,
        project_root: str,
        query: str,
        exact: bool = False,
        max_results: int = 15,
    ) -> list[dict]:
        """Search symbols scoped strictly to a project_root using SQLite indexes."""
        try:
            with self._get_connection() as conn:
                cursor = conn.cursor()
                if exact:
                    cursor.execute(
                        """
                        SELECT name, kind, file_path, line, signature, file_hash
                        FROM symbol_index
                        WHERE project_root = ? AND name = ?
                        LIMIT ?
                        """,
                        (project_root, query, max_results),
                    )
                else:
                    like_pattern = f"%{query}%"
                    cursor.execute(
                        """
                        SELECT name, kind, file_path, line, signature, file_hash
                        FROM symbol_index
                        WHERE project_root = ? AND name LIKE ?
                        LIMIT ?
                        """,
                        (project_root, like_pattern, max_results),
                    )
                rows = cursor.fetchall()
                return [
                    {
                        "name": r[0],
                        "kind": r[1],
                        "file_path": r[2],
                        "line": int(r[3]),
                        "signature": r[4],
                        "file_hash": r[5],
                    }
                    for r in rows
                ]
        except Exception:
            return []

    def get_all_project_symbols(self, project_root: str) -> list[dict]:
        """Return all indexed symbols for a specific project."""
        try:
            with self._get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute(
                    """
                    SELECT name, kind, file_path, line, signature, file_hash
                    FROM symbol_index
                    WHERE project_root = ?
                    ORDER BY file_path ASC, line ASC
                    """,
                    (project_root,),
                )
                rows = cursor.fetchall()
                return [
                    {
                        "name": r[0],
                        "kind": r[1],
                        "file_path": r[2],
                        "line": int(r[3]),
                        "signature": r[4],
                        "file_hash": r[5],
                    }
                    for r in rows
                ]
        except Exception:
            return []

    def prune_project_deleted_files(self, project_root: str, valid_file_paths: set[str]) -> int:
        """Remove symbols and metadata for files that no longer exist in the project."""
        try:
            with self._get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute(
                    "SELECT file_path FROM file_index_meta WHERE project_root = ?",
                    (project_root,),
                )
                stored_files = [r[0] for r in cursor.fetchall()]
                deleted_count = 0
                for f in stored_files:
                    if f not in valid_file_paths:
                        conn.execute(
                            "DELETE FROM symbol_index WHERE project_root = ? AND file_path = ?",
                            (project_root, f),
                        )
                        conn.execute(
                            "DELETE FROM file_index_meta WHERE project_root = ? AND file_path = ?",
                            (project_root, f),
                        )
                        deleted_count += 1
                return deleted_count
        except Exception:
            return 0
