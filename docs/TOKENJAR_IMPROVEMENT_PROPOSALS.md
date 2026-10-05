# 🚀 TokenJar Architecture & Feature Improvement Proposals
**Technical Evaluation & Implementation Roadmap**

Bu belge, TokenJar Rust yerel çekirdeği (`tokenjar-core`) ve MCP sunucusu (`tokenjar-cli`) üzerinde yapılan derin mimari inceleme, canlı testler ve test paketi (`cargo test --workspace`) analizleri sonucunda hazırlanan teknik iyileştirme ve yeni özellik önerilerini içerir.

---

## 📌 Genel Özet ve Mevcut Durum

| Bileşen | Mevcut Durum | Test Başarısı | Tasarruf Oranı |
| :--- | :--- | :---: | :---: |
| **Rust Native Engine** | Sıfır Python bağımlılığı, bağımsız binary | %100 (45/45 birim test) | - |
| **Terminal Pruner (`run_command_smart`)** | Aktif ve stabil | Başarılı | **%84 - %95** |
| **AST Skeletonizer & Symbol Index** | `tree-sitter` tabanlı, 14+ dil + Hibrit Fuzzy | Başarılı | **%80 - %88** |
| **Lockfile & Binary Shields** | Kapsamlı uzantı ve format koruması | Başarılı | **%99.3** |
| **Session Cache (Unified Diff + Slice Cache)** | In-memory + L2 SQLite WAL | Başarılı | **%97.5** |

### Uygulama & Faz Takip Durumu
- [x] **Faz 1.1:** Satır Dilimlemede Oturumu Önbelleğe Alma (`#L{start}-{end}` ve `#sym:{name}`) — *Tamamlandı*
- [x] **Faz 1.2:** Çıktı Boyutu Tavanı ve Otomatik Sayfalama (`MAX_OUTPUT_LINES = 80`) — *Tamamlandı*
- [x] **Faz 1.3:** Tipografi Standardizasyonu (`(97.5% optimized reduction)`) — *Tamamlandı*
- [x] **Faz 2:** Hibrit Sembol Arama (AST + Fuzzy / Trigram Fallback + Typo Tolerance) — *Tamamlandı*
- [x] **Faz 3:** Akıllı Cerrahi Dosya Düzenleme Aracı (`edit_file_smart`) — *`ROADMAP.md` Yol Haritasına Eklendi*

---

## 🛠️ Detaylı Teknik İyileştirme Önerileri

### 1. Satır Dilimlemede (Slicing) Oturumu Önbelleğe Alma
- **Etkilenen Dosya:** `crates/tokenjar-core/src/smart_reader.rs` (`read_file_smart`)
- **Problem Analizi:**
  `read_file_smart` fonksiyonuna `start_line` veya `end_line` parametresi verildiğinde, satır aralığı doğrudan kesilip dönülmektedir. Ancak dosyanın tam içeriği veya ilgili dilimi `cache.get(file_path, &content)` mekanizmasına kaydedilmemektedir.
  - **Sonuç:** Bir AI ajan aynı dosyanın 1–50. satırlarını iki kez üst üste okuduğunda `[CACHED] unchanged` yanıtı yerine ham satırları tekrar alır.
  - Ayrıca parça okuduktan sonra dosyanın tamamı istendiğinde sistem bunu sıfırdan `FirstRead` olarak işler.
- **Çözüm / Uygulama Planı:**
  1. Dosya okunduğunda tam içerik ve dosya hash'i arka planda `SessionCache`'e tescil edilmelidir (`cache.register_raw(file_path, &content)`).
  2. Dilimlenen aralıklar için `path_key = format!("{file_path}#L{start}-L{end}")` şeklinde ikincil önbellek anahtarı üretilerek tekrarlanan sorgularda tek satırlık `[CACHED] Lines X-Y unchanged` dönüşü sağlanmalıdır.

---

### 2. Akıllı Cerrahi Dosya Düzenleme Aracı (`edit_file_smart` / `patch_smart`)
- **Etkilenen Modüller:** `crates/tokenjar-cli/src/mcp.rs`, `crates/tokenjar-core/src/`
- **Gerekçe:**
  TokenJar şu an **Okuma (Read)**, **Sembol Arama (Symbol/AST)** ve **Komut Çalıştırma (Terminal)** fazlarında devasa token tasarrufu sağlamaktadır. Ancak model dosya üzerinde değişiklik yaparken yerel ve maliyetli yöntemleri (`replace_file_content` veya tüm dosyayı yeniden yazma) kullanmaktadır.
- **Teknik Tasarım:**
  MCP sunucusuna yeni bir araç (`edit_file_smart` / `patch_file_smart`) eklenmesi:
  ```json
  {
    "name": "edit_file_smart",
    "description": "Applies a surgical unified diff patch or updates a specific AST symbol directly without rewriting the file.",
    "parameters": {
      "file_path": "string",
      "symbol": "optional string (target function/method)",
      "patch": "string (unified diff or new symbol body)"
    }
  }
  ```
  - **Fayda:** Modelin üretim (output/generation) aşamasındaki token israfını %70–%85 oranında keser; tam dosya bozulmalarını (truncation) AST seviyesinde engeller.

---

### 3. Çıktı Boyutu Tavanı ve Otomatik Sayfalama (Max Output Ceiling / Auto-Paging)
- **Etkilenen Dosya:** `crates/tokenjar-core/src/smart_reader.rs`
- **Problem Analizi:**
  `read_file_smart` ile 100+ satırlık bir aralık okunduğunda ve metin boyutu ~4 KB'ı aştığında, Antigravity, Claude Code ve bazı IDE arayüzleri MCP çıktısını diske (`output.txt`) kaydederek ajanı fazladan `view_file` adımı atmaya zorlamaktadır.
- **Çözüm / Uygulama Planı:**
  - `smart_reader` içine varsayılan bir `max_lines` (örn. 80 satır) veya `max_bytes` (örn. 3500 byte) güvenlik tavanı eklenmelidir.
  - Eşik aşıldığında kalan kısım kesilip çıktı sonuna şu yönlendirme eklenmelidir:
    ```text
    [TOKENJAR PAGINATION] Showing lines 1-80 of 217.
    👉 To read next slice, call read_file_smart with start_line=81, end_line=160.
    ```

---

### 4. Hibrit Sembol Arama (AST + Fuzzy / Trigram Fallback)
- **Etkilenen Dosya:** `crates/tokenjar-core/src/symbols.rs`, `crates/tokenjar-core/src/skeleton.rs`
- **Problem Analizi:**
  `find_symbol_global` aracı, AST sözdizimi tanımları ve tam sembol adlarıyla çalışır. Ajan aradığı fonksiyonun tam ismini bilmediğinde (örn. `auth_validator` yerine `verify_token` veya kavramsal bir terim aradığında) AST eşleşmesi başarısız olmakta ve fallback düz `line.contains` aramasına düşmektedir.
- **Çözüm / Uygulama Planı:**
  - AST eşleşmediğinde dosya içi docstring ve sembol adları üzerinde Levenshtein mesafesi veya Trigram benzerlik algoritması çalıştırılarak en yakın adaylar (`Did you mean: ...?`) listelenmelidir.
  - Bu sayede ajan sembol adını tam hatırlamasa bile tek adımda doğru fonksiyona yönlendirilir.

---

### 5. Format ve Tipografi Tutarlılığı
- **Etkilenen Dosya:** `crates/tokenjar-cli/src/main.rs` (`tokenjar stats`)
- `tokenjar stats` çıktısında:
  `TOTAL TOKENS SAVED: 271380 (%97.5 optimized reduction)`
  Türkçe/İngilizce karışımını önlemek amacıyla uluslararası CLI standardına göre `(97.5% optimized reduction)` veya yerelleştirilmiş dil desteğine göre biçimlendirilmelidir.

---

## 🎯 Öncelik Sıralaması (Roadmap)

1. **Sprint 1 (Kritik):** `smart_reader.rs` Slicing önbellek entegrasyonu (Madde 1) & Auto-paging tavanı (Madde 3).
2. **Sprint 2 (Büyük Özellik):** `edit_file_smart` MCP cerrahi yazma aracı (Madde 2).
3. **Sprint 3 (Arama Güçlendirme):** Fuzzy/Trigram sembol fallback mekanizması (Madde 4).
