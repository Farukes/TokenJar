<p align="right">
  <a href="README.md"><b>English</b></a> | <a href="README.tr.md"><b>Türkçe</b></a>
</p>

<p align="center">
  <img src="docs/images/tokenjar_logo.jpg" alt="TokenJar Logo" width="220" style="border-radius: 16px;" />
</p>

<h1 align="center">🍯 TokenJar</h1>
<p align="center"><b>Token'ları kumbarana geri koy. Yapay zekâ kodlama asistanları için sıfır maliyetli token kumbarası ve akıllı optimizasyon motoru.</b></p>

[![Release: v1.1.1](https://img.shields.io/badge/S%C3%BCr%C3%BCm-v1.1.1-green.svg)](https://github.com/Farukes/TokenJar/releases/latest)
[![CI](https://github.com/Farukes/TokenJar/actions/workflows/ci.yml/badge.svg)](https://github.com/Farukes/TokenJar/actions/workflows/ci.yml)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%20%7C%203.11%20%7C%203.12-blue.svg)](https://www.python.org/downloads/)
[![Kurumsal Yerel Motor: Rust](https://img.shields.io/badge/Kurumsal%20Yerel%20Motor-Rust%20v1.1.1-orange.svg)](#kurumsal-motor)
[![Crates.io: v1.1.1](https://img.shields.io/badge/crates.io-v1.1.1-orange.svg?logo=rust&logoColor=white)](https://crates.io/crates/tokenjar)
[![PyPI: v1.1.1](https://img.shields.io/badge/PyPI-v1.1.1-blue.svg?logo=pypi&logoColor=white)](https://pypi.org/project/tokenjar/)
[![Token Tasarrufu](https://img.shields.io/badge/Token%20Tasarrufu-%2590%20ile%20%2599-brightgreen.svg)](#performans-testleri)
[![Lisans: BSL 1.1](https://img.shields.io/badge/Lisans-BSL%201.1-blue.svg)](LICENSE)
[![Zero Telemetry](https://img.shields.io/badge/telemetri-0%25%20--%20%25100%20yerel-success.svg)](#gizlilik-garantisi)

**Yapay zeka kodlama asistanları için işlevsellikten ödün vermeden %70-95 token tasarrufu sağlayan MCP sunucusu.**

TokenJar, yapay zeka kodlama asistanınız ile kod tabanınız arasında yer alarak kod okumalarını, terminal çıktılarını ve dosya işlemlerini akıllıca sıkıştırır; token tüketimini, bağlam sıkıştırmasını (context compaction) ve gecikmeyi önemli ölçüde azaltır.

**Claude Code**, **Cursor**, **Antigravity (AGY)**, **Windsurf**, **Continue.dev** ve tüm MCP uyumlu yapay zeka asistanlarıyla sorunsuz çalışır.

---

## ✨ Özellikler ve Mimari

| Modül | Ne Yapar | Token Tasarrufu |
|:---|:---|:---|
| 🦴 **Kod İskeleti Çıkarıcı (Code Skeletonizer)** | Tree-sitter AST ile yapısal iskelet (imzalar, tipler, docstring'ler) çıkarır | **%80-95** |
| 📖 **Akıllı Dosya Okuyucu & Dilimleme Önbelleği** | L1 RAM + L2 Kalıcı SQLite önbelleği, hedefli satır/sembol dilimleme (`#L1-30`) & otomatik sayfalama tavanı (`MAX_OUTPUT_LINES = 80`) | **%90-99.8** |
| 🛡️ **Lockfile ve Statik Varlık Kalkanı** | Devasa kilit dosyalarını ve minify edilmiş paketleri yakalayarak cerrahi sürüm sorgusu (`query="react"`) sunar | **%99** |
| 🎯 **Etki Alanı ve Hibrit Fuzzy Semboller** | AST motoru ve yazım hatası toleranslı Levenshtein + Trigram önerileriyle anlık global sembol arama (`find_symbol_global`) | **%85-95** |
| 🖥️ **Terminal Budayıcı (Terminal Pruner)** | Test/derleme/git terminal akışlarını sıkıştırır, hataları ve özet bilgileri korur | **%60-90** |
| 🗺️ **Repo Haritası (Repo Map)** | PageRank ve Graf Merkeziliği algoritmalarıyla özel token bütçelerine sığdırılan kod haritası | **Bütçeye uyarlanmış** |
| 🔄 **Güncelleme Sonrası Otomatik Eşitleme** | Tek komutla (`tokenjar update`) binary, IDE ayarları, slash komutları ve tüm projelerdeki `AGENTS.md` kurallarını eşitler | **Sıfır bakım** |
| 🧹 **Sıfır Kalıntılı Güvenli Kaldırma** | Kullanıcı kodlarına dokunmadan tüm ayarları geri alır, önbelleği sıfırlar (`tokenjar uninstall`) | **Güvenli ve Sıfır iz** |
| 🎨 **İsteğe Bağlı Kontrol Paneli (UI)** | **Sıfır Arka Plan RAM** tüketen hafif bağımsız kontrol paneli (`tokenjar ui`) | **Anlık** |

### 🛡️ Yerleşik Güvenlik Önlemleri ve Güvenilirlik
- **Lockfile ve Devasa Varlık Kalkanı:** 50.000 satırlık kilit dosyalarının bağlam penceresini yok etmesini engeller; 5 satırlık cerrahi sürüm sorgularını destekler.
- **L1 RAM + L2 SQLite Kalıcı Önbellek:** MCP sunucusu ve IDE yeniden başlatmalarından etkilenmez (`~/.tokenjar/cache.db` WAL moduyla çalışır).
- **Güvenli Hata Geri Dönüşü (Fallback Safety Guard):** Test veya komut başarısız olduğunda (`exit_code != 0`), TokenJar traceback'lerin ve hata bağlamının eksiksiz korunmasını garanti eder.
- **Küçük Dosya Anomali Koruması:** Diff başlığı dosyanın kendisinden daha fazla token tüketecekse, token şişmesini önlemek için dosyanın tam içeriği döndürülür.
- **Kontrolsüz Akış Koruması (Runaway Stream Protection):** Sonsuz döngülerde ham terminal arabelleklerini 2 MB ile sınırlandırarak bellek taşmasını önler.
- **SQLite Veritabanı Şişme Koruması:** 5 MB'tan büyük dosyalar disk alanını şişirmemek için karma referansıyla (hash) önbelleğe alınır.

---

## 🚀 Hızlı Başlangıç ve Kurulum (v1.1.0 GA)

TokenJar iki resmi sürüm halinde dağıtılmaktadır:
1. **🦀 Rust Yerel Motoru (Önerilen):** Mikrosaniyelik AST ayrıştırma, 14 MB RAM ve sıfır Python bağımlılığı içeren yüksek performanslı tekil ikili dosya.
2. **🐍 Python Sürümü:** pip ve sanal ortamlar (venv) için saf Python FastMCP paketi.

### 💡 Önerilen Kullanım Akışı (En İyi Pratik — 3 Kolay Adım)

> [!TIP]
> **Geçici dosyaları temizlerken dosya yollarının kopmasını önlemek için:**
> 1. **Kalıcı ve Güvenli Bir Klasöre Çıkarın:** Bağımsız binary veya zip dosyasını elle indiriyorsanız, `tokenjar` dosyasını silinmeyecek kalıcı bir dizine çıkarın (Örn: Windows'ta `C:\Users\<Kullanıcı>\.tokenjar\bin`, Linux/macOS'ta `~/.local/bin` veya sabit bir geliştirici klasörü). `İndirilenler (Downloads)` veya `Temp` gibi sonradan temizlenebilecek geçici klasörlerde bırakmayın. *(Not: Seçenek 1'deki tek satırlık terminal yükleyicileri bu adımı zaten otomatik olarak kalıcı dizine yapar).*
> 2. **Tek Seferlik Kurulumu Çalıştırın:** O klasörde terminal açıp şu komutu girin:
>    ```bash
>    tokenjar install
>    ```
>    Bu komut TokenJar'ı sistem `PATH` ortam değişkenine ekler, tespit edilen tüm yapay zeka asistanlarına (Cursor, Claude Desktop, Antigravity, Windsurf, Claude Code) MCP sunucusunu otomatik bağlar ve slash komutlarını yükler.
> 3. **Projenizde Aktifleştirin (Terminal/CLI Önerilir):** Kodlama yapacağınız projenin dizininde bir terminal açıp şu komutu çalıştırın:
>    ```bash
>    tokenjar on
>    ```
>    *(Bu komutu yapay zeka sohbetine yazmak yerine doğrudan kendi terminalinizde/CLI'da çalıştırmanız özellikle önerilir; çünkü sohbet token'ı harcamadan kuralları anında hazır eder ve yapay zeka ilk adımdan itibaren kuralları okuyarak başlar).* Bu işlem projenize `AGENTS.md` kurallarını ekleyerek asistanınızın TokenJar'ın yüksek tasarruflu MCP araçlarını kullanmasını sağlar.

### 📥 Doğrudan İndirme Bağlantıları (Derlenmiş v1.1.0 İkili Dosyaları)

İşletim sisteminize tıklayarak en güncel v1.1.0 sürümünü anında indirin:

| Platform | Mimari | Tıkla ve İndir | Format |
|:---|:---|:---|:---|
| 🪟 **Windows** | x86_64 (64-bit) | [**⬇️ tokenjar-windows-x64.zip İndir**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-windows-x64.zip) | Bağımsız `.exe` + Yükleyici |
| 🐧 **Linux** | x86_64 (64-bit) | [**⬇️ tokenjar-linux-x64.tar.gz İndir**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-linux-x64.tar.gz) | Bağımsız İkili Dosya |
| 🍏 **macOS** | Apple Silicon (M1/M2/M3/M4) | [**⬇️ tokenjar-macos-arm64.tar.gz İndir**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-macos-arm64.tar.gz) | Bağımsız İkili Dosya |
| 🍏 **macOS** | Intel x86_64 | [**⬇️ tokenjar-macos-x64.tar.gz İndir**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-macos-x64.tar.gz) | Bağımsız İkili Dosya |
| 🐍 **Python** | Çapraz Platform | [**⬇️ tokenjar-python.zip İndir**](https://github.com/Farukes/TokenJar/releases/latest/download/tokenjar-python.zip) | Python Wheel (.whl) |

---

### ⚡ 1. Seçenek: Rust Yerel Motoru (Tek Satır Terminal Kurulumu)
> **En İyisi:** En yüksek hız, 14 MB RAM, mikrosaniyelik Tree-sitter AST ve sıfır Python bağımlılığı.

Terminalinize tek satır yapıştırarak `tokenjar`'ı otomatik yükleyin ve sistem PATH'inize ekleyin:

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/Farukes/TokenJar/main/install.ps1 | iex
```

**Windows (CMD / Komut İstemi):**
```cmd
powershell -ExecutionPolicy Bypass -Command "iwr -useb https://raw.githubusercontent.com/Farukes/TokenJar/main/install.ps1 | iex"
```

```bash
# Linux ve macOS (Bash):
curl -fsSL https://raw.githubusercontent.com/Farukes/TokenJar/main/install.sh | bash
```

**Veya Cargo (crates.io) ile kurulum:**
```bash
cargo install tokenjar
```

---

### 🐍 2. Seçenek: Python Sürümü (pip)
> **En İyisi:** Python odaklı geliştirme ortamları, özel betik entegrasyonları veya pip iş akışları.

```bash
# PyPI üzerinden kurulum
pip install tokenjar

# Veya doğrudan GitHub main dalından kurulum:
pip install git+https://github.com/Farukes/TokenJar.git
```

---

### 🔄 TokenJar'ı Güncelleme

Mevcut TokenJar kurulumunuzu en son özelliklere ve performans iyileştirmelerine yükseltmek için:

```bash
# TokenJar CLI ile otomatik güncelleme ('update' veya 'upgrade' ikisi de geçerlidir):
tokenjar update
# veya:
tokenjar upgrade

# Varsa zorla yeniden indirip kurmak için:
tokenjar update --force

# pip ile kurulduysa:
pip install --upgrade tokenjar

# Cargo ile kurulduysa:
cargo install tokenjar --force
```

---

<a id="performans-testleri"></a>
## 📊 Kanıtlanmış Performans ve Stres Testi Sonuçları

100 adımlık gerçek geliştirici stres testi ve 50 döngülük eşit şartlardaki MCP testinden elde edilen net ölçüm sonuçları:

| Metrik | 1. Düz AI (TokenJar Yok) | 2. TokenJar Python | 3. TokenJar Rust (v1.1.0) | Rust Avantajı |
|:---|:---|:---|:---|:---|
| **Tüketilen Token (100 Adım)** | 622.892 tokens | 95.492 tokens | **68.641 tokens** | **%89.0 net tasarruf (554k token kurtarıldı)** |
| **Uçtan Uca Kodlama Tasarrufu** | 166.513 tokens | 12.400 tokens | **6.585 tokens** | **🚀 %96.0 net tasarruf (Cerrahi bloklar)** |
| **API Maliyeti (100 Adım)** | $1.8687 | $0.2865 | **$0.2059** | **Her 100 adımda $1.66 net tasarruf** |
| **Toplam Yürütme Süresi (100 Adım)** | 0.357 s (ham disk) | 2.618 s | **0.985 s** | **Python'dan 2.7 kat daha hızlı** |
| **Isınmış Önbellek Gecikmesi** | Yok | 23.6 ms | **8.1 ms** | **3.0 kat daha hızlı işlem** |
| **Bellek (RAM) Ayak İzi** | ~30.0 MB | 49.1 MB | **15.0 MB** | **%70 ile %84 daha az bellek** |
| **Kalite ve Doğruluk Skoru** | %100.0 | %100.0 | **%100.0 (100 / 100 Tam Puan)** | **Sıfır mantık/içerik kaybı** |
| **Sentaks Bütünlüğü & Sıfır Kesinti** | Ham (Doğrulanmamış) | ✅ Uygulandı | ✅ **Uygulandı** | **Tembel yorumlar (TODO) yasaklandı** |

---

### 📝 Proje Yönlendirme Kuralları (`AGENTS.md`)

TokenJar yapay zeka optimizasyon yönergelerini depo kurallarınıza ekleyin veya kaldırın:

```bash
# 🟢 TokenJar'ı mevcut projede etkinleştirir (AGENTS.md, .cursorrules oluşturur/günceller)
tokenjar on

# ⚪ TokenJar'ı mevcut projeden temizler (TokenJar bloğunu güvenle söker)
tokenjar off
```

### 🎛️ Çıktı Optimizasyonu Kontrolleri (CLI ve Terminaller)

Kompakt cerrahi diff çıktısı ile varsayılan sınırsız çıktı arasında net komutlarla geçiş yapın:

```bash
# 🟢 Kompakt cerrahi diff'leri ve sıfır kesinti kalite kuralını etkinleştirir
tokenjar output on

# ⚪ Yapay zeka asistanını varsayılan kısıtlamasız çıktı ayarlarına döndürür
tokenjar output off

# 📊 Mevcut çıktı yapılandırma durumunu görüntüler
tokenjar output
```

Yapay zeka asistanınızın sohbet ekranında slash komutları da desteklenir (`/tokenjar output on`, `/tokenjar output off`).

---

## 🔌 Yapay Zeka Asistanınızla Kurulum

### ⚡ Tek Tıkla Otomatik Kurulum (Önerilen)

Claude Desktop, Cursor, Antigravity, Windsurf, Claude Code ve VS Code genelinde TokenJar MCP sunucusunu otomatik olarak algılar ve kurar:

```bash
# 📦 Tek seferlik kurulum: PATH'e ekler, IDE'lerde MCP'yi açar, slash komutlarını kurar
tokenjar install

# 🟢 Mevcut projede TokenJar'ı etkinleştirir (AGENTS.md üretir)
tokenjar on

# 🌐 Veya MCP sunucusunu algılanan tüm IDE'lerde genel olarak açıp kapatın
tokenjar enable
tokenjar disable
```

### Manuel Yapılandırma

Manuel olarak yapılandırmayı veya diğer istemcileri kullanmayı tercih ederseniz:

<details>
<summary><b>Claude Code</b></summary>

```bash
claude mcp add tokenjar -- python -m tokenjar
```
</details>

<details>
<summary><b>Cursor</b></summary>

`.cursor/mcp.json` dosyasını oluşturun veya güncelleyin:
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

`~/.gemini/config/mcp_config.json` dosyasına ekleyin:
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

`~/.codeium/windsurf/mcp_config.json` dosyasına ekleyin:
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

`.continue/config.yaml` dosyasına ekleyin:
```yaml
mcpServers:
  - name: tokenjar
    command: python
    args: ["-m", "tokenjar"]
```
</details>

---

## 🛠️ Kullanılabilir MCP Araçları

- **`find_symbol_global(query, root_path=".", exact=False)`**: Birden fazla dosyayı okumaya gerek kalmadan kod tabanının tamamında fonksiyon, metot veya sınıfları isme göre arar. Yazım hatalarına toleranslı arama için Levenshtein mesafesi geri dönüşlü hibrit bulanık (fuzzy) eşleştirme içerir.
- **`find_symbol_references(symbol_name, root_path=".", max_results=25)`**: Etki alanı (blast radius) referans analizi. Kod düzenlemeden veya yeniden yapılandırmadan (refactor) önce kod tabanındaki tüm çağıranları, import'ları ve kullanımları bulur.
- **`tool_get_code_skeleton(file_path)`**: Bir dosyanın yapısal iskeletini çıkarır — sınıflar, fonksiyon imzaları, docstring'ler ve tip açıklamaları korunur, gövdeler `...` ile değiştirilir. (Python, JS/TS, Go, Rust, Java, C/C++, C#, Ruby, PHP, Kotlin dillerini destekler).
- **`tool_get_symbol(file_path, symbol_name)`**: İskeleti inceledikten sonra belirli bir sınıf veya fonksiyonun tam uygulamasını isme göre getirir.
- **`read_file_smart(file_path, symbol=None, force_full=False, query="", start_line=None, end_line=None)`**: Oturum önbelleği, cerrahi satır dilimleme, sembol çıkarımı ve Kilit Dosyası Kalkanı (Lockfile Shield) içeren diferansiyel dosya okuyucu. Tüm dosyayı okumadan sembolü doğrudan tek adımda çıkarmak için `symbol="fonksiyon_adi"` parametresini destekler. Satır numaralarıyla hedeflenen aralığı incelemek için `start_line` ve `end_line` (1-indeksli, dahilî) parametrelerini destekler ve otomatik sayfalama tavanı (`MAX_OUTPUT_LINES = 80`) uygular. Değişmeyen dosyalarda `[CACHED] unchanged` (~3 token) veya birleşik diff döndürür. Kilit dosyaları (`package-lock.json`, `Cargo.lock` vb.) için 50.000 satır yerine cerrahi 5 satırlık sürüm blokları almak için `query="paket-adi"` parametresini destekler.
- **`run_command_smart(command, cwd=".")`**: Kabuk komutlarını çalıştırır ve pytest, jest, npm, cargo ile git çıktılarındaki gereksiz ayrıntıları budar.
- **`filter_output(output, output_type="auto")`**: Komut çalıştırmadan test çalıştırıcıları, derleme işlem hatları ve sürüm kontrol günlükleri için saf metin filtresi uygular.
- **`get_repo_map_tool(root_path=".", max_tokens=1000)`**: Dosyalar arası import ilişkilerine göre önceliklendirilmiş graf merkeziliği kod tabanı haritası.
- **`get_directory_tree_tool(root_path=".", max_depth=4)`**: `.gitignore` kurallarına uyan ve ikili (binary) dizinleri atlayan hafif dizin ağacı.
- **`cache_stats()`**: Oturum okuma isabetlerini (hit), ıskalamalarını (miss), diff'leri ve toplam token tasarrufunu inceler.

### 📦 MCP Kaynakları ve Komut İstekleri (Prompts)

- **Kaynaklar (Resources):**
  - `tokenjar://stats`: Canlı kümülatif token ve maliyet tasarrufu paneli.
  - `tokenjar://guide`: Yapay zeka asistanı en iyi uygulama optimizasyon yönergeleri.
  - `tokenjar://config`: Etkin proje yapılandırması ve yoksayma ayarları.
- **Komut İstekleri (Prompts):**
  - `optimize_coding_task(task_description)`: Asistanları token verimli iş akışlarına yönlendiren sistem istemi şablonu.

---

## ⚙️ Proje Yapılandırması (`tokenjar.toml`)

Hariç tutulacak dosyaları ve bütçeleri özelleştirmek için deponuzun kök dizininde isteğe bağlı bir `tokenjar.toml` oluşturun:

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

## 💻 CLI Komutları ve Kabuk Kancaları

TokenJar, geliştiriciler ve yerel kabuk otomasyonu için etkileşimli bir komut satırı aracı olarak da işlev görür:

```bash
# 📦 Tek Tıkla Sistem Kurulumu ve Entegrasyonlar
tokenjar install            # Tek seferlik kurulum: PATH'e ekler, IDE'lerde MCP'yi açar, slash komutlarını kurar
tokenjar on                 # TokenJar'ı mevcut projede etkinleştirir (AGENTS.md kurallarını üretir/günceller)
tokenjar off                # TokenJar'ı mevcut projede kapatır (AGENTS.md kurallarını temizler)
tokenjar on --global        # MCP sunucusunu algılanan tüm IDE'lerde genel olarak yapılandırır
tokenjar off --global       # MCP sunucusunu algılanan tüm IDE'lerden genel olarak kaldırır
tokenjar enable             # Algılanan tüm yapay zeka asistanlarında TokenJar MCP'yi açar
tokenjar disable            # Algılanan tüm yapay zeka asistanlarında TokenJar MCP'yi kapatır

# 📊 İzleme, Telemetri ve Web Kontrol Paneli
tokenjar status             # Yapay zeka asistanları ve IDE'ler genelindeki çalışma durumunu kontrol eder
tokenjar stats              # Canlı performans, token tasarrufu ve finansal kazanç panelini görüntüler
tokenjar ui                 # Etkileşimli Web Kontrol Panelini tarayıcınızda açar (Sıfır Arka Plan RAM)
tokenjar ui --port 4141     # Web Kontrol Paneli için özel port belirtir

# 🔄 Otomatik Güncelleme ve Kendini Onarma
tokenjar update             # Binary + IDE ayarları + slash komutları + proje kurallarını tek komutla günceller
tokenjar update --force     # En son sürümde olsa bile yeniden indirmeyi ve eşitlemeyi zorlar

# 🧹 Önbellek ve Telemetri Yönetimi
tokenjar clean              # Telemetri istatistiklerini sıfırlar ve L2 SQLite önbelleğini temizler
tokenjar clean --cache      # Yalnızca L2 SQLite önbelleğini temizler
tokenjar clean --stats      # Yalnızca telemetri istatistiklerini sıfırlar
tokenjar cache-prune --ttl-days 30 --max-entries 5000 # Süresi dolmuş veya fazla önbellek kayıtlarını temizler

# ⚡ Geliştirici Araçları ve Çıktı Budama
tokenjar run "pytest"       # Kabuk komutunu akıllı token tasarruflu çıktı budama ile çalıştırır
tokenjar run "npm test"     # Hataları ve özetleri korur, binlerce satırlık tekrarlı logları budar
RAW=1 tokenjar run "pytest" # Geçici ham çıktı kuralı (veya --raw parametresi)
tokenjar hook               # Şeffaf CLI yakalama kancalarını kabuk profiline ekler (PowerShell/Bash)
tokenjar unhook             # Şeffaf CLI yakalama kancalarını kabuk profilinden kaldırır
tokenjar output on          # Kompakt cerrahi diff ve sıfır kesinti modunu açar
tokenjar output off         # Yapay zeka asistanını varsayılan sınırsız çıktıya döndürür

# ⚠️ Sıfır Kalıntılı Cerrahi Kaldırma
tokenjar uninstall          # IDE ayarlarını geri alır, kuralları temizler, önbelleği, hook'ları ve PATH'i siler
tokenjar uninstall --yes    # Onay istemini atlayarak hemen kaldırır
```

---

<a id="gizlilik-garantisi"></a>
## 🔒 Kurumsal Gizlilik ve Güvenlik Garantisi

TokenJar kesinlikle **Sıfır-Telemetri, %100 Localhost** tasarım felsefesiyle geliştirilmiştir:

- **%100 Yerel Yürütme:** Tüm ayrıştırma (Tree-sitter), önbellekleme (SQLite) ve çıktı filtreleme işlemleri yerel olarak işlemcinizde (CPU) gerçekleşir.
- **Sıfır Dış Ağ Çağrısı:** Telemetri sunucusu, analitik izleyici, giden ping veya herhangi bir bulut bağımlılığı kesinlikle yoktur.
- **Air-Gapped / Çevrimdışı Ortamlarla Uyumlu:** Gizli, çevrimdışı veya izole kurumsal şirket ağlarında güvenle çalışır.
- **Yerel Veri İzolasyonu:** Kalıcı önbellek (`~/.tokenjar/cache.db`) ve istatistikler (`~/.tokenjar/telemetry.json`) yalnızca kullanıcı dizininizde bulunur ve `tokenjar clean --stats` ile veya dizin silinerek istenildiği zaman tamamen temizlenebilir.
- **Müdahalesiz Mimari:** Yapay zeka asistanının açık talimatı olmadan proje kodunuzu asla değiştirmez.

---

<a id="kurumsal-motor"></a>
## 🦀 Kurumsal ve Yüksek Performanslı Yerel Motor (Rust Sürümü)

Kurumsal çalışma ortamları, devasa monorepolar (50.000+ dosya), CI/CD süreçleri veya sisteminde Python kurulu olmayan geliştiriciler için TokenJar, sıfır bağımlılıklı ve ultra hızlı yerel bir Rust ikili dosyası (`tokenjar.exe` / bağımsız binary) sunar.

### Neden Kurumsal Yerel Motor?
- **Sıfır Çalışma Zamanı Bağımlılığı:** Python, pip, Node.js veya sanal ortam (venv) gerektirmez. Tek bir çalıştırılabilir dosya.
- **Ultra Düşük Gecikme:** Anında başlangıç (~3 ms soğuk açılış süresi) sayesinde MCP araç çağrılarında sıfır bekleme süresi.
- **Yüksek Eşzamanlı İndeksleme:** Çok iş parçacıklı (Rayon + Tokio) paralel kod ayrıştırma ve sembol çıkarımı.
- **Gömülü 15 Dil AST Motoru:** Rust, C, C++, Go, C#, Java, Python, JavaScript, TypeScript, PHP, Ruby, Bash, HTML, CSS, JSON dilleri için statik olarak ikili dosyaya gömülü Tree-sitter ayrıştırıcıları.
- **Minimal Bellek Tüketimi:** Aktif çalışma sırasında yalnızca ~8-15 MB RAM tüketir.

### Kurumsal Hızlı Başlangıç (Bağımsız Binary)

Önceden derlenmiş ikili dosyayı [GitHub Releases](https://github.com/Farukes/TokenJar/releases) sayfasından indirin veya doğrudan Cargo ile derleyin:

```bash
# Kaynak koddan optimize yerel ikili dosyayı derleyin
cargo build --release --workspace

# Bağımsız dosya kullanıma hazır:
./target/release/tokenjar.exe status
```

### Kurumsal MCP Yapılandırması (`claude_desktop_config.json` / Cursor)
Herhangi bir Python sarmalayıcısına gerek kalmadan doğrudan binary dosyasını gösterin:

```json
{
  "mcpServers": {
    "tokenjar": {
      "command": "C:\\dosya\\yolu\\tokenjar.exe"
    }
  }
}
```

---

## 🌍 Desteklenen Diller

TokenJar, AST ayrıştırma için Tree-sitter kullanır ve kullanıma hazır olarak **130'dan fazla programlama dilini** destekler:

Python · TypeScript · JavaScript · Go · Rust · Java · C# · C / C++ · Ruby · PHP · Swift · Kotlin · Scala · Dart · Lua · Elixir · Haskell · ve daha fazlası.

---

## 🧪 Geliştirme ve Kalite Güvencesi

TokenJar, her iki uygulamada da %100 işlevsel eşliği garanti eden çift test paketi barındırır:

```bash
# Python (Topluluk Sürümü & MCP SDK)
pip install -e ".[dev]"
pytest tests/ -v           # 72 test başarılı (%100 geçer)

# Rust (Kurumsal Yerel Motor)
cargo test --workspace    # 45 çekirdek test + 3 benchmark/stres paketi başarılı (%100 geçer)
```

---

## 📄 Lisans ve Fikri Mülkiyet

Copyright © 2026 Ömer Faruk Eskitürk. Tüm hakları saklıdır.

Bu yazılım **Business Source License 1.1 (BSL 1.1)** altında lisanslanmıştır ve süresi dolduğunda otomatik olarak **Apache License, Version 2.0** lisansına dönüşür:
- **Ücretsiz Kullanım:** Kişisel, eğitim, araştırma, değerlendirme ve şirket içi dahili kullanım için tamamen ücretsizdir.
- **Ticari Kısıtlamalar:** Lisans sahibine rakip olacak şekilde ücretli bir ticari SaaS, bulut servisi veya ücretli dağıtım olarak barındırılamaz/satılamaz.
- **Apache 2.0 Geçişi:** 01.01.2030 tarihinde otomatik olarak %100 açık kaynaklı Apache 2.0 lisansına dönüşür.

Yasal şartların tamamı için [LICENSE](LICENSE) dosyasına bakın.
