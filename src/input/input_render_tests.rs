//! Tests for input rendering

use crate::app::Focus;
use crate::editor::EditorMode;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

const TEST_WIDTH: u16 = 80;
const TEST_HEIGHT: u16 = 3;

fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn render_input_field(app: &mut crate::app::App, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    terminal
        .draw(|f| {
            let area = f.area();
            super::render_field(app, f, area);
        })
        .unwrap();
    terminal.backend().to_string()
}

#[test]
fn snapshot_input_field_empty_insert_mode() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    let output = render_input_field(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_input_field_empty_normal_mode() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.input.editor_mode = EditorMode::Normal;
    app.focus = Focus::InputField;

    let output = render_input_field(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_input_field_with_query() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.input.editor_mode = EditorMode::Insert;
    app.focus = Focus::InputField;

    let output = render_input_field(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_input_field_operator_mode() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.input.editor_mode = EditorMode::Operator('d');
    app.focus = Focus::InputField;

    let output = render_input_field(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_input_field_unfocused() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.focus = Focus::ResultsPane;

    let output = render_input_field(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_input_field_with_syntax_error() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".invalid[");
    app.query.as_mut().unwrap().execute(".invalid[");
    app.focus = Focus::InputField;

    let output = render_input_field(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_input_field_narrow() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.focus = Focus::InputField;

    let output = render_input_field(&mut app, 40, TEST_HEIGHT);
    assert_snapshot!(output);
}
