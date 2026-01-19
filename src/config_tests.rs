//! Tests for config

use super::*;

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
fn test_invalid_backend_fails_parse() {
    let toml = r#"
[clipboard]
backend = "invalid"
"#;
    let result: Result<Config, _> = toml::from_str(toml);
    assert!(result.is_err(), "Invalid backend should fail to parse");
}

#[test]
fn test_missing_file_returns_defaults() {
    let config = Config::default();
    assert_eq!(config.clipboard.backend, ClipboardBackend::Auto);
}

#[test]
fn test_malformed_toml_missing_bracket() {
    let toml = "[clipboard\nbackend = \"auto\"";
    let result: Result<Config, _> = toml::from_str(toml);
    assert!(result.is_err(), "Malformed TOML should fail to parse");
}

#[test]
fn test_malformed_toml_missing_quotes() {
    let toml = "[clipboard]\nbackend = auto";
    let result: Result<Config, _> = toml::from_str(toml);
    assert!(result.is_err(), "Malformed TOML should fail to parse");
}

#[test]
fn test_malformed_toml_missing_value() {
    let toml = "[clipboard]\n backend";
    let result: Result<Config, _> = toml::from_str(toml);
    assert!(result.is_err(), "Malformed TOML should fail to parse");
}

#[test]
fn test_config_path_consistency() {
    let path1 = get_config_path();
    let path2 = get_config_path();

    assert_eq!(path1, path2, "Config path should be consistent");

    let path_str = path1.to_string_lossy();
    assert!(
        path_str.ends_with("jiq/config.toml") || path_str.ends_with("jiq\\config.toml"),
        "Config path should end with jiq/config.toml, got: {}",
        path_str
    );
}
