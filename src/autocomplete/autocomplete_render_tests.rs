//! Tests for autocomplete popup rendering

use crate::autocomplete::{JsonFieldType, Suggestion, SuggestionType};
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

const TEST_WIDTH: u16 = 80;
const TEST_HEIGHT: u16 = 15;

fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn render_autocomplete_popup(app: &crate::app::App, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    terminal
        .draw(|f| {
            let input_area = Rect::new(0, height - 3, width, 3);
            super::render_popup(app, f, input_area);
        })
        .unwrap();
    terminal.backend().to_string()
}

#[test]
fn snapshot_autocomplete_popup_empty() {
    let json = r#"{"name": "test"}"#;
    let app = test_app(json);

    let output = render_autocomplete_popup(&app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_popup_with_functions() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);

    app.autocomplete.update_suggestions(vec![
        Suggestion::new("map", SuggestionType::Function)
            .with_signature("map(f)")
            .with_description("Transform elements"),
        Suggestion::new("select", SuggestionType::Function)
            .with_signature("select(expr)")
            .with_description("Filter elements"),
        Suggestion::new("keys", SuggestionType::Function).with_description("Get object keys"),
    ]);

    let output = render_autocomplete_popup(&app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_popup_with_fields() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);

    app.autocomplete.update_suggestions(vec![
        Suggestion::new_with_type("name", SuggestionType::Field, Some(JsonFieldType::String)),
        Suggestion::new_with_type("age", SuggestionType::Field, Some(JsonFieldType::Number)),
        Suggestion::new_with_type(
            "active",
            SuggestionType::Field,
            Some(JsonFieldType::Boolean),
        ),
    ]);

    let output = render_autocomplete_popup(&app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_popup_with_operators() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);

    app.autocomplete.update_suggestions(vec![
        Suggestion::new("//", SuggestionType::Operator).with_description("Alternative operator"),
        Suggestion::new("|=", SuggestionType::Operator).with_description("Update operator"),
    ]);

    let output = render_autocomplete_popup(&app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_popup_with_patterns() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);

    app.autocomplete.update_suggestions(vec![
        Suggestion::new("[]", SuggestionType::Pattern).with_description("Array iterator"),
        Suggestion::new(".[]", SuggestionType::Pattern).with_description("Array/object iterator"),
    ]);

    let output = render_autocomplete_popup(&app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_popup_with_selection() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);

    app.autocomplete.update_suggestions(vec![
        Suggestion::new("first", SuggestionType::Function),
        Suggestion::new("second", SuggestionType::Function),
        Suggestion::new("third", SuggestionType::Function),
    ]);
    app.autocomplete.select_next();

    let output = render_autocomplete_popup(&app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_popup_mixed_types() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);

    app.autocomplete.update_suggestions(vec![
        Suggestion::new_with_type("name", SuggestionType::Field, Some(JsonFieldType::String)),
        Suggestion::new("map", SuggestionType::Function).with_signature("map(f)"),
        Suggestion::new("//", SuggestionType::Operator),
        Suggestion::new("[]", SuggestionType::Pattern),
    ]);

    let output = render_autocomplete_popup(&app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_popup_narrow() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);

    app.autocomplete.update_suggestions(vec![
        Suggestion::new("map", SuggestionType::Function).with_signature("map(f)"),
        Suggestion::new("select", SuggestionType::Function).with_signature("select(expr)"),
    ]);

    let output = render_autocomplete_popup(&app, 40, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_popup_array_field_type() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);

    app.autocomplete
        .update_suggestions(vec![Suggestion::new_with_type(
            "items",
            SuggestionType::Field,
            Some(JsonFieldType::ArrayOf(Box::new(JsonFieldType::Object))),
        )]);

    let output = render_autocomplete_popup(&app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}
