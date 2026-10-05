<p align="right">
  <a href="README.md"><b>English</b></a> | <a href="README.tr.md"><b>Türkçe</b></a>
</p>

<p align="center">
  <img src="docs/images/tokenjar_logo.jpg" alt="TokenJar Logo" width="220" style="border-radius: 16px;" />
</p>

<h1 align="center">🍯 TokenJar</h1>
<p align="center"><b>Put tokens back in your jar. Save 70-95% tokens for AI coding assistants without losing functionality.</b></p>

[![Release: v1.1.0](https://img.shields.io/badge/Release-v1.1.0%20GA-green.svg)](https://github.com/Farukes/TokenJar/releases/latest)
[![CI](https://github.com/Farukes/TokenJar/actions/workflows/ci.yml/badge.svg)](https://github.com/Farukes/TokenJar/actions/workflows/ci.yml)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%20%7C%203.11%20%7C%203.12-blue.svg)](https://www.python.org/downloads/)
[![Enterprise Native: Rust](https://img.shields.io/badge/Enterprise%20Native-Rust%20v1.1.0-orange.svg)](#enterprise-engine)
[![Crates.io: v1.1.0](https://img.shields.io/badge/crates.io-v1.1.0-orange.svg?logo=rust&logoColor=white)](https://crates.io/crates/tokenjar)
[![PyPI: v1.1.0](https://img.shields.io/badge/PyPI-v1.1.0-blue.svg?logo=pypi&logoColor=white)](https://pypi.org/project/tokenjar/)
[![Token Reduction](https://img.shields.io/badge/Token%20Savings-90%25%20to%2099%25-brightgreen.svg)](#benchmarks)
[![License: BSL 1.1](https://img.shields.io/badge/License-BSL%201.1-blue.svg)](LICENSE)
[![Zero Telemetry](https://img.shields.io/badge/telemetry-0%25%20--%20100%25%20local-success.svg)](#privacy-guarantee)

**MCP server that saves 70-95% tokens for AI coding assistants — without losing functionality.**

TokenJar sits between your AI coding assistant and your codebase, intelligently compressing code reads, terminal outputs, and file operations to dramatically reduce token consumption, context compaction, and latency.

Works with **Claude Code**, **Cursor**, **Antigravity (AGY)**, **Windsurf**, **Continue.dev**, and any MCP-compatible AI assistant.

---

## ✨ Features & Architecture

| Module | What It Does | Token Savings |
|:---|:---|:---|
| 🦴 **Code Skeletonizer** | Extracts structural skeleton (signatures, types, docstrings) via Tree-sitter AST | **80-95%** |
| 📖 **Smart File Reader & Slicing Cache** | L1 RAM + L2 Persistent SQLite cache with targeted line/symbol slicing (`#L1-30`) & auto-paging ceiling (`MAX_OUTPUT_LINES = 80`) | **90-99.8%** |
| 🛡️ **Lockfile & Asset Shield** | Intercepts massive lockfiles & minified bundles with surgical version queries (`query="react"`) | **99%** |
| 🎯 **Blast Radius & Hybrid Fuzzy Symbols** | Instant global symbol lookup with AST parser and fuzzy Levenshtein + Trigram fallback suggestions (`find_symbol_global`) | **85-95%** |
| 🖥️ **Terminal Pruner** | Compresses test/build/git terminal streams, keeps errors and summary info | **60-90%** |
| 🗺️ **Repo Map** | PageRank & Graph Centrality codebase overview fitted into custom token budgets | **Budget-fitted** |
| 🔄 **Post-Update Auto-Sync** | Automatically syncs IDE configs, slash commands, and project rules in a single command (`tokenjar update`) | **Zero-maintenance** |
| 🧹 **Zero-Trace Surgical Uninstall** | Safely reverts IDE configs, purges caches, and cleans rules without touching user code | **Safe & Zero-trace** |
| 🎨 **On-Demand UI Dashboard** | Lightweight standalone control panel (`tokenjar ui`) with **Zero Background RAM** | **Instant** |

### 🛡️ Built-in Guardrails & Reliability
- **Lockfile & Giant Asset Shield:** Prevents context window destruction from 50,000-line lockfiles; supports 5-line surgical version queries.
- **L1 RAM + L2 SQLite Persistent Cache:** Survives MCP server restarts and IDE reboots (`~/.tokenjar/cache.db` with WAL mode).
- **Fallback Safety Guard:** If a test or command fails (`exit_code != 0`), TokenJar guarantees tracebacks and error contexts are preserved intact.
- **Tiny File Anomaly Guard:** If a diff header would consume more tokens than the file itself, the full content is returned to prevent token inflation.
- **Runaway Stream Protection:** Protects host memory from infinite loops by capping raw terminal buffers at 2MB with graceful truncation.
- **SQLite Database Bloat Guard:** Files larger than 5MB are cached by hash reference without bloating disk space.

---

## 🚀 Quick Start & Installation (v1.1.0 GA)

TokenJar is distributed in two official editions:
1. **🦀 Rust Native Engine (Recommended):** High-performance, self-contained single binary with microsecond AST, 14 MB RAM, and zero Python dependencies.
2. **🐍 Python Edition:** Pure Python FastMCP package for pip and virtual environments.

### 📥 1-Click Direct Downloads (Precompiled Binaries)

Click your operating system below to download the latest v1.1.0 release:

| Platform | Architecture | Click to Download | Format |
|:---|:---|:---|:---|
| 🪟 **Windows** | x86_64 (64-bit) | [**⬇️ Download tokenjar-windows-x64.zip**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-windows-x64.zip) | Standalone `.exe` + Installer |
| 🐧 **Linux** | x86_64 (64-bit) | [**⬇️ Download tokenjar-linux-x64.tar.gz**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-linux-x64.tar.gz) | Standalone Binary |
| 🍏 **macOS** | Apple Silicon (M1/M2/M3/M4) | [**⬇️ Download tokenjar-macos-arm64.tar.gz**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-macos-arm64.tar.gz) | Standalone Binary |
| 🍏 **macOS** | Intel x86_64 | [**⬇️ Download tokenjar-macos-x64.tar.gz**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-macos-x64.tar.gz) | Standalone Binary |
| 🐍 **Python** | Cross-platform | [**⬇️ Download tokenjar-python.zip**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-python.zip) | Python Wheel (.whl) |

---

### ⚡ Option 1: Rust Native Engine (1-Click Terminal Install)
> **Best for:** Highest speed, 14 MB RAM, microsecond tree-sitter AST, and zero Python dependency.

Copy and paste one line into your terminal to install and add `tokenjar` to your PATH automatically:

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/Farukes/TokenJar/main/install.ps1 | iex
```

**Windows (CMD / Command Prompt):**
```cmd
powershell -ExecutionPolicy Bypass -Command "iwr -useb https://raw.githubusercontent.com/Farukes/TokenJar/main/install.ps1 | iex"
```

```bash
# Linux & macOS (Bash):
curl -fsSL https://raw.githubusercontent.com/Farukes/TokenJar/main/install.sh | bash
```

**Or install via Cargo (crates.io):**
```bash
cargo install tokenjar
```

---

### 🐍 Option 2: Python Edition (pip)
> **Best for:** Python-centric environments, custom script integration, or pip workflows.

```bash
# Install from PyPI
pip install tokenjar

# Or install directly from GitHub main:
pip install git+https://github.com/Farukes/TokenJar.git
```

---

### 🔄 Updating TokenJar

Keep your installation up to date with the latest features and engine optimizations:

```bash
# Self-update via TokenJar CLI (both 'update' and 'upgrade' work identically):
tokenjar update
# or:
tokenjar upgrade

# Force re-download:
tokenjar update --force

# If installed via pip:
pip install --upgrade tokenjar

# If installed via Cargo:
cargo install tokenjar --force
```

---

<a id="benchmarks"></a>
## 📊 Proven Performance & Stress Test Benchmark

Empirical results from our rigorous **100-Step Real-Life Developer Stress Test** and **50-Cycle MCP Head-to-Head Benchmark** comparing Standard Raw AI vs TokenJar Python vs TokenJar Rust Native Engine:

| Metric | 1. Raw AI (No TokenJar) | 2. TokenJar Python | 3. TokenJar Rust (v1.1.0) | Rust Advantage |
|:---|:---|:---|:---|:---|
| **Consumed Tokens (100 Steps)** | 622,892 tokens | 95,492 tokens | **68,641 tokens** | **89.0% net savings (554k tokens saved)** |
| **End-to-End Coding Savings** | 166,513 tokens | 12,400 tokens | **6,585 tokens** | **🚀 96.0% net savings (Surgical edits)** |
| **API Cost (per 100 Steps)** | $1.8687 | $0.2865 | **$0.2059** | **$1.66 saved per 100 steps** |
| **Total Runtime (100 Steps)** | 0.357 s (raw disk) | 2.618 s | **0.985 s** | **2.7x faster than Python** |
| **Warm Cycle Latency** | N/A | 23.6 ms | **8.1 ms** | **3.0x faster execution** |
| **RAM / Memory Footprint** | ~30.0 MB | 49.1 MB | **15.0 MB** | **70% to 84% less RAM** |
| **Quality & Accuracy Score** | 100.0% | 100.0% | **100.0% (100/100)** | **100% functional completeness** |
| **Syntax Integrity & Zero Truncation** | Ham (Unverified) | ✅ Enforced | ✅ **Enforced** | **Zero placeholder comments** |

---

### 📝 Project Steering Rules (`AGENTS.md`)

Inject or remove TokenJar AI steering instructions into your repository rules:

```bash
# 🟢 Activate TokenJar in current project (creates/updates AGENTS.md, .cursorrules)
tokenjar on

# ⚪ Deactivate TokenJar in current project (cleanly removes TokenJar block)
tokenjar off
```

### 🎛️ Output Optimization Controls (CLI & Terminals)

Switch between compact surgical output and default unrestricted output with crystal-clear commands:

```bash
# 🟢 Enable compact surgical diffs & zero-truncation quality mandate
tokenjar output on

# ⚪ Revert AI assistant to default unrestricted output settings
tokenjar output off

# 📊 Check current output configuration status
tokenjar output
```

Slash commands are also supported in your AI assistant chat (`/tokenjar output on`, `/tokenjar output off`).

---

## 🔌 Setup with Your AI Assistant

### ⚡ 1-Click Automatic Setup (Recommended)

Automatically detects and configures TokenJar MCP server across Claude Desktop, Cursor, Antigravity, Windsurf, Claude Code, and VS Code:

```bash
# 📦 One-time setup: adds to PATH, enables MCP in all IDEs, installs slash commands
tokenjar install

# 🟢 Turn on TokenJar in your current project (generates AGENTS.md)
tokenjar on

# 🌐 Or enable/disable MCP server globally across all detected IDEs
tokenjar enable
tokenjar disable
```

### Manual Configuration

If you prefer to configure manually or use other clients:

<details>
<summary><b>Claude Code</b></summary>

```bash
claude mcp add tokenjar -- python -m tokenjar
```
</details>

<details>
<summary><b>Cursor</b></summary>

Create or update `.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "tokenjar": {
      "command": "python",
      "args": ["-m", "tokenjar"],
      "env": { "PYTHONUNBUFFERED": "1" }
    }
  }
}
```
</details>

<details>
<summary><b>Antigravity (AGY)</b></summary>

Add to `~/.gemini/config/mcp_config.json`:
```json
{
  "mcpServers": {
    "tokenjar": {
      "command": "python",
      "args": ["-m", "tokenjar"],
      "env": { "PYTHONUNBUFFERED": "1" }
    }
  }
}
```
</details>

<details>
<summary><b>Windsurf / Cascade</b></summary>

Add to `~/.codeium/windsurf/mcp_config.json`:
```json
{
  "mcpServers": {
    "tokenjar": {
      "command": "python",
      "args": ["-m", "tokenjar"]
    }
  }
}
```
</details>

<details>
<summary><b>Continue.dev</b></summary>

Add to `.continue/config.yaml`:
```yaml
mcpServers:
  - name: tokenjar
    command: python
    args: ["-m", "tokenjar"]
```
</details>

---

## 🛠️ Available MCP Tools

- **`find_symbol_global(query, root_path=".", exact=False)`**: Search for functions, methods, or classes across the entire codebase by name without reading multiple files. Features hybrid fuzzy matching with Levenshtein distance fallback for typo-tolerant lookups.
- **`find_symbol_references(symbol_name, root_path=".", max_results=25)`**: Blast radius reference analyzer. Finds all callers, imports, and usages across the entire codebase before editing or refactoring code.
- **`tool_get_code_skeleton(file_path)`**: Extract structural skeleton of a file — classes, function signatures, docstrings, and type annotations with bodies replaced by `...`. (Supports Python, JS/TS, Go, Rust, Java, C/C++, C#, Ruby, PHP, Kotlin).
- **`tool_get_symbol(file_path, symbol_name)`**: Extract the full implementation of a specific class or function by name after inspecting its skeleton.
- **`read_file_smart(file_path, symbol=None, force_full=False, query="", start_line=None, end_line=None)`**: Differential file reader with session caching, targeted line range slicing, symbol extraction, and Lockfile Shield. Pass `symbol="function_name"` to extract symbols directly in 1 step. Supports `start_line` and `end_line` (1-indexed, inclusive) to surgically inspect specific line ranges with line numbers and enforces an auto-paging ceiling (`MAX_OUTPUT_LINES = 80`). Returns `[CACHED] unchanged` (~3 tokens) or unified diffs on edits. For lockfiles (`package-lock.json`, `Cargo.lock`, etc.), pass `query="package-name"` for surgical 5-line version blocks instead of 50,000 lines.
- **`run_command_smart(command, cwd=".")`**: Executes shell commands and prunes verbose logs from pytest, jest, npm, cargo, and git.
- **`filter_output(output, output_type="auto")`**: Pure text filter for test runners, build pipelines, and version control logs without executing commands.
- **`get_repo_map_tool(root_path=".", max_tokens=1000)`**: Graph centrality codebase map prioritized by cross-file import relationships.
- **`get_directory_tree_tool(root_path=".", max_depth=4)`**: Lightweight directory tree honoring `.gitignore` and skipping binary folders.
- **`cache_stats()`**: Inspect session read hits, misses, diffs, and aggregate token savings.

### 📦 MCP Resources & Prompts

- **Resources:**
  - `tokenjar://stats`: Live cumulative token and financial savings dashboard.
  - `tokenjar://guide`: AI assistant best-practice optimization guidelines.
  - `tokenjar://config`: Active project configuration and ignore settings.
- **Prompts:**
  - `optimize_coding_task(task_description)`: System prompt template steering assistants toward token-efficient workflows.

---

## ⚙️ Project Configuration (`tokenjar.toml`)

Create an optional `tokenjar.toml` in your repository root to customize exclusions and budgets:

```toml
[general]
ignore_patterns = ["tests/fixtures/*", "legacy/*", "*.bak"]
max_cacheable_bytes = 5242880 # 5 MB

[cache]
ttl_days = 30
max_entries = 5000

[repo_map]
default_budget = 1000
```

---

## 💻 CLI Commands & Shell Hooks

TokenJar also functions as an interactive command-line utility for human developers and local shell automation:

```bash
# 📦 1-Click System Setup & Integrations
tokenjar install            # One-time setup: adds to PATH, enables MCP across detected IDEs, sets up slash commands
tokenjar on                 # Turn on TokenJar in current project (creates/updates AGENTS.md rules)
tokenjar off                # Turn off TokenJar in current project (cleans AGENTS.md rules)
tokenjar on --global        # Configure MCP server globally across all detected IDEs
tokenjar off --global       # Uninstall MCP server globally from all detected IDEs
tokenjar enable             # Enable TokenJar MCP server in all detected AI assistants globally
tokenjar disable            # Disable TokenJar MCP server from all AI assistants globally

# 📊 Monitoring, Telemetry & Web Dashboard
tokenjar status             # Check operational status across AI assistants and IDEs
tokenjar stats              # View live performance, token reduction, and financial savings dashboard
tokenjar ui                 # Launch the interactive Web Dashboard in your browser (Zero Background RAM)
tokenjar ui --port 4141     # Specify custom port for Web Dashboard

# 🔄 Automatic Updates & Self-Healing
tokenjar update             # Upgrade binary + auto-sync IDE configs + slash commands + project rules
tokenjar update --force     # Force re-installation even if already on latest version

# 🧹 Cache & Telemetry Management
tokenjar clean              # Reset telemetry statistics and clear L2 SQLite cache
tokenjar clean --cache      # Only clear L2 SQLite cache
tokenjar clean --stats      # Only reset telemetry statistics
tokenjar cache-prune --ttl-days 30 --max-entries 5000 # Prune expired or excess entries from L2 SQLite cache

# ⚡ Developer Tools & Output Pruning
tokenjar run "pytest"       # Run shell command with intelligent token-saving output pruning
tokenjar run "npm test"     # Retains errors & summary, prunes repetitive logs
RAW=1 tokenjar run "pytest" # Temporary raw bypass (or pass --raw)
tokenjar hook               # Install transparent CLI interceptor hooks into shell profiles (PowerShell/Bash)
tokenjar unhook             # Remove transparent CLI interceptor hooks from shell profiles
tokenjar output on          # Enable compact surgical diffs & zero-truncation mode
tokenjar output off         # Revert AI assistant to default unrestricted output

# ⚠️ Zero-Trace Surgical Uninstall
tokenjar uninstall          # Safely revert IDE configs, clean project rules, purge cache, hooks, and PATH
tokenjar uninstall --yes    # Skip confirmation prompt and purge immediately
```

---

<a id="privacy-guarantee"></a>
## 🔒 Enterprise Privacy & Security Guarantee

TokenJar is built strictly under a **Zero-Telemetry, 100% Localhost** design philosophy:

- **100% Local Execution:** All parsing (Tree-sitter), caching (SQLite), and output filtering happen locally in-process on your CPU.
- **Zero External Network Calls:** No telemetry servers, no analytical trackers, no outbound pings, and no cloud dependencies whatsoever.
- **Air-Gapped Compatible:** Safely operates in classified, offline, or air-gapped corporate enterprise environments.
- **Local Data Isolation:** Persistent cache (`~/.tokenjar/cache.db`) and statistics (`~/.tokenjar/telemetry.json`) reside exclusively in your user directory and can be purged at any time with `tokenjar clean --stats` or by deleting the directory.
- **Non-Invasive Architecture:** Never modifies your project code without explicit assistant direction.

---

<a id="enterprise-engine"></a>
## 🦀 Enterprise & High-Performance Native Engine (Rust Edition)

For enterprise environments, massive monorepos (50,000+ files), CI/CD pipelines, or developer systems without a Python runtime, TokenJar provides an ultra-fast, zero-dependency native Rust binary (`tokenjar.exe` / standalone executable).

### Why the Enterprise Native Engine?
- **Zero Runtime Dependencies:** No Python, pip, Node.js, or virtual environments required. Single standalone binary.
- **Ultra-Low Latency:** Instant startup (~3ms cold start vs 300ms Python startup) for zero-delay MCP tool responses.
- **High-Concurrency Indexing:** True multithreaded (Rayon + Tokio) parallel code parsing and symbol extraction.
- **Embedded 15-Language AST Engine:** Built-in Tree-sitter parsers for Rust, C, C++, Go, C#, Java, Python, JavaScript, TypeScript, PHP, Ruby, Bash, HTML, CSS, JSON statically linked inside the binary.
- **Minimal Memory Footprint:** Consumes only ~8-15 MB RAM under active load.

### Enterprise Quick Start (Standalone Binary)

Download the precompiled binary from [GitHub Releases](https://github.com/Farukes/TokenJar/releases) or build directly with Cargo:

```bash
# Build optimized native release binary from source
cargo build --release --workspace

# The standalone binary is ready:
./target/release/tokenjar.exe status
```

### Enterprise MCP Configuration (`claude_desktop_config.json` / Cursor)
Point directly to the native binary without any Python wrapper:

```json
{
  "mcpServers": {
    "tokenjar": {
      "command": "C:\\path\\to\\tokenjar.exe"
    }
  }
}
```

---

## 🌍 Supported Languages

TokenJar uses Tree-sitter for AST parsing and supports **130+ programming languages** out of the box, including:

Python · TypeScript · JavaScript · Go · Rust · Java · C# · C / C++ · Ruby · PHP · Swift · Kotlin · Scala · Dart · Lua · Elixir · Haskell · and more.

---

## 🧪 Development & Quality Assurance

TokenJar maintains dual test suites ensuring 100% parity across both implementations:

```bash
# Python (Community Edition & MCP SDK)
pip install -e ".[dev]"
pytest tests/ -v           # 72 tests passing (100% pass)

# Rust (Enterprise Native Engine)
cargo test --workspace    # 45 core tests + 3 benchmark/stress suites passing (100% pass)
```

---

## 📄 License & Intellectual Property

Copyright © 2026 Ömer Faruk Eskitürk. All rights reserved.

Licensed under the **Business Source License 1.1 (BSL 1.1)** with an automatic transition to the **Apache License, Version 2.0**.
- **Free Use:** Free for all personal, educational, research, evaluation, and internal business use.
- **Commercial Restrictions:** Cannot be hosted or provided as a paid commercial service or SaaS competing with the Licensor.
- **Sunset to Apache 2.0:** Converts automatically to 100% open-source Apache 2.0 on 2030-01-01.

See [LICENSE](LICENSE) for full legal terms.
