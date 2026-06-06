//! Value-context insertion tests
//!
//! Covers the `SuggestionType::Value` dispatch branch in `insert_suggestion`:
//! when a Value-type suggestion is accepted while the cursor sits inside an
//! unclosed `"..."` at a comparison position, insertion must route through
//! `value_trigger::classify` + `value_insertion::apply_to_textarea` and
//! short-circuit the context-based dispatch.

use super::*;

/// Insert `suggestion` into a query whose buffer ends inside an unclosed string
/// at a comparison position, with the cursor at end-of-line.
fn insert_at_unclosed_value(query: &str, suggestion: &Suggestion) -> String {
    let (mut textarea, mut query_state) = setup_insertion_test("");
    textarea.insert_str(query);
    insert_suggestion(&mut textarea, &mut query_state, suggestion);
    textarea.lines()[0].clone()
}

#[test]
fn test_value_suggestion_inserts_into_unclosed_string() {
    // `.name == "` with the cursor inside the open quote is a value-comparison
    // trigger. A Value-type suggestion must be wrapped and quote-closed.
    let value = Suggestion::new("alice", SuggestionType::Value);
    let result = insert_at_unclosed_value(".name == \"", &value);

    assert_eq!(
        result, ".name == \"alice\"",
        "Value suggestion should be inserted into the open string and the quote closed"
    );
}

#[test]
fn test_value_branch_is_type_gated_not_field() {
    // Same buffer, but a Field-type suggestion of the identical text must NOT
    // take the value path. It falls through to context dispatch (the partial
    // here is the bare `\"`, classified as FunctionContext), so the result is
    // the raw token with no closing quote -- distinct from the Value result.
    let field = Suggestion::new("alice", SuggestionType::Field);
    let result = insert_at_unclosed_value(".name == \"", &field);

    assert_eq!(
        result, ".name == alice",
        "Field suggestion must bypass the value-insertion path (no quote closing)"
    );
    assert_ne!(
        result, ".name == \"alice\"",
        "Field suggestion must not produce the quote-closed value output"
    );
}
