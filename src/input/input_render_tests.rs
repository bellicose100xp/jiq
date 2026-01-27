use crate::app::Focus;
use crate::editor::EditorMode;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn render_to_string(app: &mut crate::app::App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| app.render(f)).unwrap();
    terminal.backend().to_string()
}

const TEST_WIDTH: u16 = 80;
const TEST_HEIGHT: u16 = 24;

#[test]
fn snapshot_query_focused_insert_mode() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_query_focused_normal_mode() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Normal;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_query_unfocused_insert_mode() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Insert;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_query_unfocused_normal_mode() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".name");
    app.query.as_mut().unwrap().execute(".name");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Normal;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_query_unfocused_empty() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);
    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_query_unfocused_long_query() {
    let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;
    let mut app = test_app(json);
    app.input
        .textarea
        .insert_str(".users | map(select(.name == \"Alice\")) | .[0].name");
    app.query
        .as_mut()
        .unwrap()
        .execute(".users | map(select(.name == \"Alice\")) | .[0].name");
    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_query_unfocused_with_syntax_error() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".invalid[");
    app.query.as_mut().unwrap().execute(".invalid[");
    app.focus = Focus::ResultsPane;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}

#[test]
fn snapshot_query_focused_with_syntax_error() {
    let json = r#"{"name": "Alice"}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".invalid[");
    app.query.as_mut().unwrap().execute(".invalid[");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    let output = render_to_string(&mut app, TEST_WIDTH, TEST_HEIGHT);
    assert_snapshot!(output);
}
