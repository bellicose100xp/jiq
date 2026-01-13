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
        .draw(|f| render_popup(&app, f, input_area))
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
