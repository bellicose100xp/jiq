use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Helper to get path to fixture file
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_cli_with_invalid_json_file() {
    let fixture = fixture_path("invalid.json");

    cargo_bin_cmd!()
        .arg(&fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON"));
}

#[test]
fn test_cli_with_nonexistent_file() {
    cargo_bin_cmd!().arg("nonexistent.json").assert().failure();
}

#[test]
fn test_cli_help_flag() {
    cargo_bin_cmd!()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Interactive JSON query tool"));
}

#[test]
fn test_cli_version_flag() {
    cargo_bin_cmd!()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("jiq"));
}

#[test]
fn test_fixture_files_exist() {
    // Verify all our test fixtures are present
    assert!(fixture_path("simple.json").exists());
    assert!(fixture_path("array.json").exists());
    assert!(fixture_path("nested.json").exists());
    assert!(fixture_path("invalid.json").exists());
}

#[test]
fn test_fixture_simple_json_content() {
    let content = fs::read_to_string(fixture_path("simple.json")).unwrap();
    assert!(content.contains("Alice"));
    assert!(content.contains("Seattle"));
}

#[test]
fn test_fixture_array_json_content() {
    let content = fs::read_to_string(fixture_path("array.json")).unwrap();
    assert!(content.contains("Alice"));
    assert!(content.contains("Bob"));
    assert!(content.contains("Charlie"));
}

#[test]
fn test_fixture_nested_json_content() {
    let content = fs::read_to_string(fixture_path("nested.json")).unwrap();
    assert!(content.contains("TechCorp"));
    assert!(content.contains("engineering"));
    assert!(content.contains("departments"));
}
