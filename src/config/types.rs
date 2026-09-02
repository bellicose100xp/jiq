// Configuration type definitions

use serde::Deserialize;

use super::ai_types::AiConfig;

/// Clipboard backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardBackend {
    #[default]
    Auto,
    System,
    Osc52,
}

/// Clipboard configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct ClipboardConfig {
    #[serde(default)]
    pub backend: ClipboardBackend,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        ClipboardConfig {
            backend: ClipboardBackend::Auto,
        }
    }
}

/// Theme mode. `auto` detects the terminal background at startup; `light`/`dark` force a palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Auto,
    Light,
    Dark,
}

/// Theme configuration section
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThemeConfig {
    #[serde(default)]
    pub mode: ThemeMode,
}

/// Tooltip configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct TooltipConfig {
    #[serde(default = "default_auto_show")]
    pub auto_show: bool,
}

fn default_auto_show() -> bool {
    true
}

impl Default for TooltipConfig {
    fn default() -> Self {
        TooltipConfig { auto_show: true }
    }
}

/// Autocomplete configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct AutocompleteConfig {
    #[serde(default = "default_array_sample_size")]
    pub array_sample_size: usize,
}

fn default_array_sample_size() -> usize {
    10
}

impl Default for AutocompleteConfig {
    fn default() -> Self {
        AutocompleteConfig {
            array_sample_size: 10,
        }
    }
}

/// Query execution configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct QueryConfig {
    /// Delay between the last keystroke and the jq re-run, in milliseconds
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
}

fn default_debounce_ms() -> u64 {
    150
}

impl Default for QueryConfig {
    fn default() -> Self {
        QueryConfig { debounce_ms: 150 }
    }
}

/// History configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryConfig {
    /// Maximum number of entries kept in the persisted history file
    #[serde(default = "default_max_history_entries")]
    pub max_entries: usize,
}

fn default_max_history_entries() -> usize {
    1000
}

impl Default for HistoryConfig {
    fn default() -> Self {
        HistoryConfig { max_entries: 1000 }
    }
}

/// Save dialog configuration section
#[derive(Debug, Clone, Deserialize)]
pub struct SaveConfig {
    /// Initial filename pattern in the save dialog.
    /// Supports `{timestamp}`, `{cwd}`, and `~` expansion.
    #[serde(default = "default_save_pattern")]
    pub default_pattern: String,
}

fn default_save_pattern() -> String {
    crate::save::DEFAULT_PATH_PATTERN.to_string()
}

impl Default for SaveConfig {
    fn default() -> Self {
        SaveConfig {
            default_pattern: default_save_pattern(),
        }
    }
}

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub clipboard: ClipboardConfig,
    #[serde(default)]
    pub tooltip: TooltipConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub autocomplete: AutocompleteConfig,
    #[serde(default)]
    pub query: QueryConfig,
    #[serde(default)]
    pub history: HistoryConfig,
    #[serde(default)]
    pub save: SaveConfig,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
