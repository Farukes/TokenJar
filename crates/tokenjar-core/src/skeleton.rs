//! AST Skeletonizer & Symbol Extractor in Rust.
//!
//! Replaces function and method bodies with '...' to extract structural APIs,
//! saving 70-90% of tokens when exploring source files.

use crate::parser::{detect_language, parse_code, SupportedLanguage};
use crate::token_counter::estimate_tokens;
use std::path::Path;

#[derive(Debug, Clone)]
struct ReplaceRange {
    start_byte: usize,
    end_byte: usize,
    replacement: &'static str,
}

/// Builds a skeleton of source code by replacing function/method bodies with '...'.
pub fn build_skeleton(source_code: &str, lang: SupportedLanguage) -> String {
    let tree = match parse_code(source_code, lang) {
        Some(t) => t,
        None => return source_code.to_string(),
    };

    let root_node = tree.root_node();
    let mut replace_ranges: Vec<ReplaceRange> = Vec::new();

    fn walk(node: tree_sitter::Node, lang: SupportedLanguage, ranges: &mut Vec<ReplaceRange>) {
        let node_kind = node.kind();

        match lang {
            SupportedLanguage::Python => {
                if node_kind == "function_definition" {
                    let mut body = None;
                    for child in node.children(&mut node.walk()) {
                        if child.kind() == "block" {
                            body = Some(child);
                            break;
                        }
                    }

                    if let Some(b) = body {
                        let mut start_byte = b.start_byte();
                        // Preserve Python docstring if present as first statement
                        if b.child_count() > 0 {
                            let first_stmt = b.child(0);
                            if let Some(stmt) = first_stmt {
                                if stmt.kind() == "expression_statement" && stmt.child_count() > 0 {
                                    if let Some(expr) = stmt.child(0) {
                                        if expr.kind() == "string" {
                                            start_byte = stmt.end_byte();
                                        }
                                    }
                                }
                            }
                        }

                        if start_byte < b.end_byte() {
                            ranges.push(ReplaceRange {
                                start_byte,
                                end_byte: b.end_byte(),
                                replacement: " ...\n",
                            });
                        }
                    }
                }
            }
            SupportedLanguage::Rust => {
                if node_kind == "function_item" {
                    let mut body = None;
                    for child in node.children(&mut node.walk()) {
                        if child.kind() == "block" {
                            body = Some(child);
                            break;
                        }
                    }
                    if let Some(b) = body {
                        if b.start_byte() + 1 < b.end_byte() {
                            ranges.push(ReplaceRange {
                                start_byte: b.start_byte() + 1,
                                end_byte: b.end_byte() - 1,
                                replacement: "\n    ...\n",
                            });
                        }
                    }
                }
            }
            SupportedLanguage::JavaScript
            | SupportedLanguage::TypeScript
            | SupportedLanguage::C
            | SupportedLanguage::Cpp
            | SupportedLanguage::Java
            | SupportedLanguage::CSharp
            | SupportedLanguage::Php
            | SupportedLanguage::Bash => {
                if matches!(
                    node_kind,
                    "function_declaration"
                        | "method_definition"
                        | "arrow_function"
                        | "method_declaration"
                        | "function_definition"
                ) {
                    let mut body = None;
                    for child in node.children(&mut node.walk()) {
                        if matches!(
                            child.kind(),
                            "statement_block" | "compound_statement" | "block"
                        ) {
                            body = Some(child);
                            break;
                        }
                    }
                    if let Some(b) = body {
                        if b.start_byte() + 1 < b.end_byte() {
                            ranges.push(ReplaceRange {
                                start_byte: b.start_byte() + 1,
                                end_byte: b.end_byte() - 1,
                                replacement: "\n  ...\n",
                            });
                        }
                    }
                }
            }
            SupportedLanguage::Ruby => {
                if matches!(node_kind, "method" | "singleton_method") {
                    let mut body = None;
                    for child in node.children(&mut node.walk()) {
                        if child.kind() == "body_statement" {
                            body = Some(child);
                            break;
                        }
                    }
                    if let Some(b) = body {
                        if b.start_byte() < b.end_byte() {
                            ranges.push(ReplaceRange {
                                start_byte: b.start_byte(),
                                end_byte: b.end_byte(),
                                replacement: "\n    ...\n  ",
                            });
                        }
                    }
                }
            }
            SupportedLanguage::Html | SupportedLanguage::Css | SupportedLanguage::Json => {}
            SupportedLanguage::Go => {
                if matches!(node_kind, "function_declaration" | "method_declaration") {
                    let mut body = None;
                    for child in node.children(&mut node.walk()) {
                        if child.kind() == "block" {
                            body = Some(child);
                            break;
                        }
                    }
                    if let Some(b) = body {
                        if b.start_byte() + 1 < b.end_byte() {
                            ranges.push(ReplaceRange {
                                start_byte: b.start_byte() + 1,
                                end_byte: b.end_byte() - 1,
                                replacement: "\n\t...\n",
                            });
                        }
                    }
                }
            }
        }

        // Recursively traverse children
        for child in node.children(&mut node.walk()) {
            walk(child, lang, ranges);
        }
    }

    walk(root_node, lang, &mut replace_ranges);

    if replace_ranges.is_empty() {
        return source_code.to_string();
    }

    // Sort by start_byte
    replace_ranges.sort_by_key(|r| r.start_byte);

    // Resolve overlapping ranges (keep outermost)
    let mut filtered_ranges: Vec<ReplaceRange> = Vec::new();
    for r in replace_ranges {
        if let Some(last) = filtered_ranges.last() {
            if r.start_byte < last.end_byte {
                continue;
            }
        }
        filtered_ranges.push(r);
    }

    // Splice string bytes
    let source_bytes = source_code.as_bytes();
    let mut result_bytes = Vec::new();
    let mut last_end = 0;

    for r in filtered_ranges {
        if r.start_byte >= last_end && r.end_byte <= source_bytes.len() {
            result_bytes.extend_from_slice(&source_bytes[last_end..r.start_byte]);
            result_bytes.extend_from_slice(r.replacement.as_bytes());
            last_end = r.end_byte;
        }
    }

    if last_end < source_bytes.len() {
        result_bytes.extend_from_slice(&source_bytes[last_end..]);
    }

    String::from_utf8(result_bytes).unwrap_or_else(|_| source_code.to_string())
}

/// Finds the full source implementation of a symbol (function, method, class, struct).
pub fn find_symbol_in_code(
    source_code: &str,
    lang: SupportedLanguage,
    symbol_name: &str,
) -> Option<String> {
    find_symbol_range_in_code(source_code, lang, symbol_name).map(|(_, _, s)| s)
}

/// Finds a specific function, class, or method by name, returning (start_line, end_line, implementation).
pub fn find_symbol_range_in_code(
    source_code: &str,
    lang: SupportedLanguage,
    symbol_name: &str,
) -> Option<(usize, usize, String)> {
    let tree = parse_code(source_code, lang)?;
    let source_bytes = source_code.as_bytes();

    fn get_node_name<'a>(node: tree_sitter::Node<'a>, source_bytes: &'a [u8]) -> Option<&'a str> {
        for child in node.children(&mut node.walk()) {
            if matches!(
                child.kind(),
                "identifier"
                    | "type_identifier"
                    | "property_identifier"
                    | "name"
                    | "constant"
                    | "field_identifier"
            ) {
                if let Ok(name_str) =
                    std::str::from_utf8(&source_bytes[child.start_byte()..child.end_byte()])
                {
                    return Some(name_str);
                }
            } else if matches!(child.kind(), "function_declarator" | "declarator") {
                if let Some(nested) = get_node_name(child, source_bytes) {
                    return Some(nested);
                }
            }
        }
        None
    }

    fn walk(
        node: tree_sitter::Node,
        source_bytes: &[u8],
        target_name: &str,
    ) -> Option<(usize, usize, String)> {
        let node_kind = node.kind();
        let is_target_def = matches!(
            node_kind,
            "function_definition"
                | "class_definition"
                | "function_declaration"
                | "class_declaration"
                | "class_specifier"
                | "struct_specifier"
                | "method_definition"
                | "method_declaration"
                | "function_item"
                | "struct_item"
                | "enum_item"
                | "impl_item"
                | "interface_declaration"
        );

        if is_target_def {
            if let Some(name_str) = get_node_name(node, source_bytes) {
                if name_str == target_name {
                    if let Ok(full_impl) =
                        std::str::from_utf8(&source_bytes[node.start_byte()..node.end_byte()])
                    {
                        let start_line = node.start_position().row + 1;
                        let end_line = node.end_position().row + 1;
                        return Some((start_line, end_line, full_impl.to_string()));
                    }
                }
            }
        }

        for child in node.children(&mut node.walk()) {
            if let Some(found) = walk(child, source_bytes, target_name) {
                return Some(found);
            }
        }

        None
    }

    walk(tree.root_node(), source_bytes, symbol_name)
}

/// High-level function: extracts code skeleton from a file on disk.
pub fn get_code_skeleton_file<P: AsRef<Path>>(file_path: P) -> String {
    let p = file_path.as_ref();
    if !p.exists() {
        return format!("Error: File {} not found.", p.display());
    }
    if p.is_dir() {
        return format!("Error: '{}' is a directory, not a file.", p.display());
    }

    let content = match std::fs::read(p) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
        },
        Err(e) => return format!("Error reading file {}: {}", p.display(), e),
    };

    let lang = match detect_language(p) {
        Some(l) => l,
        None => return content, // Return full text if language not supported
    };

    let skeleton = build_skeleton(&content, lang);
    let orig_tokens = estimate_tokens(&content);
    let skel_tokens = estimate_tokens(&skeleton);
    let savings_pct = if orig_tokens > 0 {
        ((orig_tokens.saturating_sub(skel_tokens) as f64 / orig_tokens as f64) * 100.0) as usize
    } else {
        0
    };

    let comment_prefix = if lang == SupportedLanguage::Python {
        "#"
    } else {
        "//"
    };
    format!("{skeleton}\n{comment_prefix} TokenJar: {orig_tokens} -> {skel_tokens} tokens ({savings_pct}% saved)")
}

/// High-level function: extracts a specific symbol from a file on disk.
pub fn get_symbol_file<P: AsRef<Path>>(file_path: P, symbol_name: &str) -> String {
    let p = file_path.as_ref();
    if !p.exists() {
        return format!("Error: File {} not found.", p.display());
    }
    if p.is_dir() {
        return format!("Error: '{}' is a directory, not a file.", p.display());
    }

    let content = match std::fs::read(p) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
        },
        Err(e) => return format!("Error reading file {}: {}", p.display(), e),
    };

    let lang = match detect_language(p) {
        Some(l) => l,
        None => return format!("Error: Could not detect language for {}.", p.display()),
    };

    match find_symbol_in_code(&content, lang, symbol_name) {
        Some(impl_str) => impl_str,
        None => {
            let rel = p.to_string_lossy();
            let symbols = crate::symbols::extract_symbols_from_code(&content, lang, &rel);
            let mut candidates: Vec<(&str, f64)> = symbols
                .iter()
                .map(|s| {
                    (
                        s.name.as_str(),
                        crate::symbols::compute_similarity(symbol_name, &s.name),
                    )
                })
                .filter(|(_, sim)| *sim >= 0.50)
                .collect();
            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            candidates.dedup_by(|a, b| a.0 == b.0);
            if !candidates.is_empty() {
                let suggestions: Vec<&str> = candidates.iter().take(3).map(|(n, _)| *n).collect();
                format!(
                    "Symbol '{symbol_name}' not found in {}. Did you mean: {}?",
                    p.display(),
                    suggestions.join(", ")
                )
            } else {
                format!("Symbol '{symbol_name}' not found in {}.", p.display())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_skeleton() {
        let code = r#"def calculate_total(price: float, tax: float) -> float:
    """Calculates total with tax."""
    val = price * (1.0 + tax)
    return val

def helper():
    return 42
"#;
        let skel = build_skeleton(code, SupportedLanguage::Python);
        assert!(skel.contains("def calculate_total(price: float, tax: float) -> float:"));
        assert!(skel.contains(r#""""Calculates total with tax.""""#));
        assert!(skel.contains("..."));
        assert!(!skel.contains("val = price * (1.0 + tax)"));
        assert!(!skel.contains("return 42"));
    }

    #[test]
    fn test_rust_skeleton() {
        let code = r#"pub fn compute(a: i32, b: i32) -> i32 {
    let res = a + b * 2;
    res
}
"#;
        let skel = build_skeleton(code, SupportedLanguage::Rust);
        assert!(skel.contains("pub fn compute(a: i32, b: i32) -> i32 {"));
        assert!(skel.contains("..."));
        assert!(!skel.contains("let res = a + b * 2;"));
    }

    #[test]
    fn test_find_symbol_python() {
        let code = r#"def alpha():
    return 1

def beta(x: int) -> int:
    return x * 2
"#;
        let found =
            find_symbol_in_code(code, SupportedLanguage::Python, "beta").expect("Should find beta");
        assert!(found.starts_with("def beta(x: int) -> int:"));
        assert!(found.contains("return x * 2"));
        assert!(!found.contains("alpha"));
    }

    #[test]
    fn test_get_symbol_file_fuzzy_suggestion() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("service.py");
        std::fs::write(&file, "def calculate_total(a, b):\n    return a + b\n").unwrap();

        let res = get_symbol_file(&file, "calculate_totl");
        assert!(res.contains("Did you mean: calculate_total?"));
    }
}
