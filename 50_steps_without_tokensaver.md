# 🔴 50 Adımlık Geliştirici Oturumu: Token-Saver OLMADAN (Standart AI Asistanı)

Bu belgede, Token-Saver motoru **olmadan** standart bir AI kodlama asistanının (`view_file`, `cat`, ham terminal, tam lockfile okuma) aynı 50 geliştirici görevinde tükettiği **adım adım gerçek BPE token yükü** belgelenmiştir.

**Hesaplama Standardı:** `tiktoken (cl100k_base — OpenAI & Claude Tokenizer Standardı)`

---

## 📊 Oturum İstatistikleri Özeti

- **Toplam Görev Sayısı:** 50 Adım
- **Modele Giren Toplam Ham Token:** **195,538 Token**
- **Bağlam Penceresi Durumu:** 🚨 **Yüksek Risk** (~195,538 token ile model context compaction eşiğine girer, önceki adımları unutmaya başlar).

---

## 📋 Adım Adım 50 Görevin Ham Tüketim Tablosu

| Adım | Kategori | Görev Tanımı | Tüketilen BPE Token | Kümülatif Token |
| :---: | :--- | :--- | :---: | :---: |
| **1** | Mimari & Keşif | Proje Konfigürasyonu İnceleme (Cargo.toml) | **114** | 114 |
| **2** | Mimari & Keşif | Depo Mimarisi & PageRank Haritası (repo_map) | **738** | 852 |
| **3** | Mimari & Keşif | Dizin Ağacı Keşfi (get_directory_tree) | **844** | 1,696 |
| **4** | Sembol İndeksi | Global Sembol Arama: read_file_smart | **911** | 2,607 |
| **5** | Sembol İndeksi | Global Sembol Arama: filter_output_logic | **1,472** | 4,079 |
| **6** | Sembol İndeksi | Global Sembol Arama: RulesManager | **2,770** | 6,849 |
| **7** | Sembol İndeksi | Etki Alanı Analizi (Blast Radius): read_file_smart | **14,179** | 21,028 |
| **8** | Sembol İndeksi | Etki Alanı Analizi: process_lockfile | **5,034** | 26,062 |
| **9** | AST İskeletleyici | AST Kod İskeleti: smart_reader.rs | **2,434** | 28,496 |
| **10** | AST İskeletleyici | AST Kod İskeleti: hooks/manager.py | **8,182** | 36,678 |
| **11** | Hedefli Dilimleme | Hedefli Satır Dilimleme: format_savings | **322** | 37,000 |
| **12** | Hedefli Dilimleme | Hedefli Satır Dilimleme: ensure_in_user_path (manager.py) | **8,182** | 45,182 |
| **13** | AST İskeletleyici | Tekil Sembol Çıkarımı: RulesManager | **2,770** | 47,952 |
| **14** | Hedefli Dilimleme | Hedefli Satır Dilimleme: Pruner Regex Kuralları | **1,472** | 49,424 |
| **15** | Hedefli Dilimleme | Hedefli Satır Dilimleme: SQLite Şeması | **2,583** | 52,007 |
| **16** | Hedefli Dilimleme | Hedefli Satır Dilimleme: Lockfile Parser | **2,600** | 54,607 |
| **17** | Hedefli Dilimleme | Hedefli Satır Dilimleme: TelemetryTracker Yapısı | **2,426** | 57,033 |
| **18** | Hedefli Dilimleme | Hedefli Satır Dilimleme: CLI Subcommands | **6,357** | 63,390 |
| **19** | Hedefli Dilimleme | Hedefli Satır Dilimleme: MCP Tool Dispatcher | **5,378** | 68,768 |
| **20** | Hedefli Dilimleme | Hedefli Satır Dilimleme: FastMCP Sunucu Tanımı | **619** | 69,387 |
| **21** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): Cargo.toml | **114** | 69,501 |
| **22** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): pyproject.toml | **564** | 70,065 |
| **23** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): manager.py | **8,182** | 78,247 |
| **24** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): rules/manager.py | **2,770** | 81,017 |
| **25** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): smart_reader.rs | **2,434** | 83,451 |
| **26** | Lockfile Kalkanı | Kilit Dosyası Koruma Kalkanı: Cargo.lock (Tam Okuma) | **20,009** | 103,460 |
| **27** | Lockfile Kalkanı | Kilit Dosyasında Paket Sorgulama: 'serde' | **20,009** | 123,469 |
| **28** | Lockfile Kalkanı | Kilit Dosyasında Paket Sorgulama: 'tree-sitter' | **20,009** | 143,478 |
| **29** | Lockfile Kalkanı | Kilit Dosyasında Paket Sorgulama: 'tokio' | **20,009** | 163,487 |
| **30** | Lockfile Kalkanı | Kilit Dosyası Tekrar Okuma: Cargo.lock | **20,009** | 183,496 |
| **31** | Terminal Budayıcı | Derleme Çıktısı Budama: cargo check | **95** | 183,591 |
| **32** | Terminal Budayıcı | Birim Test Çıktısı: pytest test_config.py | **117** | 183,708 |
| **33** | Terminal Budayıcı | Birim Test Çıktısı: pytest test_token_counter.py | **88** | 183,796 |
| **34** | Terminal Budayıcı | Birim Test Çıktısı: pytest test_cache.py | **100** | 183,896 |
| **35** | Terminal Budayıcı | Birim Test Çıktısı: pytest test_rules.py | **65** | 183,961 |
| **36** | Terminal Budayıcı | Rust Test Koşusu: cargo test -p tokenjar-core | **291** | 184,252 |
| **37** | Terminal Budayıcı | Depo Durumu: git status | **180** | 184,432 |
| **38** | Terminal Budayıcı | Kod Değişiklikleri: git diff | **432** | 184,864 |
| **39** | Terminal Budayıcı | Git Geçmişi: git log -n 5 --oneline | **91** | 184,955 |
| **40** | Terminal Budayıcı | Büyük Derleme Akışı Gürültüsü (Build Noise) | **429** | 185,384 |
| **41** | Çok Dilli AST | Çok Dilli AST İskeleti: TypeScript | **103** | 185,487 |
| **42** | Çok Dilli AST | Çok Dilli AST İskeleti: Go | **64** | 185,551 |
| **43** | Çok Dilli AST | Çok Dilli AST İskeleti: Rust | **83** | 185,634 |
| **44** | Çok Dilli AST | Çok Dilli AST İskeleti: Python | **73** | 185,707 |
| **45** | L2 SQLite Önbellek | SQLite L2 Önbellekten Sembol Sorgusu: read_file_smart | **2,434** | 188,141 |
| **46** | L2 SQLite Önbellek | SQLite L2 Önbellekten Sembol Sorgusu: filter_output | **1,472** | 189,613 |
| **47** | L2 SQLite Önbellek | SQLite L2 Önbellekten Sembol Sorgusu: RulesManager | **2,770** | 192,383 |
| **48** | L2 SQLite Önbellek | SQLite L2 Önbellekten Sembol Sorgusu: process_lockfile | **2,600** | 194,983 |
| **49** | L2 SQLite Önbellek | SQLite Veritabanı Sağlık Denetimi | **250** | 195,233 |
| **50** | Oturum Denetimi | Geliştirici Oturumu Doğrulama & Denetim Tamamlama | **305** | 195,538 |
| **TOPLAM** | **50 Adım** | **Tüm Geliştirici Oturumu** | **195,538 Token** | **195,538 Token** |

---

## 🔍 Adım Bazlı Ham Çalışma Detayları (İlk 15 Kritik Adım)

### Adım 1: Proje Konfigürasyonu İnceleme (Cargo.toml)
- **Kategori:** Mimari & Keşif
- **Ham Token Yükü:** 114 BPE token
- **Neden Bu Kadar Yüksek:** İlk okumada her iki sistem de dosyanın tamamını okur.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
[workspace]
members = [
    "crates/tokenjar-core",
    "crates/tokenjar-cli",
]
resolver = "2"

[workspace.package]
version = "1.0.3"
edition = "2021"
authors = ["Ömer Faruk Eskitürk"]
license = "BUS...
```

### Adım 2: Depo Mimarisi & PageRank Haritası (repo_map)
- **Kategori:** Mimari & Keşif
- **Ham Token Yükü:** 738 BPE token
- **Neden Bu Kadar Yüksek:** Standart AI ana giriş dosyalarını tek tek okumaya çalışırken TokenJar tek bir PageRank haritası ile tüm mimariyi aktarır.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
// File: Cargo.toml
[workspace]
members = [
    "crates/tokenjar-core",
    "crates/tokenjar-cli",
]
resolver = "2"

[workspace.package]
version = "1.0.3"
edi...
```

### Adım 3: Dizin Ağacı Keşfi (get_directory_tree)
- **Kategori:** Mimari & Keşif
- **Ham Token Yükü:** 844 BPE token
- **Neden Bu Kadar Yüksek:** git ls-tree tüm alt dallardaki 300+ dosyayı ham dökerken get_directory_tree derinlik kısıtı ile temiz hiyerarşi sunar.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
.cursorrules
.github/workflows/ci.yml
.github/workflows/release.yml
.gitignore
.pre-commit-config.yaml
.windsurfrules
AGENTS.md
CLAUDE.md
Cargo.lock
Cargo.toml
LICENSE
README.md
README.tr.md
crates/to...
```

### Adım 4: Global Sembol Arama: read_file_smart
- **Kategori:** Sembol İndeksi
- **Ham Token Yükü:** 911 BPE token
- **Neden Bu Kadar Yüksek:** Standart AI sembolü aramak için şüpheli dosyaları açarken TokenJar AST indeksi ile doğrudan dosya:satır:imza döndürür.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
//! Smart Reader with Session Caching & Lockfile Shielding in Rust.
//!
//! Intelligently reads source files, returns compact diffs on edits,
//! and protects context windows from massive lockfiles an...
```

### Adım 5: Global Sembol Arama: filter_output_logic
- **Kategori:** Sembol İndeksi
- **Ham Token Yükü:** 1,472 BPE token
- **Neden Bu Kadar Yüksek:** Standart AI pruner dosyasını açıp ararken TokenJar tek satırlık imza yanıtı verir.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
from __future__ import annotations

import subprocess

from tokenjar.filters.ansi import strip_ansi
from tokenjar.filters.build_tools import detect_and_filter_build
from tokenjar.filters.git import fi...
```

### Adım 6: Global Sembol Arama: RulesManager
- **Kategori:** Sembol İndeksi
- **Ham Token Yükü:** 2,770 BPE token
- **Neden Bu Kadar Yüksek:** Sınıfın yerini bulmak için 350 satırlık dosya yerine anında AST konumu döndürülür.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
"""Agent steering rules manager for TokenJar.

Generates and injects non-intrusive instructions into AGENTS.md, .cursorrules,
.windsurfrules, and CLAUDE.md to guarantee AI models actively prioritize
T...
```

### Adım 7: Etki Alanı Analizi (Blast Radius): read_file_smart
- **Kategori:** Sembol İndeksi
- **Ham Token Yükü:** 14,179 BPE token
- **Neden Bu Kadar Yüksek:** Standart AI çağıran tüm dosyaları bağlama yükler; TokenJar sadece kullanım satırlarını kompakt listeler.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
"""TokenJar MCP Server.

Central FastMCP server that registers all token-saving tools.
Communicates with AI coding assistants via stdio (JSON-RPC 2.0).
"""

import sys

from fastmcp import FastMCP

fr...
```

### Adım 8: Etki Alanı Analizi: process_lockfile
- **Kategori:** Sembol İndeksi
- **Ham Token Yükü:** 5,034 BPE token
- **Neden Bu Kadar Yüksek:** Çağrı noktaları filtrelenerek sadece ilgili satırlar gösterilir.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
//! Smart Reader with Session Caching & Lockfile Shielding in Rust.
//!
//! Intelligently reads source files, returns compact diffs on edits,
//! and protects context windows from massive lockfiles an...
```

### Adım 9: AST Kod İskeleti: smart_reader.rs
- **Kategori:** AST İskeletleyici
- **Ham Token Yükü:** 2,434 BPE token
- **Neden Bu Kadar Yüksek:** Fonksiyon gövdeleri budanarak sadece struct, impl ve fonksiyon imzaları sunulur.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
//! Smart Reader with Session Caching & Lockfile Shielding in Rust.
//!
//! Intelligently reads source files, returns compact diffs on edits,
//! and protects context windows from massive lockfiles an...
```

### Adım 10: AST Kod İskeleti: hooks/manager.py
- **Kategori:** AST İskeletleyici
- **Ham Token Yükü:** 8,182 BPE token
- **Neden Bu Kadar Yüksek:** 870 satırlık hook yöneticisinin tüm sınıf ve fonksiyon imzaları 1/5 boyutunda verilir.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
"""Transparent hooking manager for TokenJar.

Provides non-intrusive, transparent interception for terminal commands
across PowerShell, Bash, Zsh, and Claude Code environments.

Allows safe installati...
```

### Adım 11: Hedefli Satır Dilimleme: format_savings
- **Kategori:** Hedefli Dilimleme
- **Ham Token Yükü:** 322 BPE token
- **Neden Bu Kadar Yüksek:** Standart AI dosyanın tamamını okurken TokenJar yalnızca L12-24 aralığını dilimler.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
//! Fast heuristic token estimation matching OpenAI / Claude approximations.

/// Estimate tokens for a given string slice (~4 chars per token).
pub fn estimate_tokens(text: &str) -> usize {
    if te...
```

### Adım 12: Hedefli Satır Dilimleme: ensure_in_user_path (manager.py)
- **Kategori:** Hedefli Dilimleme
- **Ham Token Yükü:** 8,182 BPE token
- **Neden Bu Kadar Yüksek:** 870 satır yerine yalnızca ilgilenilen 55 satırlık fonksiyon bağlama yüklenir.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
"""Transparent hooking manager for TokenJar.

Provides non-intrusive, transparent interception for terminal commands
across PowerShell, Bash, Zsh, and Claude Code environments.

Allows safe installati...
```

### Adım 13: Tekil Sembol Çıkarımı: RulesManager
- **Kategori:** AST İskeletleyici
- **Ham Token Yükü:** 2,770 BPE token
- **Neden Bu Kadar Yüksek:** Tüm dosya yerine Tree-sitter ile sadece RulesManager sınıfı izole edilir.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
"""Agent steering rules manager for TokenJar.

Generates and injects non-intrusive instructions into AGENTS.md, .cursorrules,
.windsurfrules, and CLAUDE.md to guarantee AI models actively prioritize
T...
```

### Adım 14: Hedefli Satır Dilimleme: Pruner Regex Kuralları
- **Kategori:** Hedefli Dilimleme
- **Ham Token Yükü:** 1,472 BPE token
- **Neden Bu Kadar Yüksek:** Regex filtre kuralları tüm dosya okunmadan hedeflenerek çekilir.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
from __future__ import annotations

import subprocess

from tokenjar.filters.ansi import strip_ansi
from tokenjar.filters.build_tools import detect_and_filter_build
from tokenjar.filters.git import fi...
```

### Adım 15: Hedefli Satır Dilimleme: SQLite Şeması
- **Kategori:** Hedefli Dilimleme
- **Ham Token Yükü:** 2,583 BPE token
- **Neden Bu Kadar Yüksek:** Veritabanı DDL ve tablo oluşturma blokları dilimlenerek okunur.
- **Modele Giren Ham İçerik Önizlemesi:**
```text
"""Persistent SQLite storage for TokenJar file cache.

Enables cache persistence across MCP server restarts and independent CLI sessions.
Uses SQLite with WAL mode for fast, safe concurrent reads and ...
```

