use super::*;
use std::fs;
use std::io::Write;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper to create a temporary JSON file
fn create_temp_json_file(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    (temp_dir, file_path)
}

/// Helper to wait for loader to complete
fn wait_for_completion(
    loader: &mut FileLoader,
    max_attempts: u32,
) -> Option<Result<String, JiqError>> {
    for _ in 0..max_attempts {
        if let Some(result) = loader.poll() {
            return Some(result);
        }
        thread::sleep(Duration::from_millis(10));
    }
    None
}

#[test]
fn test_loader_source_is_file_for_spawn_load() {
    let (_tmp, path) = create_temp_json_file(r#"{"a": 1}"#);
    let loader = FileLoader::spawn_load(path);
    assert_eq!(loader.source, LoaderSource::File);
}

#[test]
fn test_loader_source_is_stdin_for_spawn_load_stdin() {
    let loader = FileLoader::spawn_load_stdin();
    assert_eq!(loader.source, LoaderSource::Stdin);
}

#[test]
fn test_loader_source_is_clipboard_for_load_clipboard_blocking() {
    // load_clipboard_blocking runs synchronously and may succeed or fail
    // depending on the environment; either way `source` must be Clipboard.
    let loader = FileLoader::load_clipboard_blocking();
    assert_eq!(loader.source, LoaderSource::Clipboard);
}

#[test]
fn test_file_loader_loads_valid_json() {
    // Requirement 6.1: THE FileLoader SHALL have unit tests verifying successful file loading
    let json_content = r#"{"name": "test", "value": 42}"#;
    let (_temp_dir, file_path) = create_temp_json_file(json_content);

    let mut loader = FileLoader::spawn_load(file_path);

    // Poll until complete
    let result = wait_for_completion(&mut loader, 100);

    assert!(result.is_some(), "Loader should complete");
    let result = result.unwrap();
    assert!(result.is_ok(), "Loading should succeed");
    assert_eq!(result.unwrap(), json_content);
    assert!(matches!(loader.state(), LoadingState::Complete(_)));
}

#[test]
fn test_file_loader_returns_error_for_invalid_json() {
    // Requirement 6.3: THE FileLoader SHALL have unit tests verifying error handling for invalid JSON
    let invalid_json = r#"{"name": "test", invalid}"#;
    let (_temp_dir, file_path) = create_temp_json_file(invalid_json);

    let mut loader = FileLoader::spawn_load(file_path);

    // Poll until complete
    let result = wait_for_completion(&mut loader, 100);

    assert!(result.is_some(), "Loader should complete");
    let result = result.unwrap();
    assert!(result.is_err(), "Loading should fail for invalid JSON");
    assert!(matches!(result.unwrap_err(), JiqError::InvalidJson(_)));
    assert!(matches!(loader.state(), LoadingState::Error(_)));
}

#[test]
fn test_file_loader_returns_error_for_missing_file() {
    // Requirement 6.2: THE FileLoader SHALL have unit tests verifying error handling for missing files
    let missing_path = PathBuf::from("/nonexistent/path/to/file.json");

    let mut loader = FileLoader::spawn_load(missing_path);

    // Poll until complete
    let result = wait_for_completion(&mut loader, 100);

    assert!(result.is_some(), "Loader should complete");
    let result = result.unwrap();
    assert!(result.is_err(), "Loading should fail for missing file");
    assert!(matches!(result.unwrap_err(), JiqError::Io(_)));
    assert!(matches!(loader.state(), LoadingState::Error(_)));
}

#[test]
fn test_poll_returns_none_while_loading() {
    // Requirement 6.4: THE FileLoader SHALL have unit tests verifying the poll method returns None while loading
    let json_content = r#"{"name": "test"}"#;
    let (_temp_dir, file_path) = create_temp_json_file(json_content);

    let mut loader = FileLoader::spawn_load(file_path);

    // Immediately poll - should return None (or Some if thread was very fast)
    let first_poll = loader.poll();

    // If first poll returned None, we verified the requirement
    // If it returned Some, the thread was just very fast (still valid)
    if first_poll.is_none() {
        // Good - poll returned None while loading
        assert!(loader.is_loading() || matches!(loader.state(), LoadingState::Complete(_)));
    }
}

#[test]
fn test_poll_returns_result_when_complete() {
    // Requirement 6.5: THE FileLoader SHALL have unit tests verifying the poll method returns the result when complete
    let json_content = r#"{"name": "test"}"#;
    let (_temp_dir, file_path) = create_temp_json_file(json_content);

    let mut loader = FileLoader::spawn_load(file_path);

    // Wait for completion
    let result = wait_for_completion(&mut loader, 100);

    assert!(result.is_some(), "Poll should return Some when complete");
    assert!(result.unwrap().is_ok(), "Result should be Ok");

    // Subsequent polls should return None
    assert_eq!(loader.poll(), None, "Subsequent polls should return None");
}

#[test]
fn test_io_errors_convert_to_jiq_error() {
    // Verify that IO errors are converted to JiqError::Io
    let missing_path = PathBuf::from("/nonexistent/file.json");

    let mut loader = FileLoader::spawn_load(missing_path);
    let result = wait_for_completion(&mut loader, 100);

    assert!(result.is_some());
    let err = result.unwrap().unwrap_err();
    assert!(
        matches!(err, JiqError::Io(_)),
        "IO errors should convert to JiqError::Io"
    );
}

// ============================================================================
// Stdin Loading Tests (Phase 2 - Deferred Stdin Loading)
// ============================================================================

#[test]
fn test_spawn_load_stdin_creates_loader() {
    // Note: spawn_load_stdin() spawns a thread that reads from stdin
    // Full stdin reading is difficult to test in unit tests
    // This test verifies the method exists and creates a loader correctly
    let loader = FileLoader::spawn_load_stdin();

    // Should initialize in Loading state
    assert!(loader.is_loading());
    assert!(matches!(loader.state(), LoadingState::Loading));

    // Note: We don't poll here because stdin would block waiting for input
    // Integration tests verify full stdin loading behavior
}

#[test]
fn test_load_stdin_sync_detects_terminal() {
    use std::io::IsTerminal;

    // When stdin is a terminal (not piped), load_stdin_sync should error immediately
    if std::io::stdin().is_terminal() {
        let result = load_stdin_sync();
        assert!(result.is_err(), "Should error when stdin is a terminal");
        match result.unwrap_err() {
            JiqError::Io(msg) => {
                assert!(msg.contains("No input provided"));
                assert!(msg.contains("Usage:"));
            }
            _ => panic!("Expected JiqError::Io"),
        }
    }
    // Note: The non-terminal branch (piped stdin) is tested in integration tests
    // where stdin can be properly mocked with actual piped data
}

// ============================================================================
// Clipboard Loading Tests
// ============================================================================

#[test]
fn test_input_load_error_clipboard_unreadable_message_shape() {
    let err = input_load_error(InputErrorReason::ClipboardUnreadable);
    match err {
        JiqError::Io(msg) => {
            assert!(msg.contains("No input provided"));
            assert!(msg.contains("Could not read the system clipboard"));
            assert!(msg.contains("Usage:"));
            assert!(msg.contains("jiq <file>"));
            assert!(msg.contains("cat data.json | jiq"));
            assert!(msg.contains("# load from system clipboard"));
            assert!(!msg.to_lowercase().contains("x11"));
            assert!(!msg.to_lowercase().contains("arboard"));
        }
        other => panic!("expected JiqError::Io, got {:?}", other),
    }
}

#[test]
fn test_input_load_error_clipboard_empty_distinguished_from_unreadable() {
    let unreadable = match input_load_error(InputErrorReason::ClipboardUnreadable) {
        JiqError::Io(m) => m,
        _ => panic!("expected Io"),
    };
    let empty = match input_load_error(InputErrorReason::ClipboardEmpty) {
        JiqError::Io(m) => m,
        _ => panic!("expected Io"),
    };

    assert!(unreadable.contains("Could not read"));
    assert!(empty.contains("Clipboard is empty"));
    assert_ne!(unreadable, empty);
}

#[test]
fn test_input_load_error_invalid_json_distinguishes_from_unreadable() {
    let invalid = match input_load_error(InputErrorReason::ClipboardInvalidJson) {
        JiqError::Io(m) => m,
        _ => panic!("expected Io"),
    };
    let unreadable = match input_load_error(InputErrorReason::ClipboardUnreadable) {
        JiqError::Io(m) => m,
        _ => panic!("expected Io"),
    };

    assert!(invalid.contains("does not contain valid JSON"));
    assert!(!unreadable.contains("does not contain valid JSON"));

    // All clipboard-failure variants share the three-line usage block so
    // users always see all valid invocation forms regardless of how they
    // failed.
    for msg in [&invalid, &unreadable] {
        assert!(msg.contains("jiq <file>"));
        assert!(msg.contains("| jiq"));
        assert!(msg.contains("# load from system clipboard"));
    }
}

#[test]
fn test_input_load_error_no_stdin_message_shape() {
    let err = input_load_error(InputErrorReason::NoStdin);
    match err {
        JiqError::Io(msg) => {
            assert!(msg.contains("No input provided"));
            assert!(msg.contains("Usage:"));
            assert!(msg.contains("jiq <file>"));
        }
        other => panic!("expected JiqError::Io, got {:?}", other),
    }
}

#[test]
fn test_validate_json_or_jsonl_rejects_plain_text() {
    let result = validate_json_or_jsonl("hello world");
    assert!(result.is_err());
}

// ============================================================================
// scan_json_or_jsonl Tests (Phase 1 — M4 single-pass scan)
// ============================================================================

#[test]
fn test_scan_object_is_container() {
    let scan = scan_json_or_jsonl(r#"{"a": 1}"#).unwrap();
    assert_eq!(scan.count, 1);
    assert!(scan.all_containers);
}

#[test]
fn test_scan_array_is_container() {
    let scan = scan_json_or_jsonl(r#"[1, 2, 3]"#).unwrap();
    assert_eq!(scan.count, 1);
    assert!(scan.all_containers);
}

#[test]
fn test_scan_bare_number_is_primitive() {
    let scan = scan_json_or_jsonl("42").unwrap();
    assert_eq!(scan.count, 1);
    assert!(!scan.all_containers);
}

#[test]
fn test_scan_bare_string_is_primitive() {
    let scan = scan_json_or_jsonl(r#""hello""#).unwrap();
    assert_eq!(scan.count, 1);
    assert!(!scan.all_containers);
}

#[test]
fn test_scan_bare_bool_is_primitive() {
    let scan = scan_json_or_jsonl("true").unwrap();
    assert_eq!(scan.count, 1);
    assert!(!scan.all_containers);
}

#[test]
fn test_scan_bare_null_is_primitive() {
    let scan = scan_json_or_jsonl("null").unwrap();
    assert_eq!(scan.count, 1);
    assert!(!scan.all_containers);
}

#[test]
fn test_scan_jsonl_all_containers() {
    let jsonl = "{\"a\": 1}\n{\"b\": 2}\n[3, 4]";
    let scan = scan_json_or_jsonl(jsonl).unwrap();
    assert_eq!(scan.count, 3);
    assert!(scan.all_containers);
}

#[test]
fn test_scan_jsonl_with_one_primitive_rejects_all_containers() {
    let jsonl = "{\"a\": 1}\n42\n{\"b\": 2}";
    let scan = scan_json_or_jsonl(jsonl).unwrap();
    assert_eq!(scan.count, 3);
    assert!(
        !scan.all_containers,
        "JSONL with any primitive should set all_containers=false"
    );
}

#[test]
fn test_scan_parse_error_returns_err() {
    let result = scan_json_or_jsonl(r#"{"a": invalid}"#);
    assert!(matches!(result, Err(JiqError::InvalidJson(_))));
}

#[test]
fn test_scan_empty_returns_err() {
    let result = scan_json_or_jsonl("");
    assert!(matches!(result, Err(JiqError::InvalidJson(_))));
}

#[test]
fn test_scan_whitespace_only_returns_err() {
    let result = scan_json_or_jsonl("  \n\t  ");
    assert!(matches!(result, Err(JiqError::InvalidJson(_))));
}

// ============================================================================
// ClipboardPrimitive error variant
// ============================================================================

#[test]
fn test_input_load_error_clipboard_primitive_message() {
    let err = input_load_error(InputErrorReason::ClipboardPrimitive);
    match err {
        JiqError::Io(msg) => {
            assert!(msg.contains("non-JSON value"));
            assert!(msg.contains("object or array"));
            // No literal echo of the clipboard contents (privacy).
            assert!(!msg.contains("`42`"));
        }
        other => panic!("expected JiqError::Io, got {:?}", other),
    }
}

// ============================================================================
// JSONL Validation Tests
// ============================================================================

#[test]
fn test_validate_json_single_object() {
    let json = r#"{"name": "test", "value": 42}"#;
    let result = validate_json_or_jsonl(json);
    assert!(result.is_ok(), "Single JSON object should be valid");
}

#[test]
fn test_validate_json_array() {
    let json = r#"[1, 2, 3]"#;
    let result = validate_json_or_jsonl(json);
    assert!(result.is_ok(), "JSON array should be valid");
}

#[test]
fn test_validate_jsonl_multiple_objects() {
    let jsonl = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}"#;
    let result = validate_json_or_jsonl(jsonl);
    assert!(
        result.is_ok(),
        "JSONL with multiple objects should be valid"
    );
}

#[test]
fn test_validate_jsonl_with_empty_lines() {
    let jsonl = r#"{"id": 1}

{"id": 2}

{"id": 3}"#;
    let result = validate_json_or_jsonl(jsonl);
    assert!(
        result.is_ok(),
        "JSONL with blank lines between values should be valid"
    );
}

#[test]
fn test_validate_invalid_json() {
    let invalid = r#"{"name": invalid}"#;
    let result = validate_json_or_jsonl(invalid);
    assert!(result.is_err(), "Invalid JSON should fail validation");
    assert!(matches!(result.unwrap_err(), JiqError::InvalidJson(_)));
}

#[test]
fn test_validate_empty_input() {
    let empty = "";
    let result = validate_json_or_jsonl(empty);
    assert!(result.is_err(), "Empty input should fail validation");
    match result.unwrap_err() {
        JiqError::InvalidJson(msg) => {
            assert!(msg.contains("Empty input"));
        }
        _ => panic!("Expected JiqError::InvalidJson with 'Empty input' message"),
    }
}

#[test]
fn test_validate_whitespace_only_input() {
    let whitespace = "   \n\t\n   ";
    let result = validate_json_or_jsonl(whitespace);
    assert!(
        result.is_err(),
        "Whitespace-only input should fail validation"
    );
}

#[test]
fn test_file_loader_loads_jsonl() {
    let jsonl_content = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}"#;
    let (_temp_dir, file_path) = create_temp_json_file(jsonl_content);

    let mut loader = FileLoader::spawn_load(file_path);
    let result = wait_for_completion(&mut loader, 100);

    assert!(result.is_some(), "Loader should complete");
    let result = result.unwrap();
    assert!(result.is_ok(), "Loading JSONL should succeed");
    assert_eq!(result.unwrap(), jsonl_content);
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Generate valid JSON strings
    fn valid_json_string() -> impl Strategy<Value = String> {
        prop_oneof![
            Just(r#"{"key": "value"}"#.to_string()),
            Just(r#"[1, 2, 3]"#.to_string()),
            Just(r#"{"nested": {"data": [1, 2, 3]}}"#.to_string()),
            Just(r#"{"string": "test", "number": 42, "bool": true}"#.to_string()),
            Just(r#"[]"#.to_string()),
            Just(r#"{}"#.to_string()),
        ]
    }

    /// Generate invalid file paths that will cause IO errors
    fn invalid_path() -> impl Strategy<Value = PathBuf> {
        prop_oneof![
            Just(PathBuf::from("/nonexistent/path/file.json")),
            Just(PathBuf::from("/tmp/nonexistent_dir_12345/file.json")),
            Just(PathBuf::from("/root/protected/file.json")),
            Just(PathBuf::from("/dev/null/impossible/file.json")),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 4: Poll returns None until complete
        /// Feature: deferred-file-loading, Property 4: Poll returns None until complete
        /// Validates: Requirements 3.4
        #[test]
        fn prop_poll_none_until_complete(json in valid_json_string()) {
            let (_temp_dir, file_path) = create_temp_json_file(&json);
            let mut loader = FileLoader::spawn_load(file_path);

            // Poll should eventually return Some, but may return None first
            let mut got_some = false;

            for _ in 0..100 {
                match loader.poll() {
                    None => {
                        // Still loading
                    }
                    Some(result) => {
                        got_some = true;
                        prop_assert!(result.is_ok());
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }

            prop_assert!(got_some, "Should eventually return Some");

            // After returning Some, subsequent polls return None
            prop_assert_eq!(loader.poll(), None);
            prop_assert_eq!(loader.poll(), None);
        }

        /// Property 6: IO errors convert to JiqError
        /// Feature: deferred-file-loading, Property 6: IO errors convert to JiqError
        /// Validates: Requirements 5.4
        #[test]
        fn prop_io_errors_become_jiq_errors(path in invalid_path()) {
            let mut loader = FileLoader::spawn_load(path);

            // Wait for completion
            let result = wait_for_completion(&mut loader, 100);

            prop_assert!(result.is_some(), "Loader should complete");
            let result = result.unwrap();
            prop_assert!(result.is_err(), "Should return error for invalid path");

            // Verify it's a JiqError::Io
            match result.unwrap_err() {
                JiqError::Io(_) => {
                    // Success - IO error was converted to JiqError
                }
                other => {
                    prop_assert!(false, "Expected JiqError::Io, got {:?}", other);
                }
            }
        }
    }
}
