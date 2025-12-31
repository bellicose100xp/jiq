//! Tests for prompt template generation

use super::*;
use crate::ai::context::ContextParams;

#[test]
fn test_build_error_prompt_includes_query() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        output_sample: None,
        error: Some("syntax error".to_string()),
        is_success: false,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_error_prompt(&ctx);
    assert!(prompt.contains(".name"));
    assert!(prompt.contains("syntax error"));
    assert!(prompt.contains("Cursor position: 5"));
}

#[test]
fn test_build_error_prompt_includes_schema() {
    let ctx = QueryContext {
        query: ".".to_string(),
        cursor_pos: 1,
        output_sample: None,
        error: Some("error".to_string()),
        is_success: false,
        is_empty_result: false,
        input_schema: Some(r#"{"name":"string","age":"number"}"#.to_string()),
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_error_prompt(&ctx);
    assert!(prompt.contains("## Input JSON Schema"));
    assert!(prompt.contains(r#"{"name":"string","age":"number"}"#));
}

#[test]
fn test_build_success_prompt_includes_query() {
    let ctx = QueryContext {
        query: ".items[]".to_string(),
        cursor_pos: 8,
        output_sample: Some("1\n2\n3".to_string()),
        error: None,
        is_success: true,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_success_prompt(&ctx);
    assert!(prompt.contains(".items[]"));
    assert!(prompt.contains("optimize"));
}

#[test]
fn test_build_success_prompt_includes_output_sample() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        output_sample: Some(r#""test""#.to_string()),
        error: None,
        is_success: true,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_success_prompt(&ctx);
    assert!(prompt.contains(r#""test""#));
    assert!(prompt.contains("Query Output Sample"));
}

#[test]
fn test_build_success_prompt_includes_schema() {
    let ctx = QueryContext {
        query: ".[]".to_string(),
        cursor_pos: 3,
        output_sample: Some("1\n2\n3".to_string()),
        error: None,
        is_success: true,
        is_empty_result: false,
        input_schema: Some(r#"["number"]"#.to_string()),
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_success_prompt(&ctx);
    assert!(prompt.contains("## Input JSON Schema"));
    assert!(prompt.contains(r#"["number"]"#));
}

#[test]
fn test_build_prompt_dispatches_to_error_prompt() {
    let ctx = QueryContext {
        query: ".invalid".to_string(),
        cursor_pos: 8,
        output_sample: None,
        error: Some("syntax error".to_string()),
        is_success: false,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_prompt(&ctx);
    assert!(prompt.contains("troubleshoot"));
    assert!(prompt.contains("syntax error"));
}

#[test]
fn test_build_prompt_dispatches_to_success_prompt() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        output_sample: Some(r#""test""#.to_string()),
        error: None,
        is_success: true,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_prompt(&ctx);
    assert!(prompt.contains("optimize"));
    assert!(!prompt.contains("troubleshoot"));
}

#[test]
fn test_build_error_prompt_includes_structured_format() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        output_sample: None,
        error: Some("error".to_string()),
        is_success: false,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_error_prompt(&ctx);
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
        output_sample: Some("test".to_string()),
        error: None,
        is_success: true,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_success_prompt(&ctx);
    assert!(prompt.contains("[Optimize]"));
    assert!(prompt.contains("[Next]"));
    assert!(prompt.contains("numbered suggestions"));
}

#[test]
fn test_build_prompt_includes_natural_language_instructions() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        output_sample: None,
        error: Some("error".to_string()),
        is_success: false,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_error_prompt(&ctx);
    assert!(prompt.contains("Natural Language"));
    assert!(prompt.contains("natural language"));
}

#[test]
fn test_error_prompt_includes_base_query() {
    let ctx = QueryContext {
        query: ".invalid".to_string(),
        cursor_pos: 8,
        output_sample: None,
        error: Some("field not found".to_string()),
        is_success: false,
        is_empty_result: false,
        input_schema: None,
        base_query: Some(".name".to_string()),
        base_query_result: Some(r#""test""#.to_string()),
    };

    let prompt = build_error_prompt(&ctx);
    assert!(prompt.contains("## Last Working Query"));
    assert!(prompt.contains(".name"));
    assert!(prompt.contains("## Its Output"));
    assert!(prompt.contains(r#""test""#));
}

#[test]
fn test_error_prompt_without_base_query() {
    let ctx = QueryContext {
        query: ".invalid".to_string(),
        cursor_pos: 8,
        output_sample: None,
        error: Some("error".to_string()),
        is_success: false,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    };

    let prompt = build_error_prompt(&ctx);
    assert!(!prompt.contains("Last Working Query"));
}

#[test]
fn test_success_prompt_excludes_base_query() {
    let ctx = QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        output_sample: Some("output".to_string()),
        error: None,
        is_success: true,
        is_empty_result: false,
        input_schema: None,
        base_query: Some(".old".to_string()),
        base_query_result: Some("old result".to_string()),
    };

    let prompt = build_success_prompt(&ctx);
    assert!(!prompt.contains("Last Non-Empty Query"));
    assert!(!prompt.contains(".old"));
}

#[test]
fn test_base_query_result_truncation_in_context() {
    use crate::ai::context::MAX_JSON_SAMPLE_LENGTH;

    let long_result = "x".repeat(30_000);
    let preprocessed =
        crate::ai::context::prepare_json_for_context(&long_result, MAX_JSON_SAMPLE_LENGTH);

    let ctx = QueryContext::new(
        ".invalid".to_string(),
        8,
        None,
        Some("error".to_string()),
        ContextParams {
            input_schema: None,
            base_query: Some(".base"),
            base_query_result: Some(&preprocessed),
            is_empty_result: false,
        },
        MAX_JSON_SAMPLE_LENGTH,
    );

    assert!(ctx.base_query_result.is_some());
    let truncated = ctx.base_query_result.unwrap();
    assert!(truncated.len() <= MAX_JSON_SAMPLE_LENGTH + 15);
    assert!(truncated.ends_with("... [truncated]"));
}
