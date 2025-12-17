//! Tests for prompt template generation

use super::*;
use crate::ai::context::JsonTypeInfo;

#[test]
fn test_build_error_prompt_includes_query() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        input_sample: r#"{"name": "test"}"#.to_string(),
        output: None,
        output_sample: None,
        error: Some("syntax error".to_string()),
        json_type_info: JsonTypeInfo::default(),
        is_success: false,
    };

    let prompt = build_error_prompt(&ctx, 200);
    assert!(prompt.contains(".name"));
    assert!(prompt.contains("syntax error"));
    assert!(prompt.contains("Cursor position: 5"));
}

#[test]
fn test_build_error_prompt_includes_json_sample() {
    let ctx = QueryContext {
        query: ".".to_string(),
        cursor_pos: 1,
        input_sample: r#"{"key": "value"}"#.to_string(),
        output: None,
        output_sample: None,
        error: Some("error".to_string()),
        json_type_info: JsonTypeInfo::default(),
        is_success: false,
    };

    let prompt = build_error_prompt(&ctx, 200);
    assert!(prompt.contains(r#"{"key": "value"}"#));
}

#[test]
fn test_build_error_prompt_includes_type_info() {
    let ctx = QueryContext {
        query: ".".to_string(),
        cursor_pos: 1,
        input_sample: "{}".to_string(),
        output: None,
        output_sample: None,
        error: Some("error".to_string()),
        json_type_info: JsonTypeInfo {
            root_type: "Object".to_string(),
            element_type: None,
            element_count: None,
            top_level_keys: vec!["name".to_string(), "age".to_string()],
            schema_hint: "Object with keys: name, age".to_string(),
        },
        is_success: false,
    };

    let prompt = build_error_prompt(&ctx, 200);
    assert!(prompt.contains("Type: Object"));
    assert!(prompt.contains("name, age"));
}

#[test]
fn test_build_help_prompt_basic() {
    let ctx = QueryContext {
        query: ".items[]".to_string(),
        cursor_pos: 8,
        input_sample: "[1, 2, 3]".to_string(),
        output: Some("1\n2\n3".to_string()),
        output_sample: Some("1\n2\n3".to_string()),
        error: None,
        json_type_info: JsonTypeInfo {
            root_type: "Array".to_string(),
            element_type: Some("numbers".to_string()),
            element_count: Some(3),
            top_level_keys: vec![],
            schema_hint: "Array of 3 numbers".to_string(),
        },
        is_success: true,
    };

    let prompt = build_help_prompt(&ctx);
    assert!(prompt.contains(".items[]"));
    assert!(prompt.contains("Array of 3 numbers"));
    assert!(prompt.contains("1\n2\n3"));
}

#[test]
fn test_build_help_prompt_uses_output_sample() {
    let output_sample = "sample output".to_string();
    let ctx = QueryContext {
        query: ".".to_string(),
        cursor_pos: 1,
        input_sample: "{}".to_string(),
        output: Some("full output".to_string()),
        output_sample: Some(output_sample.clone()),
        error: None,
        json_type_info: JsonTypeInfo::default(),
        is_success: true,
    };

    let prompt = build_help_prompt(&ctx);
    // Should use output_sample, not output
    assert!(prompt.contains(&output_sample));
}

#[test]
fn test_build_help_prompt_truncates_output_when_no_sample() {
    let long_output = "x".repeat(1000);
    let ctx = QueryContext {
        query: ".".to_string(),
        cursor_pos: 1,
        input_sample: "{}".to_string(),
        output: Some(long_output),
        output_sample: None, // No pre-truncated sample
        error: None,
        json_type_info: JsonTypeInfo::default(),
        is_success: true,
    };

    let prompt = build_help_prompt(&ctx);
    assert!(prompt.contains("[truncated]"));
}

#[test]
fn test_build_success_prompt_includes_query() {
    let ctx = QueryContext {
        query: ".items[]".to_string(),
        cursor_pos: 8,
        input_sample: "[1, 2, 3]".to_string(),
        output: Some("1\n2\n3".to_string()),
        output_sample: Some("1\n2\n3".to_string()),
        error: None,
        json_type_info: JsonTypeInfo::default(),
        is_success: true,
    };

    let prompt = build_success_prompt(&ctx, 200);
    assert!(prompt.contains(".items[]"));
    assert!(prompt.contains("optimize"));
}

#[test]
fn test_build_success_prompt_includes_output_sample() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        input_sample: r#"{"name": "test"}"#.to_string(),
        output: Some(r#""test""#.to_string()),
        output_sample: Some(r#""test""#.to_string()),
        error: None,
        json_type_info: JsonTypeInfo::default(),
        is_success: true,
    };

    let prompt = build_success_prompt(&ctx, 200);
    assert!(prompt.contains(r#""test""#));
    assert!(prompt.contains("Query Output Sample"));
}

#[test]
fn test_build_success_prompt_includes_type_info() {
    let ctx = QueryContext {
        query: ".[]".to_string(),
        cursor_pos: 3,
        input_sample: "[1, 2, 3]".to_string(),
        output: Some("1\n2\n3".to_string()),
        output_sample: Some("1\n2\n3".to_string()),
        error: None,
        json_type_info: JsonTypeInfo {
            root_type: "Array".to_string(),
            element_type: Some("numbers".to_string()),
            element_count: Some(3),
            top_level_keys: vec![],
            schema_hint: "Array of 3 numbers".to_string(),
        },
        is_success: true,
    };

    let prompt = build_success_prompt(&ctx, 200);
    assert!(prompt.contains("Type: Array"));
    assert!(prompt.contains("Element type: numbers"));
    assert!(prompt.contains("Element count: 3"));
}

#[test]
fn test_build_prompt_dispatches_to_error_prompt() {
    let ctx = QueryContext {
        query: ".invalid".to_string(),
        cursor_pos: 8,
        input_sample: "{}".to_string(),
        output: None,
        output_sample: None,
        error: Some("syntax error".to_string()),
        json_type_info: JsonTypeInfo::default(),
        is_success: false,
    };

    let prompt = build_prompt(&ctx, 200);
    // Error prompt contains "troubleshoot" and error message
    assert!(prompt.contains("troubleshoot"));
    assert!(prompt.contains("syntax error"));
}

#[test]
fn test_build_prompt_dispatches_to_success_prompt() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        input_sample: r#"{"name": "test"}"#.to_string(),
        output: Some(r#""test""#.to_string()),
        output_sample: Some(r#""test""#.to_string()),
        error: None,
        json_type_info: JsonTypeInfo::default(),
        is_success: true,
    };

    let prompt = build_prompt(&ctx, 200);
    // Success prompt contains "optimize"
    assert!(prompt.contains("optimize"));
    assert!(!prompt.contains("troubleshoot"));
}

#[test]
fn test_build_prompt_includes_word_limit() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        input_sample: "{}".to_string(),
        output: None,
        output_sample: None,
        error: Some("error".to_string()),
        json_type_info: JsonTypeInfo::default(),
        is_success: false,
    };

    let prompt = build_prompt(&ctx, 300);
    assert!(prompt.contains("300 words"));
}

#[test]
fn test_build_error_prompt_includes_structured_format() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        input_sample: "{}".to_string(),
        output: None,
        output_sample: None,
        error: Some("error".to_string()),
        json_type_info: JsonTypeInfo::default(),
        is_success: false,
    };

    let prompt = build_error_prompt(&ctx, 200);
    assert!(prompt.contains("[Fix]"));
    assert!(prompt.contains("[Optimize]"));
    assert!(prompt.contains("[Next]"));
    assert!(prompt.contains("numbered suggestions"));
}

#[test]
fn test_build_success_prompt_includes_structured_format() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        input_sample: "{}".to_string(),
        output: Some("test".to_string()),
        output_sample: Some("test".to_string()),
        error: None,
        json_type_info: JsonTypeInfo::default(),
        is_success: true,
    };

    let prompt = build_success_prompt(&ctx, 200);
    assert!(prompt.contains("[Optimize]"));
    assert!(prompt.contains("[Next]"));
    assert!(prompt.contains("numbered suggestions"));
}

#[test]
fn test_build_prompt_includes_natural_language_instructions() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        input_sample: "{}".to_string(),
        output: None,
        output_sample: None,
        error: Some("error".to_string()),
        json_type_info: JsonTypeInfo::default(),
        is_success: false,
    };

    let prompt = build_error_prompt(&ctx, 200);
    assert!(prompt.contains("Natural Language"));
    assert!(prompt.contains("natural language"));
}
