use super::*;
use crate::autocomplete::{Suggestion, SuggestionType};
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn render_autocomplete_with_suggestions(suggestions: Vec<Suggestion>) -> String {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.autocomplete.update_suggestions(suggestions);

    let mut terminal = create_test_terminal(80, 24);
    let input_area = Rect::new(0, 0, 80, 3);

    terminal
        .draw(|f| {
            let _ = render_popup(&app, f, input_area);
        })
        .unwrap();

    terminal.backend().to_string()
}

fn render_autocomplete_with_scroll(suggestions: Vec<Suggestion>, navigate_down: usize) -> String {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.autocomplete.update_suggestions(suggestions);

    for _ in 0..navigate_down {
        app.autocomplete.select_next();
    }

    let mut terminal = create_test_terminal(80, 24);
    let input_area = Rect::new(0, 0, 80, 3);

    terminal
        .draw(|f| {
            let _ = render_popup(&app, f, input_area);
        })
        .unwrap();

    terminal.backend().to_string()
}

#[test]
fn snapshot_variable_suggestions() {
    let suggestions = vec![
        Suggestion::new("$ENV", SuggestionType::Variable),
        Suggestion::new("$__loc__", SuggestionType::Variable),
        Suggestion::new("$x", SuggestionType::Variable),
    ];

    let output = render_autocomplete_with_suggestions(suggestions);
    assert_snapshot!(output);
}

#[test]
fn snapshot_operator_suggestions() {
    let suggestions = vec![
        Suggestion::new("|", SuggestionType::Operator).with_description("Pipe operator"),
        Suggestion::new("//", SuggestionType::Operator).with_description("Alternative operator"),
        Suggestion::new("and", SuggestionType::Operator).with_description("Logical AND"),
    ];

    let output = render_autocomplete_with_suggestions(suggestions);
    assert_snapshot!(output);
}

#[test]
fn snapshot_mixed_suggestion_types() {
    let suggestions = vec![
        Suggestion::new("$x", SuggestionType::Variable),
        Suggestion::new(".name", SuggestionType::Field),
        Suggestion::new("map", SuggestionType::Function),
        Suggestion::new("and", SuggestionType::Operator),
    ];

    let output = render_autocomplete_with_suggestions(suggestions);
    assert_snapshot!(output);
}

#[test]
fn snapshot_scrolled_suggestions() {
    let suggestions: Vec<Suggestion> = (0..15)
        .map(|i| Suggestion::new(format!(".field{}", i), SuggestionType::Field))
        .collect();

    let output = render_autocomplete_with_scroll(suggestions, 12);
    assert_snapshot!(output);
}

#[test]
fn snapshot_selection_at_bottom_of_visible_window() {
    let suggestions: Vec<Suggestion> = (0..15)
        .map(|i| Suggestion::new(format!(".field{}", i), SuggestionType::Field))
        .collect();

    let output = render_autocomplete_with_scroll(suggestions, 9);
    assert_snapshot!(output);
}

#[test]
fn snapshot_selection_after_scroll_then_back_up() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    let suggestions: Vec<Suggestion> = (0..15)
        .map(|i| Suggestion::new(format!(".field{}", i), SuggestionType::Field))
        .collect();
    app.autocomplete.update_suggestions(suggestions);

    for _ in 0..12 {
        app.autocomplete.select_next();
    }
    for _ in 0..10 {
        app.autocomplete.select_previous();
    }

    let mut terminal = create_test_terminal(80, 24);
    let input_area = Rect::new(0, 0, 80, 3);

    terminal
        .draw(|f| {
            let _ = render_popup(&app, f, input_area);
        })
        .unwrap();

    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn snapshot_fixed_width_type_labels_with_varying_field_lengths() {
    use crate::autocomplete::JsonFieldType;

    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    let suggestions = vec![
        Suggestion::new_with_type("a", SuggestionType::Field, Some(JsonFieldType::String)),
        Suggestion::new_with_type(
            "mediumFieldName",
            SuggestionType::Field,
            Some(JsonFieldType::Number),
        ),
        Suggestion::new_with_type(
            "veryLongFieldNameThatShouldBeTruncated",
            SuggestionType::Field,
            Some(JsonFieldType::Boolean),
        ),
        Suggestion::new_with_type(
            "anotherVeryLongFieldNameHere",
            SuggestionType::Field,
            Some(JsonFieldType::Array),
        ),
        Suggestion::new_with_type("short", SuggestionType::Field, Some(JsonFieldType::Object)),
    ];
    app.autocomplete.update_suggestions(suggestions);

    let mut terminal = create_test_terminal(80, 20);
    let input_area = Rect::new(0, 12, 80, 3);

    terminal
        .draw(|f| {
            let _ = render_popup(&app, f, input_area);
        })
        .unwrap();

    assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn snapshot_truncated_field_names_with_fixed_type_labels() {
    use crate::autocomplete::JsonFieldType;

    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    let suggestions = vec![
        Suggestion::new_with_type(
            "thisIsAnExtremelyLongFieldNameThatDefinitelyExceedsTheMaxWidth",
            SuggestionType::Field,
            Some(JsonFieldType::String),
        ),
        Suggestion::new_with_type(
            "anotherVeryVeryVeryLongFieldNameThatWillBeTruncated",
            SuggestionType::Field,
            Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Object))),
        ),
        Suggestion::new_with_type("short", SuggestionType::Field, Some(JsonFieldType::Number)),
    ];
    app.autocomplete.update_suggestions(suggestions);

    let mut terminal = create_test_terminal(80, 20);
    let input_area = Rect::new(0, 12, 80, 3);

    terminal
        .draw(|f| {
            let _ = render_popup(&app, f, input_area);
        })
        .unwrap();

    assert_snapshot!(terminal.backend().to_string());
}

// =========================================================================
// Scrollbar Position Tests - verify scrollbar reaches correct positions
// =========================================================================

fn render_autocomplete_scrollbar_test(navigate_down: usize) -> String {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    let suggestions: Vec<Suggestion> = (0..20)
        .map(|i| Suggestion::new(format!(".field{:02}", i), SuggestionType::Field))
        .collect();
    app.autocomplete.update_suggestions(suggestions);

    for _ in 0..navigate_down {
        app.autocomplete.select_next();
    }

    let mut terminal = create_test_terminal(80, 20);
    // Position input area lower so popup appears in visible area above it
    let input_area = Rect::new(0, 15, 80, 3);

    terminal
        .draw(|f| {
            let _ = render_popup(&app, f, input_area);
        })
        .unwrap();

    terminal.backend().to_string()
}

#[test]
fn snapshot_autocomplete_scrollbar_at_top() {
    // Default scroll position is at the top
    let output = render_autocomplete_scrollbar_test(0);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_scrollbar_at_middle() {
    // Navigate to the middle to scroll
    let output = render_autocomplete_scrollbar_test(10);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_scrollbar_at_bottom() {
    // Navigate to the last suggestion to scroll to the bottom
    let output = render_autocomplete_scrollbar_test(19);
    assert_snapshot!(output);
}
