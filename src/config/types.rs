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

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub clipboard: ClipboardConfig,
    #[serde(default)]
    pub tooltip: TooltipConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub autocomplete: AutocompleteConfig,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
