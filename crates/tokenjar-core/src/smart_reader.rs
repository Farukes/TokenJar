//! Smart Reader with Session Caching & Lockfile Shielding in Rust.
//!
//! Intelligently reads source files, returns compact diffs on edits,
//! and protects context windows from massive lockfiles and minified assets.

use std::path::Path;

use crate::cache::session_cache::{CacheStatus, SessionCache};
use crate::config::TokenJarConfig;
use crate::filters::lockfile::process_lockfile;
use crate::telemetry::TelemetryTracker;
use crate::token_counter::format_savings;

/// Maximum lines per read slice before auto-pagination kicks in (protects IDE context).
pub const MAX_OUTPUT_LINES: usize = 40;

/// Maximum bytes per read slice before auto-pagination kicks in (prevents framework output.txt dumping).
pub const MAX_OUTPUT_BYTES: usize = 2500;

/// Intelligently reads a file with session caching, line slicing, and lockfile protection.
#[allow(clippy::too_many_arguments)]
pub fn read_file_smart(
    file_path: &str,
    force_full: bool,
    query: Option<&str>,
    start_line: Option<usize>,
    end_line: Option<usize>,
    symbol: Option<&str>,
    cache: &SessionCache,
    config: &TokenJarConfig,
    tracker: &TelemetryTracker,
) -> String {
    let p = Path::new(file_path);

    if !force_full && config.is_ignored(p) {
        let base = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.to_string());
        return format!(
            "[TOKENJAR SECURITY] '{base}' matches security ignore patterns \
            (credentials/secrets/exclusions). Pass force_full=true if you explicitly \
            need to read this file."
        );
    }

    // Binary file safeguard
    if let Some(ext) = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
    {
        const BINARY_EXTS: &[&str] = &[
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp", "mp3", "mp4", "wav", "avi",
            "mov", "mkv", "webm", "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "exe", "dll", "so",
            "dylib", "bin", "o", "a", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "woff",
            "woff2", "ttf", "otf", "eot", "pyc", "pyo", "class", "jar", "db", "sqlite", "sqlite3",
        ];
        if BINARY_EXTS.contains(&ext.as_str()) {
            return format!(
                "[TOKENJAR] Binary file '{}' skipped to prevent context window corruption.",
                p.display()
            );
        }
    }

    if p.is_dir() {
        return format!(
            "[TOKENJAR] '{file_path}' is a directory, not a file. Use 'get_directory_tree_tool' or 'get_repo_map_tool' to explore directory contents."
        );
    }

    if let Ok(meta) = std::fs::metadata(p) {
        if meta.len() as usize > config.max_cacheable_bytes && !force_full {
            return format!(
                "[TOKENJAR] File '{file_path}' ({:.2} MB) exceeds maximum cacheable limit ({:.2} MB). Reading this entirely into context would consume massive tokens. Pass force_full=true if you explicitly need the raw content.",
                meta.len() as f64 / (1024.0 * 1024.0),
                config.max_cacheable_bytes as f64 / (1024.0 * 1024.0)
            );
        }
    }

    let content = match std::fs::read(p) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
        },
        Err(e) => return format!("Error reading file {file_path}: {e}"),
    };

    // Direct symbol slicing support (targeted AST extraction)
    if let Some(sym) = symbol {
        let sym = sym.trim();
        if !sym.is_empty() {
            let total_lines = content.lines().count();
            if let Some(lang) = crate::parser::detect_language(p) {
                if let Some((start, end, _impl_str)) =
                    crate::skeleton::find_symbol_range_in_code(&content, lang, sym)
                {
                    let lines: Vec<&str> = content.lines().collect();
                    let slice_lines: Vec<String> = lines[start - 1..end.min(lines.len())]
                        .iter()
                        .enumerate()
                        .map(|(idx, line)| format!("{}: {}", start + idx, line))
                        .collect();

                    let slice_key = format!("{file_path}#sym:{sym}");
                    let header = format!(
                        "[TOKENJAR] Symbol '{sym}' found at lines {start}-{end} of {total_lines} in '{file_path}':\n"
                    );

                    // Check slicing cache
                    let raw_sliced = format!("{header}{}", slice_lines.join("\n"));
                    let entry = cache.get(&slice_key, &raw_sliced);
                    if entry.status == CacheStatus::Unchanged {
                        let orig_tok = (content.len() / 4) as u64;
                        let opt_tok = (entry.content.len() / 4) as u64;
                        tracker.record_savings("cache", orig_tok, opt_tok);
                        let savings_msg = format_savings(&content, &entry.content);
                        return format!("{}\n\nToken savings: {savings_msg}", entry.content);
                    }

                    // Auto-pagination if symbol is larger than MAX_OUTPUT_LINES
                    let (display_slice, paginated) = if slice_lines.len() > MAX_OUTPUT_LINES
                        && !force_full
                    {
                        let p_end = start + MAX_OUTPUT_LINES - 1;
                        (
                            &slice_lines[..MAX_OUTPUT_LINES],
                            Some(format!(
                                "\n\n[TOKENJAR PAGINATION] Showing lines {start}-{p_end} of {total_lines}. (Remaining lines truncated to protect context window)\n\
                                 👉 To read the next slice, call read_file_smart with start_line={}, end_line={}.",
                                p_end + 1,
                                (p_end + MAX_OUTPUT_LINES).min(end)
                            )),
                        )
                    } else {
                        (&slice_lines[..], None)
                    };
                    let body = display_slice.join("\n");
                    let sliced_content = match paginated {
                        Some(notice) => format!("{header}{body}{notice}"),
                        None => format!("{header}{body}"),
                    };

                    let orig_tok = (content.len() / 4) as u64;
                    let opt_tok = (sliced_content.len() / 4) as u64;
                    if orig_tok > opt_tok {
                        tracker.record_savings("slice", orig_tok, opt_tok);
                        let savings_msg = format_savings(&content, &sliced_content);
                        return format!("{sliced_content}\n\nToken savings: {savings_msg}");
                    }
                    return sliced_content;
                }
            }

            // Fallback for non-AST files or if symbol not found by AST: search for symbol definition line
            let lines: Vec<&str> = content.lines().collect();
            let mut match_line = None;
            for (i, line) in lines.iter().enumerate() {
                if line.contains(sym) {
                    match_line = Some(i + 1);
                    break;
                }
            }

            if let Some(start) = match_line {
                let end = (start + 40).min(lines.len());
                let slice_lines: Vec<String> = lines[start - 1..end]
                    .iter()
                    .enumerate()
                    .map(|(idx, line)| format!("{}: {}", start + idx, line))
                    .collect();

                let slice_key = format!("{file_path}#sym:{sym}");
                let header = format!(
                    "[TOKENJAR] Symbol '{sym}' matched at line {start} (showing lines {start}-{end} of {total_lines}) in '{file_path}':\n"
                );
                let raw_sliced = format!("{header}{}", slice_lines.join("\n"));
                let entry = cache.get(&slice_key, &raw_sliced);
                if entry.status == CacheStatus::Unchanged {
                    let orig_tok = (content.len() / 4) as u64;
                    let opt_tok = (entry.content.len() / 4) as u64;
                    tracker.record_savings("cache", orig_tok, opt_tok);
                    let savings_msg = format_savings(&content, &entry.content);
                    return format!("{}\n\nToken savings: {savings_msg}", entry.content);
                }

                let orig_tok = (content.len() / 4) as u64;
                let opt_tok = (raw_sliced.len() / 4) as u64;
                if orig_tok > opt_tok {
                    tracker.record_savings("slice", orig_tok, opt_tok);
                    let savings_msg = format_savings(&content, &raw_sliced);
                    return format!("{raw_sliced}\n\nToken savings: {savings_msg}");
                }
                return raw_sliced;
            }

            return format!(
                "[TOKENJAR] Symbol '{sym}' was not found in '{file_path}'. Tip: use 'tool_get_code_skeleton' to see available functions/classes or pass start_line and end_line."
            );
        }
    }

    // Line slicing support (targeted range extraction)
    if start_line.is_some() || end_line.is_some() {
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        if total == 0 {
            return format!("[TOKENJAR] File '{file_path}' is empty (0 lines).");
        }
        let start = start_line.unwrap_or(1).max(1);
        let end = end_line.unwrap_or(total).min(total);

        if start > total {
            return format!("[TOKENJAR] Requested start_line {start} exceeds total line count ({total}) in '{file_path}'.");
        }
        if start > end {
            return format!("[TOKENJAR] Invalid line range: start_line ({start}) cannot be greater than end_line ({end}).");
        }

        let slice_lines: Vec<String> = lines[start - 1..end]
            .iter()
            .enumerate()
            .map(|(idx, line)| format!("{}: {}", start + idx, line))
            .collect();

        let slice_key = format!("{file_path}#L{start}-L{end}");
        let header = format!("[TOKENJAR] Lines {start}-{end} of {total} in '{file_path}':\n");

        // Check slicing cache
        let raw_sliced = format!("{header}{}", slice_lines.join("\n"));
        let entry = cache.get(&slice_key, &raw_sliced);
        if entry.status == CacheStatus::Unchanged {
            let orig_tok = (content.len() / 4) as u64;
            let opt_tok = (entry.content.len() / 4) as u64;
            tracker.record_savings("cache", orig_tok, opt_tok);
            let savings_msg = format_savings(&content, &entry.content);
            return format!("{}\n\nToken savings: {savings_msg}", entry.content);
        }

        // Auto-pagination if requested slice is > MAX_OUTPUT_LINES or > MAX_OUTPUT_BYTES
        let (display_slice, paginated) = if !force_full {
            let mut take_count = 0;
            let mut byte_accum = 0;
            for line in slice_lines.iter() {
                if take_count >= MAX_OUTPUT_LINES || (take_count > 0 && byte_accum + line.len() > MAX_OUTPUT_BYTES) {
                    break;
                }
                byte_accum += line.len() + 1;
                take_count += 1;
            }
            if take_count == 0 {
                take_count = 1.min(slice_lines.len());
            }

            if take_count < slice_lines.len() {
                let p_end = start + take_count - 1;
                (
                    &slice_lines[..take_count],
                    Some(format!(
                        "\n\n[TOKENJAR PAGINATION] Showing lines {start}-{p_end} of {total}. (Remaining lines truncated to protect context window)\n\
                         👉 To read the next slice, call read_file_smart with start_line={}, end_line={}.",
                        p_end + 1,
                        (p_end + MAX_OUTPUT_LINES).min(end)
                    )),
                )
            } else {
                (&slice_lines[..], None)
            }
        } else {
            (&slice_lines[..], None)
        };
        let body = display_slice.join("\n");
        let sliced_content = match paginated {
            Some(notice) => format!("{header}{body}{notice}"),
            None => format!("{header}{body}"),
        };

        let orig_tok = (content.len() / 4) as u64;
        let opt_tok = (sliced_content.len() / 4) as u64;
        if orig_tok > opt_tok {
            tracker.record_savings("slice", orig_tok, opt_tok);
            let savings_msg = format_savings(&content, &sliced_content);
            return format!("{sliced_content}\n\nToken savings: {savings_msg}");
        }
        return sliced_content;
    }

    if force_full {
        return content;
    }

    // Lockfile shielding
    if config.is_lockfile(p) {
        let shielded = process_lockfile(p, &content, query);
        let orig_tok = (content.len() / 4) as u64;
        let opt_tok = (shielded.len() / 4) as u64;
        tracker.record_savings("lockfile", orig_tok, opt_tok);
        return shielded;
    }

    // In-memory unified diff session cache
    let entry = cache.get(file_path, &content);

    if entry.status == CacheStatus::FirstRead {
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        let is_lock = config.is_lockfile(p)
            || file_path.ends_with(".lock")
            || file_path.ends_with("package-lock.json");

        let mut take_count = 0;
        let mut byte_accum = 0;
        for line in lines.iter() {
            if take_count >= MAX_OUTPUT_LINES || (take_count > 0 && byte_accum + line.len() > MAX_OUTPUT_BYTES) {
                break;
            }
            byte_accum += line.len() + 1;
            take_count += 1;
        }

        if take_count < total && !force_full && !is_lock {
            let p_end = take_count;
            let slice_lines: Vec<String> = lines[..p_end]
                .iter()
                .enumerate()
                .map(|(idx, line)| format!("{}: {}", idx + 1, line))
                .collect();
            let header =
                format!("[TOKENJAR] Showing lines 1-{p_end} of {total} in '{file_path}':\n");
            let body = slice_lines.join("\n");
            let pagination_notice = format!(
                "\n\n[TOKENJAR PAGINATION] Showing lines 1-{p_end} of {total}. (Remaining lines truncated to protect context window)\n\
                 👉 To read the next slice, call read_file_smart with start_line={}, end_line={} (or pass force_full=true).",
                p_end + 1,
                (p_end + MAX_OUTPUT_LINES).min(total)
            );
            let paginated_content = format!("{header}{body}{pagination_notice}");
            let orig_tok = (content.len() / 4) as u64;
            let opt_tok = (paginated_content.len() / 4) as u64;
            if orig_tok > opt_tok {
                tracker.record_savings("slice", orig_tok, opt_tok);
                let savings_msg = format_savings(&content, &paginated_content);
                return format!("{paginated_content}\n\nToken savings: {savings_msg}");
            }
            return paginated_content;
        }
        return entry.content;
    }

    let savings_msg = format_savings(&content, &entry.content);
    tracker.record_savings(
        "cache",
        entry.original_tokens as u64,
        entry.optimized_tokens as u64,
    );

    format!("{}\n\nToken savings: {savings_msg}", entry.content)
}

/// Returns formatted cache statistics.
pub fn cache_stats(cache: &SessionCache) -> String {
    cache.get_stats().summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_reader_first_and_second_read() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("test.txt");
        std::fs::write(&file_path, "Hello world\n").unwrap();

        let cache = SessionCache::new();
        let config = TokenJarConfig::default();
        let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
        let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

        let p_str = file_path.to_str().unwrap();

        // 1st read -> full content
        let r1 = read_file_smart(
            p_str, false, None, None, None, None, &cache, &config, &tracker,
        );
        assert_eq!(r1, "Hello world\n");

        // 2nd read -> cached hit
        let r2 = read_file_smart(
            p_str, false, None, None, None, None, &cache, &config, &tracker,
        );
        assert!(r2.contains("unchanged since last read"));
        assert!(r2.contains("Token savings:"));
    }

    #[test]
    fn test_smart_reader_line_slicing() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("lines.txt");
        let sample = "line 1\nline 2\nline 3\nline 4\nline 5\n";
        std::fs::write(&file_path, sample).unwrap();

        let cache = SessionCache::new();
        let config = TokenJarConfig::default();
        let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
        let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

        let p_str = file_path.to_str().unwrap();

        // Slice lines 2 to 4
        let r = read_file_smart(
            p_str,
            false,
            None,
            Some(2),
            Some(4),
            None,
            &cache,
            &config,
            &tracker,
        );
        assert!(r.contains("Lines 2-4 of 5"));
        assert!(r.contains("2: line 2"));
        assert!(r.contains("3: line 3"));
        assert!(r.contains("4: line 4"));
        assert!(!r.contains("1: line 1"));
        assert!(!r.contains("5: line 5"));

        // Out-of-bounds start line
        let r_err = read_file_smart(
            p_str,
            false,
            None,
            Some(10),
            Some(12),
            None,
            &cache,
            &config,
            &tracker,
        );
        assert!(r_err.contains("exceeds total line count"));

        // Invalid range (start > end)
        let r_inv = read_file_smart(
            p_str,
            false,
            None,
            Some(4),
            Some(2),
            None,
            &cache,
            &config,
            &tracker,
        );
        assert!(r_inv.contains("Invalid line range"));
    }

    #[test]
    fn test_smart_reader_symbol_slicing() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("service.py");
        let code = r#"class OrderProcessor:
    def __init__(self):
        self.orders = []

    def calculate_discount(self, total: float) -> float:
        if total > 100.0:
            return total * 0.1
        return 0.0

    def process_order(self, order_id: str) -> bool:
        """Processes payment and prints receipts."""
        print(f"Processing order: {order_id}")
        return True

    def calculate_tax(self, amount: float) -> float:
        return amount * 0.20

    def refund_order(self, order_id: str) -> dict:
        return {"status": "REFUNDED", "order_id": order_id}

def standalone_helper():
    return 42
"#;
        std::fs::write(&file_path, code).unwrap();

        let cache = SessionCache::new();
        let config = TokenJarConfig::default();
        let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
        let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

        let p_str = file_path.to_str().unwrap();

        // Extract calculate_discount symbol directly
        let res = read_file_smart(
            p_str,
            false,
            None,
            None,
            None,
            Some("calculate_discount"),
            &cache,
            &config,
            &tracker,
        );

        assert!(res.contains("Symbol 'calculate_discount' found at lines 5-8"));
        assert!(res.contains("def calculate_discount(self, total: float) -> float:"));
        assert!(res.contains("return total * 0.1"));
        assert!(!res.contains("class OrderProcessor"));
        assert!(!res.contains("standalone_helper"));
        assert!(res.contains("Token savings:"));

        // Verify telemetry recorded savings under slice
        let data = tracker.get_data();
        assert!(data.slice.saved > 0);
        assert_eq!(data.slice.count, 1);
    }

    #[test]
    fn test_smart_reader_directory() {
        let temp = tempfile::tempdir().unwrap();
        let cache = SessionCache::new();
        let config = TokenJarConfig::default();
        let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
        let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

        let dir_str = temp.path().to_str().unwrap();
        let r = read_file_smart(
            dir_str, false, None, None, None, None, &cache, &config, &tracker,
        );
        assert!(r.contains("is a directory, not a file"));
        assert!(r.contains("get_directory_tree_tool"));
    }

    #[test]
    fn test_smart_reader_non_utf8() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("latin1.txt");
        // Write byte 0xA9 (copyright in latin1) which is invalid UTF-8
        std::fs::write(&file_path, [b'c', b'o', b'p', b'y', 0xA9]).unwrap();

        let cache = SessionCache::new();
        let config = TokenJarConfig::default();
        let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
        let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

        let p_str = file_path.to_str().unwrap();
        let r = read_file_smart(
            p_str, false, None, None, None, None, &cache, &config, &tracker,
        );
        assert!(r.contains("copy"));
    }

    #[test]
    fn test_smart_reader_slice_caching() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("slice_cache.txt");
        let sample = "line 1\nline 2\nline 3\nline 4\nline 5\n";
        std::fs::write(&file_path, sample).unwrap();

        let cache = SessionCache::new();
        let config = TokenJarConfig::default();
        let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
        let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

        let p_str = file_path.to_str().unwrap();

        // 1st slice read -> extracts lines
        let r1 = read_file_smart(
            p_str,
            false,
            None,
            Some(2),
            Some(4),
            None,
            &cache,
            &config,
            &tracker,
        );
        assert!(r1.contains("Lines 2-4 of 5"));

        // 2nd slice read -> cache hit!
        let r2 = read_file_smart(
            p_str,
            false,
            None,
            Some(2),
            Some(4),
            None,
            &cache,
            &config,
            &tracker,
        );
        assert!(r2.contains("[CACHED]"));
        assert!(r2.contains("unchanged since last read"));
        assert!(r2.contains("Token savings:"));
    }

    #[test]
    fn test_smart_reader_auto_pagination() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("large_file.txt");
        let lines: Vec<String> = (1..=120).map(|i| format!("Line {i}")).collect();
        std::fs::write(&file_path, lines.join("\n")).unwrap();

        let cache = SessionCache::new();
        let config = TokenJarConfig::default();
        let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
        let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

        let p_str = file_path.to_str().unwrap();

        // Normal read -> auto-paginates to 40 lines
        let r = read_file_smart(
            p_str, false, None, None, None, None, &cache, &config, &tracker,
        );
        assert!(r.contains("[TOKENJAR PAGINATION] Showing lines 1-40 of 120"));
        assert!(r.contains("start_line=41, end_line=80"));

        // With force_full=true -> returns all 120 lines without pagination notice
        let r_full = read_file_smart(
            p_str, true, None, None, None, None, &cache, &config, &tracker,
        );
        assert!(!r_full.contains("[TOKENJAR PAGINATION]"));
        assert!(r_full.contains("Line 120"));
    }
}
