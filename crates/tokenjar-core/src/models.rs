use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CategoryStats {
    pub original: u64,
    pub optimized: u64,
    pub saved: u64,
    pub count: u64,
}

impl CategoryStats {
    pub fn savings_pct(&self) -> f64 {
        if self.original == 0 {
            0.0
        } else {
            (self.saved as f64 / self.original as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub total_original_tokens: u64,
    pub total_optimized_tokens: u64,
    pub total_tokens_saved: u64,
    pub total_commands_filtered: u64,
    pub total_files_cached: u64,
    pub total_skeletons_generated: u64,
    pub total_repo_maps_generated: u64,
    pub first_used_at: String,
    pub last_used_at: String,

    // Granular categories
    pub skeleton: CategoryStats,
    #[serde(default)]
    pub slice: CategoryStats,
    pub cache: CategoryStats,
    pub repo_map: CategoryStats,
    pub command: CategoryStats,
    pub symbol_search: CategoryStats,
    pub lockfile: CategoryStats,
}

impl Default for TelemetryData {
    fn default() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            total_original_tokens: 0,
            total_optimized_tokens: 0,
            total_tokens_saved: 0,
            total_commands_filtered: 0,
            total_files_cached: 0,
            total_skeletons_generated: 0,
            total_repo_maps_generated: 0,
            first_used_at: now.clone(),
            last_used_at: now,
            skeleton: CategoryStats::default(),
            slice: CategoryStats::default(),
            cache: CategoryStats::default(),
            repo_map: CategoryStats::default(),
            command: CategoryStats::default(),
            symbol_search: CategoryStats::default(),
            lockfile: CategoryStats::default(),
        }
    }
}

impl TelemetryData {
    pub fn savings_pct(&self) -> f64 {
        let eff_orig = self.skeleton.original
            + self.slice.original
            + self.cache.original
            + self.repo_map.original
            + self.command.original
            + self.symbol_search.original
            + self.lockfile.original;
        let eff_saved = self.skeleton.saved
            + self.slice.saved
            + self.cache.saved
            + self.repo_map.saved
            + self.command.saved
            + self.symbol_search.saved
            + self.lockfile.saved;

        if eff_orig > 0 {
            (eff_saved as f64 / eff_orig as f64) * 100.0
        } else if self.total_original_tokens > 0 {
            (self.total_tokens_saved as f64 / self.total_original_tokens as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn estimated_dollars_saved(&self) -> f64 {
        // Blended rate: $3.00 per 1M tokens
        (self.total_tokens_saved as f64 / 1_000_000.0) * 3.00
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexedSymbol {
    pub name: String,
    pub kind: String, // class, function, method, struct, enum, trait
    pub file_path: String,
    pub line: usize,
    pub signature: String,
    #[serde(default)]
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferenceKind {
    Call,
    Import,
    Inheritance,
    Usage,
}

impl std::fmt::Display for ReferenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Call => write!(f, "CALL"),
            Self::Import => write!(f, "IMPORT"),
            Self::Inheritance => write!(f, "INHERITANCE"),
            Self::Usage => write!(f, "USAGE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReference {
    pub symbol_name: String,
    pub file_path: String,
    pub line: usize,
    pub kind: ReferenceKind,
    pub snippet: String,
}
