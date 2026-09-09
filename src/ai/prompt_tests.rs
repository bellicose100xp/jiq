//! Tests for prompt assembly

use super::*;
use crate::ai::chat::ChatRole;
use crate::ai::context::ContextParams;
use crate::ai::suggestion::SuggestionType;

fn error_context() -> QueryContext {
    QueryContext {
        query: ".name".to_string(),
        cursor_pos: 5,
        output_sample: None,
        error: Some("syntax error".to_string()),
        is_success: false,
        is_empty_result: false,
        input_schema: None,
        base_query: None,
        base_query_result: None,
    }
}

fn success_context() -> QueryContext {
    QueryContext {
        query: ".users[]".to_string(),
        cursor_pos: 8,
        output_sample: Some(r#"{"name":"Alice"}"#.to_string()),
        error: None,
        is_success: true,
        is_empty_result: false,
        input_schema: Some(r#"{"users":[{"name":"string"}]}"#.to_string()),
        base_query: None,
        base_query_result: None,
    }
}

fn exchange(question: &str, raw: &str) -> ChatExchange {
    ChatExchange {
        question: question.to_string(),
        query: ".q".to_string(),
        raw_response: raw.to_string(),
        answer: Some("a".to_string()),
        suggestions: vec![],
    }
}

fn suggestion(query: &str, kind: SuggestionType) -> Suggestion {
    Suggestion {
        query: query.to_string(),
        description: format!("does {}", query),
        suggestion_type: kind,
    }
}

fn auto_inputs<'a>(context: &'a QueryContext, history: &'a [ChatExchange]) -> PromptInputs<'a> {
    PromptInputs {
        context,
        extra_instructions: None,
        history,
        question: None,
        displayed_suggestions: &[],
    }
}

// ---------------------------------------------------------------------------
// System prompt
// ---------------------------------------------------------------------------

#[test]
fn system_prompt_has_role_format_rules_and_non_ascii_rules() {
    let system = build_system_prompt(None, None);
    assert!(system.contains("jq query assistant"));
    assert!(system.contains("## Output Format (STRICT)"));
    assert!(system.contains(r#"{"suggestions":[]}"#));
    assert!(system.contains("## Non-ASCII Field Names"));
    assert!(system.contains(r#"`"query"` (a query that does what the user asked for)"#));
    assert!(system.contains("`answer`"));
    assert!(!system.contains("\"next\""), "the next type is gone");
}

#[test]
fn system_prompt_includes_schema_when_present() {
    let system = build_system_prompt(Some(r#"{"name":"string"}"#), None);
    assert!(system.contains("## Input JSON Schema"));
    assert!(system.contains(r#"{"name":"string"}"#));

    let without = build_system_prompt(None, None);
    assert!(!without.contains("## Input JSON Schema"));
}

#[test]
fn system_prompt_appends_extra_instructions_after_rules() {
    let system = build_system_prompt(None, Some("  Prefer map() over .[]  "));
    assert!(system.contains("## Additional User Preferences"));
    assert!(system.contains("Prefer map() over .[]"));
    let rules = system.find("## Output Format").unwrap();
    let extra = system.find("## Additional User Preferences").unwrap();
    assert!(rules < extra);
    assert!(system.ends_with("Prefer map() over .[]\n\n"));
}

#[test]
fn system_prompt_ignores_blank_extra_instructions() {
    let system = build_system_prompt(None, Some("   \n\t  "));
    assert!(!system.contains("## Additional User Preferences"));
    assert_eq!(system, build_system_prompt(None, None));
}

// ---------------------------------------------------------------------------
// Auto message (query changed)
// ---------------------------------------------------------------------------

#[test]
fn auto_message_for_error_includes_query_error_and_fix_task() {
    let message = build_auto_message(&error_context());
    assert!(message.contains("## Current Query\n```\n.name\n```"));
    assert!(message.contains("Cursor position: 5"));
    assert!(message.contains("## Error\n```\nsyntax error\n```"));
    assert!(message.contains("The query failed. Suggest 3-5 `fix` suggestions"));
    assert!(message.contains("Do not include an `answer`"));
    assert!(!message.contains("## Current Query Output"));
}

#[test]
fn auto_message_for_success_includes_output_and_optimize_task() {
    let message = build_auto_message(&success_context());
    assert!(message.contains("## Current Query Output\n```json\n{\"name\":\"Alice\"}\n```"));
    assert!(message.contains("The query ran successfully. Suggest up to 5 `optimize` suggestions"));
    assert!(message.contains(r#"return `{"suggestions":[]}`"#));
    assert!(!message.contains("## Error"));
    assert!(
        !message.contains("## Input JSON Schema"),
        "schema lives in the system prompt, not the per-turn message"
    );
}

#[test]
fn auto_message_handles_natural_language_queries() {
    let message = build_auto_message(&error_context());
    assert!(message.contains("natural language rather than jq"));
    assert!(message.contains("return `query` suggestions"));
}

#[test]
fn auto_message_error_includes_last_working_query_and_output() {
    let mut ctx = error_context();
    ctx.base_query = Some(".users".to_string());
    ctx.base_query_result = Some(r#"[{"name":"Alice"}]"#.to_string());

    let message = build_auto_message(&ctx);
    assert!(message.contains("## Last Working Query\n```\n.users\n```"));
    assert!(message.contains("## Last Working Query Output\n```json\n[{\"name\":\"Alice\"}]\n```"));
}

#[test]
fn auto_message_error_without_base_query_omits_section() {
    let message = build_auto_message(&error_context());
    assert!(!message.contains("Last Working Query"));
}

#[test]
fn auto_message_success_excludes_base_query_when_result_not_empty() {
    let mut ctx = success_context();
    ctx.base_query = Some(".users".to_string());
    ctx.base_query_result = Some("[]".to_string());

    let message = build_auto_message(&ctx);
    assert!(!message.contains("Last Working Query"));
    assert!(!message.contains("Last Non-Empty Query"));
}

#[test]
fn auto_message_success_with_empty_result_includes_last_non_empty_query() {
    let mut ctx = success_context();
    ctx.output_sample = None;
    ctx.is_empty_result = true;
    ctx.base_query = Some(".users".to_string());
    ctx.base_query_result = Some(r#"[{"name":"Alice"}]"#.to_string());

    let message = build_auto_message(&ctx);
    assert!(
        message.contains("The current query output is empty or consists entirely of null values.")
    );
    assert!(message.contains("## Last Non-Empty Query\n```\n.users\n```"));
    assert!(message.contains("## Last Non-Empty Query Output"));
}

// ---------------------------------------------------------------------------
// Chat message (user question)
// ---------------------------------------------------------------------------

#[test]
fn chat_message_includes_context_question_and_answer_task() {
    let message = build_chat_message(&success_context(), "  why only Alice?  ", &[]);
    assert!(message.contains("## Current Query\n```\n.users[]\n```"));
    assert!(message.contains("## Current Query Output"));
    assert!(message.contains("## Question\nwhy only Alice?\n\n"));
    assert!(message.contains("Answer the question in `answer`"));
    assert!(message.contains("put each runnable query in `suggestions` with type `query`"));
    assert!(!message.contains("## Suggestions Currently Shown"));
}

#[test]
fn chat_message_lists_displayed_suggestions_with_numbers_and_labels() {
    let shown = vec![
        suggestion(".a", SuggestionType::Fix),
        suggestion(".b", SuggestionType::Optimize),
    ];
    let message = build_chat_message(&error_context(), "explain 2", &shown);
    assert!(message.contains("## Suggestions Currently Shown to the User\n"));
    assert!(message.contains("1. [Fix] `.a` - does .a\n"));
    assert!(message.contains("2. [Optimize] `.b` - does .b\n"));
    let shown_pos = message.find("## Suggestions Currently Shown").unwrap();
    let question_pos = message.find("## Question").unwrap();
    assert!(shown_pos < question_pos);
}

#[test]
fn chat_message_for_error_context_includes_error() {
    let message = build_chat_message(&error_context(), "what went wrong?", &[]);
    assert!(message.contains("## Error\n```\nsyntax error\n```"));
}

// ---------------------------------------------------------------------------
// Full prompt assembly
// ---------------------------------------------------------------------------

#[test]
fn build_prompt_auto_has_system_and_single_user_turn() {
    let ctx = success_context();
    let prompt = build_prompt(&auto_inputs(&ctx, &[]));
    assert!(prompt.system.contains("## Input JSON Schema"));
    assert_eq!(prompt.messages.len(), 1);
    assert_eq!(prompt.messages[0].role, ChatRole::User);
    assert!(prompt.messages[0].content.contains("## Task"));
    assert!(prompt.messages[0].content.contains("`optimize`"));
}

#[test]
fn build_prompt_auto_dispatches_on_error() {
    let ctx = error_context();
    let prompt = build_prompt(&auto_inputs(&ctx, &[]));
    assert!(prompt.messages[0].content.contains("The query failed"));
}

#[test]
fn build_prompt_replays_history_as_alternating_turns() {
    let ctx = success_context();
    let history = vec![
        exchange("q1", "{\"answer\":\"a1\"}"),
        exchange("q2", "{\"answer\":\"a2\"}"),
    ];
    let prompt = build_prompt(&PromptInputs {
        context: &ctx,
        extra_instructions: None,
        history: &history,
        question: Some("q3"),
        displayed_suggestions: &[],
    });

    let roles: Vec<ChatRole> = prompt.messages.iter().map(|m| m.role).collect();
    assert_eq!(
        roles,
        vec![
            ChatRole::User,
            ChatRole::Assistant,
            ChatRole::User,
            ChatRole::Assistant,
            ChatRole::User
        ]
    );
    assert_eq!(prompt.messages[0].content, history[0].user_turn());
    assert_eq!(prompt.messages[1].content, "{\"answer\":\"a1\"}");
    assert_eq!(prompt.messages[3].content, "{\"answer\":\"a2\"}");
    assert!(prompt.messages[4].content.contains("## Question\nq3"));
}

#[test]
fn build_prompt_auto_request_also_carries_history() {
    let ctx = error_context();
    let history = vec![exchange("q1", "{}")];
    let prompt = build_prompt(&auto_inputs(&ctx, &history));
    assert_eq!(prompt.messages.len(), 3);
    assert_eq!(prompt.messages[1].role, ChatRole::Assistant);
    assert!(prompt.messages[2].content.contains("The query failed"));
}

#[test]
fn build_prompt_caps_replayed_history() {
    let ctx = success_context();
    let history: Vec<ChatExchange> = (0..(MAX_HISTORY_EXCHANGES + 4))
        .map(|i| exchange(&format!("q{}", i), "{}"))
        .collect();
    let prompt = build_prompt(&auto_inputs(&ctx, &history));

    assert_eq!(prompt.messages.len(), MAX_HISTORY_EXCHANGES * 2 + 1);
    assert!(
        prompt.messages[0].content.contains("q4"),
        "oldest exchanges are dropped: {}",
        prompt.messages[0].content
    );
}

#[test]
fn build_prompt_passes_extra_instructions_into_system() {
    let ctx = success_context();
    let prompt = build_prompt(&PromptInputs {
        context: &ctx,
        extra_instructions: Some("Keep it terse."),
        history: &[],
        question: None,
        displayed_suggestions: &[],
    });
    assert!(prompt.system.contains("Keep it terse."));
    assert!(!prompt.messages[0].content.contains("Keep it terse."));
}

#[test]
fn base_query_result_truncation_flows_through_context() {
    let long_result = "x".repeat(200);
    let truncated = crate::ai::context::truncate_json(&long_result, 50);
    let ctx = QueryContext::new(
        ".broken".to_string(),
        3,
        None,
        Some("error".to_string()),
        ContextParams {
            input_schema: None,
            base_query: Some(".ok"),
            base_query_result: Some(&truncated),
            is_empty_result: false,
        },
        1000,
    );
    let message = build_auto_message(&ctx);
    assert!(message.contains("... [truncated]"));
    assert!(!message.contains(&long_result));
}
