//! 100-Step Extreme Stress Test for TokenJar Core (Rust Native)
//!
//! Stresses performance, throughput, boundary conditions, and concurrency across 4 dimensions:
//! Dimension 1: High-Frequency File Cache, Line Slicing & Diff Engine (Steps 1-25)
//! Dimension 2: Extreme AST Skeleton Extraction & Symbol Search (Steps 26-50)
//! Dimension 3: High-Volume Output Pruner & ANSI Stream Cleaner (Steps 51-75)
//! Dimension 4: Extreme Multi-Threaded SQLite WAL Stress Test (Steps 76-100)

use std::sync::Arc;
use std::time::Instant;
use tokenjar_core::cache::persistent_cache::PersistentCache;
use tokenjar_core::cache::session_cache::SessionCache;
use tokenjar_core::config::TokenJarConfig;
use tokenjar_core::output_pruner::filter_output_logic;
use tokenjar_core::parser::SupportedLanguage;
use tokenjar_core::skeleton::build_skeleton;
use tokenjar_core::smart_reader::read_file_smart;
use tokenjar_core::telemetry::TelemetryTracker;

#[test]
fn test_extreme_100_step_stress_rust() {
    println!("\n============================================================");
    println!("🔥 TOKENJAR RUST NATIVE 100-STEP EXTREME STRESS TEST");
    println!("============================================================");

    let config = TokenJarConfig::default();
    let temp_telemetry = tempfile::NamedTempFile::new().unwrap();
    let tracker = TelemetryTracker::with_path(temp_telemetry.path().to_path_buf());

    // ------------------------------------------------------------------
    // DIMENSION 1: High-Frequency File Cache, Slicing & Diffs (Steps 1-25)
    // ------------------------------------------------------------------
    println!("\n--- [Phase 1/4] High-Frequency File Cache & Slicing (Steps 1-25) ---");
    let temp_dir = tempfile::tempdir().unwrap();
    let sample_file = temp_dir.path().join("large_stress_file.py");

    let large_code: String = (1..=600)
        .map(|i| format!("def worker_function_{i}(arg_{i}: int) -> int:\n    \"\"\"Docstring {i}\"\"\"\n    return arg_{i} * 2\n\n"))
        .collect();
    std::fs::write(&sample_file, &large_code).unwrap();
    let path_str = sample_file.to_str().unwrap();

    let cache = SessionCache::new();

    // Step 1: Cold read
    let t0 = Instant::now();
    let first = read_file_smart(
        path_str, false, None, None, None, None, &cache, &config, &tracker,
    );
    assert!(
        !first.contains("[CACHED]"),
        "Step 1: Cold read must return full content"
    );
    println!(
        "  [Step 1] Cold Read Latency: {:.3} ms",
        t0.elapsed().as_secs_f64() * 1000.0
    );

    // Steps 2-10: Repeated cache hit storm
    let t0 = Instant::now();
    for step in 2..=10 {
        let cached = read_file_smart(
            path_str, false, None, None, None, None, &cache, &config, &tracker,
        );
        assert!(
            cached.contains("[CACHED]"),
            "Step {step}: Cache hit verification failed"
        );
    }
    let hit_lat = (t0.elapsed().as_secs_f64() * 1000.0) / 9.0;
    println!(
        "  [Steps 2-10] Burst Cache Hit Avg Latency: {:.4} ms / hit",
        hit_lat
    );
    let max_hit_lat = if cfg!(debug_assertions) { 10.0 } else { 1.0 };
    assert!(
        hit_lat < max_hit_lat,
        "Cache hits must be sub-millisecond in release mode"
    );

    // Steps 11-18: Targeted line slicing
    let t0 = Instant::now();
    for step in 11..=18 {
        let start = (step - 10) * 50;
        let end = start + 40;
        let slice = read_file_smart(
            path_str,
            false,
            None,
            Some(start),
            Some(end),
            None,
            &cache,
            &config,
            &tracker,
        );
        assert!(
            slice.contains(&format!("Lines {start}-{end}")),
            "Step {step}: Slice header missing"
        );
    }
    println!(
        "  [Steps 11-18] Targeted Slicing Latency: {:.3} ms avg",
        (t0.elapsed().as_secs_f64() * 1000.0) / 8.0
    );

    // Steps 19-25: Incremental file modifications and differential unified diffs
    let mut current_code = large_code.clone();
    let t0 = Instant::now();
    for step in 19..=25 {
        current_code.push_str(&format!(
            "# Modification step {step}\ndef extra_{step}(): return {step}\n"
        ));
        std::fs::write(&sample_file, &current_code).unwrap();
        let diff_result = read_file_smart(
            path_str, false, None, None, None, None, &cache, &config, &tracker,
        );
        assert!(
            diff_result.contains('+') || diff_result.contains("extra_"),
            "Step {step}: Differential update must capture modifications"
        );
    }
    println!(
        "  [Steps 19-25] Differential Unified Diff Latency: {:.3} ms avg",
        (t0.elapsed().as_secs_f64() * 1000.0) / 7.0
    );

    // ------------------------------------------------------------------
    // DIMENSION 2: Extreme AST Skeleton Extraction (Steps 26-50)
    // ------------------------------------------------------------------
    println!("\n--- [Phase 2/4] Extreme AST Skeleton Extraction (Steps 26-50) ---");
    let python_source = r#"
class TransactionProcessor:
    """Handles enterprise high-throughput financial transactions with ACID semantics."""
    def __init__(self, merchant_id: str, secret_key: str):
        self.merchant_id = merchant_id
        self.secret_key = secret_key
        self.retry_count = 3

    def process_payment(self, customer_id: str, amount_cents: int, currency: str = "USD") -> dict:
        """Executes payment processing against Stripe API gateway with automatic retry."""
        payload = {"customer": customer_id, "amount": amount_cents, "currency": currency}
        res = self._send_request(payload)
        return {"status": "success", "id": res["id"]}

    def refund_transaction(self, tx_id: str, amount_cents: int) -> bool:
        """Refunds a settled charge either partially or fully."""
        return True
"#;

    let rust_source = r#"
pub struct NetworkManager {
    endpoint: String,
    timeout_ms: u64,
}

impl NetworkManager {
    pub fn new(endpoint: &str) -> Self {
        Self { endpoint: endpoint.to_string(), timeout_ms: 5000 }
    }

    pub fn send_packet(&self, data: &[u8]) -> Result<usize, String> {
        // Long complicated transmission algorithm
        if data.is_empty() {
            return Err("Empty payload".into());
        }
        Ok(data.len())
    }
}
"#;

    let t0 = Instant::now();
    for step in 26..=37 {
        let skel = build_skeleton(python_source, SupportedLanguage::Python);
        assert!(
            skel.contains("class TransactionProcessor:"),
            "Step {step}: Python skeleton missing class"
        );
        assert!(
            skel.contains("def process_payment"),
            "Step {step}: Python signature missing"
        );
        assert!(
            skel.contains("..."),
            "Step {step}: Function body must be replaced with ellipsis"
        );
    }
    println!(
        "  [Steps 26-37] Python AST Skeleton Avg Latency: {:.4} ms / parse",
        (t0.elapsed().as_secs_f64() * 1000.0) / 12.0
    );

    let t0 = Instant::now();
    for step in 38..=50 {
        let skel = build_skeleton(rust_source, SupportedLanguage::Rust);
        assert!(
            skel.contains("pub struct NetworkManager"),
            "Step {step}: Rust struct missing"
        );
        assert!(
            skel.contains("pub fn send_packet"),
            "Step {step}: Rust method signature missing"
        );
    }
    println!(
        "  [Steps 38-50] Rust AST Skeleton Avg Latency: {:.4} ms / parse",
        (t0.elapsed().as_secs_f64() * 1000.0) / 13.0
    );

    // ------------------------------------------------------------------
    // DIMENSION 3: High-Volume Terminal Stream Pruning (Steps 51-75)
    // ------------------------------------------------------------------
    println!("\n--- [Phase 3/4] High-Volume Terminal Stream Pruner (Steps 51-75) ---");

    // Synthesize a massive 1,500-line verbose test output
    let mut raw_test_log = String::new();
    for i in 1..=1500 {
        raw_test_log.push_str(&format!(
            "test suite::test_module::test_case_{i} ... \x1b[32mok\x1b[0m\n"
        ));
    }
    raw_test_log.push_str("test result: ok. 1500 passed; 0 failed; 0 ignored; 0 measured\n");

    let t0 = Instant::now();
    for step in 51..=62 {
        let filtered = filter_output_logic(&raw_test_log, "cargo", 0, &tracker);
        assert!(
            filtered.contains("test result: ok"),
            "Step {step}: Summary line missing"
        );
        assert!(
            filtered.len() < raw_test_log.len() / 5,
            "Step {step}: Compression ratio must exceed 80%"
        );
    }
    let prune_lat = (t0.elapsed().as_secs_f64() * 1000.0) / 12.0;
    println!("  [Steps 51-62] 1,500-Line Cargo Test Log Pruner: {:.3} ms avg (compressed 1,500 lines -> compact summary)", prune_lat);

    // Runaway output ceiling guard (2.5 MB runaway log)
    let runaway_line = "DEBUG [2026-09-26 12:00:00.123] Worker connection heartbeat tick ok\n";
    let repeat = (2_500_000 / runaway_line.len()) + 1;
    let massive_runaway: String = runaway_line.repeat(repeat);

    let t0 = Instant::now();
    for step in 63..=75 {
        let capped = filter_output_logic(&massive_runaway, "generic", 0, &tracker);
        assert!(
            capped.contains("Stream Guard")
                || capped.contains("Truncated")
                || capped.len() < 2_100_000,
            "Step {step}: Stream ceiling guard must protect context"
        );
    }
    println!(
        "  [Steps 63-75] 2.5 MB Runaway Stream Guard Capping: {:.3} ms avg",
        (t0.elapsed().as_secs_f64() * 1000.0) / 13.0
    );

    // ------------------------------------------------------------------
    // DIMENSION 4: Multi-Threaded SQLite WAL Contention Stress (Steps 76-100)
    // ------------------------------------------------------------------
    println!("\n--- [Phase 4/4] SQLite WAL Multi-Threaded Contention (Steps 76-100) ---");
    let db_path = temp_dir.path().join("stress_cache.db");
    let pc = Arc::new(PersistentCache::with_path(db_path).unwrap());

    let mut handles = Vec::new();
    let thread_count = 12;
    let ops_per_thread = 25;

    let t0 = Instant::now();
    for t_id in 0..thread_count {
        let pc_clone = Arc::clone(&pc);
        handles.push(std::thread::spawn(move || {
            let mut errs = 0;
            for i in 0..ops_per_thread {
                let file = format!("src/module_{t_id}_{i}.rs");
                let sym = tokenjar_core::models::IndexedSymbol {
                    name: format!("fn_stress_{t_id}_{i}"),
                    kind: "function".to_string(),
                    file_path: file.clone(),
                    line: 10,
                    signature: format!("fn fn_stress_{t_id}_{i}() -> bool"),
                    content_hash: format!("hash_{t_id}_{i}"),
                };
                if pc_clone
                    .set_file_symbols("proj_root", &file, "hash123", 1000.0, &[sym])
                    .is_err()
                {
                    errs += 1;
                }
                if pc_clone
                    .search_symbols("proj_root", &format!("fn_stress_{t_id}"), false, 10)
                    .is_err()
                {
                    errs += 1;
                }
            }
            errs
        }));
    }

    let mut total_errors = 0;
    for (i, h) in handles.into_iter().enumerate() {
        let errs = h.join().unwrap();
        total_errors += errs;
        let step = 76 + 2 * i;
        println!("  [Step {step}] Worker Thread #{i} completed 100 ops (Errors: {errs})");
    }

    let total_ops = thread_count * ops_per_thread * 2;
    let total_time_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let throughput = (total_ops as f64) / t0.elapsed().as_secs_f64();
    println!("  [Steps 76-100] SQLite WAL Contention: {total_ops} ops in {:.2} ms ({:.0} ops/sec, Errors: {total_errors})", total_time_ms, throughput);
    assert_eq!(
        total_errors, 0,
        "SQLite WAL mode must guarantee 0 concurrency lock errors"
    );

    println!("\n============================================================");
    println!("🏁 ALL 100 STEPS IN RUST EXTREME STRESS TEST PASSED (0 ERRORS)");
    println!("============================================================\n");
}
