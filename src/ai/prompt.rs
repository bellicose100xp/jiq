//! Prompt template generation
//!
//! Builds prose prompts for AI requests based on query context.
//! Generates different prompts for error troubleshooting vs success optimization.

use super::context::QueryContext;

/// Build a prompt based on query context
///
/// Dispatches to either error troubleshooting or success optimization prompt
/// based on the `is_success` field in the context.
///
/// # Requirements
/// - 3.3: Error context for troubleshooting
/// - 3.4: Success context for optimization suggestions
pub fn build_prompt(context: &QueryContext) -> String {
    if context.is_success {
        build_success_prompt(context)
    } else {
        build_error_prompt(context)
    }
}

/// Build a prompt for error troubleshooting
///
/// Creates a prose prompt that includes the query, error message,
/// JSON sample, and structure information to help the AI provide
/// relevant assistance.
pub fn build_error_prompt(context: &QueryContext) -> String {
    let mut prompt = String::new();

    // System context with strict brevity requirements
    prompt.push_str("You are a jq query assistant helping troubleshoot errors.\n");
    prompt
        .push_str("CRITICAL: Your response will be displayed in a small, non-scrollable window.\n");
    prompt.push_str("Keep your ENTIRE response under 200 words. Be extremely concise.\n");
    prompt.push_str("Format: 1-2 sentences explaining the error, then the corrected query.\n\n");

    // Query information
    prompt.push_str("## Current Query\n");
    prompt.push_str(&format!("```\n{}\n```\n", context.query));
    prompt.push_str(&format!("Cursor position: {}\n\n", context.cursor_pos));

    // Error information
    if let Some(ref error) = context.error {
        prompt.push_str("## Error\n");
        prompt.push_str(&format!("```\n{}\n```\n\n", error));
    }

    // JSON structure information
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

    // JSON sample
    prompt.push_str("## Input JSON Sample\n");
    prompt.push_str(&format!("```json\n{}\n```\n\n", context.input_sample));

    // Instructions - emphasize brevity
    prompt.push_str("## Instructions\n");
    prompt.push_str("Respond in this EXACT format (keep it SHORT):\n\n");
    prompt.push_str("Error: [1 sentence explaining what's wrong]\n\n");
    prompt.push_str("Fix: [the corrected jq query]\n\n");
    prompt.push_str("Why: [1 sentence explaining the fix]\n\n");
    prompt
        .push_str("REMEMBER: Total response must be under 200 words and fit in a small window.\n");

    prompt
}

/// Build a prompt for successful query optimization
///
/// Creates a prose prompt that includes the query, output sample,
/// and structure information to help the AI suggest optimizations.
///
/// # Requirements
/// - 3.4: Success context for optimization suggestions
pub fn build_success_prompt(context: &QueryContext) -> String {
    let mut prompt = String::new();

    // System context with strict brevity requirements
    prompt.push_str("You are a jq query assistant helping optimize queries.\n");
    prompt
        .push_str("CRITICAL: Your response will be displayed in a small, non-scrollable window.\n");
    prompt.push_str("Keep your ENTIRE response under 200 words. Be extremely concise.\n");
    prompt.push_str("Format: Brief analysis, then any optimization suggestions.\n\n");

    // Query information
    prompt.push_str("## Current Query\n");
    prompt.push_str(&format!("```\n{}\n```\n\n", context.query));

    // JSON structure information
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

    // Output sample (truncated)
    if let Some(ref output_sample) = context.output_sample {
        prompt.push_str("## Query Output Sample\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", output_sample));
    }

    // Instructions - emphasize brevity
    prompt.push_str("## Instructions\n");
    prompt.push_str("Respond in this EXACT format (keep it SHORT):\n\n");
    prompt.push_str("Query: [1 sentence explaining what the query does]\n\n");
    prompt.push_str(
        "Tip: [1 optimization suggestion if applicable, or 'Query looks good!' if optimal]\n\n",
    );
    prompt
        .push_str("REMEMBER: Total response must be under 200 words and fit in a small window.\n");

    prompt
}

/// Build a general help prompt (for non-error queries)
// TODO: Remove #[allow(dead_code)] when help prompts are implemented
#[allow(dead_code)] // Phase 1: Reserved for future help feature
pub fn build_help_prompt(context: &QueryContext) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are a jq query assistant.\n");
    prompt.push_str("Provide concise, helpful suggestions.\n\n");

    prompt.push_str("## Current Query\n");
    prompt.push_str(&format!("```\n{}\n```\n\n", context.query));

    prompt.push_str("## Input JSON Structure\n");
    prompt.push_str(&format!("{}\n\n", context.json_type_info.schema_hint));

    // Use output_sample if available, otherwise truncate output
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

        let prompt = build_error_prompt(&ctx);
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

        let prompt = build_error_prompt(&ctx);
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

        let prompt = build_error_prompt(&ctx);
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

    // =========================================================================
    // Tests for build_success_prompt
    // =========================================================================

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

        let prompt = build_success_prompt(&ctx);
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

        let prompt = build_success_prompt(&ctx);
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

        let prompt = build_success_prompt(&ctx);
        assert!(prompt.contains("Type: Array"));
        assert!(prompt.contains("Element type: numbers"));
        assert!(prompt.contains("Element count: 3"));
    }

    // =========================================================================
    // Tests for build_prompt (dispatcher)
    // =========================================================================

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

        let prompt = build_prompt(&ctx);
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

        let prompt = build_prompt(&ctx);
        // Success prompt contains "optimize"
        assert!(prompt.contains("optimize"));
        assert!(!prompt.contains("troubleshoot"));
    }
}
