//! Tests for JiqError type

use super::*;

#[test]
fn test_jq_not_found_error_display() {
    let error = JiqError::JqNotFound;
    let msg = error.to_string();
    assert!(msg.contains("jq binary not found"));
    assert!(msg.contains("jqlang.org"));
}

#[test]
fn test_invalid_json_error_display() {
    let error = JiqError::InvalidJson("expected ','".to_string());
    let msg = error.to_string();
    assert!(msg.contains("Invalid JSON"));
    assert!(msg.contains("expected ','"));
}

#[test]
fn test_io_error_display() {
    let error = JiqError::Io("file not found".to_string());
    let msg = error.to_string();
    assert!(msg.contains("IO error"));
    assert!(msg.contains("file not found"));
}

#[test]
fn test_io_error_from_std_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
    let jiq_err = JiqError::from(io_err);
    assert!(matches!(jiq_err, JiqError::Io(_)));
    assert!(jiq_err.to_string().contains("test error"));
}

#[test]
fn test_error_clone() {
    let error = JiqError::InvalidJson("test".to_string());
    let cloned = error.clone();
    assert_eq!(error, cloned);
}

#[test]
fn test_error_debug() {
    let error = JiqError::JqNotFound;
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("JqNotFound"));
}

#[test]
fn test_error_equality() {
    let err1 = JiqError::Io("test".to_string());
    let err2 = JiqError::Io("test".to_string());
    let err3 = JiqError::Io("different".to_string());

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn test_all_error_variants_are_cloneable() {
    let errors: Vec<JiqError> = vec![
        JiqError::JqNotFound,
        JiqError::InvalidJson("test".to_string()),
        JiqError::Io("test".to_string()),
    ];

    for error in errors {
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }
}
