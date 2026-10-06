//! L2 Persistent SQLite Cache for TokenJar.
//!
//! Stores AST digests and symbol indexes across sessions and IDE reboots in ~/.tokenjar/cache.db.

use crate::models::IndexedSymbol;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

pub struct PersistentCache {
    db_path: PathBuf,
}

/// Normalizes project roots across platforms (strips Windows \\?\ prefix and normalizes backslashes).
#[inline]
pub fn normalize_project_root(root: &str) -> String {
    let stripped = root.strip_prefix(r"\\?\").unwrap_or(root);
    stripped.replace('\\', "/")
}

impl PersistentCache {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = home.join(".tokenjar");
        std::fs::create_dir_all(&dir).ok();
        let db_path = dir.join("cache.db");

        let cache = Self { db_path };
        cache.init_schema()?;
        Ok(cache)
    }

    pub fn with_path(db_path: PathBuf) -> Result<Self> {
        let cache = Self { db_path };
        cache.init_schema()?;
        Ok(cache)
    }

    fn connect(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;
        Ok(conn)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.connect()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS file_cache (
                file_path TEXT PRIMARY KEY,
                content_hash TEXT NOT NULL,
                mtime REAL NOT NULL,
                token_count INTEGER NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS symbol_index (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_root TEXT NOT NULL,
                file_path TEXT NOT NULL,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                line INTEGER NOT NULL,
                signature TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                mtime REAL NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_sym_proj_name ON symbol_index(project_root, name);
            CREATE INDEX IF NOT EXISTS idx_sym_proj_file ON symbol_index(project_root, file_path);",
        )?;

        // Auto-migration: ensure mtime column exists if table was created in earlier beta
        let _ = conn.execute(
            "ALTER TABLE symbol_index ADD COLUMN mtime REAL DEFAULT 0.0",
            [],
        );

        Ok(())
    }

    pub fn search_symbols(
        &self,
        project_root: &str,
        query: &str,
        exact: bool,
        max_results: usize,
    ) -> Result<Vec<IndexedSymbol>> {
        let norm_root = normalize_project_root(project_root);
        let conn = self.connect()?;
        let mut symbols = Vec::new();

        if exact {
            let mut stmt = conn.prepare(
                "SELECT name, kind, file_path, line, signature, file_hash
                 FROM symbol_index
                 WHERE project_root = ? AND name = ?
                 LIMIT ?",
            )?;
            let rows = stmt.query_map(params![norm_root, query, max_results], |row| {
                Ok(IndexedSymbol {
                    name: row.get(0)?,
                    kind: row.get(1)?,
                    file_path: row.get(2)?,
                    line: row.get(3)?,
                    signature: row.get(4)?,
                    content_hash: row.get(5)?,
                })
            })?;
            for s in rows {
                symbols.push(s?);
            }
        } else {
            let escaped = query
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            let pattern = format!("%{escaped}%");
            let mut stmt = conn.prepare(
                "SELECT name, kind, file_path, line, signature, file_hash
                 FROM symbol_index
                 WHERE project_root = ? AND name LIKE ? ESCAPE '\\'
                 LIMIT ?",
            )?;
            let rows = stmt.query_map(params![norm_root, pattern, max_results], |row| {
                Ok(IndexedSymbol {
                    name: row.get(0)?,
                    kind: row.get(1)?,
                    file_path: row.get(2)?,
                    line: row.get(3)?,
                    signature: row.get(4)?,
                    content_hash: row.get(5)?,
                })
            })?;
            for s in rows {
                symbols.push(s?);
            }
        }

        Ok(symbols)
    }

    pub fn get_all_symbols(&self, project_root: &str) -> Result<Vec<IndexedSymbol>> {
        let norm_root = normalize_project_root(project_root);
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT name, kind, file_path, line, signature, file_hash
             FROM symbol_index
             WHERE project_root = ?",
        )?;
        let rows = stmt.query_map(params![norm_root], |row| {
            Ok(IndexedSymbol {
                name: row.get(0)?,
                kind: row.get(1)?,
                file_path: row.get(2)?,
                line: row.get(3)?,
                signature: row.get(4)?,
                content_hash: row.get(5)?,
            })
        })?;
        let mut symbols = Vec::new();
        for s in rows {
            symbols.push(s?);
        }
        Ok(symbols)
    }

    pub fn set_file_symbols(
        &self,
        project_root: &str,
        file_path: &str,
        file_hash: &str,
        mtime: f64,
        symbols: &[IndexedSymbol],
    ) -> Result<()> {
        let norm_root = normalize_project_root(project_root);
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;

        // Delete existing symbols for this file
        tx.execute(
            "DELETE FROM symbol_index WHERE project_root = ? AND file_path = ?",
            params![norm_root, file_path],
        )?;

        // Insert fresh symbols
        {
            let mut stmt = tx.prepare(
                "INSERT INTO symbol_index (project_root, file_path, name, kind, line, signature, file_hash, mtime)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )?;
            for s in symbols {
                stmt.execute(params![
                    norm_root,
                    file_path,
                    s.name,
                    s.kind,
                    s.line,
                    s.signature,
                    file_hash,
                    mtime
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_file_meta(
        &self,
        project_root: &str,
        file_path: &str,
    ) -> Result<Option<(String, f64)>> {
        let norm_root = normalize_project_root(project_root);
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT file_hash, mtime FROM symbol_index WHERE project_root = ? AND file_path = ? LIMIT 1",
        )?;
        let mut rows = stmt.query(params![norm_root, file_path])?;
        if let Some(row) = rows.next()? {
            let hash: String = row.get(0)?;
            let mtime: f64 = row.get(1)?;
            Ok(Some((hash, mtime)))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_file_metas(
        &self,
        project_root: &str,
    ) -> Result<std::collections::HashMap<String, (String, f64)>> {
        let norm_root = normalize_project_root(project_root);
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT file_path, file_hash, mtime FROM symbol_index WHERE project_root = ? GROUP BY file_path",
        )?;
        let rows = stmt.query_map(params![norm_root], |row| {
            let path: String = row.get(0)?;
            let hash: String = row.get(1)?;
            let mtime: f64 = row.get(2)?;
            Ok((path, (hash, mtime)))
        })?;
        let mut map = std::collections::HashMap::new();
        for r in rows {
            let (p, data) = r?;
            map.insert(p, data);
        }
        Ok(map)
    }

    pub fn prune(&self, max_entries: usize) -> Result<usize> {
        let conn = self.connect()?;
        let count: usize = conn
            .query_row("SELECT COUNT(*) FROM file_cache", [], |r| r.get(0))
            .unwrap_or(0);
        if count > max_entries {
            let to_remove = count - max_entries;
            conn.execute(
                "DELETE FROM file_cache WHERE file_path IN (SELECT file_path FROM file_cache ORDER BY cached_at ASC LIMIT ?)",
                params![to_remove],
            )?;
            return Ok(to_remove);
        }
        Ok(0)
    }

    pub fn count_entries(&self) -> Result<usize> {
        let conn = self.connect()?;
        let file_count: usize = conn
            .query_row("SELECT COUNT(*) FROM file_cache", [], |r| r.get(0))
            .unwrap_or(0);
        let symbol_count: usize = conn
            .query_row("SELECT COUNT(*) FROM symbol_index", [], |r| r.get(0))
            .unwrap_or(0);
        Ok(file_count + symbol_count)
    }

    pub fn clear(&self) -> Result<usize> {
        let conn = self.connect()?;
        let count = self.count_entries()?;
        conn.execute("DELETE FROM file_cache", [])?;
        conn.execute("DELETE FROM symbol_index", [])?;
        let _ = conn.execute("VACUUM", []);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistent_cache_symbols() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let cache = PersistentCache::with_path(temp.path().to_path_buf()).unwrap();

        let syms = vec![IndexedSymbol {
            name: "calculate_tokens".to_string(),
            kind: "function".to_string(),
            file_path: "src/counter.rs".to_string(),
            line: 42,
            signature: "pub fn calculate_tokens(s: &str) -> usize".to_string(),
            content_hash: "hash123".to_string(),
        }];

        cache
            .set_file_symbols("/test_root", "src/counter.rs", "hash123", 1000.0, &syms)
            .unwrap();

        // Exact search
        let exact = cache
            .search_symbols("/test_root", "calculate_tokens", true, 10)
            .unwrap();
        assert_eq!(exact.len(), 1);
        assert_eq!(exact[0].name, "calculate_tokens");

        // Fuzzy/substring search
        let fuzzy = cache
            .search_symbols("/test_root", "tokens", false, 10)
            .unwrap();
        assert_eq!(fuzzy.len(), 1);

        // Path normalization test: set with Windows UNC and backslashes, query with forward slashes
        cache
            .set_file_symbols(r"\\?\C:\Projects\MyApp", "src/lib.rs", "hash456", 2000.0, &syms)
            .unwrap();
        let cross = cache
            .search_symbols("C:/Projects/MyApp", "calculate_tokens", true, 10)
            .unwrap();
        assert_eq!(cross.len(), 1);
        assert_eq!(cross[0].name, "calculate_tokens");
    }
}
