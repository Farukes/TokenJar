//! Steering Rules Manager for TokenJar in Rust.
//!
//! Generates and injects non-intrusive AI steering instructions into AGENTS.md,
//! .cursorrules, .windsurfrules, and CLAUDE.md to guarantee AI models actively
//! prioritize TokenJar MCP tools over native, unoptimized tools.

use std::fs;
use std::path::{Path, PathBuf};

pub const RULES_MARKER_START: &str = "# >>> tokenjar-rules >>>";
pub const RULES_MARKER_END: &str = "# <<< tokenjar-rules <<<";

pub const SUPPORTED_RULE_FILES: &[&str] =
    &["AGENTS.md", ".cursorrules", ".windsurfrules", "CLAUDE.md"];

/// Generates steering rules instructions with optional output optimization.
pub fn generate_rules(compact_output: bool, prevent_truncation: bool) -> String {
    let mut lines = vec![
        RULES_MARKER_START.to_string(),
        "## 🔋 TokenJar AI Optimization Guidelines (STRICT ENFORCEMENT)".to_string(),
        "CRITICAL MANDATE: Do NOT use native file viewers (view_file, cat) or raw terminal commands when TokenJar MCP tools are available. Bypassing TokenJar wastes hundreds of thousands of tokens and triggers conversation context window compaction.".to_string(),
        "".to_string(),
        "1. **File Reading & Inspections (MANDATORY):**".to_string(),
        "   - ALWAYS use `read_file_smart` instead of native file viewers. It caches file contents, returns compact diffs on edits, and supports `symbol`, `start_line`, and `end_line` parameters for targeted function and line range slicing.".to_string(),
        "2. **Codebase Exploration & Symbol Search (MANDATORY):**".to_string(),
        "   - ALWAYS use `find_symbol_global` to locate functions, classes, or methods instantly across the codebase.".to_string(),
        "   - ALWAYS use `find_symbol_references` before editing or refactoring code to check blast radius (all callers, usages, and imports).".to_string(),
        "   - ALWAYS use `tool_get_code_skeleton` to inspect classes, signatures, and docstrings before reading full file implementations.".to_string(),
        "   - ALWAYS use `get_repo_map_tool` to explore repository architecture instead of listing and reading multiple files.".to_string(),
        "3. **Terminal & Test Execution (MANDATORY):**".to_string(),
        "   - For fast test suites and scripts (`pytest`, `npm test`, targeted unit tests): Use `run_command_smart` directly.".to_string(),
        "   - For heavy compilation or workspace-wide builds (e.g. `cargo test --workspace`, `cargo build --release`): Run via native terminal, then pass output through `filter_output` to prune logs without blocking the MCP connection.".to_string(),
    ];

    if compact_output {
        lines.push("4. **Output Optimization & Code Quality Mandate (STRICT):**".to_string());
        lines.push("   - Targeted File Slices: When inspecting specific functions or line ranges, pass `symbol=\"function_name\"` or `start_line`/`end_line` to `read_file_smart` to avoid dumping whole files into context.".to_string());
        lines.push("   - Surgical File Edits: When modifying code, use surgical replacement blocks targeting precise line ranges instead of rewriting entire unchanged files.".to_string());
        if prevent_truncation {
            lines.push("   - ZERO TRUNCATION MANDATE (Anti-Lazy Coder): NEVER use placeholder comments (e.g. '// ... rest of code unchanged ...' or 'TODO: keep existing logic') or omit required logic. Every generated or replaced code block must be complete, functional, and syntactically valid.".to_string());
        }
        lines.push("   - High-Density Rationale: Omit conversational pleasantries, introductory filler, and restating line-by-line code changes. Prioritize direct, rigorous technical justification, architectural context, and concrete solutions.".to_string());
    }

    lines.push(RULES_MARKER_END.to_string());
    lines.join("\n")
}

#[derive(Debug, Clone)]
pub struct RuleInstallResult {
    pub file_name: String,
    pub path: PathBuf,
    pub success: bool,
    pub message: String,
}

pub fn record_project(target_dir: &Path) {
    if let Some(home) = dirs::home_dir() {
        let dir = home.join(".tokenjar");
        let _ = std::fs::create_dir_all(&dir);
        let p_file = dir.join("projects.json");
        let resolved =
            std::fs::canonicalize(target_dir).unwrap_or_else(|_| target_dir.to_path_buf());
        let resolved_str = resolved.to_string_lossy().to_string();

        let mut projects: std::collections::BTreeSet<String> = if p_file.exists() {
            std::fs::read_to_string(&p_file)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            std::collections::BTreeSet::new()
        };

        projects.insert(resolved_str);
        if let Ok(formatted) = serde_json::to_string_pretty(&projects) {
            let _ = std::fs::write(p_file, formatted);
        }
    }
}

pub fn get_known_project_roots() -> Vec<PathBuf> {
    let mut roots = std::collections::BTreeSet::new();

    // 1. From projects.json
    if let Some(home) = dirs::home_dir() {
        let p_file = home.join(".tokenjar").join("projects.json");
        if p_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&p_file) {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(&content) {
                    for p in list {
                        let pb = PathBuf::from(&p);
                        if pb.exists() {
                            roots.insert(pb);
                        }
                    }
                }
            }
        }
    }

    // 2. Current working directory
    if let Ok(curr) = std::env::current_dir() {
        roots.insert(curr);
    }
    roots.insert(PathBuf::from("."));

    roots.into_iter().collect()
}

/// Injects or updates TokenJar rules into target project rule files.
pub fn install_rules(
    target_dir: &Path,
    compact_output: bool,
    prevent_truncation: bool,
) -> Vec<RuleInstallResult> {
    record_project(target_dir);
    let rules_text = generate_rules(compact_output, prevent_truncation);
    let mut results = Vec::new();

    for file_name in SUPPORTED_RULE_FILES {
        let file_path = target_dir.join(file_name);
        let existing = fs::read_to_string(&file_path).unwrap_or_default();

        let updated =
            if existing.contains(RULES_MARKER_START) && existing.contains(RULES_MARKER_END) {
                let re = regex::Regex::new(&format!(
                    r"(?s){}.*?{}",
                    regex::escape(RULES_MARKER_START),
                    regex::escape(RULES_MARKER_END)
                ))
                .unwrap();
                re.replace(&existing, rules_text.as_str()).to_string()
            } else if !existing.is_empty() {
                format!("{existing}\n\n{rules_text}\n")
            } else {
                format!("{rules_text}\n")
            };

        match fs::write(&file_path, updated) {
            Ok(_) => results.push(RuleInstallResult {
                file_name: file_name.to_string(),
                path: file_path,
                success: true,
                message: "Rules successfully updated/injected.".to_string(),
            }),
            Err(e) => results.push(RuleInstallResult {
                file_name: file_name.to_string(),
                path: file_path,
                success: false,
                message: format!("Failed to write: {e}"),
            }),
        }
    }

    results
}

/// Removes TokenJar steering blocks from project rule files.
pub fn remove_rules(target_dir: &Path) -> Vec<RuleInstallResult> {
    let mut results = Vec::new();

    for file_name in SUPPORTED_RULE_FILES {
        let file_path = target_dir.join(file_name);
        if !file_path.exists() {
            continue;
        }

        let existing = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                results.push(RuleInstallResult {
                    file_name: file_name.to_string(),
                    path: file_path,
                    success: false,
                    message: format!("Failed to read: {e}"),
                });
                continue;
            }
        };

        let mut cleaned = existing.clone();
        for (m_start, m_end) in [
            (RULES_MARKER_START, RULES_MARKER_END),
            ("# >>> token-saver-rules >>>", "# <<< token-saver-rules <<<"),
        ] {
            if cleaned.contains(m_start) && cleaned.contains(m_end) {
                let re = regex::Regex::new(&format!(
                    r"(?s)\n*{}.*?{}\n*",
                    regex::escape(m_start),
                    regex::escape(m_end)
                ))
                .unwrap();
                cleaned = re.replace(&cleaned, "\n").trim_matches('\n').to_string();
            }
        }

        if cleaned != existing {
            if cleaned.is_empty() {
                let _ = fs::remove_file(&file_path);
                results.push(RuleInstallResult {
                    file_name: file_name.to_string(),
                    path: file_path,
                    success: true,
                    message: "Rules removed (empty file cleaned up).".to_string(),
                });
            } else {
                let _ = fs::write(&file_path, format!("{cleaned}\n"));
                results.push(RuleInstallResult {
                    file_name: file_name.to_string(),
                    path: file_path,
                    success: true,
                    message: "Rules removed from existing file.".to_string(),
                });
            }
        }
    }

    // Also remove project-level config if present
    let toml_path = target_dir.join("tokenjar.toml");
    if toml_path.exists() && fs::remove_file(&toml_path).is_ok() {
        results.push(RuleInstallResult {
            file_name: "tokenjar.toml".to_string(),
            path: toml_path,
            success: true,
            message: "Removed tokenjar.toml configuration.".to_string(),
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_and_remove_rules() {
        let temp = tempfile::tempdir().unwrap();
        let results = install_rules(temp.path(), true, true);
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|r| r.success));

        let agents_md = temp.path().join("AGENTS.md");
        let content = fs::read_to_string(&agents_md).unwrap();
        assert!(content.contains(RULES_MARKER_START));
        assert!(content.contains("TokenJar AI Optimization Guidelines"));

        let remove_res = remove_rules(temp.path());
        assert!(!remove_res.is_empty());
        assert!(!agents_md.exists()); // was empty except rules, so removed
    }
}
