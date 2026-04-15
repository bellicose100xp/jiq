//! Prompt template generation
//!
//! Builds prose prompts for AI requests based on query context.
//! Generates different prompts for error troubleshooting vs success optimization.

use super::context::QueryContext;

/// Shared guidance for non-ASCII field names.
///
/// jq's `.field` shorthand only accepts ASCII `[A-Za-z_][A-Za-z_0-9]*` —
/// CJK, emoji, accented Latin, hyphens, spaces, and digit-start keys must
/// use bracket notation `.["key"]`. Without this rule the model often
/// suggests invalid queries like `.名前` which jq rejects as syntax errors.
/// Shared, strict output-format rules to maximize parse reliability.
///
/// The response reaches a deterministic parser that expects exactly
/// `{"suggestions": [...]}`. Any deviation (code fences, prose wrapper,
/// trailing commentary) must be extracted by fallback heuristics, which
/// is brittle. These rules tell the model exactly what shape to produce.
fn build_output_format_rules(example_type: &str) -> String {
    format!(
        "## Output Format (STRICT)\n\
Your entire response MUST be a single JSON object and NOTHING else. \
Follow these rules exactly:\n\
\n\
1. The FIRST character of your response MUST be `{{` (an opening brace).\n\
2. The LAST character of your response MUST be `}}` (a closing brace).\n\
3. Do NOT wrap the JSON in markdown code fences (no ```json, no ```).\n\
4. Do NOT prepend explanations like \"Here are the suggestions:\".\n\
5. Do NOT append commentary like \"Hope this helps!\" after the JSON.\n\
6. Do NOT include newlines outside of JSON string values — \
emit the whole object on a single line OR with standard JSON indentation, \
never both, never with trailing prose.\n\
7. Use STRAIGHT double quotes `\"` for all JSON strings. Never use \
curly/smart quotes like `\u{201c}` `\u{201d}`.\n\
8. Escape inner quotes with `\\\"`. Escape backslashes with `\\\\`.\n\
9. Non-ASCII characters inside string values (CJK, emoji, accented Latin) \
should appear literally, NOT as `\\uXXXX` escapes.\n\
\n\
Schema:\n\
`{{\"suggestions\": [{{\"type\": \"{example_type}\", \"query\": \"jq_query\", \"details\": \"1 line description\"}}]}}`\n\
\n\
Field rules:\n\
- `type`: one of `\"fix\"` (error corrections), `\"optimize\"` (improvements), `\"next\"` (next steps / related queries)\n\
- `query`: valid jq syntax, single line, no trailing whitespace\n\
- `details`: ONE sentence, no line breaks\n\
- Provide 3-5 suggestions total\n\
\n\
If you cannot comply with every rule above, return this exact string instead: \
`{{\"suggestions\":[]}}`\n\n",
    )
}

const NON_ASCII_KEY_RULES: &str = "\
## Non-ASCII Field Names (CRITICAL)\n\
jq's `.field` shorthand is restricted to ASCII identifiers matching \
`[A-Za-z_][A-Za-z_0-9]*`. ANY key containing non-ASCII characters \
(CJK like `名前`, emoji like `👋`, accented Latin like `café`, \
Cyrillic, Arabic, etc.) OR ASCII characters outside the identifier \
set (hyphens, spaces, dots, digit-start) MUST use bracket notation:\n\
- Correct:   `.[\"名前\"]`, `.[\"👋\"]`, `.[\"café\"]`, `.[\"my-field\"]`\n\
- Incorrect: `.名前`, `.👋`, `.café`, `.my-field` (all produce jq syntax errors)\n\
Bracket notation composes without a leading dot between segments: \
`.users[][\"名前\"]`, not `.users[].\"名前\"`.\n\
When suggesting fixes for queries that reference non-ASCII keys, \
ALWAYS emit bracket notation. When emitting `optimize` suggestions, \
do NOT propose removing brackets around non-ASCII keys — the brackets \
are required, not optional.\n\n\
";

/// Build a prompt based on query context
///
/// Dispatches to either error troubleshooting or success optimization prompt
/// based on the `is_success` field in the context.
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

    prompt.push_str("You are a jq query assistant helping troubleshoot errors.\n");

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

    if let Some(ref base_query) = context.base_query {
        prompt.push_str("## Last Working Query\n");
        prompt.push_str(&format!("```\n{}\n```\n\n", base_query));

        if let Some(ref result) = context.base_query_result {
            prompt.push_str("## Last Working Query Output\n");
            prompt.push_str(&format!("```json\n{}\n```\n\n", result));
        }
    }

    prompt.push_str(&build_output_format_rules("fix"));
    prompt.push_str(NON_ASCII_KEY_RULES);

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

    prompt
}

/// Build a prompt for successful query optimization
///
/// Creates a prose prompt that includes the query, output sample,
/// and structure information to help the AI suggest optimizations.
pub fn build_success_prompt(context: &QueryContext) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are a jq query assistant helping optimize queries.\n");

    prompt.push_str("## Current Query\n");
    prompt.push_str(&format!("```\n{}\n```\n", context.query));
    prompt.push_str(&format!("Cursor position: {}\n\n", context.cursor_pos));

    if let Some(ref schema) = context.input_schema {
        prompt.push_str("## Input JSON Schema\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", schema));
    }

    if let Some(ref output_sample) = context.output_sample {
        prompt.push_str("## Current Query Output\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", output_sample));
    } else if context.is_empty_result {
        prompt.push_str("## Current Query Output\n");
        prompt
            .push_str("The current query output is empty or consists entirely of null values.\n\n");
    }

    if context.is_empty_result
        && let Some(ref base_query) = context.base_query
    {
        prompt.push_str("## Last Non-Empty Query\n");
        prompt.push_str(&format!("```\n{}\n```\n\n", base_query));

        if let Some(ref result) = context.base_query_result {
            prompt.push_str("## Last Non-Empty Query Output (displayed in results)\n");
            prompt.push_str(&format!("```json\n{}\n```\n\n", result));
        }
    }

    prompt.push_str(&build_output_format_rules("optimize"));
    prompt.push_str(
        "If the query is already optimal, provide \"next\" suggestions for related operations.\n\n",
    );
    prompt.push_str(NON_ASCII_KEY_RULES);

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

    prompt
}

#[cfg(test)]
#[path = "prompt_tests.rs"]
mod prompt_tests;
