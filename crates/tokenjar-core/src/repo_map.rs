//! Repository Map & Directory Tree Engine in Rust.
//!
//! Generates structural codebase overviews ranked by call-graph centrality,
//! fitting into strict token budgets (e.g. 1000 tokens) to orient AI agents
//! at the start of any coding session without context bloat.

use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::TokenJarConfig;
use crate::models::IndexedSymbol;
use crate::parser::{detect_language, parse_code, SupportedLanguage};
use crate::symbols::extract_symbols_from_code;
use crate::token_counter::estimate_tokens;

pub const ALWAYS_SKIP_DIRS: &[&str] = &[
    "target", "node_modules", "__pycache__", ".git", "data", "logs",
    "dist", "build", ".venv", "venv", "env", ".cache", ".pytest_cache",
    ".ruff_cache", ".mypy_cache", ".idea", ".vscode", ".vs", "coverage",
    "htmlcov", ".tox",
];

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub rel_path: String,
    pub language: &'static str,
    pub symbols: Vec<IndexedSymbol>,
    pub import_count: usize,
    pub score: f64,
}

/// Counts imports/uses in AST for graph ranking.
pub fn extract_import_count(source_code: &str, lang: SupportedLanguage) -> usize {
    let tree = match parse_code(source_code, lang) {
        Some(t) => t,
        None => return 0,
    };

    let mut count = 0;
    fn walk(node: tree_sitter::Node, count: &mut usize) {
        let kind = node.kind();
        if matches!(
            kind,
            "import_statement"
                | "import_from_statement"
                | "import_declaration"
                | "use_declaration"
                | "using_directive"
                | "preproc_include"
        ) {
            *count += 1;
        }

        for child in node.children(&mut node.walk()) {
            walk(child, count);
        }
    }

    walk(tree.root_node(), &mut count);
    count
}

fn is_skip_dir(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    name.starts_with('.')
        || name == "target"
        || name == "node_modules"
        || name == "__pycache__"
        || name == "venv"
        || name == ".venv"
        || name == "dist"
        || name == "build"
}

/// Generates a structural map of the entire repository fitted to a token budget.
pub fn get_repo_map(root_path: &Path, max_tokens: usize, focus_files: &[String]) -> String {
    let root = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.to_path_buf());
    let config = TokenJarConfig::load_from_dir(&root);

    #[allow(clippy::type_complexity)]
    let mut raw_files: Vec<(
        String,
        String,
        &'static str,
        usize,
        Vec<IndexedSymbol>,
        usize,
    )> = Vec::new();
    let mut symbol_to_file: HashMap<String, String> = HashMap::new();

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

        let content = match std::fs::read(p) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
            },
            Err(_) => continue,
        };

        let line_count = content.lines().count();
        let symbols = extract_symbols_from_code(&content, lang, &rel_path);
        let import_count = extract_import_count(&content, lang);

        for sym in &symbols {
            if sym.name.len() > 2 && sym.name != "main" && sym.name != "init" {
                symbol_to_file.insert(sym.name.clone(), rel_path.clone());
            }
        }

        raw_files.push((
            rel_path,
            content,
            lang.as_str(),
            line_count,
            symbols,
            import_count,
        ));

        if raw_files.len() >= config.max_source_files {
            break;
        }
    }

    if raw_files.is_empty() {
        return format!(
            "📁 Repository Map (0 files, budget: {max_tokens} tokens)\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nNo supported source files found."
        );
    }

    // Call-graph centrality calculation
    let mut inbound_references: HashMap<String, usize> = HashMap::new();
    for (rel_path, content, _, _, _, _) in &raw_files {
        for (sym_name, def_file) in &symbol_to_file {
            if def_file != rel_path && content.contains(sym_name) {
                *inbound_references.entry(def_file.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut file_infos = Vec::new();
    for (rel_path, _, lang_str, line_count, symbols, import_count) in raw_files {
        let top_level_symbols = symbols.len();
        let base_score = (top_level_symbols * 10) as f64;
        let import_score = (import_count * 5) as f64;
        let focus_boost = if focus_files.contains(&rel_path) {
            100.0
        } else {
            0.0
        };
        let length_penalty = -0.001 * (line_count as f64);
        let call_graph_centrality =
            (inbound_references.get(&rel_path).copied().unwrap_or(0) * 12) as f64;

        let score =
            base_score + import_score + focus_boost + length_penalty + call_graph_centrality;

        file_infos.push(FileInfo {
            rel_path,
            language: lang_str,
            symbols,
            import_count,
            score,
        });
    }

    // Sort descending by centrality score
    file_infos.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut output_lines = Vec::new();
    output_lines.push(format!(
        "📁 Repository Map ({} files, budget: {} tokens)",
        file_infos.len(),
        max_tokens
    ));
    output_lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());

    let mut current_tokens = estimate_tokens(&output_lines.join("\n"));
    let mut included_count = 0;

    #[allow(clippy::explicit_counter_loop)]
    for f in &file_infos {
        let mut file_lines = Vec::new();
        file_lines.push(format!("{}:", f.rel_path));

        for sym in &f.symbols {
            file_lines.push(format!("  [{}] {}", sym.kind, sym.signature));
        }

        if f.symbols.is_empty() {
            file_lines.push("  (no symbols detected)".to_string());
        }

        let file_text = file_lines.join("\n") + "\n";
        let file_tokens = estimate_tokens(&file_text);

        if current_tokens + file_tokens > max_tokens {
            if included_count > 0 {
                output_lines.push(format!(
                    "\n... (truncating remaining {} files to respect token budget)",
                    file_infos.len() - included_count
                ));
            } else {
                output_lines
                    .push("\n... (budget too small to include even the top file)".to_string());
            }
            break;
        }

        output_lines.push(file_text);
        included_count += 1;
        current_tokens += file_tokens;
    }

    output_lines.join("\n")
}

/// Lightweight directory tree generator honoring ignore patterns.
pub fn get_directory_tree(root_path: &Path, max_depth: usize) -> String {
    let root = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.to_path_buf());
    if !root.exists() || !root.is_dir() {
        return format!("Error: Directory {} does not exist.", root_path.display());
    }

    let config = TokenJarConfig::load_from_dir(&root);
    let mut lines = Vec::new();

    fn walk_dir(
        dir: &Path,
        depth: usize,
        max_depth: usize,
        prefix: &str,
        config: &TokenJarConfig,
        lines: &mut Vec<String>,
    ) {
        if lines.len() >= 60 {
            return;
        }

        if depth > max_depth {
            lines.push(format!("{prefix}..."));
            return;
        }

        let mut entries = match std::fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect::<Vec<_>>(),
            Err(_) => {
                lines.push(format!("{prefix}<Permission Denied>"));
                return;
            }
        };

        // Sort dirs first, then files alphabetically
        entries.sort_by_key(|e| {
            let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            (!is_dir, e.file_name().to_string_lossy().to_lowercase())
        });

        // Filter ignored
        let filtered: Vec<_> = entries
            .into_iter()
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') && name != ".gitignore" && name != ".env.example" {
                    return false;
                }
                if ALWAYS_SKIP_DIRS.contains(&name.as_str()) {
                    return false;
                }
                !config.is_ignored(&e.path())
            })
            .collect();

        let total_count = filtered.len();
        let (display_items, truncated_count) = if total_count > 15 {
            (&filtered[..10], total_count - 10)
        } else {
            (&filtered[..], 0)
        };

        let count = display_items.len();
        for (i, entry) in display_items.iter().enumerate() {
            if lines.len() >= 60 {
                break;
            }
            let is_last = (i == count - 1) && (truncated_count == 0);
            let connector = if is_last { "└── " } else { "├── " };
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            if is_dir {
                lines.push(format!("{prefix}{connector}{file_name}/"));
                let ext_prefix = if is_last { "    " } else { "│   " };
                walk_dir(
                    &entry.path(),
                    depth + 1,
                    max_depth,
                    &format!("{prefix}{ext_prefix}"),
                    config,
                    lines,
                );
            } else {
                lines.push(format!("{prefix}{connector}{file_name}"));
            }
        }

        if truncated_count > 0 && lines.len() < 60 {
            lines.push(format!("{prefix}└── ... (+{truncated_count} more items)"));
        }
    }

    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "root".to_string());
    lines.push(format!("{root_name}/"));
    walk_dir(&root, 1, max_depth, "", &config, &mut lines);

    if lines.len() >= 60 {
        lines.push("\n... [TokenJar Guard: Directory tree capped at 60 entries. Use get_directory_tree_tool with subpath or smaller max_depth] ...".to_string());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_tree() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(temp.path().join("README.md"), "# Hello\n").unwrap();

        let tree = get_directory_tree(temp.path(), 3);
        assert!(tree.contains("src/"));
        assert!(tree.contains("main.rs"));
        assert!(tree.contains("README.md"));
    }

    #[test]
    fn test_repo_map_budget() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("lib.rs"),
            "pub fn calculate() -> i32 { 42 }\npub fn render() { println!(\"render\"); }\n",
        )
        .unwrap();

        let map = get_repo_map(temp.path(), 500, &[]);
        assert!(map.contains("📁 Repository Map"));
        assert!(map.contains("src/lib.rs"));
        assert!(map.contains("calculate"));
    }
}
