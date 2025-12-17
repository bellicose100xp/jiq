//! Tests for input reader

use super::*;
use std::path::PathBuf;

#[test]
fn test_read_valid_json_from_file() {
    let path = PathBuf::from("tests/fixtures/simple.json");
    let result = InputReader::read_json(Some(&path));

    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.contains("Alice"));
    assert!(json.contains("Seattle"));
}

#[test]
fn test_read_array_json_from_file() {
    let path = PathBuf::from("tests/fixtures/array.json");
    let result = InputReader::read_json(Some(&path));

    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.contains("Alice"));
    assert!(json.contains("Bob"));
    assert!(json.contains("Charlie"));
}

#[test]
fn test_read_nested_json_from_file() {
    let path = PathBuf::from("tests/fixtures/nested.json");
    let result = InputReader::read_json(Some(&path));

    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.contains("TechCorp"));
    assert!(json.contains("engineering"));
}

#[test]
fn test_invalid_json_returns_error() {
    let path = PathBuf::from("tests/fixtures/invalid.json");
    let result = InputReader::read_json(Some(&path));

    assert!(result.is_err());
    match result {
        Err(JiqError::InvalidJson(_)) => {
            // Expected error type
        }
        _ => panic!("Expected InvalidJson error"),
    }
}

#[test]
fn test_file_not_found_returns_error() {
    let path = PathBuf::from("tests/fixtures/nonexistent.json");
    let result = InputReader::read_json(Some(&path));

    assert!(result.is_err());
    match result {
        Err(JiqError::Io(_)) => {
            // Expected IO error
        }
        _ => panic!("Expected IO error"),
    }
}

#[test]
fn test_valid_json_string() {
    let json = r#"{"name": "Test", "value": 42}"#;
    let result = InputReader::read_json_from_string(json);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output, json);
}

#[test]
fn test_invalid_json_string() {
    let json = r#"{"name": "Test", invalid}"#;
    let result = InputReader::read_json_from_string(json);

    assert!(result.is_err());
    match result {
        Err(JiqError::InvalidJson(_)) => {
            // Expected error type
        }
        _ => panic!("Expected InvalidJson error"),
    }
}

#[test]
fn test_empty_json_object() {
    let json = "{}";
    let result = InputReader::read_json_from_string(json);

    assert!(result.is_ok());
}

#[test]
fn test_empty_json_array() {
    let json = "[]";
    let result = InputReader::read_json_from_string(json);

    assert!(result.is_ok());
}
