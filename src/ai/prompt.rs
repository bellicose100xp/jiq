//! Prompt template generation
//!
//! Builds prose prompts for AI requests based on query context.
//! Generates different prompts for error troubleshooting vs success optimization.

use super::context::QueryContext;

/// Build a prompt based on query context
///
/// Dispatches to either error troubleshooting or success optimization prompt
/// based on the `is_success` field in the context.
pub fn build_prompt(context: &QueryContext, word_limit: u16) -> String {
    if context.is_success {
        build_success_prompt(context, word_limit)
    } else {
        build_error_prompt(context, word_limit)
    }
}

/// Build a prompt for error troubleshooting
///
/// Creates a prose prompt that includes the query, error message,
/// JSON sample, and structure information to help the AI provide
/// relevant assistance.
pub fn build_error_prompt(context: &QueryContext, word_limit: u16) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are a jq query assistant helping troubleshoot errors.\n");
    prompt.push_str("CRITICAL: Your response will be displayed in a popup window.\n");
    prompt.push_str(&format!(
        "Keep your ENTIRE response under {} words.\n\n",
        word_limit
    ));

    prompt.push_str("## Current Query\n");
    prompt.push_str(&format!("```\n{}\n```\n", context.query));
    prompt.push_str(&format!("Cursor position: {}\n\n", context.cursor_pos));

    if let Some(ref error) = context.error {
        prompt.push_str("## Error\n");
        prompt.push_str(&format!("```\n{}\n```\n\n", error));
    }

    prompt.push_str("## Input JSON Structure\n");
    prompt.push_str(&format!("Type: {}\n", context.json_type_info.root_type));
    if let Some(ref elem_type) = context.json_type_info.element_type {
        prompt.push_str(&format!("Element type: {}\n", elem_type));
    }
    if let Some(count) = context.json_type_info.element_count {
        prompt.push_str(&format!("Element count: {}\n", count));
    }
    if !context.json_type_info.top_level_keys.is_empty() {
        prompt.push_str(&format!(
            "Top-level keys: {}\n",
            context.json_type_info.top_level_keys.join(", ")
        ));
    }
    prompt.push_str(&format!(
        "Summary: {}\n\n",
        context.json_type_info.schema_hint
    ));

    prompt.push_str("## Input JSON Sample\n");
    prompt.push_str(&format!("```json\n{}\n```\n\n", context.input_sample));

    prompt.push_str("## Response Format\n");
    prompt.push_str("Provide 3-5 numbered suggestions in this EXACT format:\n\n");
    prompt.push_str("1. [Fix] corrected_jq_query_here\n");
    prompt.push_str("   Brief explanation (1 sentence)\n\n");
    prompt.push_str("2. [Optimize] alternative_query_if_applicable\n");
    prompt.push_str("   Why this is better (1 sentence)\n\n");
    prompt.push_str(
        "Use [Fix] for error corrections, [Optimize] for improvements, [Next] for next steps.\n",
    );
    prompt.push_str("Each query must be valid jq syntax on a single line.\n\n");

    prompt.push_str("## Natural Language\n");
    prompt.push_str(
        "If the query contains natural language (e.g., 'how to get emails', 'filter by age'),\n",
    );
    prompt.push_str(
        "interpret the user's intent and provide jq query suggestions using [Next] type.\n",
    );
    prompt.push_str("Natural language can appear anywhere in the query, not just at the end.\n\n");

    prompt.push_str(&format!(
        "REMEMBER: Total response must be under {} words.\n",
        word_limit
    ));

    prompt
}

/// Build a prompt for successful query optimization
///
/// Creates a prose prompt that includes the query, output sample,
/// and structure information to help the AI suggest optimizations.
pub fn build_success_prompt(context: &QueryContext, word_limit: u16) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are a jq query assistant helping optimize queries.\n");
    prompt.push_str("CRITICAL: Your response will be displayed in a popup window.\n");
    prompt.push_str(&format!(
        "Keep your ENTIRE response under {} words.\n\n",
        word_limit
    ));

    prompt.push_str("## Current Query\n");
    prompt.push_str(&format!("```\n{}\n```\n\n", context.query));

    prompt.push_str("## Input JSON Structure\n");
    prompt.push_str(&format!("Type: {}\n", context.json_type_info.root_type));
    if let Some(ref elem_type) = context.json_type_info.element_type {
        prompt.push_str(&format!("Element type: {}\n", elem_type));
    }
    if let Some(count) = context.json_type_info.element_count {
        prompt.push_str(&format!("Element count: {}\n", count));
    }
    if !context.json_type_info.top_level_keys.is_empty() {
        prompt.push_str(&format!(
            "Top-level keys: {}\n",
            context.json_type_info.top_level_keys.join(", ")
        ));
    }
    prompt.push_str(&format!(
        "Summary: {}\n\n",
        context.json_type_info.schema_hint
    ));

    if let Some(ref output_sample) = context.output_sample {
        prompt.push_str("## Query Output Sample\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", output_sample));
    }

    prompt.push_str("## Response Format\n");
    prompt.push_str("Provide 3-5 numbered suggestions in this EXACT format:\n\n");
    prompt.push_str("1. [Optimize] optimized_jq_query_here\n");
    prompt.push_str("   Brief explanation (1 sentence)\n\n");
    prompt.push_str("2. [Next] next_step_query_if_applicable\n");
    prompt.push_str("   What this does (1 sentence)\n\n");
    prompt.push_str("Use [Optimize] for improvements, [Next] for next steps or related queries.\n");
    prompt.push_str("Each query must be valid jq syntax on a single line.\n");
    prompt.push_str(
        "If the query is already optimal, provide [Next] suggestions for related operations.\n\n",
    );

    prompt.push_str("## Natural Language\n");
    prompt.push_str(
        "If the query contains natural language (e.g., 'how to get emails', 'filter by age'),\n",
    );
    prompt.push_str(
        "interpret the user's intent and provide jq query suggestions using [Next] type.\n",
    );
    prompt.push_str("Natural language can appear anywhere in the query, not just at the end.\n\n");

    prompt.push_str(&format!(
        "REMEMBER: Total response must be under {} words.\n",
        word_limit
    ));

    prompt
}

/// Build a general help prompt (for non-error queries)
#[allow(dead_code)]
pub fn build_help_prompt(context: &QueryContext) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are a jq query assistant.\n");
    prompt.push_str("Provide concise, helpful suggestions.\n\n");

    prompt.push_str("## Current Query\n");
    prompt.push_str(&format!("```\n{}\n```\n\n", context.query));

    prompt.push_str("## Input JSON Structure\n");
    prompt.push_str(&format!("{}\n\n", context.json_type_info.schema_hint));

    if let Some(ref output_sample) = context.output_sample {
        prompt.push_str("## Current Output\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", output_sample));
    } else if let Some(ref output) = context.output {
        let truncated_output = if output.len() > 500 {
            format!("{}... [truncated]", &output[..500])
        } else {
            output.clone()
        };
        prompt.push_str("## Current Output\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", truncated_output));
    }

    prompt.push_str("Suggest improvements or explain what the query does.\n");

    prompt
}

#[cfg(test)]
mod tests {
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
}
