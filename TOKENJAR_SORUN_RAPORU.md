# 🔋 TokenJar MCP Araçları — Windows Ortamı Teknik Sorun Analiz Raporu

**Tarih:** 06.10.2026  
**Platform:** Windows 11 (x64)  
**Kapsam:** TokenJar MCP Sunucusu ve Sağlanan Araçlar (`tokenjar:*`)  
**Durum:** Sorunlar Tespit Edildi, Kök Nedenleri Çıkarıldı ve Geçici Çözümler (Workarounds) Dokümante Edildi  

---

## 📑 İÇİNDEKİLER

1. [Genel Bakış](#1-genel-bakış)
2. [Sorun 1: `run_command_smart` Windows Tırnak ve Argüman Kaçırma Hatası (Quote Mangling)](#2-sorun-1-run_command_smart-windows-tırnak-ve-argüman-kaçırma-hatası-quote-mangling)
3. [Sorun 2: `get_directory_tree_tool` ve Sınırsız / Filtresiz Çıktı Taşması](#3-sorun-2-get_directory_tree_tool-ve-sınırsız--filtresiz-çıktı-taşması)
4. [Sorun 3: `read_file_smart` Çıktı Boyutu ve Zincirleme Dosyaya Yönlendirme (Output Interception Loop)](#4-sorun-3-read_file_smart-çıktı-boyutu-ve-zincirleme-dosyaya-yönlendirme-output-interception-loop)
5. [Sorun 4: `run_command_smart` Senkron Bloklanma ve Canlı Süreçler (Daemons)](#5-sorun-4-run_command_smart-senkron-bloklanma-ve-canlı-süreçler-daemons)
6. [Sorun 5: Windows `cp1254` Konsol Karakter Kodlaması ve Unicode/Emoji Çökmesi](#6-sorun-5-windows-cp1254-konsol-karakter-kodlaması-ve-unicodeemoji-çökmesi)
7. [Uygulanan Geçici Çözümler (Workarounds)](#7-uygulanan-geçici-çözümler-workarounds)
8. [TokenJar MCP Geliştiricileri İçin Mimari İyileştirme Tavsiyeleri](#8-tokenjar-mcp-geliştiricileri-için-mimari-iyileştirme-tavsiyeleri)

---

## 1. Genel Bakış

TokenJar MCP, context token tasarrufu sağlamak üzere optimize edilmiş akıllı dosya okuma ve komut çalıştırma araçları sunmaktadır. Ancak Windows işletim sistemi üzerinde çalışırken `cmd.exe` argüman işleme mekanizması, Windows varsayılan karakter kodlama tablosu (`cp1254`) ve büyük çıktı filtreleme eşikleri nedeniyle birtakım kritik sorunlar yaşanmaktadır.

---

## 2. Sorun 1: `run_command_smart` Windows Tırnak ve Argüman Kaçırma Hatası (Quote Mangling)

Windows alt süreçlerinde (`cmd.exe /c ...`), tırnak işaretleri (`"` ve `'`) TokenJar tarafından ekstradan tırnak içine alınmakta veya ters eğik çizgi (`\"`) ile hatalı şekilde kaçırılmaktadır.

### Örnek A: Çift Tırnaklı Komutlar
* **Komut:** `cmd.exe /c "dir /b"`
* **Hata Çıktısı:**
  ```text
  Exit Code: 1
  '\"dir /b\"' is not recognized as an internal or external command,
  operable program or batch file.
  ```
* **Kök Neden:** Argüman bütünlüğü bozulduğu için Windows, `"dir /b"` ifadesini argümanlı bir komut değil, var olmayan bir çalıştırılabilir dosya adı olarak aramaktadır.

### Örnek B: Python `-c` Tek Satırlık Kod Yürütme
* **Komut:** `python -c "import os; print(1)"`
* **Hata Çıktısı:**
  ```text
  Exit Code: 1
    File "<string>", line 1
      "import
      ^
  SyntaxError: unterminated string literal (detected at line 1)
  ```
* **Kök Neden:** Komuttaki ilk tırnak içeriye literal karakter olarak kaçırıldığı için Python dizeyi başlatamamakta ve kapatılmamış string hatası vermektedir.

### Örnek C: PowerShell Boru Hattı (`|`) Karışıklığı
* **Komut:** `powershell -Command "Get-Content logs\stock_bot.log | Select-Object -First 10"`
* **Sonuç:**
  ```text
  Exit Code: 0
  Get-Content logs\stock_bot.log | Select-Object -First 10
  ```
* **Kök Neden:** Komut icra edilmek yerine ekrana düz metin (`echo`) olarak basılmış, pipeline işletilmemiştir.

---

## 3. Sorun 2: `get_directory_tree_tool` ve Sınırsız / Filtresiz Çıktı Taşması

* **Gözlem:** Proje kök dizininde `get_directory_tree_tool` çağrıldığında, `data/` altındaki derin veri klasörleri, `.git`, `__pycache__` veya çok sayıda dosya içeren dizinler için varsayılan bir derinlik sınırı (`max_depth`) ya da akıllı filtreleme uygulanmamaktadır.
* **Sonuç:** Üretilen ağaç çıktısı doğrudan binlerce satıra ulaştığı için context window koruma mekanizması devreye girmiş ve çıktıyı doğrudan `.system_generated/steps/XX/output.txt` dosyasına yönlendirmiştir. Ağacı incelemek pratikte imkansız hale gelmiştir.

---

## 4. Sorun 3: `read_file_smart` Çıktı Boyutu ve Zincirleme Dosyaya Yönlendirme (Output Interception Loop)

* **Gözlem:** TokenJar'ın `read_file_smart` aracı satır bazlı okuma (`start_line`, `end_line`) desteğine sahip olsa da, TokenJar'ın eklediği meta veriler ve satır içeriği yaklaşık 1–2 KB'yi aştığında şu durum yaşanmaktadır:
  1. `read_file_smart` çağrılır.
  2. Dönen metin framework boyut eşiğini aşar.
  3. Çıktı `.system_generated/steps/A/output.txt` dosyasına yazılır.
  4. Bu dosyayı okumak için tekrar `read_file_smart` çağrıldığında, eğer dilim yeterince küçük tutulmazsa o da `.system_generated/steps/B/output.txt` üretir.
* **Kritik Eşik:** Güvenli okuma yapabilmek için satır dilimlerinin maksimum 20–30 satırla sınırlandırılması gerektiği görülmüştür.

---

## 5. Sorun 4: `run_command_smart` Senkron Bloklanma ve Canlı Süreçler (Daemons)

* **Gözlem:** `python src/main.py` veya `streamlit run ...` gibi sürekli açık kalan borsa botu ve web paneli süreçleri `run_command_smart` ile çağrıldığında, araç sürecin tamamlanmasını senkron olarak beklemektedir.
* **Sonuç:** Araç asenkron mod / zaman aşımı ayrımına sahip olmadığı için botun sonsuz döngüsü nedeniyle kilitlenmekte; bu nedenle uzun soluklu süreçlerin yerel `run_command (IsDaemon/WaitMsBeforeAsync)` ve `manage_task` araçlarına devredilmesi zorunlu olmaktadır.

---

## 6. Sorun 5: Windows `cp1254` Konsol Karakter Kodlaması ve Unicode/Emoji Çökmesi

* **Gözlem:** TokenJar üzerinden komut çalıştırılırken Windows Türkiye yerel ayarı (`cp1254`) aktif olduğunda, Python çıktılarındaki modern emojiler (`🏛️`, `🚀`, `👑`, `🛡️`) konsol akışında `UnicodeEncodeError: 'charmap' codec can't encode characters` hatası üretmektedir.
* **Kök Neden:** TokenJar komut çalıştırırken alt süreç ortamına otomatik olarak `PYTHONIOENCODING=utf-8` ve `PYTHONUTF8=1` çevre değişkenlerini enjekte etmemektedir.

---

## 7. Uygulanan Geçici Çözümler (Workarounds)

Bu teknik aksaklıkları aşmak ve tüm analizleri %100 doğrulukla tamamlamak için tarafımızca şu pratik çözümler geliştirilmiştir:

1. **Scratch Betik Mimarisi (Tırnak Sorununa Karşı):**
   * Terminal parametresi içine tırnaklı Python kodları yazmak yerine, geçici `.py` dosyaları oluşturulmuş (`write_to_file`) ve `python scratch_script.py` şeklinde argümansız çağrılmıştır.
2. **UTF-8 Çıktı Sarmalayıcısı:**
   * Betiklerin başına `sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')` eklenerek Windows `cp1254` çökmeleri tamamen engellenmiştir.
3. **Mikro Dilimli Dosya Okuma:**
   * `read_file_smart` çağrılarında `start_line` ve `end_line` aralıkları maksimum 30 satır tutularak framework'ün zincirleme dosya yönlendirmesi devre dışı bırakılmıştır.

---

## 8. TokenJar MCP Geliştiricileri İçin Mimari İyileştirme Tavsiyeleri

1. **Windows Komut Ayrıştırma:**
   * Windows üzerinde komutlar ham string birleştirmesi yerine `subprocess.list2cmdline` kurallarına tam uyumlu hale getirilmelidir. Halihazırda tırnak içine alınmış parametrelere ek kaçış (`\"`) uygulanmamalıdır.
2. **Otomatik UTF-8 Çevre Değişkenleri:**
   * `run_command_smart`, Windows ortamında komut çalıştırırken alt sürece varsayılan olarak `PYTHONIOENCODING=utf-8` ve `PYTHONUTF8=1` değişkenlerini otomatik geçmelidir.
3. **`get_directory_tree_tool` Akıllı Filtreleme:**
   * Varsayılan olarak maksimum derinlik (`max_depth=3`) uygulanmalı ve `data/`, `.git`, `node_modules`, `__pycache__` gibi hacimli dizinler özetlenerek sunulmalıdır.
4. **Zaman Aşımı / Arka Plan Desteği:**
   * `run_command_smart` aracına `timeout_seconds` veya `background: bool` parametresi eklenerek sonsuz döngülü betiklerde askıda kalması önlenmelidir.
