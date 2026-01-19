use std::fs;
use std::io::Write;
use std::path::PathBuf;

use tempfile::TempDir;

use super::*;

#[test]
fn test_snippets_path_returns_config_path() {
    let path = snippets_path();
    assert!(path.is_some());
    let path = path.unwrap();
    assert!(path.to_string_lossy().contains(".config/jiq"));
    assert!(path.to_string_lossy().ends_with("snippets.toml"));
}

#[test]
fn test_parse_snippets_toml_empty_string() {
    let snippets = parse_snippets_toml("");
    assert!(snippets.is_empty());
}

#[test]
fn test_parse_snippets_toml_empty_array() {
    let content = "snippets = []";
    let snippets = parse_snippets_toml(content);
    assert!(snippets.is_empty());
}

#[test]
fn test_parse_snippets_toml_valid() {
    let content = r#"
[[snippets]]
name = "Select all keys"
query = "keys"
description = "Returns array of all keys in an object"

[[snippets]]
name = "Flatten nested arrays"
query = "flatten"
"#;

    let snippets = parse_snippets_toml(content);
    assert_eq!(snippets.len(), 2);

    assert_eq!(snippets[0].name, "Select all keys");
    assert_eq!(snippets[0].query, "keys");
    assert_eq!(
        snippets[0].description,
        Some("Returns array of all keys in an object".to_string())
    );

    assert_eq!(snippets[1].name, "Flatten nested arrays");
    assert_eq!(snippets[1].query, "flatten");
    assert_eq!(snippets[1].description, None);
}

#[test]
fn test_parse_snippets_toml_invalid_syntax() {
    let content = "this is not valid toml { [ }";
    let snippets = parse_snippets_toml(content);
    assert!(snippets.is_empty());
}

#[test]
fn test_parse_snippets_toml_missing_required_fields() {
    let content = r#"
[[snippets]]
name = "Missing query field"
"#;

    let snippets = parse_snippets_toml(content);
    assert!(snippets.is_empty());
}

#[test]
fn test_parse_snippets_toml_with_extra_fields() {
    let content = r#"
[[snippets]]
name = "Snippet with extra"
query = "."
extra_field = "ignored"
another_extra = 123
"#;

    let snippets = parse_snippets_toml(content);
    assert_eq!(snippets.len(), 1);
    assert_eq!(snippets[0].name, "Snippet with extra");
    assert_eq!(snippets[0].query, ".");
}

#[test]
fn test_load_snippets_from_path_missing_file() {
    let path = PathBuf::from("/nonexistent/path/snippets.toml");
    let snippets = load_snippets_from_path(&path);
    assert!(snippets.is_empty());
}

#[test]
fn test_load_snippets_from_path_valid_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("snippets.toml");

    let content = r#"
[[snippets]]
name = "Test snippet"
query = ".test"
"#;

    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let snippets = load_snippets_from_path(&file_path);
    assert_eq!(snippets.len(), 1);
    assert_eq!(snippets[0].name, "Test snippet");
    assert_eq!(snippets[0].query, ".test");
}

#[test]
fn test_load_snippets_from_path_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("snippets.toml");

    fs::File::create(&file_path).unwrap();

    let snippets = load_snippets_from_path(&file_path);
    assert!(snippets.is_empty());
}

#[test]
fn test_snippet_struct_serialization() {
    let snippet = Snippet {
        name: "Test".to_string(),
        query: ".foo".to_string(),
        description: Some("A test snippet".to_string()),
    };

    let toml_str = toml::to_string(&snippet).unwrap();
    assert!(toml_str.contains("name = \"Test\""));
    assert!(toml_str.contains("query = \".foo\""));
    assert!(toml_str.contains("description = \"A test snippet\""));
}

#[test]
fn test_snippet_struct_serialization_without_description() {
    let snippet = Snippet {
        name: "Test".to_string(),
        query: ".foo".to_string(),
        description: None,
    };

    let toml_str = toml::to_string(&snippet).unwrap();
    assert!(toml_str.contains("name = \"Test\""));
    assert!(toml_str.contains("query = \".foo\""));
    assert!(!toml_str.contains("description"));
}

#[test]
fn test_parse_snippets_toml_single_snippet() {
    let content = r#"
[[snippets]]
name = "Identity"
query = "."
"#;

    let snippets = parse_snippets_toml(content);
    assert_eq!(snippets.len(), 1);
    assert_eq!(snippets[0].name, "Identity");
    assert_eq!(snippets[0].query, ".");
    assert_eq!(snippets[0].description, None);
}

#[test]
fn test_parse_snippets_toml_multiple_snippets() {
    let content = r#"
[[snippets]]
name = "First"
query = ".first"

[[snippets]]
name = "Second"
query = ".second"

[[snippets]]
name = "Third"
query = ".third"
"#;

    let snippets = parse_snippets_toml(content);
    assert_eq!(snippets.len(), 3);
    assert_eq!(snippets[0].name, "First");
    assert_eq!(snippets[1].name, "Second");
    assert_eq!(snippets[2].name, "Third");
}

#[test]
fn test_parse_snippets_toml_with_special_characters() {
    let content = r#"
[[snippets]]
name = "Special chars"
query = ".[] | select(.type == \"error\")"
description = "Filter \"error\" types"
"#;

    let snippets = parse_snippets_toml(content);
    assert_eq!(snippets.len(), 1);
    assert_eq!(snippets[0].query, ".[] | select(.type == \"error\")");
    assert_eq!(
        snippets[0].description,
        Some("Filter \"error\" types".to_string())
    );
}
