//! Results pane event handling
//!
//! This module handles keyboard events when the results pane is focused.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::clipboard;

/// Handle keys when Results pane is focused
pub fn handle_results_pane_key(app: &mut App, key: KeyEvent) {
    match key.code {
        // Toggle help popup
        KeyCode::Char('?') => {
            app.help.visible = !app.help.visible;
        }

        // Yank (copy) result to clipboard
        KeyCode::Char('y') => {
            // yy command - copy result to clipboard
            // Note: Ctrl+Y is handled globally in events.rs before this
            clipboard::events::handle_yank_key(app, app.clipboard_backend);
        }

        // Basic line scrolling (1 line)
        KeyCode::Up | KeyCode::Char('k') => {
            app.results_scroll.scroll_up(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.results_scroll.scroll_down(1);
        }

        // 10 line scrolling
        KeyCode::Char('K') => {
            app.results_scroll.scroll_up(10);
        }
        KeyCode::Char('J') => {
            app.results_scroll.scroll_down(10);
        }

        // Horizontal scrolling (1 column)
        KeyCode::Left | KeyCode::Char('h') => {
            app.results_scroll.scroll_left(1);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.results_scroll.scroll_right(1);
        }

        // Horizontal scrolling (10 columns)
        KeyCode::Char('H') => {
            app.results_scroll.scroll_left(10);
        }
        KeyCode::Char('L') => {
            app.results_scroll.scroll_right(10);
        }


        // Jump to left edge
        KeyCode::Char('0') | KeyCode::Char('^') => {
            app.results_scroll.jump_to_left();
        }

        // Jump to right edge
        KeyCode::Char('$') => {
            app.results_scroll.jump_to_right();
        }

        // Jump to top
        KeyCode::Home | KeyCode::Char('g') => {
            app.results_scroll.jump_to_top();
        }

        // Jump to bottom
        KeyCode::End | KeyCode::Char('G') => {
            app.results_scroll.jump_to_bottom();
        }

        // Half page scrolling
        KeyCode::PageUp | KeyCode::Char('u') if key.code == KeyCode::PageUp || key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.results_scroll.page_up();
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.code == KeyCode::PageDown || key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.results_scroll.page_down();
        }

        _ => {
            // Ignore other keys in Results pane
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, Focus};
    use crate::config::ClipboardBackend;
    use crate::history::HistoryState;

    // Test fixture data
    const TEST_JSON: &str = r#"{"name": "test", "age": 30, "city": "NYC"}"#;

    /// Helper to create App with default clipboard backend for tests
    fn test_app(json: &str) -> App {
        App::new(json.to_string(), ClipboardBackend::Auto)
    }

    // Helper to create a KeyEvent without modifiers
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    // Helper to create a KeyEvent with specific modifiers
    fn key_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    // Helper to set up an app with text in the query field
    fn app_with_query(query: &str) -> App {
        let mut app = test_app(TEST_JSON);
        app.input.textarea.insert_str(query);
        // Use empty in-memory history for all tests to prevent disk writes
        app.history = HistoryState::empty();
        app
    }


    // ========== Results Scrolling Tests ==========

    #[test]
    fn test_j_scrolls_down_one_line() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;

        // Set up content with enough lines for scrolling
        let content: String = (0..20).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);

        // Set up bounds so scrolling works
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

        // Should saturate at 0, not go negative
        assert_eq!(app.results_scroll.offset, 0);
    }

    #[test]
    fn test_capital_j_scrolls_down_ten_lines() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;

        // Set up content with 30 lines so max_offset = 30 - 10 = 20
        let content: String = (0..30).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);

        // Update bounds and set initial scroll
        let line_count = app.results_line_count_u32();
        app.results_scroll.update_bounds(line_count, 10);
        app.results_scroll.offset = 5;

        app.handle_key_event(key(KeyCode::Char('J')));

        // Should scroll from 5 to 15 (10 lines down, within max_offset of 20)
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
        app.results_scroll.viewport_height = 2; // Small viewport to ensure max_offset > 0

        // Update bounds to calculate max_offset
        let line_count = app.results_line_count_u32();
        app.results_scroll.update_bounds(line_count, 2);
        let max_scroll = app.results_scroll.max_offset;

        app.handle_key_event(key(KeyCode::Char('G')));

        // Should jump to max_offset position
        assert_eq!(app.results_scroll.offset, max_scroll);
    }

    #[test]
    fn test_page_up_scrolls_half_page() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll.offset = 20;
        app.results_scroll.viewport_height = 20;

        app.handle_key_event(key(KeyCode::PageUp));

        // Should scroll up by half viewport (10 lines)
        assert_eq!(app.results_scroll.offset, 10);
    }

    #[test]
    fn test_page_down_scrolls_half_page() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;

        // Set up content with 50 lines so max_offset = 50 - 20 = 30
        let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);

        // Update bounds
        let line_count = app.results_line_count_u32();
        app.results_scroll.update_bounds(line_count, 20);
        app.results_scroll.offset = 0;

        app.handle_key_event(key(KeyCode::PageDown));

        // Should scroll down by half viewport (10 lines), within max_offset of 30
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

        // Set up content with 50 lines so max_offset = 50 - 20 = 30
        let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);

        // Update bounds
        let line_count = app.results_line_count_u32();
        app.results_scroll.update_bounds(line_count, 20);
        app.results_scroll.offset = 0;

        app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

        // Should scroll down by half viewport (10 lines), within max_offset of 30
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

        // Set up content with enough lines for scrolling
        let content: String = (0..20).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);

        // Set up bounds so scrolling works
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


    // ========== Scroll clamping tests ==========

    #[test]
    fn test_scroll_clamped_to_max() {
        let mut app = app_with_query("");
        app.focus = Focus::ResultsPane;

        // Set up a short content with few lines
        app.query.result = Ok("line1\nline2\nline3".to_string());

        // Update bounds - viewport larger than content
        let line_count = app.results_line_count_u32();
        app.results_scroll.update_bounds(line_count, 10);

        // max_offset should be 0 since content fits in viewport
        assert_eq!(app.results_scroll.max_offset, 0);

        // Try to scroll down - should stay at 0
        app.handle_key_event(key(KeyCode::Char('j')));
        assert_eq!(app.results_scroll.offset, 0);

        // Try to scroll down multiple times - should stay at 0
        for _ in 0..100 {
            app.handle_key_event(key(KeyCode::Char('j')));
        }
        assert_eq!(app.results_scroll.offset, 0);
    }

    #[test]
    fn test_scroll_clamped_with_content() {
        let mut app = app_with_query("");
        app.focus = Focus::ResultsPane;

        // Set up content with 20 lines
        let content: String = (0..20).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);

        // Update bounds
        let line_count = app.results_line_count_u32();
        app.results_scroll.update_bounds(line_count, 10);

        // max_offset should be 20 - 10 = 10
        assert_eq!(app.results_scroll.max_offset, 10);

        // Scroll down many times
        for _ in 0..100 {
            app.handle_key_event(key(KeyCode::Char('j')));
        }

        // Should be clamped to max_offset
        assert_eq!(app.results_scroll.offset, 10);
    }

    #[test]
    fn test_scroll_page_down_clamped() {
        let mut app = app_with_query("");
        app.focus = Focus::ResultsPane;

        // 15 lines content, 10 line viewport
        let content: String = (0..15).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);

        // Update bounds
        let line_count = app.results_line_count_u32();
        app.results_scroll.update_bounds(line_count, 10);

        // max_offset = 15 - 10 = 5
        assert_eq!(app.results_scroll.max_offset, 5);

        // Page down (half page = 5) should go to max
        app.handle_key_event(key(KeyCode::PageDown));
        assert_eq!(app.results_scroll.offset, 5);

        // Another page down should stay at max
        app.handle_key_event(key(KeyCode::PageDown));
        assert_eq!(app.results_scroll.offset, 5);
    }

    #[test]
    fn test_scroll_j_clamped() {
        let mut app = app_with_query("");
        app.focus = Focus::ResultsPane;

        // 5 lines content, 3 line viewport
        let content: String = (0..5).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);

        // Update bounds
        let line_count = app.results_line_count_u32();
        app.results_scroll.update_bounds(line_count, 3);

        // max_offset = 5 - 3 = 2
        assert_eq!(app.results_scroll.max_offset, 2);

        // Big scroll (J = 10 lines) should clamp to max
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

    // ========== Horizontal Scroll Tests ==========

    fn app_with_wide_content() -> App {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        // Content with long lines (100 chars each)
        let content: String = (0..10)
            .map(|i| format!("{}{}\n", i, "x".repeat(100)))
            .collect();
        app.query.result = Ok(content);
        app.results_scroll.update_h_bounds(101, 40); // max_h_offset = 61
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

        assert_eq!(app.results_scroll.h_offset, 61); // max_h_offset
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
        app.results_scroll.h_offset = 61; // at max

        app.handle_key_event(key(KeyCode::Char('l')));

        assert_eq!(app.results_scroll.h_offset, 61);
    }

    #[test]
    fn test_end_jumps_to_bottom() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;

        let content: String = (0..20).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(content);
        app.results_scroll.update_bounds(20, 10);
        app.results_scroll.offset = 0;

        app.handle_key_event(key(KeyCode::End));

        assert_eq!(app.results_scroll.offset, 10); // max_offset
    }
}
