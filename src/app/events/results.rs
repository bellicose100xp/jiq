use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::state::App;

/// Handle keys when Results pane is focused
pub fn handle_results_pane_key(app: &mut App, key: KeyEvent) {
    match key.code {
        // Toggle help popup
        KeyCode::Char('?') => {
            app.help.visible = !app.help.visible;
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

        // Jump to top
        KeyCode::Home | KeyCode::Char('g') => {
            app.results_scroll.jump_to_top();
        }

        // Jump to bottom
        KeyCode::Char('G') => {
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
    use crate::app::state::{App, Focus};
    use crate::history::HistoryState;

    // Test fixture data
    const TEST_JSON: &str = r#"{"name": "test", "age": 30, "city": "NYC"}"#;

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
        let mut app = App::new(TEST_JSON.to_string());
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
        let mut app = App::new(json.to_string());
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
}
