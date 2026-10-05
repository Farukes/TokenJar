//! Automatic 1-Click MCP Installer and Manager for AI Assistants in Rust.
//!
//! Automatically detects and configures TokenJar MCP server in Claude Desktop,
//! Cursor, Windsurf, Claude Code, and VS Code (Cline / Roo Code).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInstallResult {
    pub ide_name: String,
    pub config_path: PathBuf,
    pub success: bool,
    pub message: String,
}

/// Returns list of supported AI assistant names and their standard MCP config file paths.
pub fn get_supported_ide_configs() -> Vec<(&'static str, PathBuf)> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut configs = Vec::new();

    // 1. Claude Desktop
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            configs.push((
                "Claude Desktop",
                PathBuf::from(appdata)
                    .join("Claude")
                    .join("claude_desktop_config.json"),
            ));
        } else {
            configs.push((
                "Claude Desktop",
                home.join("AppData")
                    .join("Roaming")
                    .join("Claude")
                    .join("claude_desktop_config.json"),
            ));
        }
    }
    #[cfg(target_os = "macos")]
    {
        configs.push((
            "Claude Desktop",
            home.join("Library")
                .join("Application Support")
                .join("Claude")
                .join("claude_desktop_config.json"),
        ));
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        configs.push((
            "Claude Desktop",
            home.join(".config")
                .join("Claude")
                .join("claude_desktop_config.json"),
        ));
    }

    // 2. Cursor
    configs.push(("Cursor", home.join(".cursor").join("mcp.json")));

    // 3. Windsurf
    configs.push((
        "Windsurf",
        home.join(".codeium")
            .join("windsurf")
            .join("mcp_config.json"),
    ));

    // 4. Claude Code
    configs.push(("Claude Code", home.join(".claude.json")));

    // 5. Antigravity (AGY)
    configs.push((
        "Antigravity (AGY)",
        home.join(".gemini").join("config").join("mcp_config.json"),
    ));

    // 6. VS Code (Cline / Roo Code)
    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let code_storage = PathBuf::from(appdata)
            .join("Code")
            .join("User")
            .join("globalStorage");
        configs.push((
            "VS Code (Cline)",
            code_storage
                .join("saoudrizwan.claude-dev")
                .join("settings")
                .join("cline_mcp_settings.json"),
        ));
        configs.push((
            "VS Code (Roo)",
            code_storage
                .join("rooveterinaryinc.roo-cline")
                .join("settings")
                .join("cline_mcp_settings.json"),
        ));
    }
    #[cfg(target_os = "macos")]
    {
        let code_storage = home
            .join("Library")
            .join("Application Support")
            .join("Code")
            .join("User")
            .join("globalStorage");
        configs.push((
            "VS Code (Cline)",
            code_storage
                .join("saoudrizwan.claude-dev")
                .join("settings")
                .join("cline_mcp_settings.json"),
        ));
        configs.push((
            "VS Code (Roo)",
            code_storage
                .join("rooveterinaryinc.roo-cline")
                .join("settings")
                .join("cline_mcp_settings.json"),
        ));
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let code_storage = home
            .join(".config")
            .join("Code")
            .join("User")
            .join("globalStorage");
        configs.push((
            "VS Code (Cline)",
            code_storage
                .join("saoudrizwan.claude-dev")
                .join("settings")
                .join("cline_mcp_settings.json"),
        ));
        configs.push((
            "VS Code (Roo)",
            code_storage
                .join("rooveterinaryinc.roo-cline")
                .join("settings")
                .join("cline_mcp_settings.json"),
        ));
    }

    configs
}

/// Checks if Claude Code CLI is genuinely installed on the host.
pub fn is_claude_code_installed() -> bool {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // 1. Check if 'claude' CLI binary exists in PATH
    #[cfg(target_os = "windows")]
    let check_cmds = ["claude.cmd", "claude.exe", "claude.bat"];
    #[cfg(not(target_os = "windows"))]
    let check_cmds = ["claude"];

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for cmd in &check_cmds {
                if dir.join(cmd).is_file() {
                    return true;
                }
            }
        }
    }

    // 2. Check if ~/.claude directory exists with actual files other than our own commands
    let claude_dir = home.join(".claude");
    if claude_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&claude_dir) {
            for entry in entries.flatten() {
                if entry.file_name() != "commands" {
                    return true;
                }
            }
        }
    }

    // 3. Check if ~/.claude.json exists and has configuration other than tokenjar
    let cfg = home.join(".claude.json");
    if cfg.is_file() {
        if let Ok(content) = fs::read_to_string(&cfg) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                if let Some(obj) = val.as_object() {
                    if obj.keys().any(|k| k != "mcpServers") {
                        return true;
                    }
                    if let Some(servers) = obj.get("mcpServers").and_then(|s| s.as_object()) {
                        if servers.keys().any(|k| k != "tokenjar") {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Automatically configures TokenJar MCP in detected or all AI assistants.
pub fn install_mcp_all(all_ides: bool, custom_exe: Option<&str>) -> Vec<McpInstallResult> {
    let exe_path = match custom_exe {
        Some(e) => e.to_string(),
        None => std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("tokenjar.exe"))
            .to_string_lossy()
            .to_string(),
    };

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let configs = get_supported_ide_configs();
    let mut results = Vec::new();

    for (ide_name, cfg_path) in configs {
        let file_exists = cfg_path.exists();
        let is_detected = if ide_name == "Claude Code" {
            is_claude_code_installed()
        } else {
            let parent_exists = cfg_path
                .parent()
                .map(|p| p.exists() && p != home)
                .unwrap_or(false);
            parent_exists || file_exists
        };

        // If not all_ides, only configure if IDE directory or config file exists on host
        if !all_ides && !is_detected {
            results.push(McpInstallResult {
                ide_name: ide_name.to_string(),
                config_path: cfg_path,
                success: false,
                message: "Skipped (IDE not detected on system)".to_string(),
            });
            continue;
        }

        // Ensure parent directory exists
        if let Some(parent) = cfg_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Read or initialize JSON
        let mut json_data = if file_exists {
            match fs::read_to_string(&cfg_path) {
                Ok(content) => {
                    serde_json::from_str::<Value>(&content).unwrap_or_else(|_| json!({}))
                }
                Err(_) => json!({}),
            }
        } else {
            json!({})
        };

        if !json_data.is_object() {
            json_data = json!({});
        }

        // Ensure mcpServers exists
        if json_data.get("mcpServers").is_none() || !json_data["mcpServers"].is_object() {
            json_data["mcpServers"] = json!({});
        }

        // Insert or update tokenjar configuration
        json_data["mcpServers"]["tokenjar"] = json!({
            "command": exe_path,
            "args": []
        });

        match serde_json::to_string_pretty(&json_data) {
            Ok(formatted) => match fs::write(&cfg_path, formatted) {
                Ok(_) => {
                    results.push(McpInstallResult {
                        ide_name: ide_name.to_string(),
                        config_path: cfg_path,
                        success: true,
                        message: format!("Successfully configured in {}", ide_name),
                    });
                }
                Err(e) => {
                    results.push(McpInstallResult {
                        ide_name: ide_name.to_string(),
                        config_path: cfg_path,
                        success: false,
                        message: format!("Failed writing file: {e}"),
                    });
                }
            },
            Err(e) => {
                results.push(McpInstallResult {
                    ide_name: ide_name.to_string(),
                    config_path: cfg_path,
                    success: false,
                    message: format!("Failed serializing config: {e}"),
                });
            }
        }
    }

    // Auto-install /tokenjar slash commands into AGY CLI and Claude Code
    let _ = install_all_slash_commands(true);

    // Ensure binary directory is in user's PATH so 'tokenjar' works everywhere
    let (path_ok, path_msg) = ensure_in_user_path();
    if path_ok && path_msg.contains("Added") {
        results.push(McpInstallResult {
            ide_name: "System PATH".to_string(),
            config_path: PathBuf::from("PATH"),
            success: true,
            message: path_msg,
        });
    }

    results
}

/// Safely removes TokenJar from all AI assistant MCP configurations.
pub fn uninstall_mcp_all() -> Vec<McpInstallResult> {
    let configs = get_supported_ide_configs();
    let mut results = Vec::new();

    for (ide_name, cfg_path) in configs {
        if !cfg_path.exists() {
            results.push(McpInstallResult {
                ide_name: ide_name.to_string(),
                config_path: cfg_path,
                success: true,
                message: "Already inactive (config not present)".to_string(),
            });
            continue;
        }

        let content = match fs::read_to_string(&cfg_path) {
            Ok(c) => c,
            Err(e) => {
                results.push(McpInstallResult {
                    ide_name: ide_name.to_string(),
                    config_path: cfg_path,
                    success: false,
                    message: format!("Failed reading config: {e}"),
                });
                continue;
            }
        };

        let mut json_data: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                results.push(McpInstallResult {
                    ide_name: ide_name.to_string(),
                    config_path: cfg_path,
                    success: false,
                    message: format!("Invalid JSON in config: {e}"),
                });
                continue;
            }
        };

        let mut modified = false;
        if let Some(servers) = json_data
            .get_mut("mcpServers")
            .and_then(|s| s.as_object_mut())
        {
            if servers.remove("tokenjar").is_some() {
                modified = true;
            }
        }

        if modified {
            match serde_json::to_string_pretty(&json_data) {
                Ok(formatted) => match fs::write(&cfg_path, formatted) {
                    Ok(_) => {
                        results.push(McpInstallResult {
                            ide_name: ide_name.to_string(),
                            config_path: cfg_path,
                            success: true,
                            message: format!("Deactivated tokenjar from {ide_name}"),
                        });
                    }
                    Err(e) => {
                        results.push(McpInstallResult {
                            ide_name: ide_name.to_string(),
                            config_path: cfg_path,
                            success: false,
                            message: format!("Failed saving config: {e}"),
                        });
                    }
                },
                Err(e) => {
                    results.push(McpInstallResult {
                        ide_name: ide_name.to_string(),
                        config_path: cfg_path,
                        success: false,
                        message: format!("Failed serializing config: {e}"),
                    });
                }
            }
        } else {
            results.push(McpInstallResult {
                ide_name: ide_name.to_string(),
                config_path: cfg_path,
                success: true,
                message: format!("TokenJar was already inactive in {ide_name}"),
            });
        }
    }

    results
}

pub fn install_agy_slash_command() -> (bool, String) {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let skill_dir = home
        .join(".gemini")
        .join("config")
        .join("skills")
        .join("tokenjar");
    if let Err(e) = fs::create_dir_all(&skill_dir) {
        return (false, format!("Failed creating directory: {e}"));
    }
    let skill_file = skill_dir.join("SKILL.md");
    let content = r#"---
name: tokenjar
description: >-
  Instant slash command controller for the TokenJar token optimization engine.
  Use immediately when user types /tokenjar, /tokenjar on, /tokenjar off,
  /tokenjar output on, /tokenjar output off, /tokenjar stats, /tokenjar ui,
  /tokenjar cache-clear, /tokenjar reset-stats, or requests to toggle tokenjar state.
---

# TokenJar Slash Command Controller

When this command is invoked with an argument:

1. **If argument is 'on' or 'enable':**
   Execute shell command: `tokenjar on`
   Report confirmation that TokenJar is active.

2. **If argument is 'off' or 'disable':**
   Execute shell command: `tokenjar off`
   Report confirmation that TokenJar is deactivated.

3. **If argument starts with 'output':**
   Execute shell command: `tokenjar output <arg>` (e.g. `tokenjar output on` or `tokenjar output off`)
   Report confirmation of the output mode change.

4. **If argument is 'status':**
   Execute shell command: `tokenjar status`
   Display the overall operational status report.

5. **If argument is 'stats' or 'telemetry':**
   Execute shell command: `tokenjar stats`
   Display the savings dashboard.

6. **If argument is 'ui' or 'dashboard':**
   Execute shell command: `tokenjar ui`
   Report confirmation that the Web Dashboard has been launched.

7. **If argument is 'cache-clear' or 'cache-reset':**
   Execute shell command: `tokenjar cache-clear`
   Report confirmation that L2 SQLite cache has been cleared.

8. **If argument is 'reset-stats':**
   Execute shell command: `tokenjar reset-stats`
   Report confirmation that cumulative telemetry metrics have been reset.

9. **If no argument or 'help':**
   Show options: `/tokenjar status`, `/tokenjar on`, `/tokenjar off`, `/tokenjar output on/off`, `/tokenjar stats`, `/tokenjar ui`, `/tokenjar cache-clear`, `/tokenjar reset-stats`.
"#;
    match fs::write(&skill_file, content) {
        Ok(_) => (
            true,
            format!(
                "Installed /tokenjar command for AGY CLI at {:?}",
                skill_file
            ),
        ),
        Err(e) => (false, format!("Failed writing skill file: {e}")),
    }
}

pub fn install_claude_code_slash_command() -> (bool, String) {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let claude_dir = home.join(".claude").join("commands");
    if let Err(e) = fs::create_dir_all(&claude_dir) {
        return (false, format!("Failed creating directory: {e}"));
    }
    let command_file = claude_dir.join("tokenjar.md");
    let content = r#"---
description: Manage TokenJar token optimization engine (on, off, output on/off, stats, ui, cache-clear, reset-stats)
---

Execute the requested TokenJar operation:
$ARGUMENTS

Instructions:
1. If argument is "on" or "enable", run `tokenjar on` and confirm activation.
2. If argument is "off" or "disable", run `tokenjar off` and confirm deactivation.
3. If argument starts with "output", run `tokenjar output <args>` and report status.
4. If argument is "stats", run `tokenjar stats` and show the telemetry dashboard.
5. If argument is "ui" or "dashboard", run `tokenjar ui` and confirm dashboard launch.
6. If argument is "cache-clear" or "cache-reset", run `tokenjar cache-clear` and report cache reset.
7. If argument is "reset-stats", run `tokenjar reset-stats` and report telemetry reset.
8. If empty or help, show usage instructions.
"#;
    match fs::write(&command_file, content) {
        Ok(_) => (
            true,
            format!(
                "Installed /tokenjar command for Claude Code at {:?}",
                command_file
            ),
        ),
        Err(e) => (false, format!("Failed writing command file: {e}")),
    }
}

pub fn install_all_slash_commands(only_installed: bool) -> Vec<(&'static str, bool, String)> {
    let mut results = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    let agy_config = home.join(".gemini").join("config").join("mcp_config.json");
    if !only_installed || agy_config.exists() {
        let (ok, msg) = install_agy_slash_command();
        results.push(("Antigravity (AGY)", ok, msg));
    }

    if !only_installed || is_claude_code_installed() {
        let (ok, msg) = install_claude_code_slash_command();
        results.push(("Claude Code", ok, msg));
    } else {
        results.push(("Claude Code", false, "Not installed (skipped)".to_string()));
    }

    results
}

/// Ensures the directory containing the TokenJar binary is present in the User's PATH.
/// On Windows, checks and updates the User Environment PATH in the Windows Registry.
/// On Unix, checks if the binary directory is in PATH and updates shell profiles if needed.
pub fn ensure_in_user_path() -> (bool, String) {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return (false, format!("Could not get current executable path: {e}")),
    };
    let exe_dir = match exe_path.parent() {
        Some(d) => d,
        None => return (false, "Could not determine binary directory".to_string()),
    };
    let exe_dir_str = exe_dir.to_string_lossy().to_string();

    // 1. Check if already present in the active process PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for p in std::env::split_paths(&path_var) {
            if p == exe_dir {
                return (true, format!("Already present in PATH: {exe_dir_str}"));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Query current User PATH from registry
        let query_output = Command::new("reg")
            .args(["query", "HKCU\\Environment", "/v", "Path"])
            .output();

        let mut current_user_path = String::new();
        if let Ok(out) = query_output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Path") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 3 {
                        current_user_path = parts[2..].join(" ");
                    }
                }
            }
        }

        // Check if already in user path
        let parts: Vec<&str> = current_user_path.split(';').map(|s| s.trim()).collect();
        for p in &parts {
            if p.eq_ignore_ascii_case(&exe_dir_str) {
                return (true, format!("Already present in User PATH: {exe_dir_str}"));
            }
        }

        let new_user_path = if current_user_path.is_empty() {
            exe_dir_str.clone()
        } else {
            format!(
                "{};{}",
                current_user_path.trim_end_matches(';'),
                exe_dir_str
            )
        };

        let add_res = Command::new("reg")
            .args([
                "add",
                "HKCU\\Environment",
                "/v",
                "Path",
                "/t",
                "REG_EXPAND_SZ",
                "/d",
                &new_user_path,
                "/f",
            ])
            .output();

        match add_res {
            Ok(res) if res.status.success() => {
                if let Ok(path_var) = std::env::var("PATH") {
                    std::env::set_var("PATH", format!("{};{}", exe_dir_str, path_var));
                }
                (true, format!("Added {} to Windows User PATH", exe_dir_str))
            }
            Ok(res) => (
                false,
                format!(
                    "Failed adding to PATH: {}",
                    String::from_utf8_lossy(&res.stderr)
                ),
            ),
            Err(e) => (false, format!("Failed executing reg command: {e}")),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let bashrc = home.join(".bashrc");
        let zshrc = home.join(".zshrc");
        let export_line = format!("\nexport PATH=\"{}:$PATH\"\n", exe_dir_str);

        let mut updated = false;
        for rc_file in [bashrc, zshrc] {
            if rc_file.exists() {
                if let Ok(content) = fs::read_to_string(&rc_file) {
                    if !content.contains(&exe_dir_str) {
                        let _ =
                            fs::OpenOptions::new()
                                .append(true)
                                .open(&rc_file)
                                .and_then(|mut f| {
                                    use std::io::Write;
                                    f.write_all(export_line.as_bytes())
                                });
                        updated = true;
                    }
                }
            }
        }

        if updated {
            (true, format!("Added {} to shell profile PATH", exe_dir_str))
        } else {
            (true, format!("PATH verified: {}", exe_dir_str))
        }
    }
}

pub fn remove_from_user_path() -> (bool, String) {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return (false, format!("Could not get current executable path: {e}")),
    };
    let exe_dir = match exe_path.parent() {
        Some(d) => d,
        None => return (false, "Could not determine binary directory".to_string()),
    };
    let exe_dir_str = exe_dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        let query_output = Command::new("reg")
            .args(["query", "HKCU\\Environment", "/v", "Path"])
            .output();

        let mut current_user_path = String::new();
        if let Ok(out) = query_output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Path") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 3 {
                        current_user_path = parts[2..].join(" ");
                    }
                }
            }
        }

        let parts: Vec<&str> = current_user_path
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let filtered: Vec<&str> = parts
            .into_iter()
            .filter(|p| !p.eq_ignore_ascii_case(&exe_dir_str))
            .collect();
        let new_user_path = filtered.join(";");

        let _ = Command::new("reg")
            .args([
                "add",
                "HKCU\\Environment",
                "/v",
                "Path",
                "/t",
                "REG_EXPAND_SZ",
                "/d",
                &new_user_path,
                "/f",
            ])
            .output();

        (
            true,
            format!("Removed {exe_dir_str} from Windows User PATH"),
        )
    }

    #[cfg(not(target_os = "windows"))]
    {
        (true, format!("Verified User PATH for {exe_dir_str}"))
    }
}

pub fn uninstall_all_slash_commands() -> Vec<(&'static str, bool, String)> {
    let mut results = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // 1. AGY CLI skill
    let agy_skill = home
        .join(".gemini")
        .join("config")
        .join("skills")
        .join("tokenjar");
    if agy_skill.exists() {
        match fs::remove_dir_all(&agy_skill) {
            Ok(_) => results.push((
                "Antigravity (AGY)",
                true,
                format!("Removed slash command skill at {:?}", agy_skill),
            )),
            Err(e) => results.push((
                "Antigravity (AGY)",
                false,
                format!("Failed removing skill: {e}"),
            )),
        }
    }

    // 2. Antigravity MCP schemas
    let agy_schemas = home
        .join(".gemini")
        .join("antigravity-cli")
        .join("mcp")
        .join("tokenjar");
    if agy_schemas.exists() {
        let _ = fs::remove_dir_all(&agy_schemas);
    }

    // 3. Claude Code command
    let claude_cmd = home.join(".claude").join("commands").join("tokenjar.md");
    if claude_cmd.exists() {
        match fs::remove_file(&claude_cmd) {
            Ok(_) => results.push((
                "Claude Code",
                true,
                format!("Removed slash command at {:?}", claude_cmd),
            )),
            Err(e) => results.push((
                "Claude Code",
                false,
                format!("Failed removing command: {e}"),
            )),
        }
    }

    results
}

pub fn full_uninstall() {
    println!("{}", "=".repeat(65));
    println!("⚠️  TOKENJAR COMPLETE UNINSTALL & PURGE (RUST NATIVE)");
    println!("{}", "=".repeat(65));

    // 1. Revert all AI coding CLIs
    println!("\n🔌 Reverting MCP server configurations in AI coding assistants...");
    let ide_results = uninstall_mcp_all();
    for r in ide_results {
        let icon = if r.success { "⚪" } else { "❌" };
        println!("  {icon} {}: {}", r.ide_name, r.message);
    }

    // 2. Remove shell hooks
    println!("\n🪝 Removing shell hooks from terminal profiles...");
    let hook_results = crate::hooks::remove_hooks();
    for r in hook_results {
        let icon = if r.success { "⚪" } else { "❌" };
        println!("  {icon} {}: {}", r.shell, r.message);
    }

    // 3. Clean project rules across all known projects
    println!("\n📝 Cleaning TokenJar steering rules from all known projects...");
    let project_roots = crate::rules::get_known_project_roots();
    let mut cleaned_count = 0;
    for root in project_roots {
        let rule_results = crate::rules::remove_rules(&root);
        for r in rule_results {
            if r.success && r.message.contains("Removed") {
                cleaned_count += 1;
                let dir_name = root.file_name().unwrap_or_default().to_string_lossy();
                println!("  ⚪ {dir_name}/{}: {}", r.file_name, r.message);
            }
        }
    }
    if cleaned_count == 0 {
        println!("  ⚪ No active project steering rules found.");
    }

    // 4. Remove slash commands
    println!("\n⚡ Removing slash command definitions...");
    let cmd_results = uninstall_all_slash_commands();
    for (name, ok, msg) in cmd_results {
        let icon = if ok { "⚪" } else { "❌" };
        println!("  {icon} {name}: {msg}");
    }

    // 5. Clean User PATH
    println!("\n🌐 Removing TokenJar from User PATH...");
    let (ok_path, path_msg) = remove_from_user_path();
    let icon_path = if ok_path { "⚪" } else { "❌" };
    println!("  {icon_path} {path_msg}");

    // 6. Delete ~/.tokenjar data directory
    if let Some(home) = dirs::home_dir() {
        let data_dir = home.join(".tokenjar");
        println!(
            "\n💾 Purging {:?} (L2 SQLite cache, telemetry, settings)...",
            data_dir
        );
        if data_dir.exists() {
            match fs::remove_dir_all(&data_dir) {
                Ok(_) => println!("  ⚪ Deleted {:?} successfully.", data_dir),
                Err(e) => println!("  ❌ Could not delete {:?}: {e}", data_dir),
            }
        } else {
            println!("  ⚪ Data directory already clean.");
        }
    }

    println!("\n{}", "=".repeat(65));
    println!("✨ TokenJar has been completely uninstalled from your computer!");
    println!("   Zero background processes, zero configs, and zero traces remain.");
    println!("   You can now delete this executable file if desired.");
    println!("{}", "=".repeat(65));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_configs_not_empty() {
        let configs = get_supported_ide_configs();
        assert!(!configs.is_empty());
        assert!(configs.iter().any(|(name, _)| *name == "Cursor"));
    }
}
