//! Tests for search bar rendering

use crate::test_utils::test_helpers::test_app;
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use super::SEARCH_BAR_HEIGHT;

const TEST_WIDTH: u16 = 80;

fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

fn render_search_bar(app: &mut crate::app::App, width: u16) -> String {
    let mut terminal = create_test_terminal(width, SEARCH_BAR_HEIGHT);
    terminal
        .draw(|f| {
            let area = f.area();
            super::render_bar(app, f, area);
        })
        .unwrap();
    terminal.backend().to_string()
}

#[test]
fn test_search_bar_height_constant() {
    assert_eq!(SEARCH_BAR_HEIGHT, 3);
}

#[test]
fn snapshot_search_bar_empty() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.search.open();

    let output = render_search_bar(&mut app, TEST_WIDTH);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_bar_with_query() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.search.open();
    app.search.search_textarea_mut().insert_str("test");

    let output = render_search_bar(&mut app, TEST_WIDTH);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_bar_with_no_matches() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.search.open();
    app.search.search_textarea_mut().insert_str("xyz");

    let output = render_search_bar(&mut app, TEST_WIDTH);
    assert_snapshot!(output);
}

#[test]
fn snapshot_search_bar_narrow() {
    let json = r#"{"name": "test"}"#;
    let mut app = test_app(json);
    app.search.open();
    app.search.search_textarea_mut().insert_str("test");

    let output = render_search_bar(&mut app, 40);
    assert_snapshot!(output);
}
