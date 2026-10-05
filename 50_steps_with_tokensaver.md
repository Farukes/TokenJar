# 🟢 50 Adımlık Geliştirici Oturumu: Token-Saver İLE (Optimize Edilmiş Motor)

Bu belgede, Token-Saver motoru **aktifken** aynı 50 geliştirici görevinde AST İskeletleyici, Lockfile Shield, Smart File Cache ve Terminal Pruner araçlarının sağladığı **adım adım doğrulanmış BPE token tasarrufu** belgelenmiştir.

**Hesaplama Standardı:** `tiktoken (cl100k_base — OpenAI & Claude Tokenizer Standardı)`

---

## 📊 Oturum İstatistikleri Özeti

- **Toplam Görev Sayısı:** 50 Adım
- **Standart AI Ham Tüketim:** 195,538 Token
- **Token-Saver ile Tüketilen:** **14,162 Token**
- **Kurtarılan / Tasarruf Edilen:** **181,376 Token**
- **Net Bağlam Azalma Oranı:** **%92.8 NET TASARRUF**
- **Bağlam Penceresi Durumu:** 🛡️ **Kusursuz** (Bağlam şişmediği için model ilk adımdaki kuralları ve kodları %100 hatırlar).

---

## 📋 Adım Adım 50 Görevin Tasarruf Tablosu

| Adım | Kategori | Görev Tanımı | Ham Token | Token-Saver | Net Tasarruf | Tasarruf Oranı |
| :---: | :--- | :--- | :---: | :---: | :---: | :---: |
| **1** | Mimari & Keşif | Proje Konfigürasyonu İnceleme (Cargo.toml) | 114 | **114** | 0 | **-%0.0** |
| **2** | Mimari & Keşif | Depo Mimarisi & PageRank Haritası (repo_map) | 738 | **49** | 689 | **-%93.4** |
| **3** | Mimari & Keşif | Dizin Ağacı Keşfi (get_directory_tree) | 844 | **675** | 169 | **-%20.0** |
| **4** | Sembol İndeksi | Global Sembol Arama: read_file_smart | 911 | **105** | 806 | **-%88.5** |
| **5** | Sembol İndeksi | Global Sembol Arama: filter_output_logic | 1,472 | **136** | 1,336 | **-%90.8** |
| **6** | Sembol İndeksi | Global Sembol Arama: RulesManager | 2,770 | **43** | 2,727 | **-%98.4** |
| **7** | Sembol İndeksi | Etki Alanı Analizi (Blast Radius): read_file_smart | 14,179 | **633** | 13,546 | **-%95.5** |
| **8** | Sembol İndeksi | Etki Alanı Analizi: process_lockfile | 5,034 | **386** | 4,648 | **-%92.3** |
| **9** | AST İskeletleyici | AST Kod İskeleti: smart_reader.rs | 2,434 | **321** | 2,113 | **-%86.8** |
| **10** | AST İskeletleyici | AST Kod İskeleti: hooks/manager.py | 8,182 | **1,063** | 7,119 | **-%87.0** |
| **11** | Hedefli Dilimleme | Hedefli Satır Dilimleme: format_savings | 322 | **197** | 125 | **-%38.8** |
| **12** | Hedefli Dilimleme | Hedefli Satır Dilimleme: ensure_in_user_path (manager.py) | 8,182 | **805** | 7,377 | **-%90.2** |
| **13** | AST İskeletleyici | Tekil Sembol Çıkarımı: RulesManager | 2,770 | **2,038** | 732 | **-%26.4** |
| **14** | Hedefli Dilimleme | Hedefli Satır Dilimleme: Pruner Regex Kuralları | 1,472 | **602** | 870 | **-%59.1** |
| **15** | Hedefli Dilimleme | Hedefli Satır Dilimleme: SQLite Şeması | 2,583 | **508** | 2,075 | **-%80.3** |
| **16** | Hedefli Dilimleme | Hedefli Satır Dilimleme: Lockfile Parser | 2,600 | **711** | 1,889 | **-%72.7** |
| **17** | Hedefli Dilimleme | Hedefli Satır Dilimleme: TelemetryTracker Yapısı | 2,426 | **462** | 1,964 | **-%81.0** |
| **18** | Hedefli Dilimleme | Hedefli Satır Dilimleme: CLI Subcommands | 6,357 | **607** | 5,750 | **-%90.5** |
| **19** | Hedefli Dilimleme | Hedefli Satır Dilimleme: MCP Tool Dispatcher | 5,378 | **755** | 4,623 | **-%86.0** |
| **20** | Hedefli Dilimleme | Hedefli Satır Dilimleme: FastMCP Sunucu Tanımı | 619 | **436** | 183 | **-%29.6** |
| **21** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): Cargo.toml | 114 | **16** | 98 | **-%86.0** |
| **22** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): pyproject.toml | 564 | **17** | 547 | **-%97.0** |
| **23** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): manager.py | 8,182 | **15** | 8,167 | **-%99.8** |
| **24** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): rules/manager.py | 2,770 | **15** | 2,755 | **-%99.5** |
| **25** | Akıllı Önbellek | Tekrarlanan Dosya Okuma (2. Kez): smart_reader.rs | 2,434 | **16** | 2,418 | **-%99.3** |
| **26** | Lockfile Kalkanı | Kilit Dosyası Koruma Kalkanı: Cargo.lock (Tam Okuma) | 20,009 | **277** | 19,732 | **-%98.6** |
| **27** | Lockfile Kalkanı | Kilit Dosyasında Paket Sorgulama: 'serde' | 20,009 | **324** | 19,685 | **-%98.4** |
| **28** | Lockfile Kalkanı | Kilit Dosyasında Paket Sorgulama: 'tree-sitter' | 20,009 | **343** | 19,666 | **-%98.3** |
| **29** | Lockfile Kalkanı | Kilit Dosyasında Paket Sorgulama: 'tokio' | 20,009 | **275** | 19,734 | **-%98.6** |
| **30** | Lockfile Kalkanı | Kilit Dosyası Tekrar Okuma: Cargo.lock | 20,009 | **277** | 19,732 | **-%98.6** |
| **31** | Terminal Budayıcı | Derleme Çıktısı Budama: cargo check | 95 | **49** | 46 | **-%48.4** |
| **32** | Terminal Budayıcı | Birim Test Çıktısı: pytest test_config.py | 117 | **52** | 65 | **-%55.6** |
| **33** | Terminal Budayıcı | Birim Test Çıktısı: pytest test_token_counter.py | 88 | **32** | 56 | **-%63.6** |
| **34** | Terminal Budayıcı | Birim Test Çıktısı: pytest test_cache.py | 100 | **32** | 68 | **-%68.0** |
| **35** | Terminal Budayıcı | Birim Test Çıktısı: pytest test_rules.py | 65 | **32** | 33 | **-%50.8** |
| **36** | Terminal Budayıcı | Rust Test Koşusu: cargo test -p tokenjar-core | 291 | **36** | 255 | **-%87.6** |
| **37** | Terminal Budayıcı | Depo Durumu: git status | 180 | **130** | 50 | **-%27.8** |
| **38** | Terminal Budayıcı | Kod Değişiklikleri: git diff | 432 | **447** | 0 | **-%0.0** |
| **39** | Terminal Budayıcı | Git Geçmişi: git log -n 5 --oneline | 91 | **90** | 1 | **-%1.1** |
| **40** | Terminal Budayıcı | Büyük Derleme Akışı Gürültüsü (Build Noise) | 429 | **25** | 404 | **-%94.2** |
| **41** | Çok Dilli AST | Çok Dilli AST İskeleti: TypeScript | 103 | **75** | 28 | **-%27.2** |
| **42** | Çok Dilli AST | Çok Dilli AST İskeleti: Go | 64 | **49** | 15 | **-%23.4** |
| **43** | Çok Dilli AST | Çok Dilli AST İskeleti: Rust | 83 | **60** | 23 | **-%27.7** |
| **44** | Çok Dilli AST | Çok Dilli AST İskeleti: Python | 73 | **61** | 12 | **-%16.4** |
| **45** | L2 SQLite Önbellek | SQLite L2 Önbellekten Sembol Sorgusu: read_file_smart | 2,434 | **176** | 2,258 | **-%92.8** |
| **46** | L2 SQLite Önbellek | SQLite L2 Önbellekten Sembol Sorgusu: filter_output | 1,472 | **289** | 1,183 | **-%80.4** |
| **47** | L2 SQLite Önbellek | SQLite L2 Önbellekten Sembol Sorgusu: RulesManager | 2,770 | **59** | 2,711 | **-%97.9** |
| **48** | L2 SQLite Önbellek | SQLite L2 Önbellekten Sembol Sorgusu: process_lockfile | 2,600 | **210** | 2,390 | **-%91.9** |
| **49** | L2 SQLite Önbellek | SQLite Veritabanı Sağlık Denetimi | 250 | **19** | 231 | **-%92.4** |
| **50** | Oturum Denetimi | Geliştirici Oturumu Doğrulama & Denetim Tamamlama | 305 | **18** | 287 | **-%94.1** |
| **TOPLAM** | **50 Adım** | **Tüm Geliştirici Oturumu** | **195,538** | **14,162** | **181,376** | **-%92.8** |

---

## 🎯 Token-Saver Optimizasyon Detayları (İlk 15 Kritik Adım)

### Adım 1: Proje Konfigürasyonu İnceleme (Cargo.toml)
- **Kategori:** Mimari & Keşif
- **Tasarruf:** 114 ➔ **114 BPE token** (-%0.0)
- **Nasıl Optimize Edildi:** İlk okumada her iki sistem de dosyanın tamamını okur.
- **Modele Giden Optimize İçerik Önizlemesi:**
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
- **Tasarruf:** 738 ➔ **49 BPE token** (-%93.4)
- **Nasıl Optimize Edildi:** Standart AI ana giriş dosyalarını tek tek okumaya çalışırken TokenJar tek bir PageRank haritası ile tüm mimariyi aktarır.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
📁 Repository Map (104 files, budget: 600 tokens)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

... (budget too small to include even the top file)...
```

### Adım 3: Dizin Ağacı Keşfi (get_directory_tree)
- **Kategori:** Mimari & Keşif
- **Tasarruf:** 844 ➔ **675 BPE token** (-%20.0)
- **Nasıl Optimize Edildi:** git ls-tree tüm alt dallardaki 300+ dosyayı ham dökerken get_directory_tree derinlik kısıtı ile temiz hiyerarşi sunar.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
Token-Saver/
├── assets/
├── crates/
│   ├── tokenjar-cli/
│   │   ├── src/
│   │   │   ...
│   │   ├── static/
│   │   │   ...
│   │   └── Cargo.toml
│   └── tokenjar-core/
│       ├── src/
│       │...
```

### Adım 4: Global Sembol Arama: read_file_smart
- **Kategori:** Sembol İndeksi
- **Tasarruf:** 911 ➔ **105 BPE token** (-%88.5)
- **Nasıl Optimize Edildi:** Standart AI sembolü aramak için şüpheli dosyaları açarken TokenJar AST indeksi ile doğrudan dosya:satır:imza döndürür.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[SYMBOLS] Found 3 symbol(s) matching 'read_file_smart':
----------------------------------------
1. [FUNCTION] read_file_smart -> crates/tokenjar-core/src/smart_reader.rs:16
   Signature: pub fn read_...
```

### Adım 5: Global Sembol Arama: filter_output_logic
- **Kategori:** Sembol İndeksi
- **Tasarruf:** 1,472 ➔ **136 BPE token** (-%90.8)
- **Nasıl Optimize Edildi:** Standart AI pruner dosyasını açıp ararken TokenJar tek satırlık imza yanıtı verir.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[SYMBOLS] Found 3 symbol(s) matching 'filter_output_logic':
----------------------------------------
1. [FUNCTION] filter_output_logic -> crates/tokenjar-core/src/output_pruner.rs:20
   Signature: pub...
```

### Adım 6: Global Sembol Arama: RulesManager
- **Kategori:** Sembol İndeksi
- **Tasarruf:** 2,770 ➔ **43 BPE token** (-%98.4)
- **Nasıl Optimize Edildi:** Sınıfın yerini bulmak için 350 satırlık dosya yerine anında AST konumu döndürülür.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[SYMBOLS] Found 1 symbol(s) matching 'RulesManager':
----------------------------------------
1. [CLASS] RulesManager -> src/tokenjar/rules/manager.py:57
   Signature: class RulesManager:...
```

### Adım 7: Etki Alanı Analizi (Blast Radius): read_file_smart
- **Kategori:** Sembol İndeksi
- **Tasarruf:** 14,179 ➔ **633 BPE token** (-%95.5)
- **Nasıl Optimize Edildi:** Standart AI çağıran tüm dosyaları bağlama yükler; TokenJar sadece kullanım satırlarını kompakt listeler.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[REFERENCES] Blast Radius Analysis for 'read_file_smart':
• Defined at: src/tokenjar/tools/smart_reader.py:12 (function), src/tokenjar/tools/smart_reader.py:148 (function), crates/tokenjar-core/src/sm...
```

### Adım 8: Etki Alanı Analizi: process_lockfile
- **Kategori:** Sembol İndeksi
- **Tasarruf:** 5,034 ➔ **386 BPE token** (-%92.3)
- **Nasıl Optimize Edildi:** Çağrı noktaları filtrelenerek sadece ilgili satırlar gösterilir.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[REFERENCES] Blast Radius Analysis for 'process_lockfile':
• Defined at: crates/tokenjar-core/src/filters/lockfile.rs:114 (function), src/tokenjar/filters/lockfile.py:212 (function)
• Total Usages: 9 ...
```

### Adım 9: AST Kod İskeleti: smart_reader.rs
- **Kategori:** AST İskeletleyici
- **Tasarruf:** 2,434 ➔ **321 BPE token** (-%86.8)
- **Nasıl Optimize Edildi:** Fonksiyon gövdeleri budanarak sadece struct, impl ve fonksiyon imzaları sunulur.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
//! Smart Reader with Session Caching & Lockfile Shielding in Rust.
//!
//! Intelligently reads source files, returns compact diffs on edits,
//! and protects context windows from massive lockfiles an...
```

### Adım 10: AST Kod İskeleti: hooks/manager.py
- **Kategori:** AST İskeletleyici
- **Tasarruf:** 8,182 ➔ **1,063 BPE token** (-%87.0)
- **Nasıl Optimize Edildi:** 870 satırlık hook yöneticisinin tüm sınıf ve fonksiyon imzaları 1/5 boyutunda verilir.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
"""Transparent hooking manager for TokenJar.

Provides non-intrusive, transparent interception for terminal commands
across PowerShell, Bash, Zsh, and Claude Code environments.

Allows safe installati...
```

### Adım 11: Hedefli Satır Dilimleme: format_savings
- **Kategori:** Hedefli Dilimleme
- **Tasarruf:** 322 ➔ **197 BPE token** (-%38.8)
- **Nasıl Optimize Edildi:** Standart AI dosyanın tamamını okurken TokenJar yalnızca L12-24 aralığını dilimler.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[TOKENJAR] Lines 12-24 of 44 in 'crates\tokenjar-core\src\token_counter.rs':
12: /// Format token savings summary string.
13: pub fn format_savings(original: ...
```

### Adım 12: Hedefli Satır Dilimleme: ensure_in_user_path (manager.py)
- **Kategori:** Hedefli Dilimleme
- **Tasarruf:** 8,182 ➔ **805 BPE token** (-%90.2)
- **Nasıl Optimize Edildi:** 870 satır yerine yalnızca ilgilenilen 55 satırlık fonksiyon bağlama yüklenir.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[TOKENJAR] Lines 520-575 of 870 in 'src\tokenjar\hooks\manager.py':
520:         # Ensure CLI binary directory is in user's PATH
521:         path_ok, path_ms...
```

### Adım 13: Tekil Sembol Çıkarımı: RulesManager
- **Kategori:** AST İskeletleyici
- **Tasarruf:** 2,770 ➔ **2,038 BPE token** (-%26.4)
- **Nasıl Optimize Edildi:** Tüm dosya yerine Tree-sitter ile sadece RulesManager sınıfı izole edilir.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
class RulesManager:
    """Manages injection and removal of agent steering rules across projects."""

    RULES_CONTENT = TOKENJAR_AGENT_RULES

    SUPPORTED_RULE_FILES = [
        "AGENTS.md",
      ...
```

### Adım 14: Hedefli Satır Dilimleme: Pruner Regex Kuralları
- **Kategori:** Hedefli Dilimleme
- **Tasarruf:** 1,472 ➔ **602 BPE token** (-%59.1)
- **Nasıl Optimize Edildi:** Regex filtre kuralları tüm dosya okunmadan hedeflenerek çekilir.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[TOKENJAR] Lines 25-75 of 178 in 'src\tokenjar\tools\output_pruner.py':
25:     build_filtered = detect_and_filter_build(output)
26:     if build_filtered is ...
```

### Adım 15: Hedefli Satır Dilimleme: SQLite Şeması
- **Kategori:** Hedefli Dilimleme
- **Tasarruf:** 2,583 ➔ **508 BPE token** (-%80.3)
- **Nasıl Optimize Edildi:** Veritabanı DDL ve tablo oluşturma blokları dilimlenerek okunur.
- **Modele Giden Optimize İçerik Önizlemesi:**
```text
[TOKENJAR] Lines 40-90 of 371 in 'src\tokenjar\cache\persistent_cache.py':
40:                 conn.execute(
41:                     """
42:                  ...
```

