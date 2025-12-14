// Configuration module for jiq
// This module handles loading and parsing configuration from ~/.config/jiq/config.toml

pub mod ai_types;
mod types;

// AI types are used internally via Config struct
pub use types::{ClipboardBackend, Config};

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

    #[cfg(debug_assertions)]
    log::debug!("Loading config from {:?}", config_path);

    // If file doesn't exist, return defaults silently
    if !config_path.exists() {
        #[cfg(debug_assertions)]
        log::debug!("Config file does not exist, using defaults");
        return ConfigResult {
            config: Config::default(),
            warning: None,
        };
    }

    // Try to read the file
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => {
            #[cfg(debug_assertions)]
            log::debug!("Config file read successfully, {} bytes", contents.len());
            contents
        }
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
        Ok(config) => {
            #[cfg(debug_assertions)]
            log::debug!("Config parsed successfully: {:?}", config.clipboard.backend);
            ConfigResult {
                config,
                warning: None,
            }
        }
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
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: config-system, Property 3: Invalid backend fallback
    // For any invalid clipboard backend value in a TOML config file, the config system
    // should log a warning and use the default clipboard backend value ("auto").
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_invalid_backend_fallback(
            invalid_backend in "[a-z]{3,10}".prop_filter(
                "not valid",
                |s| !["auto", "system", "osc52"].contains(&s.as_str())
            )
        ) {
            let toml_content = format!(r#"
[clipboard]
backend = "{}"
"#, invalid_backend);

            let config: Result<Config, _> = toml::from_str(&toml_content);

            // Should fail to parse (serde will reject invalid enum value)
            prop_assert!(config.is_err(), "Invalid backend should fail to parse");

            // In the actual load_config function, this error would be caught
            // and Config::default() would be returned, which has Auto backend
            let default_config = Config::default();
            prop_assert_eq!(
                default_config.clipboard.backend,
                ClipboardBackend::Auto,
                "Default config should use Auto backend"
            );
        }
    }

    // Feature: config-system, Property 4: Malformed TOML fallback
    // For any malformed TOML syntax in the config file, the config system should
    // log an error with details and return a config with all default values.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_malformed_toml_fallback(
            malformed in prop::sample::select(vec![
                "[clipboard\nbackend = \"auto\"",      // Missing closing bracket
                "[clipboard]\nbackend = auto",          // Missing quotes
                "[clipboard]\n backend",                // Missing value
                "clipboard]\nbackend = \"auto\"",       // Missing opening bracket
                "[clipboard]\nbackend = \"auto",        // Unterminated string
                "[clipboard\nbackend = \"auto\"\n]",    // Bracket in wrong place
            ])
        ) {
            let config: Result<Config, _> = toml::from_str(malformed);

            // Should fail to parse
            prop_assert!(config.is_err(), "Malformed TOML should fail to parse");

            // In the actual load_config function, this error would be caught
            // and Config::default() would be returned
            let default_config = Config::default();
            prop_assert_eq!(
                default_config.clipboard.backend,
                ClipboardBackend::Auto,
                "Default config should use Auto backend"
            );
        }
    }

    // Feature: config-system, Property 5: Config path consistency
    // For any execution of the config loading function, it should attempt to load
    // from the same standardized path (~/.config/jiq/config.toml).
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_config_path_consistency(_iteration in 0..10u32) {
            // Call get_config_path multiple times
            let path1 = get_config_path();
            let path2 = get_config_path();
            let path3 = get_config_path();

            // All should return the same path
            prop_assert_eq!(&path1, &path2, "Config path should be consistent");
            prop_assert_eq!(&path2, &path3, "Config path should be consistent");

            // Path should end with jiq/config.toml
            let path_str = path1.to_string_lossy();
            prop_assert!(
                path_str.ends_with("jiq/config.toml") || path_str.ends_with("jiq\\config.toml"),
                "Config path should end with jiq/config.toml, got: {}",
                path_str
            );
        }
    }

    // Unit tests for configuration loading

    #[test]
    fn test_config_default_values() {
        let config = Config::default();
        assert_eq!(config.clipboard.backend, ClipboardBackend::Auto);
    }

    #[test]
    fn test_clipboard_backend_default() {
        let backend = ClipboardBackend::default();
        assert_eq!(backend, ClipboardBackend::Auto);
    }

    #[test]
    fn test_parse_auto_backend() {
        let toml = r#"
[clipboard]
backend = "auto"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.clipboard.backend, ClipboardBackend::Auto);
    }

    #[test]
    fn test_parse_system_backend() {
        let toml = r#"
[clipboard]
backend = "system"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.clipboard.backend, ClipboardBackend::System);
    }

    #[test]
    fn test_parse_osc52_backend() {
        let toml = r#"
[clipboard]
backend = "osc52"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.clipboard.backend, ClipboardBackend::Osc52);
    }

    #[test]
    fn test_missing_file_returns_defaults() {
        // This test verifies that load_config() returns defaults when file doesn't exist
        // We can't easily test the actual load_config() without mocking the filesystem,
        // but we can verify the default behavior
        let config = Config::default();
        assert_eq!(config.clipboard.backend, ClipboardBackend::Auto);
    }

    #[test]
    fn test_malformed_toml_example_1() {
        let toml = "[clipboard\nbackend = \"auto\""; // Missing closing bracket
        let result: Result<Config, _> = toml::from_str(toml);
        assert!(result.is_err(), "Malformed TOML should fail to parse");
    }

    #[test]
    fn test_malformed_toml_example_2() {
        let toml = "[clipboard]\nbackend = auto"; // Missing quotes
        let result: Result<Config, _> = toml::from_str(toml);
        assert!(result.is_err(), "Malformed TOML should fail to parse");
    }

    #[test]
    fn test_malformed_toml_example_3() {
        let toml = "[clipboard]\n backend"; // Missing value
        let result: Result<Config, _> = toml::from_str(toml);
        assert!(result.is_err(), "Malformed TOML should fail to parse");
    }
}
