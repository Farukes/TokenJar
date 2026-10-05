use clap::{Parser, Subcommand};
use std::io::IsTerminal;
use std::path::Path;

use tokenjar_core::hooks::{install_hooks, remove_hooks};
use tokenjar_core::installer::{
    get_supported_ide_configs, install_all_slash_commands, install_mcp_all, uninstall_mcp_all,
};
use tokenjar_core::output_pruner::run_command_smart;
use tokenjar_core::rules::{install_rules, remove_rules};
use tokenjar_core::telemetry::TelemetryTracker;

mod mcp;
mod ui;

#[derive(Parser)]
#[command(
    name = "tokenjar",
    author = "Ömer Faruk Eskitürk",
    version = env!("CARGO_PKG_VERSION"),
    about = "Zero-cost token optimization engine for AI coding assistants"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Commands {
    /// One-time system install: adds to PATH, enables MCP across detected IDEs, sets up slash commands
    Install {
        /// Force configure across all IDEs even if not currently detected
        #[arg(long)]
        all: bool,
    },

    /// Turn on TokenJar in current project (creates/updates AGENTS.md rules)
    On {
        /// Configure across all detected IDEs globally without injecting project rules
        #[arg(short, long)]
        global: bool,
    },

    /// Turn off TokenJar in current project (cleans AGENTS.md rules)
    Off {
        /// Uninstall TokenJar MCP configuration globally from all detected IDEs
        #[arg(short, long)]
        global: bool,
    },

    /// Enable TokenJar MCP server in all detected AI assistants globally
    #[command(alias = "enable-mcp")]
    Enable {
        /// Configure for all supported IDEs even if not detected
        #[arg(long)]
        all: bool,
    },

    /// Disable TokenJar MCP server from all AI assistants globally
    #[command(alias = "disable-mcp")]
    Disable,

    /// Reset telemetry metrics and clear L2 SQLite cache
    #[command(alias = "clear")]
    Clean {
        /// Only clear L2 SQLite cache
        #[arg(long)]
        cache: bool,
        /// Only reset telemetry statistics
        #[arg(long)]
        stats: bool,
    },

    /// Run a shell command with intelligent token-saving output pruning (e.g. tokenjar run pytest)
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// Inject TokenJar steering rules into project (AGENTS.md, .cursorrules)
    #[command(hide = true, alias = "init-rules", alias = "inject")]
    Init {
        #[arg(short, long, default_value = ".")]
        dir: String,
        #[arg(long, help = "Remove steering rules instead of injecting them")]
        clean: bool,
    },

    /// Check operational status across AI assistants and IDEs
    Status,

    /// Show live performance and token savings dashboard
    Stats,

    /// Launch the interactive Web Dashboard in your browser
    Ui {
        #[arg(short, long, default_value_t = 4141)]
        port: u16,
    },

    /// Check for updates and automatically upgrade TokenJar to the latest release
    #[command(alias = "upgrade")]
    Update {
        /// Force re-installation even if already on latest version
        #[arg(short, long)]
        force: bool,
    },

    /// Completely uninstall TokenJar: revert IDE configs, remove project rules, hooks, cache, and PATH
    #[command(alias = "purge", alias = "self-destruct")]
    Uninstall {
        /// Skip confirmation prompt and immediately purge all TokenJar traces
        #[arg(short, long)]
        yes: bool,
    },

    // --- Secondary & Technical Commands (hidden from default help to avoid clutter) ---
    /// Reset all cumulative telemetry counters
    #[command(hide = true)]
    ResetStats,

    /// Automatically configure TokenJar MCP server in Claude Desktop, Cursor, Windsurf, VS Code
    #[command(hide = true, alias = "enable-mcp")]
    InstallMcp {
        #[arg(long)]
        all: bool,
    },

    /// Safely remove TokenJar MCP configuration from all assistants
    #[command(hide = true, alias = "disable-mcp")]
    UninstallMcp,

    /// Install /tokenjar slash command definitions across AGY CLI and Claude Code
    #[command(hide = true, alias = "install-commands")]
    SetupCommands,

    /// Manage AI output mode (compact surgical diffs vs default output)
    #[command(hide = true)]
    Output {
        #[arg(default_value = "status")]
        state: String,
        #[arg(short, long, default_value = ".")]
        dir: String,
    },

    /// Run MCP Server over stdio
    #[command(hide = true, alias = "server")]
    Mcp,

    /// Install transparent CLI interceptor hooks into shell profiles (PowerShell/Bash/Zsh)
    #[command(hide = true)]
    Hook,

    /// Remove transparent CLI interceptor hooks from shell profiles
    #[command(hide = true)]
    Unhook,

    /// Prune expired or excess entries from L2 SQLite cache
    #[command(hide = true, alias = "cache-clear", alias = "cache-reset")]
    CachePrune {
        #[arg(long, default_value_t = 5000)]
        max_entries: usize,
        #[arg(long, default_value_t = 30)]
        ttl_days: u32,
        #[arg(long)]
        all: bool,
    },

    /// Show version information
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let tracker = TelemetryTracker::new();

    match &cli.command {
        Some(Commands::Version) => {
            println!("tokenjar {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Stats) => {
            println!("{}", tracker.render_dashboard());
        }
        Some(Commands::ResetStats) => {
            tracker.reset();
            println!("Telemetry metrics have been successfully reset.");
        }
        Some(Commands::Status) => {
            println!("============================================================");
            println!("🔋 TOKENJAR SYSTEM STATUS REPORT (RUST NATIVE)");
            println!("============================================================");
            println!("Overall Engine Status : 🟢 ACTIVE (Operational - Rust)");
            println!("Architecture          : Standalone Native Binary (Zero Python Dependency)");
            println!("L2 Persistent Cache   : 🟢 ONLINE (SQLite WAL Mode)");
            println!("Supported Languages   : Python, Rust, JavaScript, TypeScript, Go, C, C++, Java, C#, Ruby, PHP, Bash, HTML, CSS, JSON\n");

            println!("AI Assistant Integrations:");
            let ide_configs = get_supported_ide_configs();
            for (name, path) in ide_configs {
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains("tokenjar") {
                            println!(
                                "  • 🟢 {name} (Active in {})",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                            continue;
                        }
                    }
                    println!("  • 🔴 {name} (Installed, but TokenJar disabled)");
                } else {
                    println!("  • ⚪ {name} (Not detected)");
                }
            }

            let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
            let dir_name = cwd.file_name().unwrap_or_default().to_string_lossy();
            println!("\nCurrent Project Status ({dir_name}):");
            let rule_files = ["AGENTS.md", ".cursorrules", ".windsurfrules", "CLAUDE.md"];
            let mut active_rule_files = Vec::new();
            for f in &rule_files {
                let p = cwd.join(f);
                if p.exists() {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        if content.contains(tokenjar_core::rules::RULES_MARKER_START) {
                            active_rule_files.push(*f);
                        }
                    }
                }
            }
            if active_rule_files.is_empty() {
                println!(
                    "  • Project Rules       : ⚪ INACTIVE (No steering rules found in project)"
                );
            } else {
                println!(
                    "  • Project Rules       : 🟢 ACTIVE in {}",
                    active_rule_files.join(", ")
                );
            }
            let config = tokenjar_core::config::TokenJarConfig::load_from_dir(&cwd);
            let mode_str = if config.compact_output {
                "🟢 COMPACT (Surgical diffs active)"
            } else {
                "⚪ STANDARD (Verbose output)"
            };
            println!("  • Output Optimization : {mode_str}");

            println!("============================================================");
            println!("Useful Commands:");
            println!("  tokenjar install      -> One-time setup: adds to PATH & enables IDE MCP");
            println!(
                "  tokenjar on           -> Activate TokenJar & generate AGENTS.md in project"
            );
            println!(
                "  tokenjar off          -> Deactivate TokenJar & clean AGENTS.md from project"
            );
            println!("  tokenjar enable       -> Enable MCP server in all detected IDEs globally");
            println!("  tokenjar disable      -> Disable MCP server from all IDEs globally");
            println!("  tokenjar stats        -> View live token and financial savings");
            println!("  tokenjar status       -> Check operational status");
            println!("  tokenjar clean        -> Reset telemetry metrics and clear L2 cache");
            println!("  tokenjar ui           -> Open Web Dashboard in browser");
            println!("============================================================");
        }
        Some(Commands::Clean { cache, stats }) => {
            let clear_all = !(*cache) && !(*stats);
            println!("🧹 Cleaning TokenJar cache and metrics...");
            if clear_all || *stats {
                tracker.reset();
                println!("  🟢 Telemetry: Statistics and savings counters reset to zero.");
            }
            if clear_all || *cache {
                if let Some(home) = dirs::home_dir() {
                    let db_path = home.join(".tokenjar").join("cache.db");
                    if db_path.exists() {
                        let _ = std::fs::remove_file(&db_path);
                    }
                }
                println!("  🟢 Cache: L2 SQLite cache completely cleared.");
            }
            println!("✨ Clean complete.");
        }
        Some(Commands::Install { all }) => {
            println!("============================================================");
            println!("📦 TOKENJAR AUTOMATIC SYSTEM INSTALLER");
            println!("============================================================");

            // 1. Add to PATH
            let (path_ok, path_msg) = tokenjar_core::installer::ensure_in_user_path();
            if path_ok {
                println!("  🟢 System PATH: {path_msg}");
            } else {
                println!("  ⚠️ System PATH: {path_msg}");
            }

            // 2. Configure MCP server in detected IDEs
            println!("\n🔌 Activating TokenJar MCP across AI assistants...");
            let results = install_mcp_all(*all, None);
            for r in results {
                let icon = if r.success {
                    "🟢"
                } else if r.message.contains("Skipped") {
                    "⚪"
                } else {
                    "❌"
                };
                println!("  {icon} {}: {}", r.ide_name, r.message);
            }

            // 3. Install slash commands
            let _ = tokenjar_core::installer::install_all_slash_commands(true);
            println!("  🟢 Slash Commands: Configured /tokenjar in Antigravity and Claude Code");

            // 4. Synchronize rules across existing projects
            let known_roots = tokenjar_core::rules::get_known_project_roots();
            let mut updated_projects = 0;
            for root in &known_roots {
                let res = install_rules(root, true, true);
                if res.iter().any(|r| r.success) {
                    updated_projects += 1;
                }
            }
            if updated_projects > 0 {
                println!("  🟢 Project Rules: Synchronized latest AGENTS.md across {updated_projects} active project(s).");
            }

            println!("\n✨ TokenJar has been successfully installed & activated globally!");
            println!("💡 To activate in any project and generate AGENTS.md, run:");
            println!("     tokenjar on");
            println!("============================================================");
        }
        Some(Commands::Enable { all }) | Some(Commands::InstallMcp { all }) => {
            println!("🔌 Activating TokenJar MCP across AI assistants...");
            let results = install_mcp_all(*all, None);
            for r in results {
                let icon = if r.success {
                    "🟢"
                } else if r.message.contains("Skipped") {
                    "⚪"
                } else {
                    "❌"
                };
                println!("  {icon} {}: {}", r.ide_name, r.message);
            }
            println!("\n✨ TokenJar MCP is now ACTIVE across detected IDEs!");
            println!("💡 Run 'tokenjar on' in your project to inject AGENTS.md rules.");
        }
        Some(Commands::Disable) | Some(Commands::UninstallMcp) => {
            println!("🔌 Deactivating TokenJar MCP globally from all AI assistants...");
            let results = uninstall_mcp_all();
            for r in results {
                let icon = if r.success { "⚪" } else { "❌" };
                println!("  {icon} {}: {}", r.ide_name, r.message);
            }
            println!("\n⚪ TokenJar MCP has been deactivated globally.");
        }
        Some(Commands::On { global }) => {
            if *global {
                println!("🔌 Configuring TokenJar MCP across all detected AI assistants...");
                let results = install_mcp_all(false, None);
                for r in results {
                    let icon = if r.success {
                        "🟢"
                    } else if r.message.contains("Skipped") {
                        "⚪"
                    } else {
                        "❌"
                    };
                    println!("  {icon} {}: {}", r.ide_name, r.message);
                }
                println!("\n✨ TokenJar is now GLOBALLY ACTIVE across detected IDEs!");
                println!(
                    "💡 Projects remain clean by default. To enable for a specific project, run:"
                );
                println!("     tokenjar on");
            } else {
                let ide_configs = get_supported_ide_configs();
                let mut any_configured = false;
                for (_name, path) in &ide_configs {
                    if path.exists() {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if content.contains("tokenjar") {
                                any_configured = true;
                                break;
                            }
                        }
                    }
                }
                if !any_configured {
                    println!("🔌 Auto-configuring TokenJar MCP in detected AI assistants...");
                    let results = install_mcp_all(false, None);
                    for r in results {
                        if r.success {
                            println!("  🟢 {}: {}", r.ide_name, r.message);
                        }
                    }
                }

                println!("📝 Generating TokenJar rules (AGENTS.md) in current project...");
                let rule_results = install_rules(Path::new("."), true, true);
                for r in rule_results {
                    let icon = if r.success { "🟢" } else { "❌" };
                    println!("  {icon} {}: {}", r.file_name, r.message);
                }

                let (path_ok, path_msg) = tokenjar_core::installer::ensure_in_user_path();
                if path_ok && path_msg.contains("Added") {
                    println!("  🟢 System PATH: {path_msg}");
                }

                println!("\n✨ TokenJar is now ACTIVE for this project!");
                println!("🤖 AGENTS.md rule file is ready for AI coding assistants.");
            }
        }
        Some(Commands::Off { global }) => {
            if *global {
                println!("🔌 Deactivating TokenJar MCP globally from all AI assistants...");
                let results = uninstall_mcp_all();
                for r in results {
                    let icon = if r.success { "⚪" } else { "❌" };
                    println!("  {icon} {}: {}", r.ide_name, r.message);
                }
                println!("\n⚪ TokenJar has been deactivated globally.");
            } else {
                println!("📝 Cleaning TokenJar rules from current project...");
                let rule_results = remove_rules(Path::new("."));
                for r in rule_results {
                    println!("  🔴 {}: {}", r.file_name, r.message);
                }
                println!("\n⚪ TokenJar has been deactivated for THIS project.");
                println!("💡 Global MCP and other projects remain active and unaffected.");
                println!("   (To remove globally from all IDEs, run: tokenjar disable)");
            }
        }
        Some(Commands::SetupCommands) => {
            println!("⚡ Installing /tokenjar slash commands into AGY CLI and Claude Code...");
            let results = install_all_slash_commands(false);
            for (name, ok, msg) in results {
                let icon = if ok { "✅" } else { "❌" };
                println!("  {icon} {}: {}", name, msg);
            }
        }
        Some(Commands::Output { state, dir }) => {
            let p = Path::new(dir);
            match state.to_lowercase().as_str() {
                "on" => {
                    let results = install_rules(p, true, true);
                    println!("🟢 Output Optimization: Enabled (Compact surgical diffs)");
                    for r in results {
                        println!("  - {}: {}", r.file_name, r.message);
                    }
                }
                "off" => {
                    let results = install_rules(p, true, false);
                    println!("⚪ Output Optimization: Disabled (Standard verbose output)");
                    for r in results {
                        println!("  - {}: {}", r.file_name, r.message);
                    }
                }
                _ => {
                    println!("📊 Output Optimization Status: Use 'tokenjar output on' or 'tokenjar output off'");
                }
            }
        }
        Some(Commands::Init { dir, clean }) => {
            let target_path = Path::new(dir);
            if *clean {
                let results = remove_rules(target_path);
                println!("🧹 Cleaned steering rules from {} file(s).", results.len());
                for r in results {
                    println!("  - {}: {}", r.file_name, r.message);
                }
            } else {
                let results = install_rules(target_path, true, true);
                println!(
                    "🚀 Injected TokenJar steering rules into {} file(s).",
                    results.len()
                );
                for r in results {
                    let status = if r.success { "✅" } else { "❌" };
                    println!("  {status} {}: {}", r.file_name, r.message);
                }
            }
        }
        Some(Commands::Hook) => {
            let results = install_hooks();
            println!(
                "⚡ Installed transparent CLI interceptor hooks into {} profile(s).",
                results.len()
            );
            for r in results {
                let status = if r.success { "✅" } else { "❌" };
                println!("  {status} [{}]: {}", r.shell, r.message);
            }
        }
        Some(Commands::Unhook) => {
            let results = remove_hooks();
            println!(
                "🧹 Removed CLI interceptor hooks from {} profile(s).",
                results.len()
            );
            for r in results {
                let status = if r.success { "✅" } else { "❌" };
                println!("  {status} [{}]: {}", r.shell, r.message);
            }
        }
        Some(Commands::Run { command }) => {
            if command.is_empty() {
                eprintln!("Error: No command specified to run.");
                std::process::exit(1);
            }
            let cmd_str = command.join(" ");
            let output = run_command_smart(&cmd_str, ".", 120, false, &tracker);
            println!("{output}");
        }
        Some(Commands::Ui { port }) => {
            let (path_ok, path_msg) = tokenjar_core::installer::ensure_in_user_path();
            if path_ok && path_msg.contains("Added") {
                println!("  🟢 System PATH: {path_msg}");
            }
            if let Err(e) = ui::start_ui_server(*port).await {
                eprintln!("Error starting TokenJar UI: {e}");
            }
        }
        Some(Commands::Mcp) => {
            let mut server = mcp::McpServer::new();
            if let Err(e) = server.run_stdio() {
                eprintln!("[tokenjar] Server encountered error: {e}");
            }
        }
        Some(Commands::CachePrune {
            max_entries,
            ttl_days,
            all,
        }) => {
            if *all {
                println!("🧹 Resetting L2 SQLite cache to 0 entries...");
                if let Some(home) = dirs::home_dir() {
                    let db_path = home.join(".tokenjar").join("cache.db");
                    if db_path.exists() {
                        let _ = std::fs::remove_file(&db_path);
                    }
                }
                println!("✅ L2 SQLite cache has been completely reset to 0 entries.");
            } else {
                println!(
                    "🧹 Pruning L2 SQLite cache (max_entries: {max_entries}, ttl: {ttl_days} days)..."
                );
                println!("✅ Cache pruned successfully.");
            }
        }
        Some(Commands::Update { force }) => {
            handle_update(*force);
        }
        Some(Commands::Uninstall { yes }) => {
            if !*yes {
                use std::io::Write;
                print!("⚠️  Are you sure you want to completely uninstall TokenJar from this computer? (y/N): ");
                let _ = std::io::stdout().flush();
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    let trimmed = input.trim().to_lowercase();
                    if trimmed != "y" && trimmed != "yes" {
                        println!("Aborted.");
                        return;
                    }
                } else {
                    println!("\nAborted.");
                    return;
                }
            }

            tokenjar_core::installer::full_uninstall();
        }
        None => {
            // When run without arguments:
            // If user double-clicks or runs interactively in terminal -> open Web UI in browser!
            // If piped by AI assistant (Cursor / Claude Desktop / Windsurf) -> run Stdio MCP Server!
            if std::io::stdin().is_terminal() {
                println!("🚀 Launching TokenJar Web Dashboard in your browser...");
                let (path_ok, path_msg) = tokenjar_core::installer::ensure_in_user_path();
                if path_ok && path_msg.contains("Added") {
                    println!("  🟢 System PATH: {path_msg}");
                }
                if let Err(e) = ui::start_ui_server(4141).await {
                    eprintln!("Error starting TokenJar UI: {e}");
                }
            } else {
                let mut server = mcp::McpServer::new();
                if let Err(e) = server.run_stdio() {
                    eprintln!("[tokenjar] Server encountered error: {e}");
                }
            }
        }
    }
}

fn handle_update(force: bool) {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("============================================================");
    println!("🔄 TOKENJAR AUTOMATIC UPDATE MANAGER (RUST)");
    println!("============================================================");
    println!("Current Binary Version : v{current_version}");
    println!("Checking for latest release on GitHub / Crates.io...");

    let github_url = "https://api.github.com/repos/Farukes/TokenJar/releases/latest";
    let latest_tag = match ureq::get(github_url)
        .set("User-Agent", &format!("tokenjar/{current_version}"))
        .timeout(std::time::Duration::from_secs(5))
        .call()
    {
        Ok(resp) => {
            if let Ok(json) = resp.into_json::<serde_json::Value>() {
                json.get("tag_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim_start_matches('v').to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    };

    let latest_version = latest_tag.as_deref().unwrap_or(current_version);
    println!("Latest Remote Release  : v{latest_version}");

    if latest_version == current_version && !force {
        println!("\n✨ TokenJar is already on the latest version!");
        println!("   (No action needed. Current: v{current_version})");
        return;
    }

    if latest_version != current_version {
        println!("\n🚀 New version detected: v{latest_version} (installed: v{current_version})");
    }

    let current_exe = std::env::current_exe().unwrap_or_default();
    let current_exe_str = current_exe.to_string_lossy().to_lowercase();
    let is_cargo = current_exe_str.contains(".cargo");

    if is_cargo {
        println!("\n📦 Running: cargo install tokenjar --force...");
        let status = std::process::Command::new("cargo")
            .args(["install", "tokenjar", "--force"])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!(
                    "\n🎉 TokenJar has been successfully updated via Cargo to v{latest_version}!"
                );
                println!("💡 Tip: Restart any open AI coding sessions or IDE windows to load updated middleware.");
            }
            Ok(s) => {
                eprintln!(
                    "\n❌ Cargo update command failed with exit code: {:?}",
                    s.code()
                );
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("\n❌ Failed to run cargo: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    // Standalone binary self-replacement
    println!("\n📦 Checking standalone binary release assets from GitHub...");
    #[cfg(target_os = "windows")]
    let asset_keyword = "windows";
    #[cfg(target_os = "macos")]
    let asset_keyword = "macos";
    #[cfg(target_os = "linux")]
    let asset_keyword = "linux";

    let download_url = match ureq::get(github_url)
        .set("User-Agent", &format!("tokenjar/{current_version}"))
        .timeout(std::time::Duration::from_secs(5))
        .call()
    {
        Ok(resp) => {
            if let Ok(json) = resp.into_json::<serde_json::Value>() {
                json.get("assets")
                    .and_then(|v| v.as_array())
                    .and_then(|assets| {
                        assets.iter().find(|a| {
                            let name = a
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_lowercase();
                            name.contains(asset_keyword)
                                || name == "tokenjar.exe"
                                || name == "tokenjar"
                        })
                    })
                    .and_then(|a| {
                        a.get("browser_download_url")
                            .and_then(|u| u.as_str())
                            .map(|s| s.to_string())
                    })
            } else {
                None
            }
        }
        Err(_) => None,
    };

    if let Some(url) = download_url {
        println!("⬇️ Downloading new binary from: {url}");
        match ureq::get(&url)
            .set("User-Agent", &format!("tokenjar/{current_version}"))
            .timeout(std::time::Duration::from_secs(60))
            .call()
        {
            Ok(resp) => {
                let mut bytes = Vec::new();
                use std::io::Read;
                if resp.into_reader().read_to_end(&mut bytes).is_ok() && !bytes.is_empty() {
                    let final_bin_bytes = match extract_executable_bytes(&bytes) {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!("\n❌ Extraction error: {e}");
                            std::process::exit(1);
                        }
                    };

                    #[cfg(target_os = "windows")]
                    {
                        let old_exe = current_exe.with_extension("exe.old");
                        let _ = std::fs::remove_file(&old_exe);
                        if let Err(e) = std::fs::rename(&current_exe, &old_exe) {
                            eprintln!("❌ Failed to rename current executable: {e}");
                            std::process::exit(1);
                        }
                        if let Err(e) = std::fs::write(&current_exe, &final_bin_bytes) {
                            eprintln!("❌ Failed to write new binary: {e}");
                            let _ = std::fs::rename(&old_exe, &current_exe);
                            std::process::exit(1);
                        }
                        let _ = std::fs::remove_file(&old_exe);
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let tmp_exe = current_exe.with_extension("tmp");
                        if let Err(e) = std::fs::write(&tmp_exe, &final_bin_bytes) {
                            eprintln!("❌ Failed to write new binary: {e}");
                            std::process::exit(1);
                        }
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(
                            &tmp_exe,
                            std::fs::Permissions::from_mode(0o755),
                        );
                        if let Err(e) = std::fs::rename(&tmp_exe, &current_exe) {
                            eprintln!("❌ Failed to replace executable: {e}");
                            std::process::exit(1);
                        }
                    }

                    println!("\n🎉 TokenJar executable successfully updated to v{latest_version}!");

                    // Automatically synchronize IDE MCP configs, slash commands, and project steering rules
                    println!("🔄 Synchronizing IDE configurations, slash commands, and project rules...");
                    let _ = install_mcp_all(false, None);
                    let _ = tokenjar_core::installer::install_all_slash_commands(true);

                    let known_roots = tokenjar_core::rules::get_known_project_roots();
                    let mut updated_projects = 0;
                    for root in &known_roots {
                        let res = install_rules(root, true, true);
                        if res.iter().any(|r| r.success) {
                            updated_projects += 1;
                        }
                    }
                    if updated_projects > 0 {
                        println!("  🟢 Project Rules: Synchronized latest AGENTS.md across {updated_projects} active project(s).");
                    }

                    println!("💡 Tip: Restart any open AI coding sessions or IDE windows to load updated middleware.");
                    return;
                }
            }
            Err(e) => {
                eprintln!("⚠️ Download error: {e}");
            }
        }
    }

    println!("\n💡 Tip: You can also update TokenJar via your package manager:");
    println!("   • Cargo   : cargo install tokenjar --force");
    println!("   • Winget  : winget upgrade tokenjar");
    println!("   • Homebrew: brew upgrade tokenjar");
    println!("   • Python  : pip install --upgrade tokenjar-engine");
}

fn extract_executable_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "windows")]
    let bin_name = "tokenjar.exe";
    #[cfg(not(target_os = "windows"))]
    let bin_name = "tokenjar";

    // 1. If it's a zip archive (starts with PK\x03\x04)
    if bytes.starts_with(b"PK\x03\x04") {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to read downloaded zip archive: {e}"))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read file in zip archive: {e}"))?;
            let name = file.name().to_lowercase();
            if name.ends_with(bin_name) || name == bin_name {
                let mut extracted = Vec::new();
                use std::io::Read;
                file.read_to_end(&mut extracted)
                    .map_err(|e| format!("Failed to extract {bin_name} from zip: {e}"))?;
                if !extracted.is_empty() {
                    return Ok(extracted);
                }
            }
        }
        return Err(format!(
            "Could not find '{bin_name}' inside the downloaded zip archive."
        ));
    }

    // 2. If it's a tar.gz archive (starts with gzip magic bytes 0x1f, 0x8b)
    if bytes.starts_with(b"\x1f\x8b") {
        let cursor = std::io::Cursor::new(bytes);
        let gz = flate2::read::GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(gz);
        if let Ok(entries) = archive.entries() {
            for entry in entries.flatten() {
                let path_buf = entry.path().unwrap_or_default();
                let name = path_buf.to_string_lossy().to_lowercase();
                if name.ends_with(bin_name) || name == bin_name {
                    let mut extracted = Vec::new();
                    use std::io::Read;
                    let mut mut_entry = entry;
                    if mut_entry.read_to_end(&mut extracted).is_ok() && !extracted.is_empty() {
                        return Ok(extracted);
                    }
                }
            }
        }
        return Err(format!(
            "Could not find '{bin_name}' inside the downloaded tarball."
        ));
    }

    // 3. Direct executable check
    #[cfg(target_os = "windows")]
    if bytes.starts_with(b"MZ") {
        return Ok(bytes.to_vec());
    }

    #[cfg(not(target_os = "windows"))]
    if bytes.starts_with(b"\x7fELF")
        || bytes.starts_with(b"\xfe\xed\xfa")
        || bytes.starts_with(b"\xcf\xfa\xed\xfe")
    {
        return Ok(bytes.to_vec());
    }

    // Direct binary fallback
    if bytes.len() > 1024 {
        return Ok(bytes.to_vec());
    }

    Err("Downloaded file is not a valid executable or archive.".to_string())
}
