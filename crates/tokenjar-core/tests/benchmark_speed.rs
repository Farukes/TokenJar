use std::path::PathBuf;
use std::time::Instant;
use tokenjar_core::cache::session_cache::SessionCache;
use tokenjar_core::config::TokenJarConfig;
use tokenjar_core::smart_reader::read_file_smart;
use tokenjar_core::telemetry::TelemetryTracker;

#[test]
fn benchmark_savings_and_speed_rust() {
    let cache = SessionCache::new();
    let config = TokenJarConfig::default();
    let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
    let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let test_cases = vec![
        (
            "src/installer.rs",
            "get_supported_ide_configs",
            "Core Installer IDE configs",
        ),
        (
            "src/installer.rs",
            "install_mcp_all",
            "Core Installer MCP batch installer",
        ),
        (
            "src/smart_reader.rs",
            "read_file_smart",
            "Smart Reader core function",
        ),
        (
            "src/skeleton.rs",
            "find_symbol_range_in_code",
            "AST symbol range finder",
        ),
        ("src/rules.rs", "generate_rules", "Rules generator"),
        (
            "src/token_counter.rs",
            "format_savings",
            "Token counter savings formatter",
        ),
    ];

    println!("\n=========================================================================================");
    println!("⚡ TOKENJAR NATIVE RUST ENGINE: TASARRUF VE HIZ PERFORMANS BENCHMARK'I");
    println!(
        "========================================================================================="
    );
    println!(
        "{:<25} | {:<27} | {:<7} | {:<7} | {:<8} | {:<10}",
        "Dosya", "Sembol", "Ham Tok", "Dilim", "Tasarruf", "Hız (Süre)"
    );
    println!("{:-<95}", "");

    let mut total_orig = 0u64;
    let mut total_opt = 0u64;

    for (rel_path, symbol, _desc) in test_cases {
        let abs_path = manifest_dir.join(rel_path);
        let path_str = abs_path.to_str().unwrap();
        let content = std::fs::read_to_string(&abs_path).expect("Failed to read file");
        let orig_tok = (content.len() / 4) as u64;

        // Warmup
        let _ = read_file_smart(
            path_str,
            false,
            None,
            None,
            None,
            Some(symbol),
            &cache,
            &config,
            &tracker,
        );

        // Precise Timing (average over 10 iterations)
        let iters = 10;
        let start = Instant::now();
        let mut result = String::new();
        for _ in 0..iters {
            result = read_file_smart(
                path_str,
                false,
                None,
                None,
                None,
                Some(symbol),
                &cache,
                &config,
                &tracker,
            );
        }
        let elapsed = start.elapsed() / iters;
        let duration_str = if elapsed.as_micros() < 1000 {
            format!("{} µs", elapsed.as_micros())
        } else {
            format!("{:.2} ms", elapsed.as_secs_f64() * 1000.0)
        };

        let opt_tok = (result.len() / 4) as u64;
        let saved = orig_tok.saturating_sub(opt_tok);
        let pct = if orig_tok > 0 {
            (saved as f64 / orig_tok as f64) * 100.0
        } else {
            0.0
        };

        total_orig += orig_tok;
        total_opt += opt_tok;

        let short_name = std::path::Path::new(rel_path)
            .file_name()
            .unwrap()
            .to_string_lossy();
        println!(
            "{:<25} | {:<27} | {:<7} | {:<7} | %{:<7.1} | {:<10}",
            short_name, symbol, orig_tok, opt_tok, pct, duration_str
        );
    }

    println!("{:-<95}", "");
    let total_saved = total_orig.saturating_sub(total_opt);
    let total_pct = (total_saved as f64 / total_orig as f64) * 100.0;
    println!(
        "TOPLAM: Ham: {} token | Dilim: {} token | Net Tasarruf: {} token (%{:.1})",
        total_orig, total_opt, total_saved, total_pct
    );
    println!("=========================================================================================\n");
}
