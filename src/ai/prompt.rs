//! Prompt assembly for AI requests.
//!
//! Every request is an [`AiPrompt`]: a system prompt that carries the
//! assistant's role, the output contract, and the input JSON schema, followed
//! by the replayed chat history and one final user turn. The final turn is
//! either an *auto* message (the query changed: ask for fixes or
//! optimizations) or a *chat* message (the user typed a question in the AI
//! popup).

use super::chat::{AiPrompt, ChatExchange, ChatMessage};
use super::context::QueryContext;
use super::suggestion::Suggestion;

/// Upper bound on replayed exchanges, so long sessions stay within model
/// context limits and request cost.
pub const MAX_HISTORY_EXCHANGES: usize = 12;

/// Everything needed to assemble one request.
pub struct PromptInputs<'a> {
    /// Current query, result or error, and JSON context.
    pub context: &'a QueryContext,
    /// User-supplied guidance from `[ai] extra_instructions`.
    pub extra_instructions: Option<&'a str>,
    /// Completed chat exchanges to replay, oldest first.
    pub history: &'a [ChatExchange],
    /// The question typed in the AI popup. `None` means an auto request.
    pub question: Option<&'a str>,
    /// Suggestions currently on screen, so a chat question can refer to them
    /// ("explain 2", "why does the first one work").
    pub displayed_suggestions: &'a [Suggestion],
}

/// Strict output rules shared by auto and chat requests.
///
/// The response reaches a deterministic parser that expects exactly one JSON
/// object. Any deviation (code fences, prose wrapper, trailing commentary)
/// must be recovered by fallback heuristics, which is brittle, so the rules
/// spell out the exact shape.
const OUTPUT_FORMAT_RULES: &str = "\
## Output Format (STRICT)\n\
Your entire response MUST be a single JSON object and NOTHING else. \
Follow these rules exactly:\n\
\n\
1. The FIRST character of your response MUST be `{` (an opening brace).\n\
2. The LAST character of your response MUST be `}` (a closing brace).\n\
3. Do NOT wrap the JSON in markdown code fences (no ```json, no ```).\n\
4. Do NOT prepend explanations like \"Here are the suggestions:\".\n\
5. Do NOT append commentary like \"Hope this helps!\" after the JSON.\n\
6. Do NOT include newlines outside of JSON string values.\n\
7. Use STRAIGHT double quotes `\"` for all JSON strings. Never use \
curly/smart quotes like `\u{201c}` `\u{201d}`.\n\
8. Escape inner quotes with `\\\"`. Escape backslashes with `\\\\`.\n\
9. Non-ASCII characters inside string values (CJK, emoji, accented Latin) \
should appear literally, NOT as `\\uXXXX` escapes.\n\
\n\
Schema:\n\
`{\"answer\": \"optional prose\", \"suggestions\": [{\"type\": \"fix\", \"query\": \"jq_query\", \"details\": \"1 line description\"}]}`\n\
\n\
Field rules:\n\
- `answer`: plain prose for the user. Omit it (or use \"\") unless the request \
asks for one. Short paragraphs, no markdown headings, no code fences; put \
runnable queries in `suggestions`, not in `answer`.\n\
- `type`: one of `\"fix\"` (error corrections), `\"optimize\"` (improvements to a \
working query), `\"query\"` (a query that does what the user asked for)\n\
- `query`: valid jq syntax, single line, no trailing whitespace\n\
- `details`: ONE sentence, no line breaks\n\
- At most 5 suggestions\n\
\n\
If you have nothing to suggest, return exactly: `{\"suggestions\":[]}`\n\n";

/// jq's `.field` shorthand only accepts ASCII identifiers. Without this rule
/// the model often suggests invalid queries like `.名前` which jq rejects.
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

/// Assemble the full request: system prompt, replayed history, final user turn.
pub fn build_prompt(inputs: &PromptInputs) -> AiPrompt {
    let system = build_system_prompt(
        inputs.context.input_schema.as_deref(),
        inputs.extra_instructions,
    );

    let mut messages = Vec::new();
    let start = inputs.history.len().saturating_sub(MAX_HISTORY_EXCHANGES);
    for exchange in &inputs.history[start..] {
        messages.push(ChatMessage::user(exchange.user_turn()));
        messages.push(ChatMessage::assistant(exchange.raw_response.clone()));
    }

    let final_turn = match inputs.question {
        Some(question) => {
            build_chat_message(inputs.context, question, inputs.displayed_suggestions)
        }
        None => build_auto_message(inputs.context),
    };
    messages.push(ChatMessage::user(final_turn));

    AiPrompt { system, messages }
}

/// The system prompt: role, output contract, jq rules, input schema, and the
/// user's extra instructions. Stable across a session so providers can cache it.
pub fn build_system_prompt(input_schema: Option<&str>, extra_instructions: Option<&str>) -> String {
    let mut prompt = String::new();

    prompt.push_str(
        "You are a jq query assistant embedded in jiq, an interactive terminal tool. \
         The user is editing a jq query against a JSON document and sees your \
         suggestions as a numbered list they can apply with one keystroke. \
         You also answer questions the user types about the data, the query, or jq itself. \
         Later turns may refer to earlier ones; keep the conversation's context in mind.\n\n",
    );

    if let Some(schema) = input_schema {
        prompt.push_str("## Input JSON Schema\n");
        prompt.push_str(&format!("```json\n{}\n```\n\n", schema));
    }

    prompt.push_str(OUTPUT_FORMAT_RULES);
    prompt.push_str(NON_ASCII_KEY_RULES);

    if let Some(extra) = extra_instructions {
        let extra = extra.trim();
        if !extra.is_empty() {
            prompt.push_str("## Additional User Preferences\n");
            prompt.push_str(
                "Apply these user preferences where they don't conflict with the rules above. \
                 The Output Format rules always take precedence.\n",
            );
            prompt.push_str(extra);
            prompt.push_str("\n\n");
        }
    }

    prompt
}

/// The current query plus whatever the run produced: error text, output
/// sample, and the last working query when the current one is broken or empty.
fn build_query_context(context: &QueryContext) -> String {
    let mut section = String::new();

    section.push_str("## Current Query\n");
    section.push_str(&format!("```\n{}\n```\n", context.query));
    section.push_str(&format!("Cursor position: {}\n\n", context.cursor_pos));

    if let Some(ref error) = context.error {
        section.push_str("## Error\n");
        section.push_str(&format!("```\n{}\n```\n\n", error));
    }

    if let Some(ref output_sample) = context.output_sample {
        section.push_str("## Current Query Output\n");
        section.push_str(&format!("```json\n{}\n```\n\n", output_sample));
    } else if context.is_success && context.is_empty_result {
        section.push_str("## Current Query Output\n");
        section
            .push_str("The current query output is empty or consists entirely of null values.\n\n");
    }

    let show_base = !context.is_success || context.is_empty_result;
    if show_base && let Some(ref base_query) = context.base_query {
        let label = if context.is_success {
            "Last Non-Empty Query"
        } else {
            "Last Working Query"
        };
        section.push_str(&format!("## {}\n", label));
        section.push_str(&format!("```\n{}\n```\n\n", base_query));

        if let Some(ref result) = context.base_query_result {
            section.push_str(&format!("## {} Output\n", label));
            section.push_str(&format!("```json\n{}\n```\n\n", result));
        }
    }

    section
}

/// Final user turn for an auto request (the query changed).
///
/// Asks for `fix` suggestions when the query errored and `optimize`
/// suggestions when it ran, with no prose answer either way.
pub fn build_auto_message(context: &QueryContext) -> String {
    let mut message = build_query_context(context);

    message.push_str("## Task\n");
    if context.is_success {
        message.push_str(
            "The query ran successfully. Suggest up to 5 `optimize` suggestions: \
             more concise, more idiomatic, or more robust forms that produce the same \
             result, or a better way to get what the query is clearly reaching for. \
             If the query is already optimal, return `{\"suggestions\":[]}`.\n",
        );
    } else {
        message.push_str(
            "The query failed. Suggest 3-5 `fix` suggestions that correct the error while \
             preserving the user's evident intent.\n",
        );
    }
    message.push_str(
        "If the query text is natural language rather than jq, treat it as the user's \
         intent and return `query` suggestions that implement it.\n\
         Do not include an `answer`; the user has not asked a question.\n",
    );

    message
}

/// Final user turn for a chat request (the user typed a question).
///
/// Includes the current query context and the suggestions on screen so the
/// question can refer to them, then asks for a prose `answer` plus any
/// `query` suggestions that carry it out.
pub fn build_chat_message(
    context: &QueryContext,
    question: &str,
    displayed_suggestions: &[Suggestion],
) -> String {
    let mut message = build_query_context(context);

    if !displayed_suggestions.is_empty() {
        message.push_str("## Suggestions Currently Shown to the User\n");
        for (i, suggestion) in displayed_suggestions.iter().enumerate() {
            message.push_str(&format!(
                "{}. {} `{}` - {}\n",
                i + 1,
                suggestion.suggestion_type.label(),
                suggestion.query,
                suggestion.description
            ));
        }
        message.push('\n');
    }

    message.push_str("## Question\n");
    message.push_str(question.trim());
    message.push_str("\n\n");

    message.push_str("## Task\n");
    message.push_str(
        "Answer the question in `answer`: direct, specific to this data and query, \
         a few sentences at most. When a jq query would help (the user asked how to do \
         something, asked for a fix, or asked for an alternative), put each runnable \
         query in `suggestions` with type `query` (or `fix` / `optimize` when that is \
         what it is) so the user can apply it directly. If the question needs no query, \
         return `\"suggestions\": []`.\n",
    );

    message
}

#[cfg(test)]
#[path = "prompt_tests.rs"]
mod prompt_tests;
