# 🍯 TokenJar V2.0: Universal MCP Gateway & Intelligent Compression Proxy

## 💡 The Vision: "Cloudflare for the MCP Ecosystem"
As the Model Context Protocol (MCP) becomes the universal standard for AI coding assistants and enterprise agents, a critical bottleneck has emerged: **third-party MCP servers are built for data access, not LLM token economy.**

TokenJar V2.0 transforms TokenJar from a standalone local file/terminal optimizer into an **Intelligent MCP Proxy Gateway** that intercepts, filters, and compresses tool responses between AI assistants and downstream MCP servers.

---

## 🎯 The Core Problem in V1 vs V2

```mermaid
flowchart LR
    subgraph V1 ["TokenJar v1.0.2 (Current Architecture)"]
        IDE1["AI Assistant"] <==>|"AST + Diff Cache + Pruner"| TJ1["TokenJar MCP Engine"]
        TJ1 --- LocalFiles["Local Codebase & Terminal"]
    end

    subgraph V2 ["TokenJar v2.0 (Next Major Evolution)"]
        IDE2["AI Assistant"] <==>|"Single Gateway Port"| TGW["TokenJar Gateway Middleware"]
        TGW <==> Ext1["PostgreSQL / DB MCP"]
        TGW <==> Ext2["GitHub / Jira MCP"]
        TGW <==> Ext3["Slack / Cloud MCP"]
        TGW <==> TJ2["TokenJar Native Tools"]
    end
```

### Why V2 is Essential:
1. **Unbounded Data Dumps:** A simple `SELECT * FROM users` via PostgreSQL MCP returns 10,000 JSON rows, destroying context windows and costing $2–$5 per prompt.
2. **Metadata Pollution:** Slack, Jira, and GitHub MCP responses are filled with avatar URLs, permission trees, nested nulls, and internal IDs that LLMs never need.
3. **IDE Connection Limit:** Configuring 15 different MCP servers in Cursor, Windsurf, Claude Code, and Antigravity leads to config bloat and process contention.

---

## 🏗️ TokenJar V2 Architecture Blueprint

```mermaid
flowchart TD
    IDE["AI Coding Assistant (Cursor / Antigravity / Claude Code / Windsurf)"]
    
    subgraph Gateway ["TokenJar V2 Universal Gateway Engine"]
        Router["Dynamic MCP Router & Multiplexer"]
        
        subgraph Pipeline ["Intelligent Compression Pipeline"]
            F1["🛡️ SQL & Tabular Squeezer (Top 5 + Stat Summary)"]
            F2["🧹 JSON Schema & Noise Stripper (Nulls, Avatars, Blobs)"]
            F3["⚡ Differential Semantic Cache (L1 Memory + L2 SQLite)"]
            F4["🚨 Circuit Breaker & Hard Token Ceiling (e.g. 2,500 tokens max)"]
        end
        
        Telemetry["📊 Gateway Telemetry & Cost Auditor"]
    end
    
    subgraph Downstream ["Downstream MCP Servers"]
        D1["PostgreSQL / MySQL / Snowflake"]
        D2["GitHub / GitLab / Bitbucket"]
        D3["Slack / Discord / Teams"]
        D4["AWS / Kubernetes / Docker"]
    end

    IDE <==>|"Single stdio / SSE Channel"| Router
    Router <==> Pipeline
    Pipeline <==> Downstream
    Pipeline -.-> Telemetry
```

---

## 🔑 Key Features Planned for V2.0

### 1. 🗄️ SQL & Tabular Squeezer (95–99% Savings)
- Intercepts tabular datasets from database MCP servers.
- Automatically renders a compact Markdown table containing the **first 5 rows + statistical distribution** (total rows, column types, min/max, null counts).
- LLMs get full structural insight without paying for 50,000 repetitive data rows.

### 2. 🧹 Universal JSON Metadata Stripper (40–70% Savings)
- Cleans nested JSON responses before sending to the model:
  - Prunes empty strings, `null` fields, and boolean defaults.
  - Strips image URLs, profile icons, and base64 payloads.
  - Collapses verbose schemas into concise TypeScript-like type definitions.

### 3. 🚨 Token Budget Guard & Circuit Breaker (100% Budget Protection)
- Set hard limits per tool call (e.g. max 2,500 tokens).
- If a downstream tool generates runaway output, TokenJar gracefully caps it with a concise summary and offers paginated access (`offset`, `limit`).

### 4. 🔌 Single-Pipe Multiplexing
- Instead of adding 10 MCP servers to Cursor or Claude Code, add **just one**: `tokenjar gateway`.
- TokenJar transparently manages child server processes, routing requests to the appropriate downstream MCP server based on tool namespace (`postgres:query`, `github:issue`).

### 5. 📈 Enterprise Cost & Savings Analytics
- Enhanced Web Dashboard (`tokenjar ui`) displaying:
  - Per-tool token consumption breakdown.
  - Real-time financial savings per model (Claude 3.5 Sonnet, GPT-4o, Gemini 1.5 Pro).
  - Configurable compression rules per server.

### 6. ⚡ Surgical Code Modification Engine (`edit_file_smart` / `patch_smart`)
- **Core Value:** While TokenJar slashes token costs during inspection (reading, AST skeletons, symbol lookups), model output generation during file modification remains a major token drain (full file dumps, repeated line rewrites, truncations).
- **Architecture:**
  - **AST Node Replacement:** Target specific functions, methods, or classes by symbol name and replace the exact AST node via Tree-sitter without rewriting or disturbing surrounding code or imports.
  - **Surgical Unified Diff Patcher:** Apply compact multi-hunk unified diffs with fuzzy context line alignment and whitespace resilience.
  - **Zero-Truncation Guarantee (Anti-Lazy Coder):** Automatically parse and validate syntax before committing changes to disk, rejecting corrupted, truncated, or broken patches with instant feedback.
  - **Expected Impact:** 70–85% reduction in generation token expenditure; eliminates destructive file overwrite errors.

---

## 📅 Roadmap & Milestones

- **Phase 1: Gateway Core & Router** — Subprocess multiplexer over stdio/JSON-RPC.
- **Phase 2: Tabular & JSON Filter Pipeline** — SQL summarizer and metadata stripper.
- **Phase 3: Surgical File Modifier (`edit_file_smart` / `patch_smart`)** — Tree-Sitter AST replacement and zero-truncation diff patcher.
- **Phase 4: Rate Limiting & Circuit Breaker** — Hard token ceiling guards.
- **Phase 5: Web UI Gateway Configuration** — 1-Click downstream server management.
- **Phase 6: Release v2.0 GA** — Crates.io, PyPI, Homebrew, and Winget updates.

---
*Preserved for future development. Created: September 2026. Updated: October 2026.*
