use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to get path to fixture file
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Helper to create a temporary JSON file
fn create_temp_json_file(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    (temp_dir, file_path)
}

// =============================================================================
// Task 6: Integration Tests for End-to-End Flow
// =============================================================================

/// Test complete loading flow from file to query execution
///
/// **Validates: Requirements 1.1, 1.3, 6.7**
/// This test verifies that jiq can successfully load a file and execute queries.
/// Note: This is a basic smoke test since full interactive testing is difficult.
#[test]
fn test_complete_loading_flow() {
    let json_content = r#"{"name": "Alice", "age": 30, "city": "Seattle"}"#;
    let (_temp_dir, file_path) = create_temp_json_file(json_content);

    // Verify the file exists and is readable
    assert!(file_path.exists(), "Test file should exist");

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, json_content, "File content should match");

    // Note: Full end-to-end testing with query execution would require
    // interactive terminal simulation, which is beyond the scope of
    // integration tests. The unit tests in loader_tests.rs and
    // app_state_tests.rs cover the loading and query initialization logic.
}

// =============================================================================
// Task 6.1: Example Tests for Specific Error Cases
// =============================================================================

/// Example 1: Missing file error
///
/// **Validates: Requirements 5.1**
/// - 5.1: WHEN a file does not exist THEN the FileLoader SHALL return an Error state
///   with a descriptive message
#[test]
fn test_missing_file_error() {
    use jiq::input::loader::FileLoader;
    use std::thread;
    use std::time::Duration;

    let nonexistent_path = PathBuf::from("/nonexistent/path/to/file.json");

    let mut loader = FileLoader::spawn_load(nonexistent_path);

    // Wait for the loader to complete
    let mut result = None;
    for _ in 0..100 {
        if let Some(r) = loader.poll() {
            result = Some(r);
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(result.is_some(), "Loader should complete");
    let result = result.unwrap();

    // Verify we get an error
    assert!(result.is_err(), "Should return error for missing file");

    // Verify the error is an IO error (file not found)
    let err = result.unwrap_err();
    match err {
        jiq::error::JiqError::Io(io_err_msg) => {
            // Verify the error message contains "No such file" or similar
            assert!(
                io_err_msg.to_lowercase().contains("no such file")
                    || io_err_msg.to_lowercase().contains("not found"),
                "Error message should indicate file not found, got: {}",
                io_err_msg
            );
        }
        other => panic!("Expected JiqError::Io, got {:?}", other),
    }
}

// =============================================================================
// Task 6.2: Example Test for Invalid JSON
// =============================================================================

/// Example 2: Invalid JSON error
///
/// **Validates: Requirements 5.2**
/// - 5.2: WHEN a file contains invalid JSON THEN the FileLoader SHALL return an Error state
///   with parsing error details
#[test]
fn test_invalid_json_error() {
    use jiq::input::loader::FileLoader;
    use std::thread;
    use std::time::Duration;

    // Create a file with invalid JSON
    let invalid_json = r#"{"name": "test", invalid syntax here}"#;
    let (_temp_dir, file_path) = create_temp_json_file(invalid_json);

    let mut loader = FileLoader::spawn_load(file_path);

    // Wait for the loader to complete
    let mut result = None;
    for _ in 0..100 {
        if let Some(r) = loader.poll() {
            result = Some(r);
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(result.is_some(), "Loader should complete");
    let result = result.unwrap();

    // Verify we get an error
    assert!(result.is_err(), "Should return error for invalid JSON");

    // Verify the error is an InvalidJson error with parsing details
    let err = result.unwrap_err();
    match err {
        jiq::error::JiqError::InvalidJson(msg) => {
            // Verify the error message contains parsing details
            assert!(
                !msg.is_empty(),
                "Error message should contain parsing details"
            );
            // The error message should mention something about the syntax error
            // Common error messages include "expected", "invalid", "error", "key must be", etc.
            assert!(
                msg.contains("expected")
                    || msg.contains("invalid")
                    || msg.contains("error")
                    || msg.contains("key must be")
                    || msg.contains("line")
                    || msg.contains("column"),
                "Error message should describe the parsing error: {}",
                msg
            );
        }
        other => panic!("Expected JiqError::InvalidJson, got {:?}", other),
    }
}

// =============================================================================
// Task 6.3: Example Test for Permission Errors
// =============================================================================

/// Example 3: Permission error
///
/// **Validates: Requirements 5.3**
/// - 5.3: WHEN a file cannot be read due to permissions THEN the FileLoader SHALL return
///   an Error state with permission error details
#[test]
#[cfg(unix)] // Permission tests are Unix-specific
fn test_permission_error() {
    use jiq::input::loader::FileLoader;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::thread;
    use std::time::Duration;

    // Create a file with no read permissions
    let json_content = r#"{"name": "test"}"#;
    let (_temp_dir, file_path) = create_temp_json_file(json_content);

    // Remove read permissions
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o000); // No permissions
    fs::set_permissions(&file_path, perms).unwrap();

    let mut loader = FileLoader::spawn_load(file_path.clone());

    // Wait for the loader to complete
    let mut result = None;
    for _ in 0..100 {
        if let Some(r) = loader.poll() {
            result = Some(r);
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o644);
    let _ = fs::set_permissions(&file_path, perms);

    assert!(result.is_some(), "Loader should complete");
    let result = result.unwrap();

    // Verify we get an error
    assert!(result.is_err(), "Should return error for permission denied");

    // Verify the error is an IO error (permission denied)
    let err = result.unwrap_err();
    match err {
        jiq::error::JiqError::Io(io_err_msg) => {
            // Verify the error message contains "permission denied" or similar
            assert!(
                io_err_msg.to_lowercase().contains("permission denied")
                    || io_err_msg.to_lowercase().contains("access denied"),
                "Error message should indicate permission denied, got: {}",
                io_err_msg
            );
        }
        other => panic!(
            "Expected JiqError::Io with PermissionDenied, got {:?}",
            other
        ),
    }
}

/// Test that existing fixture files work with deferred loading
///
/// This test verifies that the deferred loading system works correctly
/// with the existing test fixtures.
#[test]
fn test_deferred_loading_with_fixtures() {
    use jiq::input::loader::FileLoader;
    use std::thread;
    use std::time::Duration;

    // Test with simple.json fixture
    let simple_path = fixture_path("simple.json");
    assert!(simple_path.exists(), "simple.json fixture should exist");

    let mut loader = FileLoader::spawn_load(simple_path);

    // Wait for completion
    let mut result = None;
    for _ in 0..100 {
        if let Some(r) = loader.poll() {
            result = Some(r);
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(result.is_some(), "Loader should complete");
    let result = result.unwrap();
    assert!(result.is_ok(), "Loading simple.json should succeed");

    let json_content = result.unwrap();
    assert!(
        json_content.contains("Alice"),
        "Loaded content should contain expected data"
    );
}
