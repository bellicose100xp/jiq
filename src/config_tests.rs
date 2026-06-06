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
fn test_autocomplete_array_sample_size_default() {
    let config = Config::default();
    assert_eq!(config.autocomplete.array_sample_size, 10);
}

#[test]
fn test_autocomplete_array_sample_size_parsed() {
    let toml = r#"
[autocomplete]
array_sample_size = 25
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.autocomplete.array_sample_size, 25);
}

#[test]
fn test_autocomplete_array_sample_size_clamp_zero() {
    let toml = r#"
[autocomplete]
array_sample_size = 0
"#;
    let mut config: Config = toml::from_str(toml).unwrap();
    config.autocomplete.array_sample_size = config.autocomplete.array_sample_size.clamp(1, 1000);
    assert_eq!(config.autocomplete.array_sample_size, 1);
}

#[test]
fn test_autocomplete_array_sample_size_clamp_above_max() {
    let toml = r#"
[autocomplete]
array_sample_size = 5000
"#;
    let mut config: Config = toml::from_str(toml).unwrap();
    config.autocomplete.array_sample_size = config.autocomplete.array_sample_size.clamp(1, 1000);
    assert_eq!(config.autocomplete.array_sample_size, 1000);
}

#[test]
fn test_autocomplete_array_sample_size_within_range() {
    let toml = r#"
[autocomplete]
array_sample_size = 500
"#;
    let mut config: Config = toml::from_str(toml).unwrap();
    config.autocomplete.array_sample_size = config.autocomplete.array_sample_size.clamp(1, 1000);
    assert_eq!(config.autocomplete.array_sample_size, 500);
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

// --- load_config() end-to-end tests ---
//
// load_config() resolves its path from dirs::home_dir(), which reads the
// process-global HOME env var. Cargo runs tests in parallel threads and other
// tests also read dirs::home_dir(), so these tests must be serialized against
// each other and against any sibling that touches HOME. We guard every HOME
// mutation with a single static Mutex and always restore the previous value.

use std::path::Path;
use std::sync::Mutex;

static HOME_LOCK: Mutex<()> = Mutex::new(());

/// Sets HOME to `home`, runs `body` (which calls load_config), then restores the
/// previous HOME. Serialized via HOME_LOCK so concurrent tests don't race on the
/// shared env var. Returns whatever `body` produces.
fn with_home<R>(home: &Path, body: impl FnOnce() -> R) -> R {
    // SAFETY: HOME_LOCK serializes every HOME mutation in this binary so no
    // other test reads HOME concurrently while we swap it.
    let _guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let saved = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", home);
    }
    let result = body();
    unsafe {
        match saved {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }
    result
}

/// Builds <home>/.config/jiq/'s dir and returns the config.toml file path.
fn jiq_config_path(home: &Path) -> std::path::PathBuf {
    let dir = home.join(".config").join("jiq");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("config.toml")
}

#[test]
fn test_load_config_missing_file_returns_silent_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    // No .config/jiq/config.toml created under this HOME.
    let result = with_home(tmp.path(), load_config);

    assert!(
        result.warning.is_none(),
        "absent config file must be silent (no warning), got {:?}",
        result.warning
    );
    assert_eq!(
        result.config.clipboard.backend,
        ClipboardBackend::Auto,
        "missing config must fall back to defaults"
    );
}

#[test]
fn test_load_config_invalid_toml_returns_defaults_with_warning() {
    let tmp = tempfile::tempdir().unwrap();
    let config_file = jiq_config_path(tmp.path());
    std::fs::write(&config_file, "[clipboard\nbackend = \"auto\"").unwrap();

    let result = with_home(tmp.path(), load_config);

    assert_eq!(
        result.config.clipboard.backend,
        ClipboardBackend::Auto,
        "malformed TOML must fall back to defaults"
    );
    let warning = result
        .warning
        .expect("malformed TOML must produce a warning");
    assert!(
        warning.starts_with("Invalid config:"),
        "parse-error warning must use the 'Invalid config:' prefix, got: {}",
        warning
    );
}

#[test]
fn test_load_config_unreadable_file_returns_read_warning() {
    let tmp = tempfile::tempdir().unwrap();
    // Make config.toml a DIRECTORY: it exists() == true but read_to_string
    // fails with EISDIR, driving the read-error arm (root-immune unlike chmod).
    let config_file = jiq_config_path(tmp.path());
    std::fs::create_dir_all(&config_file).unwrap();
    assert!(
        config_file.exists(),
        "directory-as-config-file should exist"
    );

    let result = with_home(tmp.path(), load_config);

    assert_eq!(
        result.config.clipboard.backend,
        ClipboardBackend::Auto,
        "unreadable config must fall back to defaults"
    );
    let warning = result
        .warning
        .expect("unreadable config must produce a warning");
    assert!(
        warning.starts_with("Failed to read config:"),
        "read-error warning must use the 'Failed to read config:' prefix, got: {}",
        warning
    );
}
