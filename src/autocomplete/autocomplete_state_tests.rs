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
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_select_previous_stops_at_first() {
    let mut state = AutocompleteState::new();
    let suggestions = vec![
        Suggestion::new("first", SuggestionType::Field),
        Suggestion::new("second", SuggestionType::Field),
        Suggestion::new("third", SuggestionType::Field),
    ];
    state.update_suggestions(suggestions);
    assert_eq!(state.selected_index(), 0);

    state.select_previous();

    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_previous_on_empty() {
    let mut state = AutocompleteState::new();
    state.select_previous();
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_next_on_empty() {
    let mut state = AutocompleteState::new();
    state.select_next();
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

#[test]
fn test_scroll_offset_resets_on_update_suggestions() {
    let mut state = AutocompleteState::new();
    state.scroll_offset = 5;
    state.update_suggestions(vec![Suggestion::new("test", SuggestionType::Field)]);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_offset_resets_on_hide() {
    let mut state = AutocompleteState::new();
    state.update_suggestions(vec![Suggestion::new("test", SuggestionType::Field)]);
    state.scroll_offset = 5;
    state.hide();
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_select_next_stops_at_last() {
    let mut state = AutocompleteState::new();
    let suggestions = vec![
        Suggestion::new("first", SuggestionType::Field),
        Suggestion::new("second", SuggestionType::Field),
    ];
    state.update_suggestions(suggestions);
    state.select_next();
    assert_eq!(state.selected_index(), 1);
    state.select_next();
    assert_eq!(state.selected_index(), 1);
}

#[test]
fn test_scroll_adjusts_when_selection_moves_below_visible() {
    let mut state = AutocompleteState::new();
    let suggestions: Vec<Suggestion> = (0..15)
        .map(|i| Suggestion::new(format!("item{}", i), SuggestionType::Field))
        .collect();
    state.update_suggestions(suggestions);

    for _ in 0..10 {
        state.select_next();
    }
    assert_eq!(state.selected_index(), 10);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_adjusts_when_selection_moves_above_visible() {
    let mut state = AutocompleteState::new();
    let suggestions: Vec<Suggestion> = (0..15)
        .map(|i| Suggestion::new(format!("item{}", i), SuggestionType::Field))
        .collect();
    state.update_suggestions(suggestions);
    state.selected_index = 10;
    state.scroll_offset = 5;

    state.select_previous();
    state.select_previous();
    state.select_previous();
    state.select_previous();
    state.select_previous();
    state.select_previous();

    assert_eq!(state.selected_index(), 4);
    assert_eq!(state.scroll_offset(), 4);
}

#[test]
fn test_visible_suggestions_returns_correct_window() {
    let mut state = AutocompleteState::new();
    let suggestions: Vec<Suggestion> = (0..15)
        .map(|i| Suggestion::new(format!("item{}", i), SuggestionType::Field))
        .collect();
    state.update_suggestions(suggestions);
    state.scroll_offset = 3;

    let visible: Vec<(usize, &Suggestion)> = state.visible_suggestions().collect();
    assert_eq!(visible.len(), 10);
    assert_eq!(visible[0].0, 3);
    assert_eq!(visible[0].1.text, "item3");
    assert_eq!(visible[9].0, 12);
    assert_eq!(visible[9].1.text, "item12");
}

#[test]
fn test_visible_suggestions_with_fewer_than_max() {
    let mut state = AutocompleteState::new();
    let suggestions: Vec<Suggestion> = (0..5)
        .map(|i| Suggestion::new(format!("item{}", i), SuggestionType::Field))
        .collect();
    state.update_suggestions(suggestions);

    let visible: Vec<(usize, &Suggestion)> = state.visible_suggestions().collect();
    assert_eq!(visible.len(), 5);
}
