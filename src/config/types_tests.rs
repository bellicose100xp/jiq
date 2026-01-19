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
