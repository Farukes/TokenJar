//! Session-level in-memory cache with ultra-fast unified diffing via `similar`.

use similar::TextDiff;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub enum CacheStatus {
    FirstRead,
    Unchanged,
    Modified,
}

#[derive(Debug, Clone)]
pub struct CacheResult {
    pub status: CacheStatus,
    pub content: String,
    pub original_tokens: usize,
    pub optimized_tokens: usize,
    pub read_count: usize,
}

#[derive(Debug, Default)]
struct SessionEntry {
    content: String,
    hash: String,
    read_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_reads: usize,
    pub hits: usize,
    pub diffs: usize,
    pub first_reads: usize,
}

impl CacheStats {
    pub fn summary(&self) -> String {
        format!(
            "Session Cache Stats:\n  Total Reads: {}\n  Hits (Unchanged): {}\n  Diffs (Modified): {}\n  First Reads: {}",
            self.total_reads, self.hits, self.diffs, self.first_reads
        )
    }
}

#[derive(Debug, Default)]
pub struct SessionCache {
    entries: Mutex<HashMap<String, SessionEntry>>,
    stats: Mutex<CacheStats>,
}

fn normalize_path_key(path: &str) -> String {
    let (base_path, fragment) = match path.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (path, None),
    };
    let clean = base_path.replace('\\', "/");
    let trimmed = clean.strip_prefix("./").unwrap_or(&clean);
    let normalized_base = if let Ok(canon) = std::path::Path::new(base_path).canonicalize() {
        let raw = canon.to_string_lossy();
        let stripped = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
        stripped.replace('\\', "/")
    } else {
        trimmed.to_string()
    };
    if let Some(frag) = fragment {
        format!("{normalized_base}#{frag}")
    } else {
        normalized_base
    }
}

impl SessionCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn get(&self, file_path: &str, current_content: &str) -> CacheResult {
        let mut map = self.entries.lock().unwrap();
        let path_key = normalize_path_key(file_path);
        let orig_tokens = crate::token_counter::estimate_tokens(current_content);

        // Compute fast 64-bit or sha256 hash
        use sha2::{Digest, Sha256};
        let current_hash = format!("{:x}", Sha256::digest(current_content.as_bytes()));

        let mut stats = self.stats.lock().unwrap();
        stats.total_reads += 1;

        if let Some(entry) = map.get_mut(&path_key) {
            entry.read_count += 1;
            let read_no = entry.read_count;

            if entry.hash == current_hash {
                stats.hits += 1;
                // File or slice unchanged
                let (base_for_name, fragment) = match file_path.split_once('#') {
                    Some((b, f)) => (b, Some(f)),
                    None => (file_path, None),
                };
                let file_name = std::path::Path::new(base_for_name)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(base_for_name);

                let cached_msg = if let Some(frag) = fragment {
                    format!("[CACHED] {file_name} ({frag}) — unchanged since last read (read #{read_no})")
                } else {
                    format!("[CACHED] {file_name} — unchanged since last read (read #{read_no})")
                };
                let opt_tokens = crate::token_counter::estimate_tokens(&cached_msg);

                return CacheResult {
                    status: CacheStatus::Unchanged,
                    content: cached_msg,
                    original_tokens: orig_tokens,
                    optimized_tokens: opt_tokens,
                    read_count: read_no,
                };
            } else {
                stats.diffs += 1;
                // File modified: generate unified diff
                let diff = TextDiff::from_lines(entry.content.as_str(), current_content);
                let unified = diff
                    .unified_diff()
                    .header("original", "modified")
                    .to_string();

                let diff_tokens = crate::token_counter::estimate_tokens(&unified);

                // Tiny file anomaly guard: if diff takes more tokens than full file, return full file
                let (final_content, opt_tokens) = if diff_tokens >= orig_tokens {
                    (current_content.to_string(), orig_tokens)
                } else {
                    let file_name = std::path::Path::new(&path_key)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(file_path);
                    let diff_output = format!("[DIFF] {file_name} — modified since last read (read #{read_no}):\n{unified}");
                    let tokens = crate::token_counter::estimate_tokens(&diff_output);
                    (diff_output, tokens)
                };

                entry.content = current_content.to_string();
                entry.hash = current_hash;

                return CacheResult {
                    status: CacheStatus::Modified,
                    content: final_content,
                    original_tokens: orig_tokens,
                    optimized_tokens: opt_tokens,
                    read_count: read_no,
                };
            }
        }

        // First read: insert into cache
        stats.first_reads += 1;
        map.insert(
            path_key,
            SessionEntry {
                content: current_content.to_string(),
                hash: current_hash,
                read_count: 1,
            },
        );

        CacheResult {
            status: CacheStatus::FirstRead,
            content: current_content.to_string(),
            original_tokens: orig_tokens,
            optimized_tokens: orig_tokens,
            read_count: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_cache_flow() {
        let cache = SessionCache::new();
        let content_v1 = "fn main() {\n    println!(\"v1\");\n}\n";
        let content_v2 = "fn main() {\n    println!(\"v2\");\n}\n";

        // 1. First read
        let r1 = cache.get("main.rs", content_v1);
        assert_eq!(r1.status, CacheStatus::FirstRead);
        assert_eq!(r1.content, content_v1);

        // 2. Second read unchanged
        let r2 = cache.get("main.rs", content_v1);
        assert_eq!(r2.status, CacheStatus::Unchanged);
        assert!(r2.content.contains("[CACHED]"));

        // 3. Third read modified
        let r3 = cache.get("main.rs", content_v2);
        assert_eq!(r3.status, CacheStatus::Modified);
    }
}
