use super::*;
use crate::ai::suggestion::SuggestionType;

fn suggestion(query: &str) -> Suggestion {
    Suggestion {
        query: query.to_string(),
        description: "desc".to_string(),
        suggestion_type: SuggestionType::Query,
    }
}

#[test]
fn chat_message_constructors_set_role() {
    let user = ChatMessage::user("hi");
    assert_eq!(user.role, ChatRole::User);
    assert_eq!(user.content, "hi");

    let assistant = ChatMessage::assistant("hello");
    assert_eq!(assistant.role, ChatRole::Assistant);
    assert_eq!(assistant.content, "hello");
}

#[test]
fn single_prompt_has_one_user_turn() {
    let prompt = AiPrompt::single("system", "user");
    assert_eq!(prompt.system, "system");
    assert_eq!(prompt.messages, vec![ChatMessage::user("user")]);
}

#[test]
fn total_len_sums_system_and_turns() {
    let prompt = AiPrompt {
        system: "abc".to_string(),
        messages: vec![ChatMessage::user("de"), ChatMessage::assistant("f")],
    };
    assert_eq!(prompt.total_len(), 6);
}

#[test]
fn user_turn_includes_query_when_present() {
    let exchange = ChatExchange {
        question: "why empty?".to_string(),
        query: ".users[]".to_string(),
        raw_response: "{}".to_string(),
        answer: None,
        suggestions: vec![suggestion(".users")],
    };
    assert_eq!(
        exchange.user_turn(),
        "Query at the time: `.users[]`\n\nwhy empty?"
    );
}

#[test]
fn user_turn_is_bare_question_without_query() {
    let exchange = ChatExchange {
        question: "what is jq?".to_string(),
        query: "   ".to_string(),
        raw_response: "{}".to_string(),
        answer: Some("A JSON processor.".to_string()),
        suggestions: vec![],
    };
    assert_eq!(exchange.user_turn(), "what is jq?");
}
