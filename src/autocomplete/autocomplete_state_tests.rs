//! Tests for autocomplete_state

use super::*;
use crate::test_utils::test_helpers::test_app;

#[test]
fn test_update_suggestions_from_app_when_query_none() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.query = None;
    app.autocomplete
        .update_suggestions(vec![Suggestion::new("test", SuggestionType::Field)]);
    assert!(app.autocomplete.is_visible());

    update_suggestions_from_app(&mut app);

    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_suggestion_type_display() {
    assert_eq!(SuggestionType::Function.to_string(), "function");
    assert_eq!(SuggestionType::Field.to_string(), "field");
    assert_eq!(SuggestionType::Operator.to_string(), "operator");
    assert_eq!(SuggestionType::Pattern.to_string(), "iterator");
    assert_eq!(SuggestionType::Variable.to_string(), "variable");
}

#[test]
fn test_json_field_type_display() {
    assert_eq!(JsonFieldType::String.to_string(), "String");
    assert_eq!(JsonFieldType::Number.to_string(), "Number");
    assert_eq!(JsonFieldType::Boolean.to_string(), "Boolean");
    assert_eq!(JsonFieldType::Null.to_string(), "Null");
    assert_eq!(JsonFieldType::Object.to_string(), "Object");
    assert_eq!(JsonFieldType::Array.to_string(), "Array");
    assert_eq!(
        JsonFieldType::ArrayOf(Box::new(JsonFieldType::String)).to_string(),
        "Array[String]"
    );
}

#[test]
fn test_autocomplete_state_default() {
    let state = AutocompleteState::default();
    assert!(!state.is_visible());
    assert_eq!(state.suggestions().len(), 0);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_previous_wraps_to_end() {
    let mut state = AutocompleteState::new();
    let suggestions = vec![
        Suggestion::new("first", SuggestionType::Field),
        Suggestion::new("second", SuggestionType::Field),
        Suggestion::new("third", SuggestionType::Field),
    ];
    state.update_suggestions(suggestions);
    assert_eq!(state.selected_index(), 0);

    state.select_previous();

    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_select_previous_on_empty() {
    let mut state = AutocompleteState::new();
    state.select_previous();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_selected_returns_none_when_not_visible() {
    let mut state = AutocompleteState::new();
    let suggestions = vec![Suggestion::new("test", SuggestionType::Field)];
    state.suggestions = suggestions;
    state.is_visible = false;

    assert!(state.selected().is_none());
}

#[test]
fn test_selected_returns_none_when_out_of_bounds() {
    let mut state = AutocompleteState::new();
    let suggestions = vec![Suggestion::new("test", SuggestionType::Field)];
    state.suggestions = suggestions;
    state.is_visible = true;
    state.selected_index = 100;

    assert!(state.selected().is_none());
}
