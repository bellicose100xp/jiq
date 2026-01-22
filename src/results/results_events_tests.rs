//! Tests for results_events

use super::*;
use crate::app::Focus;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods, test_app};
use std::sync::Arc;

#[test]
fn test_j_scrolls_down_one_line() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    let content: String = (0..20).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 10);
    app.results_scroll.offset = 0;

    app.handle_key_event(key(KeyCode::Char('j')));

    assert_eq!(app.results_scroll.offset, 1);
}

#[test]
fn test_k_scrolls_up_one_line() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 5;

    app.handle_key_event(key(KeyCode::Char('k')));

    assert_eq!(app.results_scroll.offset, 4);
}

#[test]
fn test_k_at_top_stays_at_zero() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 0;

    app.handle_key_event(key(KeyCode::Char('k')));

    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_capital_j_scrolls_down_ten_lines() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    let content: String = (0..30).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 10);
    app.results_scroll.offset = 5;

    app.handle_key_event(key(KeyCode::Char('J')));

    assert_eq!(app.results_scroll.offset, 15);
}

#[test]
fn test_capital_k_scrolls_up_ten_lines() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 20;

    app.handle_key_event(key(KeyCode::Char('K')));

    assert_eq!(app.results_scroll.offset, 10);
}

#[test]
fn test_g_jumps_to_top() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 50;

    app.handle_key_event(key(KeyCode::Char('g')));

    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_capital_g_jumps_to_bottom() {
    let json = r#"{"line1": 1, "line2": 2, "line3": 3}"#;
    let mut app = test_app(json);
    app.input.textarea.insert_str(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 0;
    app.results_scroll.viewport_height = 2;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 2);
    let max_scroll = app.results_scroll.max_offset;

    app.handle_key_event(key(KeyCode::Char('G')));

    assert_eq!(app.results_scroll.offset, max_scroll);
}

#[test]
fn test_page_up_scrolls_half_page() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 20;
    app.results_scroll.viewport_height = 20;

    app.handle_key_event(key(KeyCode::PageUp));

    assert_eq!(app.results_scroll.offset, 10);
}

#[test]
fn test_page_down_scrolls_half_page() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 20);
    app.results_scroll.offset = 0;

    app.handle_key_event(key(KeyCode::PageDown));

    assert_eq!(app.results_scroll.offset, 10);
}

#[test]
fn test_ctrl_u_scrolls_half_page_up() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 20;
    app.results_scroll.viewport_height = 20;

    app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));

    assert_eq!(app.results_scroll.offset, 10);
}

#[test]
fn test_ctrl_d_scrolls_half_page_down() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 20);
    app.results_scroll.offset = 0;

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    assert_eq!(app.results_scroll.offset, 10);
}

#[test]
fn test_up_arrow_scrolls_in_results_pane() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 5;

    app.handle_key_event(key(KeyCode::Up));

    assert_eq!(app.results_scroll.offset, 4);
}

#[test]
fn test_down_arrow_scrolls_in_results_pane() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    let content: String = (0..20).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 10);
    app.results_scroll.offset = 0;

    app.handle_key_event(key(KeyCode::Down));

    assert_eq!(app.results_scroll.offset, 1);
}

#[test]
fn test_home_jumps_to_top() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.results_scroll.offset = 50;

    app.handle_key_event(key(KeyCode::Home));

    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_scroll_clamped_to_max() {
    let mut app = app_with_query("");
    app.focus = Focus::ResultsPane;

    let content = "line1\nline2\nline3".to_string();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 10);

    assert_eq!(app.results_scroll.max_offset, 0);

    app.handle_key_event(key(KeyCode::Char('j')));
    assert_eq!(app.results_scroll.offset, 0);

    for _ in 0..100 {
        app.handle_key_event(key(KeyCode::Char('j')));
    }
    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_scroll_clamped_with_content() {
    let mut app = app_with_query("");
    app.focus = Focus::ResultsPane;

    let content: String = (0..20).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 10);

    assert_eq!(app.results_scroll.max_offset, 10);

    for _ in 0..100 {
        app.handle_key_event(key(KeyCode::Char('j')));
    }

    assert_eq!(app.results_scroll.offset, 10);
}

#[test]
fn test_scroll_page_down_clamped() {
    let mut app = app_with_query("");
    app.focus = Focus::ResultsPane;

    let content: String = (0..15).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 10);

    assert_eq!(app.results_scroll.max_offset, 5);

    app.handle_key_event(key(KeyCode::PageDown));
    assert_eq!(app.results_scroll.offset, 5);

    app.handle_key_event(key(KeyCode::PageDown));
    assert_eq!(app.results_scroll.offset, 5);
}

#[test]
fn test_scroll_j_clamped() {
    let mut app = app_with_query("");
    app.focus = Focus::ResultsPane;

    let content: String = (0..5).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    app.results_scroll.update_bounds(line_count, 3);

    assert_eq!(app.results_scroll.max_offset, 2);

    app.handle_key_event(key(KeyCode::Char('J')));
    assert_eq!(app.results_scroll.offset, 2);
}

#[test]
fn test_question_mark_toggles_help_in_results_pane() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key(KeyCode::Char('?')));
    assert!(app.help.visible);
}

fn app_with_wide_content() -> App {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    let content: String = (0..10)
        .map(|i| format!("{}{}\n", i, "x".repeat(100)))
        .collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;
    query_state.cached_max_line_width = content.lines().map(|l| l.len()).max().unwrap_or(0) as u16;
    app.results_scroll.update_h_bounds(101, 40);
    app
}

#[test]
fn test_h_scrolls_left_one_column() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 10;

    app.handle_key_event(key(KeyCode::Char('h')));

    assert_eq!(app.results_scroll.h_offset, 9);
}

#[test]
fn test_l_scrolls_right_one_column() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Char('l')));

    assert_eq!(app.results_scroll.h_offset, 1);
}

#[test]
fn test_left_arrow_scrolls_left() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 10;

    app.handle_key_event(key(KeyCode::Left));

    assert_eq!(app.results_scroll.h_offset, 9);
}

#[test]
fn test_right_arrow_scrolls_right() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Right));

    assert_eq!(app.results_scroll.h_offset, 1);
}

#[test]
fn test_capital_h_scrolls_left_ten_columns() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 30;

    app.handle_key_event(key(KeyCode::Char('H')));

    assert_eq!(app.results_scroll.h_offset, 20);
}

#[test]
fn test_capital_l_scrolls_right_ten_columns() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Char('L')));

    assert_eq!(app.results_scroll.h_offset, 10);
}

#[test]
fn test_zero_jumps_to_left_edge() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 50;

    app.handle_key_event(key(KeyCode::Char('0')));

    assert_eq!(app.results_scroll.h_offset, 0);
}

#[test]
fn test_caret_jumps_to_left_edge() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 50;

    app.handle_key_event(key(KeyCode::Char('^')));

    assert_eq!(app.results_scroll.h_offset, 0);
}

#[test]
fn test_dollar_jumps_to_right_edge() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Char('$')));

    assert_eq!(app.results_scroll.h_offset, 61);
}

#[test]
fn test_h_scroll_left_clamped_at_zero() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 0;

    app.handle_key_event(key(KeyCode::Char('h')));

    assert_eq!(app.results_scroll.h_offset, 0);
}

#[test]
fn test_l_scroll_right_clamped_at_max() {
    let mut app = app_with_wide_content();
    app.results_scroll.h_offset = 61;

    app.handle_key_event(key(KeyCode::Char('l')));

    assert_eq!(app.results_scroll.h_offset, 61);
}

#[test]
fn test_end_jumps_to_bottom() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    let content: String = (0..20).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(content.clone());
    query_state.last_successful_result = Some(Arc::new(content.clone()));
    query_state.cached_line_count = content.lines().count() as u32;
    app.results_scroll.update_bounds(20, 10);
    app.results_scroll.offset = 0;

    app.handle_key_event(key(KeyCode::End));

    assert_eq!(app.results_scroll.offset, 10);
}

#[test]
fn test_tab_switches_focus_to_input_field() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key(KeyCode::Tab));

    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_tab_with_ctrl_does_not_switch_focus() {
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;

    app.handle_key_event(key_with_mods(KeyCode::Tab, KeyModifiers::CONTROL));

    assert_eq!(app.focus, Focus::ResultsPane);
}

#[test]
fn test_i_key_switches_to_input_field_in_insert_mode() {
    use crate::editor::EditorMode;
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Normal;

    app.handle_key_event(key(KeyCode::Char('i')));

    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.input.editor_mode, EditorMode::Insert);
}

#[test]
fn test_i_key_switches_to_insert_mode_even_if_already_in_insert() {
    use crate::editor::EditorMode;
    let mut app = app_with_query(".");
    app.focus = Focus::ResultsPane;
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key(KeyCode::Char('i')));

    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.input.editor_mode, EditorMode::Insert);
}
