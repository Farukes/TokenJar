# 🍯 Changelog & Release Notes

All notable changes to the **TokenJar** optimization engine across both the Native Rust Core (`crates/tokenjar-core`, `crates/tokenjar-cli`) and Python Engine (`tokenjar-engine`) are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.1.0] - 2026-10-06 (AI Token Optimizer — GA Release)

### 🚀 Highlights & Major Features

- **Phase 1: Slicing Session Cache & Auto-Paging Ceiling:**
  - Targeted file slices (`start_line`/`end_line` and `symbol`) now generate fragment keys (`#L{start}-{end}` and `#sym:{name}`) in both L1 RAM and L2 SQLite persistent cache.
  - Repeated reads of sliced functions or line ranges return instant compact `[CACHED]` diffs with **99.8% token savings** and `<1ms` latency.
  - Introduced automatic pagination ceiling (`MAX_OUTPUT_LINES = 80`) with continuation links (`start_line=81`), completely preventing LLM context compaction and IDE disk dumping (`output.txt`).
  - Standardized reduction typography to `(97.5% optimized reduction)`.

- **Phase 2: Hybrid Fuzzy Symbol Search & AST Typo Tolerance:**
  - Integrated hybrid fallback symbol resolution combining Tree-sitter AST queries with Levenshtein distance, case-normalization, and Trigram Dice similarity.
  - Zero performance overhead: fuzzy matching activates only when exact/substring match count is zero.
  - Automatically suggests `💡 Did you mean: ...?` with similarity scores, file paths, line numbers, and exact code signatures.
  - Applied fuzzy suggestions to both global symbol discovery (`find_symbol_global`) and file-level inspections (`get_symbol` / `get_symbol_file`).

- **Post-Update Auto-Sync Engine:**
  - Executing `tokenjar update` (or `tokenjar install`) now automatically refreshes IDE configurations across all detected assistants (Cursor, Claude Desktop, Antigravity, Windsurf, VS Code).
  - Automatically regenerates slash command definitions (`/tokenjar` in AGY and Claude Code).
  - Automatically scans all registered projects in `~/.tokenjar/projects.json` and updates existing `AGENTS.md` and `.cursorrules` steering blocks in-place with the latest guidelines.

- **Universal Zero-Trace Surgical Uninstall:**
  - `tokenjar uninstall` (or `tokenjar purge`) guarantees complete, zero-trace removal.
  - Safely reverts IDE MCP configurations, restores backups (`.ts_bak`), removes terminal hooks, and purges `~/.tokenjar` SQLite databases.
  - Surgically removes TokenJar steering rules from `AGENTS.md` without ever touching or modifying user custom instructions or source code.
  - Backward-compatible cleanup across all legacy prototypes and version tags.

- **Modernized CLI Commands & Migration Suite:**
  - Streamlined developer CLI commands with clean, intuitive semantics (full backward compatibility preserved via aliases):
    - `tokenjar on` / `off`: Replaces `init-rules` / `inject`. Toggles `AGENTS.md` rules per project.
    - `tokenjar on --global` / `off --global`: Replaces `install-mcp` / `uninstall-mcp`. Configures MCP globally across all IDEs.
    - `tokenjar enable` / `disable`: Explicit IDE MCP registration toggle.
    - `tokenjar clean`: Unified cache & metrics management (supports `--cache` and `--stats`).
    - `tokenjar update`: Self-updater that upgrades binary, re-syncs IDEs, and updates project rules.
    - `tokenjar uninstall`: Zero-trace uninstaller.
    - `tokenjar output on` / `off`: Toggles compact diffs and zero-truncation mandate.

### 🧪 Benchmarks & Stress Test Metrics

- **50-Step Real-Life Developer Scenario:**
  - 50/50 steps passed (100% success rate) in **2.17 seconds** (43.4 ms/step).
  - Raw Tokens Processed: **150,060 tokens** -> Delivered to Model: **4,641 tokens**.
  - **Net Token Reduction: 96.9%** (145,419 tokens saved in a single session).
  - Zero assertion errors, zero data loss, 100% Zero-Truncation Guarantee.
- **100-Step Extreme Stress Test:**
  - SQLite WAL Multi-threaded Contention: **407 ops/sec** across 12 worker threads.
  - AST Skeletons: **0.10ms – 0.42ms** per file.
  - Terminal Stream Pruning: **4.40ms** on 1,500-line logs.

---

## [1.0.3] - 2026-09-26

- Hardened macOS cross-compilation matrix in CI/CD.
- Enhanced lockfile streaming parser for pnpm, cargo, and npm lockfiles.
- Resolved Windows file locking during native binary updates.

---

## [1.0.0] - 2026-09-25

- Initial General Availability (GA) release of TokenJar.
- Dual-engine architecture: Enterprise Native Rust (`tokenjar`) + Python (`tokenjar-engine`).
- Tree-sitter multi-language AST extraction (Rust, Python, TypeScript, Go, C++, Java, and 9 others).
- In-memory differential session cache and SQLite persistent WAL cache.
- Intelligent terminal output pruner (`filter_output` and `run_command_smart`).
- PageRank repository architecture mapper (`get_repo_map`).
- Interactive standalone Web UI Dashboard (`tokenjar ui`).
