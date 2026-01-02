// Configuration module for jiq
// This module handles loading and parsing configuration from ~/.config/jiq/config.toml

pub mod ai_types;
mod types;

// AI types are used internally via Config struct
pub use types::{ClipboardBackend, Config};

// Re-export for integration tests
#[allow(unused_imports)]
pub use ai_types::{AiConfig, AiProviderType, AnthropicConfig};
#[allow(unused_imports)]
pub use types::TooltipConfig;

use std::fs;
use std::path::PathBuf;

/// Result of loading configuration
pub struct ConfigResult {
    pub config: Config,
    pub warning: Option<String>,
}

/// Loads configuration from ~/.config/jiq/config.toml
/// Returns default configuration if file doesn't exist or on parse errors
pub fn load_config() -> ConfigResult {
    let config_path = get_config_path();

    // If file doesn't exist, return defaults silently
    if !config_path.exists() {
        return ConfigResult {
            config: Config::default(),
            warning: None,
        };
    }

    // Try to read the file
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(e) => {
            #[cfg(debug_assertions)]
            log::error!("Failed to read config file {:?}: {}", config_path, e);
            return ConfigResult {
                config: Config::default(),
                warning: Some(format!("Failed to read config: {}", e)),
            };
        }
    };

    // Try to parse TOML
    match toml::from_str::<Config>(&contents) {
        Ok(config) => ConfigResult {
            config,
            warning: None,
        },
        Err(e) => {
            #[cfg(debug_assertions)]
            log::error!("Failed to parse config file {:?}: {}", config_path, e);
            ConfigResult {
                config: Config::default(),
                warning: Some(format!("Invalid config: {}", e)),
            }
        }
    }
}

/// Returns the path to the configuration file
///
/// Always uses ~/.config/jiq/config.toml on all platforms for consistency.
fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("jiq")
        .join("config.toml")
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
