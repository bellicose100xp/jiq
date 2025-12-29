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

    if let Some(ref schema) = context.input_schema {
        prompt.push_str("## Input JSON Schema\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", schema));
    }

    prompt.push_str("## Input JSON Sample\n");
    prompt.push_str(&format!("```json\n{}\n```\n\n", context.input_sample));

    if let Some(ref base_query) = context.base_query {
        prompt.push_str("## Last Working Query\n");
        prompt.push_str(&format!("```\n{}\n```\n\n", base_query));

        if let Some(ref result) = context.base_query_result {
            prompt.push_str("## Its Output\n");
            prompt.push_str(&format!("```json\n{}\n```\n\n", result));
        }
    }

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

    prompt.push_str("## Natural Language in Query\n");
    prompt.push_str("The query may contain natural language. Two patterns:\n\n");
    prompt.push_str("### Pattern A: `<jq_query> <natural_language>`\n");
    prompt.push_str("User has a partial jq query followed by natural language.\n");
    prompt.push_str("The natural language could be:\n");
    prompt.push_str("- Debugging: 'why no results', 'why empty'\n");
    prompt.push_str("- Extending: 'how to filter by age', 'add sorting'\n");
    prompt.push_str("- Understanding: 'what does this do'\n");
    prompt.push_str("- Alternatives: 'is there a better way'\n");
    prompt.push_str("- Next steps: 'now get their names too'\n\n");
    prompt.push_str("You must:\n");
    prompt.push_str("1. IDENTIFY the jq query portion (valid jq syntax before natural language)\n");
    prompt.push_str("2. UNDERSTAND what the user is asking about that query\n");
    prompt.push_str("3. RESPOND appropriately (debug, extend, explain, or suggest alternatives)\n");
    prompt.push_str(
        "CRITICAL: Do NOT suggest 'remove trailing text'. ADDRESS the user's intent!\n\n",
    );
    prompt.push_str("### Pattern B: `<natural_language>` only\n");
    prompt.push_str(
        "Entire query is natural language. Interpret intent and provide [Next] suggestions.\n\n",
    );

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

    if let Some(ref schema) = context.input_schema {
        prompt.push_str("## Input JSON Schema\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", schema));
    }

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

    prompt.push_str("## Natural Language in Query\n");
    prompt.push_str("The query may contain natural language. Two patterns:\n\n");
    prompt.push_str("### Pattern A: `<jq_query> <natural_language>`\n");
    prompt.push_str("User has a partial jq query followed by natural language.\n");
    prompt.push_str("The natural language could be:\n");
    prompt.push_str("- Debugging: 'why no results', 'why empty'\n");
    prompt.push_str("- Extending: 'how to filter by age', 'add sorting'\n");
    prompt.push_str("- Understanding: 'what does this do'\n");
    prompt.push_str("- Alternatives: 'is there a better way'\n");
    prompt.push_str("- Next steps: 'now get their names too'\n\n");
    prompt.push_str("You must:\n");
    prompt.push_str("1. IDENTIFY the jq query portion (valid jq syntax before natural language)\n");
    prompt.push_str("2. UNDERSTAND what the user is asking about that query\n");
    prompt.push_str("3. RESPOND appropriately (debug, extend, explain, or suggest alternatives)\n");
    prompt.push_str(
        "CRITICAL: Do NOT suggest 'remove trailing text'. ADDRESS the user's intent!\n\n",
    );
    prompt.push_str("### Pattern B: `<natural_language>` only\n");
    prompt.push_str(
        "Entire query is natural language. Interpret intent and provide [Next] suggestions.\n\n",
    );

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
#[path = "prompt_tests.rs"]
mod prompt_tests;
