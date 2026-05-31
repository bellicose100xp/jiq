//! Tests for types

use super::*;

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
fn test_empty_config_uses_defaults() {
    let config: Config = toml::from_str("").unwrap();
    assert_eq!(config.clipboard.backend, ClipboardBackend::Auto);
    assert!(config.tooltip.auto_show);
}

#[test]
fn test_missing_backend_field_uses_default() {
    let toml = r#"
[clipboard]
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.clipboard.backend, ClipboardBackend::Auto);
}

#[test]
fn test_theme_mode_default() {
    let mode = ThemeMode::default();
    assert_eq!(mode, ThemeMode::Auto);
}

#[test]
fn test_theme_config_default() {
    let config = ThemeConfig::default();
    assert_eq!(config.mode, ThemeMode::Auto);
}

#[test]
fn test_parse_auto_theme_mode() {
    let toml = r#"
[theme]
mode = "auto"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.theme.mode, ThemeMode::Auto);
}

#[test]
fn test_parse_light_theme_mode() {
    let toml = r#"
[theme]
mode = "light"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.theme.mode, ThemeMode::Light);
}

#[test]
fn test_parse_dark_theme_mode() {
    let toml = r#"
[theme]
mode = "dark"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.theme.mode, ThemeMode::Dark);
}

#[test]
fn test_invalid_theme_mode_fails_parse() {
    let toml = r#"
[theme]
mode = "solarized"
"#;
    let result: Result<Config, _> = toml::from_str(toml);
    assert!(result.is_err(), "Invalid theme mode should fail to parse");
}

#[test]
fn test_missing_theme_section_uses_default() {
    let toml = r#"
[clipboard]
backend = "auto"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.theme.mode, ThemeMode::Auto);
}

#[test]
fn test_empty_theme_section_uses_default() {
    let toml = r#"
[theme]
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.theme.mode, ThemeMode::Auto);
}

#[test]
fn test_autocomplete_config_default() {
    let config = AutocompleteConfig::default();
    assert_eq!(config.array_sample_size, 10);
}

#[test]
fn test_parse_autocomplete_array_sample_size() {
    let toml = r#"
[autocomplete]
array_sample_size = 50
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.autocomplete.array_sample_size, 50);
}

#[test]
fn test_missing_autocomplete_section_uses_default() {
    let toml = r#"
[clipboard]
backend = "auto"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.autocomplete.array_sample_size, 10);
}

#[test]
fn test_empty_autocomplete_section_uses_default() {
    let toml = r#"
[autocomplete]
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.autocomplete.array_sample_size, 10);
}

#[test]
fn test_empty_config_includes_autocomplete_default() {
    let config: Config = toml::from_str("").unwrap();
    assert_eq!(config.autocomplete.array_sample_size, 10);
}
