//! Global Symbol Index & Blast Radius Reference Search in Rust.
//!
//! Enables instant repository-wide symbol lookup and blast radius analysis
//! without reading dozens of files or triggering context window compaction.

use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

static LAST_INDEX_TIME: LazyLock<Mutex<HashMap<String, Instant>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

use crate::cache::persistent_cache::PersistentCache;
use crate::config::TokenJarConfig;
use crate::models::{IndexedSymbol, ReferenceKind, SymbolReference};
use crate::parser::{detect_language, parse_code, SupportedLanguage};

/// Extracts all top-level and method symbols from source code using Tree-sitter.
pub fn extract_symbols_from_code(
    source_code: &str,
    lang: SupportedLanguage,
    rel_path: &str,
) -> Vec<IndexedSymbol> {
    let tree = match parse_code(source_code, lang) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let mut symbols = Vec::new();
    let lines: Vec<&str> = source_code.lines().collect();
    let source_bytes = source_code.as_bytes();

    fn get_signature(node: tree_sitter::Node, lines: &[&str]) -> String {
        let line_idx = node.start_position().row;
        if line_idx < lines.len() {
            lines[line_idx].trim().to_string()
        } else {
            String::new()
        }
    }

    fn walk(
        node: tree_sitter::Node,
        lines: &[&str],
        source_bytes: &[u8],
        rel_path: &str,
        parent_kind: &str,
        symbols: &mut Vec<IndexedSymbol>,
    ) {
        let node_kind = node.kind();
        let mut kind = None;

        if matches!(
            node_kind,
            "function_definition" | "function_declaration" | "function_item"
        ) {
            kind = Some(if parent_kind == "class" || parent_kind == "impl" {
                "method"
            } else {
                "function"
            });
        } else if matches!(
            node_kind,
            "class_definition" | "class_declaration" | "class" | "class_specifier"
        ) {
            kind = Some("class");
        } else if matches!(node_kind, "method_definition" | "method_declaration") {
            kind = Some("method");
        } else if matches!(
            node_kind,
            "struct_item" | "struct_declaration" | "struct_specifier"
        ) {
            kind = Some("struct");
        } else if matches!(node_kind, "enum_item") {
            kind = Some("enum");
        } else if matches!(node_kind, "interface_declaration") {
            kind = Some("interface");
        }

        if let Some(k) = kind {
            fn get_symbol_name_node(n: tree_sitter::Node) -> Option<tree_sitter::Node> {
                for child in n.children(&mut n.walk()) {
                    if matches!(
                        child.kind(),
                        "identifier"
                            | "type_identifier"
                            | "property_identifier"
                            | "name"
                            | "constant"
                            | "field_identifier"
                    ) {
                        return Some(child);
                    } else if matches!(child.kind(), "function_declarator" | "declarator") {
                        if let Some(nested) = get_symbol_name_node(child) {
                            return Some(nested);
                        }
                    }
                }
                None
            }

            let name_node = get_symbol_name_node(node);

            if let Some(n) = name_node {
                if let Ok(name_str) =
                    std::str::from_utf8(&source_bytes[n.start_byte()..n.end_byte()])
                {
                    let line = node.start_position().row + 1;
                    let signature = get_signature(node, lines);
                    let body_slice = &source_bytes[node.start_byte()..node.end_byte()];
                    let mut hasher = Sha256::new();
                    hasher.update(body_slice);
                    let content_hash = format!("{:x}", hasher.finalize())[..16].to_string();

                    symbols.push(IndexedSymbol {
                        name: name_str.to_string(),
                        kind: k.to_string(),
                        file_path: rel_path.to_string(),
                        line,
                        signature,
                        content_hash,
                    });
                }
            }
        }

        let next_parent_kind = if kind == Some("class") {
            "class"
        } else if node_kind == "impl_item" {
            "impl"
        } else {
            parent_kind
        };

        for child in node.children(&mut node.walk()) {
            walk(
                child,
                lines,
                source_bytes,
                rel_path,
                next_parent_kind,
                symbols,
            );
        }
    }

    walk(
        tree.root_node(),
        &lines,
        source_bytes,
        rel_path,
        "",
        &mut symbols,
    );
    symbols
}

/// Extracts all usages, calls, and imports of target_symbol in source code.
pub fn extract_references_from_code(
    source_code: &str,
    lang: SupportedLanguage,
    rel_path: &str,
    target_symbol: &str,
    def_locations: &HashSet<(String, usize)>,
) -> Vec<SymbolReference> {
    if !source_code.contains(target_symbol) {
        return Vec::new();
    }

    let lines: Vec<&str> = source_code.lines().collect();
    let mut refs = Vec::new();
    let source_bytes = source_code.as_bytes();

    if let Some(tree) = parse_code(source_code, lang) {
        fn walk(
            node: tree_sitter::Node,
            lines: &[&str],
            source_bytes: &[u8],
            rel_path: &str,
            target_symbol: &str,
            def_locations: &HashSet<(String, usize)>,
            refs: &mut Vec<SymbolReference>,
        ) {
            let kind = node.kind();
            if matches!(
                kind,
                "identifier"
                    | "type_identifier"
                    | "property_identifier"
                    | "name"
                    | "field_identifier"
            ) {
                if let Ok(text) =
                    std::str::from_utf8(&source_bytes[node.start_byte()..node.end_byte()])
                {
                    if text == target_symbol {
                        let line_no = node.start_position().row + 1;
                        if !def_locations.contains(&(rel_path.to_string(), line_no)) {
                            let mut ref_kind = ReferenceKind::Usage;
                            let mut parent = node.parent();
                            while let Some(p) = parent {
                                let ptype = p.kind();
                                if matches!(
                                    ptype,
                                    "call_expression" | "call" | "invocation_expression"
                                ) {
                                    ref_kind = ReferenceKind::Call;
                                    break;
                                } else if matches!(
                                    ptype,
                                    "import_statement"
                                        | "import_from_statement"
                                        | "import_specifier"
                                        | "use_declaration"
                                        | "using_directive"
                                ) {
                                    ref_kind = ReferenceKind::Import;
                                    break;
                                } else if matches!(
                                    ptype,
                                    "class_inheritance"
                                        | "extends_clause"
                                        | "implements_clause"
                                        | "base_class_clause"
                                ) {
                                    ref_kind = ReferenceKind::Inheritance;
                                    break;
                                }
                                parent = p.parent();
                            }

                            let snippet = if line_no > 0 && line_no <= lines.len() {
                                lines[line_no - 1].trim().to_string()
                            } else {
                                String::new()
                            };

                            refs.push(SymbolReference {
                                symbol_name: target_symbol.to_string(),
                                file_path: rel_path.to_string(),
                                line: line_no,
                                kind: ref_kind,
                                snippet,
                            });
                        }
                    }
                }
            }

            for child in node.children(&mut node.walk()) {
                walk(
                    child,
                    lines,
                    source_bytes,
                    rel_path,
                    target_symbol,
                    def_locations,
                    refs,
                );
            }
        }

        walk(
            tree.root_node(),
            &lines,
            source_bytes,
            rel_path,
            target_symbol,
            def_locations,
            &mut refs,
        );
    } else {
        // Fallback line scan
        for (idx, line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            if def_locations.contains(&(rel_path.to_string(), line_no)) {
                continue;
            }
            if line.contains(target_symbol) {
                let trimmed = line.trim();
                let ref_kind = if trimmed.starts_with("import ") || trimmed.starts_with("use ") {
                    ReferenceKind::Import
                } else if trimmed.contains(&format!("{target_symbol}(")) {
                    ReferenceKind::Call
                } else {
                    ReferenceKind::Usage
                };

                refs.push(SymbolReference {
                    symbol_name: target_symbol.to_string(),
                    file_path: rel_path.to_string(),
                    line: line_no,
                    kind: ref_kind,
                    snippet: trimmed.to_string(),
                });
            }
        }
    }

    // Deduplicate references on identical (file_path, line)
    let mut seen = HashSet::new();
    let mut unique_refs = Vec::new();
    for r in refs {
        if seen.insert((r.file_path.clone(), r.line)) {
            unique_refs.push(r);
        }
    }

    unique_refs
}

pub fn clean_canonical_root(root_path: &Path) -> (std::path::PathBuf, String) {
    let canonical = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.to_path_buf());
    let raw_str = canonical.to_string_lossy();
    let stripped = raw_str.strip_prefix(r"\\?\").unwrap_or(&raw_str);
    let normalized = stripped.replace('\\', "/");
    (canonical, normalized)
}

fn is_skip_dir(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    name.starts_with('.') || crate::repo_map::ALWAYS_SKIP_DIRS.contains(&name.as_ref())
}

/// Recursively scans and indexes the repository incrementally into SQLite.
pub fn index_repository(root_path: &Path) -> Vec<IndexedSymbol> {
    let (root, root_str) = clean_canonical_root(root_path);

    {
        if let Ok(mut last_map) = LAST_INDEX_TIME.lock() {
            if let Some(last_time) = last_map.get(&root_str) {
                if last_time.elapsed() < Duration::from_secs(3) {
                    return Vec::new();
                }
            }
            last_map.insert(root_str.clone(), Instant::now());
        }
    }

    let config = TokenJarConfig::load_from_dir(&root);
    let cache = match PersistentCache::new() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let cached_files = cache.get_all_file_metas(&root_str).unwrap_or_default();
    let mut all_symbols = Vec::new();
    let mut file_count = 0;

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| !is_skip_dir(e))
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let p = entry.path();
        let rel_path = match p.strip_prefix(&root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => p.to_string_lossy().replace('\\', "/"),
        };

        if config.is_ignored(Path::new(&rel_path)) {
            continue;
        }

        let lang = match detect_language(p) {
            Some(l) => l,
            None => continue,
        };

        file_count += 1;
        if file_count > config.max_source_files {
            break;
        }

        let mtime = std::fs::metadata(p)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64()
            })
            .unwrap_or(0.0);

        // Incremental cache check: skip file if mtime is unchanged
        if let Some((_, cached_mtime)) = cached_files.get(&rel_path) {
            if (cached_mtime - mtime).abs() < 0.001 {
                continue;
            }
        }

        let content = match std::fs::read(p) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
            },
            Err(_) => continue,
        };

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let file_hash = format!("{:x}", hasher.finalize())[..16].to_string();

        let symbols = extract_symbols_from_code(&content, lang, &rel_path);
        let _ = cache.set_file_symbols(&root_str, &rel_path, &file_hash, mtime, &symbols);
        all_symbols.extend(symbols);
    }

    all_symbols
}

/// Computes similarity score between a search query and a symbol name [0.0 - 1.0].
/// Combines case-insensitive exact matching, separator-stripped matching (snake/camel normalization),
/// normalized Levenshtein distance, substring overlap, and trigram/dice coefficient.
pub fn compute_similarity(q: &str, target: &str) -> f64 {
    let q_lower = q.to_lowercase();
    let t_lower = target.to_lowercase();
    if q_lower == t_lower {
        return 1.0;
    }

    let q_clean: String = q_lower
        .chars()
        .filter(|c| *c != '_' && *c != '-' && !c.is_whitespace())
        .collect();
    let t_clean: String = t_lower
        .chars()
        .filter(|c| *c != '_' && *c != '-' && !c.is_whitespace())
        .collect();
    if q_clean == t_clean {
        return 0.98;
    }

    let mut score: f64 = 0.0;

    // Substring containment on normalized clean strings
    if !q_clean.is_empty()
        && !t_clean.is_empty()
        && (t_clean.contains(&q_clean) || q_clean.contains(&t_clean))
    {
        let ratio =
            (q_clean.len().min(t_clean.len()) as f64) / (q_clean.len().max(t_clean.len()) as f64);
        if ratio >= 0.4 {
            score = score.max(0.75 + 0.20 * ratio);
        }
    }

    // Levenshtein similarity on lowercase characters
    let a_chars: Vec<char> = q_lower.chars().collect();
    let b_chars: Vec<char> = t_lower.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();
    if m > 0 && n > 0 {
        let mut prev: Vec<usize> = (0..=n).collect();
        let mut curr = vec![0; n + 1];

        for i in 1..=m {
            curr[0] = i;
            for j in 1..=n {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            }
            prev.copy_from_slice(&curr);
        }
        let dist = prev[n];
        let max_len = m.max(n);
        let lev_sim = 1.0 - (dist as f64 / max_len as f64);
        score = score.max(lev_sim);
    }

    // Levenshtein similarity on cleaned characters (snake_case to CamelCase tolerance)
    let ac_chars: Vec<char> = q_clean.chars().collect();
    let bc_chars: Vec<char> = t_clean.chars().collect();
    let mc = ac_chars.len();
    let nc = bc_chars.len();
    if mc > 0 && nc > 0 {
        let mut prev: Vec<usize> = (0..=nc).collect();
        let mut curr = vec![0; nc + 1];

        for i in 1..=mc {
            curr[0] = i;
            for j in 1..=nc {
                let cost = if ac_chars[i - 1] == bc_chars[j - 1] {
                    0
                } else {
                    1
                };
                curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            }
            prev.copy_from_slice(&curr);
        }
        let dist = prev[nc];
        let max_len = mc.max(nc);
        let lev_clean_sim = 1.0 - (dist as f64 / max_len as f64);
        score = score.max(lev_clean_sim);
    }

    // Trigram similarity if lengths >= 3
    if m >= 3 && n >= 3 {
        let q_trigrams: HashSet<&[char]> = a_chars.windows(3).collect();
        let t_trigrams: HashSet<&[char]> = b_chars.windows(3).collect();
        let matches = q_trigrams.intersection(&t_trigrams).count();
        let total = q_trigrams.len() + t_trigrams.len();
        if total > 0 {
            let trigram_sim = (2.0 * matches as f64) / (total as f64);
            score = score.max(trigram_sim);
        }
    }

    score
}

/// Searches for code symbols (classes, functions, methods, structs) across the repository.
pub fn find_symbol_global(
    query: &str,
    root_path: &Path,
    exact: bool,
    max_results: usize,
) -> String {
    let q = query.trim();
    if q.is_empty() {
        return "Error: Empty query provided.".to_string();
    }

    let (root, root_str) = clean_canonical_root(root_path);

    // Ensure index is populated
    let _ = index_repository(&root);

    let cache = match PersistentCache::new() {
        Ok(c) => c,
        Err(e) => return format!("Cache Error: {e}"),
    };

    let matches = match cache.search_symbols(&root_str, q, exact, max_results) {
        Ok(m) => m,
        Err(e) => return format!("Search Error: {e}"),
    };

    if matches.is_empty() {
        // Fallback: Hybrid fuzzy / trigram search across repository symbols
        if let Ok(all_symbols) = cache.get_all_symbols(&root_str) {
            let mut candidates: Vec<(IndexedSymbol, f64)> = all_symbols
                .into_iter()
                .filter_map(|s| {
                    let score = compute_similarity(q, &s.name);
                    if score >= 0.50 {
                        Some((s, score))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by score descending, then by file_path and line
            candidates.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.file_path.cmp(&b.0.file_path))
                    .then_with(|| a.0.line.cmp(&b.0.line))
            });

            // Deduplicate same name in same file
            let mut seen = HashSet::new();
            candidates
                .retain(|(s, _)| seen.insert(format!("{}:{}:{}", s.file_path, s.name, s.line)));

            if !candidates.is_empty() {
                let display_candidates = &candidates[..candidates.len().min(max_results)];
                let mut lines = Vec::new();
                lines.push(format!(
                    "[SYMBOLS] No exact match for '{query}'. Did you mean one of these symbols?"
                ));
                lines.push("----------------------------------------".to_string());
                for (i, (m, score)) in display_candidates.iter().enumerate() {
                    let pct = (score * 100.0).round() as usize;
                    lines.push(format!(
                        "{}. [{}] {} -> {}:{} ({}% match)",
                        i + 1,
                        m.kind.to_uppercase(),
                        m.name,
                        m.file_path,
                        m.line,
                        pct
                    ));
                    if !m.signature.is_empty() {
                        lines.push(format!("   Signature: {}", m.signature));
                    }
                }
                return lines.join("\n");
            }
        }

        return format!("No symbols found matching '{query}' across the codebase.");
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "[SYMBOLS] Found {} symbol(s) matching '{query}':",
        matches.len()
    ));
    lines.push("----------------------------------------".to_string());

    for (i, m) in matches.iter().enumerate() {
        lines.push(format!(
            "{}. [{}] {} -> {}:{}",
            i + 1,
            m.kind.to_uppercase(),
            m.name,
            m.file_path,
            m.line
        ));
        if !m.signature.is_empty() {
            lines.push(format!("   Signature: {}", m.signature));
        }
    }

    lines.join("\n")
}

/// Finds all usages, calls, and imports of a symbol across the entire codebase.
pub fn find_symbol_references(symbol_name: &str, root_path: &Path, max_results: usize) -> String {
    let sym = symbol_name.trim();
    if sym.is_empty() {
        return "Error: Empty symbol_name provided.".to_string();
    }

    let (root, root_str) = clean_canonical_root(root_path);
    let _ = index_repository(&root);

    let cache = match PersistentCache::new() {
        Ok(c) => c,
        Err(e) => return format!("Cache Error: {e}"),
    };

    let cached_defs = cache
        .search_symbols(&root_str, sym, true, 10)
        .unwrap_or_default();
    let def_locations: HashSet<(String, usize)> = cached_defs
        .iter()
        .map(|d| (d.file_path.clone(), d.line))
        .collect();

    let config = TokenJarConfig::load_from_dir(&root);
    let mut all_refs = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| !is_skip_dir(e))
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let p = entry.path();
        let rel_path = match p.strip_prefix(&root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => p.to_string_lossy().replace('\\', "/"),
        };

        if config.is_ignored(Path::new(&rel_path)) {
            continue;
        }

        let lang = match detect_language(p) {
            Some(l) => l,
            None => continue,
        };

        if let Ok(meta) = std::fs::metadata(p) {
            if meta.len() as usize > config.max_cacheable_bytes {
                continue;
            }
        }

        let bytes = match std::fs::read(p) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let needle = sym.as_bytes();
        if needle.is_empty() || !bytes.windows(needle.len()).any(|w| w == needle) {
            continue;
        }

        let content = String::from_utf8_lossy(&bytes);
        let refs = extract_references_from_code(&content, lang, &rel_path, sym, &def_locations);
        all_refs.extend(refs);
    }

    if all_refs.is_empty() && cached_defs.is_empty() {
        return format!(
            "No references or definitions found for '{symbol_name}' across the codebase."
        );
    }

    let mut lines = Vec::new();
    lines.push(format!("[REFERENCES] Blast Radius Analysis for '{sym}':"));
    if !cached_defs.is_empty() {
        let def_strs: Vec<String> = cached_defs
            .iter()
            .take(3)
            .map(|d| format!("{}:{} ({})", d.file_path, d.line, d.kind))
            .collect();
        lines.push(format!("• Defined at: {}", def_strs.join(", ")));
    } else {
        lines.push("• Defined at: External / Unindexed symbol".to_string());
    }

    let unique_files: HashSet<&str> = all_refs.iter().map(|r| r.file_path.as_str()).collect();
    lines.push(format!(
        "• Total Usages: {} reference(s) found across {} file(s):",
        all_refs.len(),
        unique_files.len()
    ));
    lines.push("-".repeat(60));

    for (i, r) in all_refs.iter().take(max_results).enumerate() {
        lines.push(format!(
            "{}. [{}] {}:{}",
            i + 1,
            r.kind,
            r.file_path,
            r.line
        ));
        if !r.snippet.is_empty() {
            lines.push(format!("   Line {}: {}", r.line, r.snippet));
        }
    }

    if all_refs.len() > max_results {
        lines.push(format!(
            "\n... and {} more reference(s) omitted.",
            all_refs.len() - max_results
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_symbols_rust() {
        let code = r#"
pub struct User {
    pub name: String,
}

impl User {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

pub fn greet() {
    println!("Hello");
}
"#;
        let syms = extract_symbols_from_code(code, SupportedLanguage::Rust, "src/user.rs");
        assert!(syms.iter().any(|s| s.name == "User" && s.kind == "struct"));
        assert!(syms.iter().any(|s| s.name == "new" && s.kind == "method"));
        assert!(syms
            .iter()
            .any(|s| s.name == "greet" && s.kind == "function"));
    }

    #[test]
    fn test_extract_references_python() {
        let code = r#"
from auth import login_user

def process_login(user):
    token = login_user(user)
    return token
"#;
        let def_locs = HashSet::new();
        let refs = extract_references_from_code(
            code,
            SupportedLanguage::Python,
            "app.py",
            "login_user",
            &def_locs,
        );
        assert_eq!(refs.len(), 2);
        assert!(refs.iter().any(|r| r.kind == ReferenceKind::Import));
        assert!(refs.iter().any(|r| r.kind == ReferenceKind::Call));
    }

    #[test]
    fn test_extract_symbols_go_cpp_java() {
        let go_code = r#"
package main
type Server struct{}
func (s *Server) Start() error { return nil }
func NewServer() *Server { return &Server{} }
"#;
        let syms_go = extract_symbols_from_code(go_code, SupportedLanguage::Go, "main.go");
        assert!(syms_go
            .iter()
            .any(|s| s.name == "Start" && s.kind == "method"));
        assert!(syms_go
            .iter()
            .any(|s| s.name == "NewServer" && s.kind == "function"));

        let cpp_code = r#"
class DatabaseConnection {
public:
    void connect() {}
};
int query(int q) { return q; }
"#;
        let syms_cpp = extract_symbols_from_code(cpp_code, SupportedLanguage::Cpp, "db.cpp");
        assert!(syms_cpp
            .iter()
            .any(|s| s.name == "DatabaseConnection" && s.kind == "class"));
        assert!(syms_cpp
            .iter()
            .any(|s| s.name == "connect" && s.kind == "method"));
        assert!(syms_cpp
            .iter()
            .any(|s| s.name == "query" && s.kind == "function"));

        let java_code = r#"
package com.example;
public class AuthProvider {
    public boolean verifyToken(String token) {
        return true;
    }
}
"#;
        let syms_java =
            extract_symbols_from_code(java_code, SupportedLanguage::Java, "AuthProvider.java");
        assert!(syms_java
            .iter()
            .any(|s| s.name == "AuthProvider" && s.kind == "class"));
        assert!(syms_java
            .iter()
            .any(|s| s.name == "verifyToken" && s.kind == "method"));
    }

    #[test]
    fn test_find_symbol_global_repo() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("service.rs");
        std::fs::write(
            &file,
            "pub struct PaymentService;\nimpl PaymentService {\n    pub fn pay() {}\n}\n",
        )
        .unwrap();

        let result = find_symbol_global("PaymentService", temp.path(), false, 10);
        assert!(result.contains("PaymentService"));
        assert!(result.contains("service.rs"));
    }

    #[test]
    fn test_compute_similarity() {
        assert!((compute_similarity("OrderProcessor", "OrderProcessor") - 1.0).abs() < 1e-6);
        assert!(compute_similarity("order_processor", "OrderProcessor") >= 0.95);
        assert!(compute_similarity("OrderProcesor", "OrderProcessor") >= 0.85);
        assert!(compute_similarity("validate_tokn", "validate_token") >= 0.85);
        assert!(compute_similarity("something_completely_different", "OrderProcessor") < 0.40);
    }

    #[test]
    fn test_find_symbol_global_fuzzy_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("billing.rs");
        std::fs::write(
            &file,
            "pub struct OrderProcessor;\nimpl OrderProcessor {\n    pub fn process_order() {}\n}\n",
        )
        .unwrap();

        // Exact typo that fails substring search
        let typo_result = find_symbol_global("OrderProcesor", temp.path(), false, 10);
        assert!(typo_result.contains("Did you mean one of these symbols?"));
        assert!(typo_result.contains("OrderProcessor"));
        assert!(typo_result.contains("billing.rs"));

        // Non existent symbol with zero similarity
        let nonexistent = find_symbol_global("XyzZqwUnknown", temp.path(), false, 10);
        assert!(
            nonexistent.contains("No symbols found matching 'XyzZqwUnknown' across the codebase.")
        );
    }
}
