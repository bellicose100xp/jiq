// Configuration type definitions

use serde::Deserialize;

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

/// Root configuration structure
#[derive(Debug, Clone, Deserialize)]
#[derive(Default)]
pub struct Config {
    #[serde(default)]
    pub clipboard: ClipboardConfig,
}


#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: config-system, Property 1: Valid backend parsing
    // For any valid clipboard backend value ("auto", "system", or "osc52") in a TOML config file,
    // parsing the config should successfully extract and store that backend preference without errors.
    // Validates: Requirements 1.2
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_valid_backend_parsing(backend in prop::sample::select(vec!["auto", "system", "osc52"])) {
            let toml_content = format!(r#"
[clipboard]
backend = "{}"
"#, backend);

            let config: Result<Config, _> = toml::from_str(&toml_content);
            
            // Should parse successfully
            prop_assert!(config.is_ok(), "Failed to parse valid backend: {}", backend);
            
            let config = config.unwrap();
            
            // Should match the expected backend
            let expected = match backend {
                "auto" => ClipboardBackend::Auto,
                "system" => ClipboardBackend::System,
                "osc52" => ClipboardBackend::Osc52,
                _ => unreachable!(),
            };
            
            prop_assert_eq!(config.clipboard.backend, expected);
        }
    }

    // Feature: config-system, Property 2: Missing fields use defaults
    // For any TOML config file with missing optional fields, parsing the config should
    // successfully complete and use default values for all missing fields.
    // Validates: Requirements 2.3
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_missing_fields_use_defaults(
            include_clipboard_section in prop::bool::ANY,
            include_backend_field in prop::bool::ANY
        ) {
            let toml_content = if !include_clipboard_section {
                // Empty config - no clipboard section at all
                String::new()
            } else if !include_backend_field {
                // Clipboard section exists but backend field is missing
                "[clipboard]\n".to_string()
            } else {
                // Both section and field exist (control case)
                r#"
[clipboard]
backend = "system"
"#.to_string()
            };

            let config: Result<Config, _> = toml::from_str(&toml_content);
            
            // Should always parse successfully
            prop_assert!(config.is_ok(), "Failed to parse config with missing fields");
            
            let config = config.unwrap();
            
            // When fields are missing, should use defaults
            if !include_clipboard_section || !include_backend_field {
                prop_assert_eq!(
                    config.clipboard.backend,
                    ClipboardBackend::Auto,
                    "Missing fields should default to Auto"
                );
            }
        }
    }
}
