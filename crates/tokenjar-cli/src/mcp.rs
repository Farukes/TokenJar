//! Native Stdio JSON-RPC 2.0 MCP Server for TokenJar in Rust.
//!
//! Provides the complete Model Context Protocol interface over stdio,
//! serving 10 tools, 3 live resources, and prompt templates with zero Python dependency.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;

use tokenjar_core::cache::session_cache::SessionCache;
use tokenjar_core::config::TokenJarConfig;
use tokenjar_core::output_pruner::{filter_output_logic, run_command_smart};
use tokenjar_core::repo_map::{get_directory_tree, get_repo_map};
use tokenjar_core::skeleton::{get_code_skeleton_file, get_symbol_file};
use tokenjar_core::smart_reader::{cache_stats, read_file_smart};
use tokenjar_core::symbols::{find_symbol_global, find_symbol_references};
use tokenjar_core::telemetry::TelemetryTracker;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

pub struct McpServer {
    cache: SessionCache,
    tracker: TelemetryTracker,
    config: TokenJarConfig,
}

impl McpServer {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let config = TokenJarConfig::load_from_dir(&current_dir);
        Self {
            cache: SessionCache::new(),
            tracker: TelemetryTracker::new(),
            config,
        }
    }

    pub fn run_stdio(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        eprintln!("[tokenjar] Native Rust MCP server ready on stdio.");

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[tokenjar] Invalid JSON received: {e}");
                    let resp = JsonRpcResponse {
                        jsonrpc: "2.0",
                        id: Value::Null,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {e}"),
                        }),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                    stdout.flush()?;
                    continue;
                }
            };

            let req_id = req.id.clone();
            let is_notification = req_id.is_none();

            let result = self.handle_method(&req.method, req.params);

            if !is_notification {
                let id = req_id.unwrap_or(Value::Null);
                let resp = match result {
                    Ok(val) => JsonRpcResponse {
                        jsonrpc: "2.0",
                        id,
                        result: Some(val),
                        error: None,
                    },
                    Err((code, msg)) => JsonRpcResponse {
                        jsonrpc: "2.0",
                        id,
                        result: None,
                        error: Some(JsonRpcError { code, message: msg }),
                    },
                };

                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    fn handle_method(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, (i32, String)> {
        match method {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "tokenjar",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),

            "notifications/initialized" | "initialized" => Ok(json!({})),

            "ping" => Ok(json!({})),

            "tools/list" => Ok(json!({
                "tools": self.list_tools()
            })),

            "tools/call" => {
                let params = params.ok_or((-32602, "Missing params for tools/call".to_string()))?;
                let tool_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'name' in tools/call params".to_string()))?;
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                let content_text = self.call_tool(tool_name, &arguments)?;
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": content_text
                        }
                    ]
                }))
            }

            "resources/list" => Ok(json!({
                "resources": [
                    {
                        "uri": "tokenjar://stats",
                        "name": "TokenJar Live Telemetry Dashboard",
                        "description": "Live cumulative token savings dashboard and metrics across coding sessions.",
                        "mimeType": "text/plain"
                    },
                    {
                        "uri": "tokenjar://guide",
                        "name": "TokenJar AI Optimization Guidelines",
                        "description": "Strict mandates for AI coding assistants to prevent token waste and context compaction.",
                        "mimeType": "text/markdown"
                    },
                    {
                        "uri": "tokenjar://config",
                        "name": "TokenJar Active Configuration",
                        "description": "Current active security ignore patterns, cache thresholds, and lockfile rules.",
                        "mimeType": "application/json"
                    }
                ]
            })),

            "resources/read" => {
                let params =
                    params.ok_or((-32602, "Missing params for resources/read".to_string()))?;
                let uri = params
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'uri' in resources/read params".to_string()))?;

                let contents = match uri {
                    "tokenjar://stats" => self.tracker.render_dashboard(),
                    "tokenjar://guide" => {
                        "# 🔋 TokenJar AI Optimization Guidelines (STRICT ENFORCEMENT)\n\
                         1. File Reading: ALWAYS use read_file_smart instead of native file viewers.\n\
                         2. Symbol Search: ALWAYS use find_symbol_global to locate functions/classes.\n\
                         3. Blast Radius: ALWAYS use find_symbol_references before editing code.\n\
                         4. Skeleton: ALWAYS use tool_get_code_skeleton to inspect classes before reading full implementations.\n\
                         5. Repo Map: ALWAYS use get_repo_map_tool to explore architecture.\n\
                         6. Command Execution: Use run_command_smart for test runners (pytest, cargo test).\n\
                         7. ZERO TRUNCATION MANDATE: Never use placeholder comments. Every edit must be complete.\n".to_string()
                    }
                    "tokenjar://config" => {
                        serde_json::to_string_pretty(&self.config).unwrap_or_else(|_| "{}".to_string())
                    }
                    _ => return Err((-32602, format!("Resource not found: {uri}"))),
                };

                Ok(json!({
                    "contents": [
                        {
                            "uri": uri,
                            "mimeType": if uri.ends_with(".json") || uri.contains("config") { "application/json" } else { "text/plain" },
                            "text": contents
                        }
                    ]
                }))
            }

            "prompts/list" => Ok(json!({
                "prompts": [
                    {
                        "name": "optimize_coding_task",
                        "description": "System prompt template directing the AI assistant to execute a coding task with minimal tokens.",
                        "arguments": [
                            {
                                "name": "task_description",
                                "description": "The specific coding task or bugfix to execute.",
                                "required": true
                            }
                        ]
                    }
                ]
            })),

            "prompts/get" => {
                let params =
                    params.ok_or((-32602, "Missing params for prompts/get".to_string()))?;
                let prompt_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'name' in prompts/get params".to_string()))?;

                if prompt_name == "optimize_coding_task" {
                    let task = params
                        .get("arguments")
                        .and_then(|a| a.get("task_description"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("coding task");

                    let text = format!(
                        "You are executing the following coding task: '{task}'\n\n\
                         Strictly adhere to the TokenJar AI Optimization Guidelines:\n\
                         1. Start by calling 'find_symbol_global' or 'tool_get_code_skeleton' to inspect signatures.\n\
                         2. Do NOT read entire files with standard viewers; use 'read_file_smart' to fetch compact diffs.\n\
                         3. Run all tests through 'run_command_smart' to prune repetitive passing output.\n\
                         4. Focus edits strictly on necessary symbols with surgical block replacements.\n\
                         5. ZERO TRUNCATION MANDATE: Never use placeholder comments. Code must be 100% complete.\n"
                    );

                    Ok(json!({
                        "description": "System prompt directing minimal token usage.",
                        "messages": [
                            {
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": text
                                }
                            }
                        ]
                    }))
                } else {
                    Err((-32602, format!("Prompt not found: {prompt_name}")))
                }
            }

            _ => Err((-32601, format!("Method not found: {method}"))),
        }
    }

    fn list_tools(&self) -> Value {
        json!([
            {
                "name": "read_file_smart",
                "description": "Intelligently read a file with session-level caching, targeted line slicing, and lockfile protection. Returns full content on first read, compact diffs on modifications, or surgical line range slices with line numbers.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Absolute or relative path to the file." },
                        "force_full": { "type": "boolean", "default": false, "description": "If true, bypasses caching and lockfile shields." },
                        "query": { "type": ["string", "null"], "default": null, "description": "Optional search term for querying within lockfiles." },
                        "start_line": { "type": ["integer", "null"], "default": null, "description": "Optional starting line number (1-indexed, inclusive) to slice specific line ranges." },
                        "end_line": { "type": ["integer", "null"], "default": null, "description": "Optional ending line number (1-indexed, inclusive) to slice specific line ranges." },
                        "symbol": { "type": ["string", "null"], "default": null, "description": "Optional name of a function, class, or method to extract directly. Instantly returns the full symbol implementation with line numbers, eliminating 2-step lookups and preventing output truncation." }
                    },
                    "required": ["file_path"]
                }
            },
            {
                "name": "tool_get_code_skeleton",
                "description": "Extracts a structural skeleton from a source file, replacing bodies with '...'. Use this INSTEAD of reading full files to understand structure.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Path to the source file." }
                    },
                    "required": ["file_path"]
                }
            },
            {
                "name": "tool_get_symbol",
                "description": "Extracts the FULL implementation of a specific function, method, or class from a file by name.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Path to the source file." },
                        "symbol_name": { "type": "string", "description": "Name of the symbol (function, method, class, struct) to extract." }
                    },
                    "required": ["file_path", "symbol_name"]
                }
            },
            {
                "name": "find_symbol_global",
                "description": "Search for functions, methods, or classes across the entire codebase by name. Use this tool to instantly locate where a symbol is defined without reading multiple files or guessing paths.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Symbol name or substring to search for." },
                        "root_path": { "type": "string", "default": ".", "description": "Project root directory." },
                        "exact": { "type": "boolean", "default": false, "description": "If true, only match exact symbol names." },
                        "max_results": { "type": "integer", "default": 15, "description": "Maximum number of results to return." }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "find_symbol_references",
                "description": "Search for all usages, calls, and imports of a symbol across the entire codebase. Use this tool before editing, refactoring, or deleting functions/classes to inspect blast radius.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "symbol_name": { "type": "string", "description": "Exact name of the symbol to trace." },
                        "root_path": { "type": "string", "default": ".", "description": "Project root directory." },
                        "max_results": { "type": "integer", "default": 25, "description": "Maximum number of references to return." }
                    },
                    "required": ["symbol_name"]
                }
            },
            {
                "name": "run_command_smart",
                "description": "Executes a shell command and returns intelligently filtered output. Prunes test runners and build output to minimize token consumption.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Shell command line to execute." },
                        "cwd": { "type": "string", "default": ".", "description": "Working directory." },
                        "timeout": { "type": "integer", "default": 120, "description": "Timeout in seconds." },
                        "background": { "type": "boolean", "default": false, "description": "If true, spawns as a background task." }
                    },
                    "required": ["command"]
                }
            },
            {
                "name": "filter_output",
                "description": "Filters raw text output to save tokens. Useful when you already have output from an external source.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "raw_output": { "type": "string", "description": "Raw terminal output to filter." },
                        "output_type": { "type": "string", "default": "auto", "description": "Filter type: auto, pytest, cargo, generic." }
                    },
                    "required": ["raw_output"]
                }
            },
            {
                "name": "get_repo_map_tool",
                "description": "Generates a structural map of the entire repository fitted to a token budget. Use this at the START of any task to get a bird's-eye view of the codebase.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "root_path": { "type": "string", "default": ".", "description": "Project root directory." },
                        "max_tokens": { "type": "integer", "default": 1000, "description": "Maximum token budget for the map." },
                        "focus_files": { "type": "array", "items": { "type": "string" }, "default": [], "description": "Files to prioritize in ranking." }
                    }
                }
            },
            {
                "name": "get_directory_tree_tool",
                "description": "Simple directory tree listing (respects skip dirs and binary files) — a lightweight alternative to repo_map.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "root_path": { "type": "string", "default": ".", "description": "Project root directory." },
                        "max_depth": { "type": "integer", "default": 4, "description": "Maximum directory traversal depth." }
                    }
                }
            },
            {
                "name": "cache_stats",
                "description": "Get a summary of the smart reader session cache performance.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }
        ])
    }

    fn call_tool(&mut self, name: &str, args: &Value) -> Result<String, (i32, String)> {
        match name {
            "read_file_smart" => {
                let file_path = args
                    .get("file_path")
                    .or_else(|| args.get("path"))
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'file_path'".to_string()))?;
                let force_full = args
                    .get("force_full")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let query = args.get("query").and_then(|v| v.as_str());
                let start_line = args
                    .get("start_line")
                    .or_else(|| args.get("offset"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                let end_line = args
                    .get("end_line")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .or_else(|| {
                        args.get("limit")
                            .and_then(|l| l.as_u64())
                            .and_then(|lim| start_line.map(|s| s + lim as usize - 1))
                    });

                let symbol = args
                    .get("symbol")
                    .or_else(|| args.get("symbol_name"))
                    .and_then(|v| v.as_str());

                Ok(read_file_smart(
                    file_path,
                    force_full,
                    query,
                    start_line,
                    end_line,
                    symbol,
                    &self.cache,
                    &self.config,
                    &self.tracker,
                ))
            }

            "tool_get_code_skeleton" => {
                let file_path = args
                    .get("file_path")
                    .or_else(|| args.get("path"))
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'file_path'".to_string()))?;
                let res = get_code_skeleton_file(file_path);
                if let Ok(raw) = std::fs::read_to_string(file_path) {
                    let orig_tok = (raw.len() / 4) as u64;
                    let opt_tok = (res.len() / 4) as u64;
                    if orig_tok > opt_tok {
                        self.tracker.record_savings("skeleton", orig_tok, opt_tok);
                    }
                }
                Ok(res)
            }

            "tool_get_symbol" => {
                let file_path = args
                    .get("file_path")
                    .or_else(|| args.get("path"))
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'file_path'".to_string()))?;
                let symbol_name = args
                    .get("symbol_name")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'symbol_name'".to_string()))?;
                Ok(get_symbol_file(file_path, symbol_name))
            }

            "find_symbol_global" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'query'".to_string()))?;
                let root_path_str = args
                    .get("root_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let exact = args.get("exact").and_then(|v| v.as_bool()).unwrap_or(false);
                let max_results = args
                    .get("max_results")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(15) as usize;

                let res = find_symbol_global(query, Path::new(root_path_str), exact, max_results);
                let opt_tok = (res.len() / 4) as u64;
                let raw_tok = opt_tok.saturating_mul(3).clamp(150, 1500);
                if raw_tok > opt_tok {
                    self.tracker
                        .record_savings("symbol_search", raw_tok, opt_tok);
                }
                Ok(res)
            }

            "find_symbol_references" => {
                let symbol_name = args
                    .get("symbol_name")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'symbol_name'".to_string()))?;
                let root_path_str = args
                    .get("root_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let max_results = args
                    .get("max_results")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(25) as usize;

                let res =
                    find_symbol_references(symbol_name, Path::new(root_path_str), max_results);
                let opt_tok = (res.len() / 4) as u64;
                let raw_tok = opt_tok.saturating_mul(3).clamp(200, 2000);
                if raw_tok > opt_tok {
                    self.tracker
                        .record_savings("symbol_search", raw_tok, opt_tok);
                }
                Ok(res)
            }

            "run_command_smart" => {
                let command = args
                    .get("command")
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'command'".to_string()))?;
                let cwd = args.get("cwd").and_then(|v| v.as_str()).unwrap_or(".");
                let timeout = args.get("timeout").and_then(|v| v.as_u64()).unwrap_or(120);
                let background = args
                    .get("background")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                Ok(run_command_smart(
                    command,
                    cwd,
                    timeout,
                    background,
                    &self.tracker,
                ))
            }

            "filter_output" => {
                let raw_output = args
                    .get("raw_output")
                    .or_else(|| args.get("output"))
                    .and_then(|v| v.as_str())
                    .ok_or((-32602, "Missing 'raw_output'".to_string()))?;
                let output_type = args
                    .get("output_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("auto");

                Ok(filter_output_logic(
                    raw_output,
                    output_type,
                    0,
                    &self.tracker,
                ))
            }

            "get_repo_map_tool" => {
                let root_path_str = args
                    .get("root_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let max_tokens = args
                    .get("max_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000) as usize;
                let focus_files: Vec<String> = args
                    .get("focus_files")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let res = get_repo_map(Path::new(root_path_str), max_tokens, &focus_files);
                let opt_tok = (res.len() / 4) as u64;
                let root = Path::new(root_path_str);
                let mut manifest_tok: u64 = 0;
                for fname in &["README.md", "Cargo.toml", "package.json", "pyproject.toml", "src/main.rs", "src/lib.rs"] {
                    if let Ok(meta) = std::fs::metadata(root.join(fname)) {
                        manifest_tok += meta.len() / 4;
                    }
                }
                let raw_tok = manifest_tok.clamp(1500, 5000);
                self.tracker.record_savings("repo_map", raw_tok, opt_tok);
                Ok(res)
            }

            "get_directory_tree_tool" => {
                let root_path_str = args
                    .get("root_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let max_depth =
                    args.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(4) as usize;

                Ok(get_directory_tree(Path::new(root_path_str), max_depth))
            }

            "cache_stats" => Ok(cache_stats(&self.cache)),

            _ => Err((-32601, format!("Tool not found: {name}"))),
        }
    }
}
