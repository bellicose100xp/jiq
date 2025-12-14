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

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub clipboard: ClipboardConfig,
    #[serde(default)]
    pub tooltip: TooltipConfig,
    #[serde(default)]
    pub ai: AiConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: config-system, Property 1: Valid backend parsing
    // For any valid clipboard backend value ("auto", "system", or "osc52") in a TOML config file,
    // parsing the config should successfully extract and store that backend preference without errors.
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

    // Feature: tooltip-config, Property 1: Valid auto_show parsing
    // For any valid boolean value in tooltip.auto_show, parsing should succeed.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_valid_tooltip_auto_show_parsing(auto_show: bool) {
            let toml_content = format!(r#"
[tooltip]
auto_show = {}
"#, auto_show);

            let config: Result<Config, _> = toml::from_str(&toml_content);

            prop_assert!(config.is_ok(), "Failed to parse valid auto_show: {}", auto_show);

            let config = config.unwrap();
            prop_assert_eq!(config.tooltip.auto_show, auto_show);
        }
    }

    // Feature: tooltip-config, Property 2: Missing tooltip section defaults to auto_show = true
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_missing_tooltip_section_defaults(
            include_tooltip_section in prop::bool::ANY,
            include_auto_show_field in prop::bool::ANY
        ) {
            let toml_content = if !include_tooltip_section {
                // No tooltip section
                String::new()
            } else if !include_auto_show_field {
                // Tooltip section exists but auto_show field is missing
                "[tooltip]\n".to_string()
            } else {
                // Both section and field exist with false value
                r#"
[tooltip]
auto_show = false
"#.to_string()
            };

            let config: Result<Config, _> = toml::from_str(&toml_content);

            prop_assert!(config.is_ok(), "Failed to parse config with missing tooltip fields");

            let config = config.unwrap();

            // When fields are missing, should default to true
            if !include_tooltip_section || !include_auto_show_field {
                prop_assert!(
                    config.tooltip.auto_show,
                    "Missing tooltip.auto_show should default to true"
                );
            }
        }
    }

    // Unit tests for tooltip config
    #[test]
    fn test_tooltip_config_default() {
        let config = TooltipConfig::default();
        assert!(config.auto_show);
    }

    #[test]
    fn test_parse_tooltip_auto_show_true() {
        let toml = r#"
[tooltip]
auto_show = true
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.tooltip.auto_show);
    }

    #[test]
    fn test_parse_tooltip_auto_show_false() {
        let toml = r#"
[tooltip]
auto_show = false
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.tooltip.auto_show);
    }

    #[test]
    fn test_missing_tooltip_section_uses_default() {
        let toml = r#"
[clipboard]
backend = "auto"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.tooltip.auto_show);
    }

    #[test]
    fn test_empty_tooltip_section_uses_default() {
        let toml = r#"
[tooltip]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.tooltip.auto_show);
    }
}
