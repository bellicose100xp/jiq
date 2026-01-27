//! Tests for help_line_render

use super::*;
use crate::app::Focus;
use crate::editor::EditorMode;
use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn render_help_line_to_string(app: &App, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    terminal
        .draw(|f| {
            let area = f.area();
            render_line(app, f, area);
        })
        .unwrap();
    terminal.backend().to_string()
}

#[test]
fn snapshot_help_line_insert_mode_empty_query() {
    let mut app = test_app("{}");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    let output = render_help_line_to_string(&app, 120, 1);
    assert_snapshot!(output);
}

#[test]
fn snapshot_help_line_insert_mode_with_query() {
    let mut app = test_app("{}");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;
    app.input.textarea.insert_str(".foo");

    if let Some(query_state) = &mut app.query {
        query_state.execute(".foo");
    }

    let output = render_help_line_to_string(&app, 120, 1);
    assert_snapshot!(output);
}

#[test]
fn snapshot_help_line_normal_mode() {
    let mut app = test_app("{}");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Normal;

    let output = render_help_line_to_string(&app, 120, 1);
    assert_snapshot!(output);
}

#[test]
fn snapshot_help_line_results_pane() {
    let mut app = test_app("{}");
    app.focus = Focus::ResultsPane;

    let output = render_help_line_to_string(&app, 120, 1);
    assert_snapshot!(output);
}

#[test]
fn test_help_text_contains_snippets_shortcut_insert_empty() {
    let mut app = test_app("{}");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;

    let output = render_help_line_to_string(&app, 120, 1);
    assert!(output.contains("Ctrl+S") && output.contains("Snippets"));
}

#[test]
fn test_help_text_contains_snippets_shortcut_insert_with_query() {
    let mut app = test_app("{}");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Insert;
    app.input.textarea.insert_str(".foo");

    if let Some(query_state) = &mut app.query {
        query_state.execute(".foo");
    }

    let output = render_help_line_to_string(&app, 120, 1);
    assert!(output.contains("Ctrl+S") && output.contains("Snippets"));
}

#[test]
fn test_help_text_contains_snippets_shortcut_normal_mode() {
    let mut app = test_app("{}");
    app.focus = Focus::InputField;
    app.input.editor_mode = EditorMode::Normal;

    let output = render_help_line_to_string(&app, 120, 1);
    assert!(output.contains("Ctrl+S") && output.contains("Snippets"));
}

#[test]
fn snapshot_help_line_search_unconfirmed() {
    let mut app = test_app("{}");
    app.search.open();

    let output = render_help_line_to_string(&app, 120, 1);
    assert_snapshot!(output);
}

#[test]
fn snapshot_help_line_search_confirmed() {
    let mut app = test_app("{}");
    app.search.open();
    app.search.confirm();

    let output = render_help_line_to_string(&app, 120, 1);
    assert_snapshot!(output);
}

#[test]
fn snapshot_help_line_snippet_manager() {
    let mut app = test_app("{}");
    app.snippets.open();

    let output = render_help_line_to_string(&app, 120, 1);
    assert_snapshot!(output);
}

#[test]
fn test_help_text_excludes_snippets_shortcut_when_search_active() {
    let mut app = test_app("{}");
    app.search.open();

    let output = render_help_line_to_string(&app, 120, 1);
    assert!(!output.contains("Ctrl+S"));
    assert!(output.contains("Esc") && output.contains("Close"));
}

#[test]
fn test_help_text_excludes_snippets_shortcut_when_snippet_manager_active() {
    let mut app = test_app("{}");
    app.snippets.open();

    let output = render_help_line_to_string(&app, 120, 1);
    assert!(!output.contains("Ctrl+S"));
    assert!(output.contains("Esc") && output.contains("Close"));
}
