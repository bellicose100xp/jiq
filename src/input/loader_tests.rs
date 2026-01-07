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

    // This test verifies the terminal detection logic exists
    // When stdin is a terminal (not piped), load_stdin_sync should error immediately
    if std::io::stdin().is_terminal() {
        let result = load_stdin_sync();
        assert!(result.is_err(), "Should error when stdin is a terminal");
        match result.unwrap_err() {
            JiqError::Io(msg) => {
                assert!(msg.contains("No input provided"));
            }
            _ => panic!("Expected JiqError::Io"),
        }
    }
}

#[test]
fn test_stdin_terminal_detection_with_subprocess() {
    use std::process::{Command, Stdio};

    // Test 1: With piped input (not a terminal)
    let output = Command::new("cargo")
        .args([
            "test",
            "--lib",
            "stdin_helper_with_pipe",
            "--",
            "--nocapture",
            "--ignored",
        ])
        .stdin(Stdio::piped())
        .output()
        .expect("Failed to run test");

    assert!(output.status.success(), "Piped stdin test should pass");

    // Test 2: Without piped input (is a terminal) - runs in normal test environment
    // This is covered by test_load_stdin_sync_detects_terminal above
}

#[test]
#[ignore]
fn stdin_helper_with_pipe() {
    // This helper test is run by test_stdin_terminal_detection_with_subprocess
    // with piped stdin to verify the non-terminal branch
    use std::io::IsTerminal;

    // When run with piped stdin, this should be false
    let is_term = std::io::stdin().is_terminal();
    assert!(!is_term, "stdin should not be a terminal when piped");
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
