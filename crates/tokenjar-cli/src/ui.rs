//! Embedded Web Dashboard HTTP Server for TokenJar CLI in Rust.
//!
//! Zero background RAM: runs an asynchronous local HTTP server on demand,
//! serves the embedded dashboard UI, and cleanly shuts down on exit.

use serde_json::json;
use std::net::SocketAddr;
use std::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use tokenjar_core::config::TokenJarConfig;
use tokenjar_core::telemetry::TelemetryTracker;

const HTML_CONTENT: &str = include_str!("../static/index.html");

/// Builds system status JSON matching the dashboard frontend expectation.
pub fn build_system_status(tracker: &TelemetryTracker) -> serde_json::Value {
    let t_data = tracker.get_data();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cfg = TokenJarConfig::load_from_dir(&current_dir);

    let rules_installed = ["AGENTS.md", ".cursorrules", ".windsurfrules", "CLAUDE.md"]
        .iter()
        .any(|f| current_dir.join(f).exists());

    let categories = json!({
        "lockfile": {
            "name": "Lockfile Shield",
            "saved": t_data.lockfile.saved,
            "count": t_data.lockfile.count,
            "pct": (t_data.lockfile.savings_pct() * 10.0).round() / 10.0,
            "unit": "shields",
            "icon": "🛡️"
        },
        "symbol_search": {
            "name": "Global Symbol Search",
            "saved": t_data.symbol_search.saved,
            "count": t_data.symbol_search.count,
            "pct": (t_data.symbol_search.savings_pct() * 10.0).round() / 10.0,
            "unit": "searches",
            "icon": "🔍"
        },
        "repo_map": {
            "name": "Repo Map Engine",
            "saved": t_data.repo_map.saved,
            "count": t_data.repo_map.count,
            "pct": (t_data.repo_map.savings_pct() * 10.0).round() / 10.0,
            "unit": "maps",
            "icon": "🗺️"
        },
        "skeleton": {
            "name": "AST Skeletonizer",
            "saved": t_data.skeleton.saved,
            "count": t_data.skeleton.count,
            "pct": (t_data.skeleton.savings_pct() * 10.0).round() / 10.0,
            "unit": "files",
            "icon": "🦴"
        },
        "cache": {
            "name": "Smart File Cache",
            "saved": t_data.cache.saved,
            "count": t_data.cache.count,
            "pct": (t_data.cache.savings_pct() * 10.0).round() / 10.0,
            "unit": "reads",
            "icon": "⚡"
        },
        "command": {
            "name": "Terminal Pruner",
            "saved": t_data.command.saved,
            "count": t_data.command.count,
            "pct": (t_data.command.savings_pct() * 10.0).round() / 10.0,
            "unit": "runs",
            "icon": "✂️"
        },
        "slice": {
            "name": "Smart Line Slicer",
            "saved": t_data.slice.saved,
            "count": t_data.slice.count,
            "pct": (t_data.slice.savings_pct() * 10.0).round() / 10.0,
            "unit": "slices",
            "icon": "🔪"
        }
    });

    let ide_configs = tokenjar_core::installer::get_supported_ide_configs();
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let mut any_active = false;
    let mut ides = Vec::new();

    for (name, path) in ide_configs {
        let file_exists = path.exists();
        let installed = if name == "Claude Code" {
            tokenjar_core::installer::is_claude_code_installed()
        } else {
            let parent_exists = path
                .parent()
                .map(|p| p.exists() && p != home)
                .unwrap_or(false);
            file_exists || parent_exists
        };
        let mut active = false;

        if file_exists && installed {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.contains("tokenjar") {
                    active = true;
                    any_active = true;
                }
            }
        }

        ides.push(json!({
            "name": name,
            "installed": installed,
            "active": active,
            "path": path.to_string_lossy().to_string()
        }));
    }

    let is_system_active = any_active || rules_installed;

    let cache_bytes = tracker.get_l2_cache_disk_bytes();
    let cache_mb = ((cache_bytes as f64 / (1024.0 * 1024.0)) * 100.0).round() / 100.0;
    let cache_entries = tokenjar_core::cache::persistent_cache::PersistentCache::new()
        .map(|c| c.count_entries().unwrap_or(0))
        .unwrap_or(0);

    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "active": is_system_active,
        "overall_status": if is_system_active { "ACTIVE" } else { "STANDBY" },
        "telemetry": {
            "total_saved": t_data.total_tokens_saved,
            "total_processed": t_data.total_original_tokens,
            "savings_pct": (t_data.savings_pct() * 10.0).round() / 10.0,
            "dollars_saved": (t_data.estimated_dollars_saved() * 100.0).round() / 100.0,
            "categories": categories,
            "l2_cache": {
                "disk_bytes": cache_bytes,
                "disk_mb": cache_mb,
                "entries": cache_entries,
                "warning": cache_mb > 50.0
            }
        },
        "ides": ides,
        "rules": {
            "installed": rules_installed,
            "compact_output": cfg.compact_output
        },
        "config": {
            "lockfile_shield": cfg.lockfile_shield,
            "compact_output": cfg.compact_output,
            "prevent_truncation": cfg.prevent_truncation
        }
    })
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd").args(["/C", "start", url]).spawn();

    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let _ = Command::new("xdg-open").arg(url).spawn();
}

/// Runs the local dashboard web server on localhost:8080 (or next open port).
pub async fn start_ui_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_port = port;
    let listener = loop {
        let addr = SocketAddr::from(([127, 0, 0, 1], current_port));
        match TcpListener::bind(addr).await {
            Ok(l) => break l,
            Err(_) => {
                current_port += 1;
                if current_port > port + 50 {
                    return Err("Could not find an available port for TokenJar UI".into());
                }
            }
        }
    };

    let url = format!("http://127.0.0.1:{current_port}");
    println!("============================================================");
    println!("🔋 TOKENJAR CONTROL DASHBOARD (RUST NATIVE)");
    println!("============================================================");
    println!("Dashboard URL : {url}");
    println!("Status        : 🟢 Running (Press Ctrl+C to terminate)");
    println!("============================================================");

    open_browser(&url);

    let tracker = TelemetryTracker::new();

    loop {
        let (mut socket, _) = listener.accept().await?;
        let mut buf = [0u8; 4096];
        let bytes_read = match socket.read(&mut buf).await {
            Ok(0) | Err(_) => continue,
            Ok(n) => n,
        };

        let request = String::from_utf8_lossy(&buf[..bytes_read]);
        let first_line = request.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();

        if parts.len() < 2 {
            continue;
        }

        let method = parts[0];
        let path = parts[1];

        // Security: Host and Origin header validation against DNS rebinding and CSRF
        let mut host_header = "";
        let mut origin_header = "";
        for line in request.lines().skip(1) {
            let line_lower = line.to_lowercase();
            if line_lower.starts_with("host:") {
                host_header = line[5..].trim();
            } else if line_lower.starts_with("origin:") {
                origin_header = line[7..].trim();
            }
        }

        if !host_header.is_empty()
            && !host_header.starts_with("127.0.0.1")
            && !host_header.starts_with("localhost")
        {
            let resp = "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\n\r\nForbidden Host";
            let _ = socket.write_all(resp.as_bytes()).await;
            continue;
        }

        if !origin_header.is_empty()
            && !origin_header.starts_with("http://127.0.0.1")
            && !origin_header.starts_with("http://localhost")
            && !origin_header.starts_with("https://127.0.0.1")
            && !origin_header.starts_with("https://localhost")
        {
            let resp = "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\n\r\nForbidden Origin";
            let _ = socket.write_all(resp.as_bytes()).await;
            continue;
        }

        let post_json: Option<serde_json::Value> = if method == "POST" {
            request
                .find("\r\n\r\n")
                .or_else(|| request.find("\n\n"))
                .and_then(|pos| serde_json::from_str(request[pos..].trim()).ok())
        } else {
            None
        };

        let (status_code, content_type, body) = match (method, path) {
            ("GET", "/") | ("GET", "/index.html") => (
                "200 OK",
                "text/html; charset=utf-8",
                HTML_CONTENT.to_string(),
            ),
            ("GET", "/api/status") => {
                let status_json = build_system_status(&tracker);
                ("200 OK", "application/json", status_json.to_string())
            }
            ("POST", "/api/reset-stats") => {
                tracker.reset();
                let status_json = build_system_status(&tracker);
                (
                    "200 OK",
                    "application/json",
                    json!({ "ok": true, "msg": "Stats reset", "status": status_json }).to_string(),
                )
            }
            ("POST", "/api/shutdown") => {
                tokio::spawn(async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    std::process::exit(0);
                });
                (
                    "200 OK",
                    "application/json",
                    json!({ "ok": true, "msg": "Server shutting down" }).to_string(),
                )
            }
            ("POST", "/api/toggle-output") => {
                let compact = post_json
                    .as_ref()
                    .and_then(|j| j.get("compact"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                tokenjar_core::rules::install_rules(std::path::Path::new("."), compact, true);
                let status_json = build_system_status(&tracker);
                ("200 OK", "application/json", json!({
                    "ok": true,
                    "msg": format!("Output mode set to: {}", if compact { "Compact Surgical" } else { "Standard Verbose" }),
                    "status": status_json
                }).to_string())
            }
            ("POST", "/api/toggle-all") => {
                let enable = post_json
                    .as_ref()
                    .and_then(|j| j.get("enable"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if enable {
                    tokenjar_core::installer::install_mcp_all(false, None);
                    tokenjar_core::rules::install_rules(std::path::Path::new("."), true, true);
                } else {
                    tokenjar_core::installer::uninstall_mcp_all();
                    tokenjar_core::rules::remove_rules(std::path::Path::new("."));
                }
                let status_json = build_system_status(&tracker);
                ("200 OK", "application/json", json!({
                    "ok": true,
                    "msg": if enable { "Activated TokenJar across detected IDEs" } else { "Deactivated TokenJar across all IDEs" },
                    "status": status_json
                }).to_string())
            }
            ("POST", "/api/toggle-rules") => {
                let enable = post_json
                    .as_ref()
                    .and_then(|j| j.get("enable"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if enable {
                    tokenjar_core::rules::install_rules(std::path::Path::new("."), true, true);
                } else {
                    tokenjar_core::rules::remove_rules(std::path::Path::new("."));
                }
                let status_json = build_system_status(&tracker);
                ("200 OK", "application/json", json!({
                    "ok": true,
                    "msg": if enable { "Steering rules installed" } else { "Steering rules removed" },
                    "status": status_json
                }).to_string())
            }
            ("POST", "/api/toggle-ide") => {
                let ide_name = post_json
                    .as_ref()
                    .and_then(|j| j.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let enable = post_json
                    .as_ref()
                    .and_then(|j| j.get("enable"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let ide_configs = tokenjar_core::installer::get_supported_ide_configs();
                if let Some((_, cfg_path)) = ide_configs.iter().find(|(n, _)| *n == ide_name) {
                    if enable {
                        let exe = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("tokenjar.exe"))
                            .to_string_lossy()
                            .to_string();
                        if let Some(parent) = cfg_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let mut json_data = if cfg_path.exists() {
                            std::fs::read_to_string(cfg_path)
                                .ok()
                                .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                                .unwrap_or_else(|| json!({}))
                        } else {
                            json!({})
                        };
                        if !json_data.is_object() {
                            json_data = json!({});
                        }
                        if json_data.get("mcpServers").is_none()
                            || !json_data["mcpServers"].is_object()
                        {
                            json_data["mcpServers"] = json!({});
                        }
                        json_data["mcpServers"]["tokenjar"] = json!({
                            "command": exe,
                            "args": []
                        });
                        if let Ok(formatted) = serde_json::to_string_pretty(&json_data) {
                            let _ = std::fs::write(cfg_path, formatted);
                        }
                    } else if cfg_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(cfg_path) {
                            if let Ok(mut json_data) =
                                serde_json::from_str::<serde_json::Value>(&content)
                            {
                                if let Some(servers) = json_data
                                    .get_mut("mcpServers")
                                    .and_then(|s| s.as_object_mut())
                                {
                                    servers.remove("tokenjar");
                                }
                                if let Ok(formatted) = serde_json::to_string_pretty(&json_data) {
                                    let _ = std::fs::write(cfg_path, formatted);
                                }
                            }
                        }
                    }
                }
                let status_json = build_system_status(&tracker);
                ("200 OK", "application/json", json!({
                    "ok": true,
                    "msg": format!("{}: {}", ide_name, if enable { "Connected" } else { "Disconnected" }),
                    "status": status_json
                }).to_string())
            }
            ("POST", "/api/prune-cache") => {
                let msg = match tokenjar_core::cache::persistent_cache::PersistentCache::new() {
                    Ok(c) => match c.prune(1000) {
                        Ok(n) => format!("L2 Cache pruned ({n} entries cleared)"),
                        Err(e) => format!("Prune failed: {e}"),
                    },
                    Err(e) => format!("Cache connection failed: {e}"),
                };
                let status_json = build_system_status(&tracker);
                (
                    "200 OK",
                    "application/json",
                    json!({ "ok": true, "msg": msg, "status": status_json }).to_string(),
                )
            }
            ("POST", "/api/clear-cache") | ("POST", "/api/reset-cache") => {
                let msg = match tokenjar_core::cache::persistent_cache::PersistentCache::new() {
                    Ok(c) => match c.clear() {
                        Ok(n) => format!("L2 Cache completely cleared ({n} entries removed)"),
                        Err(e) => format!("Clear failed: {e}"),
                    },
                    Err(e) => format!("Cache connection failed: {e}"),
                };
                let status_json = build_system_status(&tracker);
                (
                    "200 OK",
                    "application/json",
                    json!({ "ok": true, "msg": msg, "status": status_json }).to_string(),
                )
            }
            _ => ("404 Not Found", "text/plain", "Not Found".to_string()),
        };

        let response = format!(
            "HTTP/1.1 {status_code}\r\n\
             Content-Type: {content_type}\r\n\
             Content-Length: {}\r\n\
             Cache-Control: no-store, no-cache, must-revalidate, max-age=0\r\n\
             Pragma: no-cache\r\n\
             Expires: 0\r\n\
             Connection: close\r\n\
             Access-Control-Allow-Origin: *\r\n\
             \r\n\
             {body}",
            body.len()
        );

        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.flush().await;
    }
}
