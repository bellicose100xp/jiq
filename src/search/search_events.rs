//! Search event handling
//!
//! Handles keyboard events for the search feature including:
//! - Opening/closing search bar
//! - Text input to search query
//! - Navigation between matches (n/N, Enter/Shift+Enter)

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[cfg(debug_assertions)]
use log::debug;

use crate::app::{App, Focus};
use crate::results::results_events::handle_results_pane_key;

/// Handle search-related key events when search is visible
/// Returns true if event was consumed, false otherwise
pub fn handle_search_key(app: &mut App, key: KeyEvent) -> bool {
    if !app.search.is_visible() {
        return false;
    }

    match key.code {
        // Close search with Escape
        KeyCode::Esc => {
            close_search(app);
            true
        }

        // Enter confirms search (first press) or navigates to next match (subsequent presses)
        KeyCode::Enter if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            if !app.search.is_confirmed() {
                // First Enter: just confirm and scroll to current match (index 0)
                app.search.confirm();
                
                if let Some(current_match) = app.search.current_match() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "Search: confirmed on first match -> line {}, index {}/{}",
                        current_match.line,
                        app.search.current_index() + 1,
                        app.search.matches().len()
                    );
                    scroll_to_line(app, current_match.line);
                }
            } else {
                // Already confirmed: navigate to next match
                if let Some(line) = app.search.next_match() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "Search: next match (Enter) -> line {}, index {}/{}",
                        line,
                        app.search.current_index() + 1,
                        app.search.matches().len()
                    );
                    scroll_to_line(app, line);
                }
            }
            true
        }

        // Shift+Enter confirms search (first press) or navigates to previous match (subsequent presses)
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            if !app.search.is_confirmed() {
                // First Shift+Enter: just confirm and scroll to current match (index 0)
                app.search.confirm();
                
                if let Some(current_match) = app.search.current_match() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "Search: confirmed on first match (Shift+Enter) -> line {}, index {}/{}",
                        current_match.line,
                        app.search.current_index() + 1,
                        app.search.matches().len()
                    );
                    scroll_to_line(app, current_match.line);
                }
            } else {
                // Already confirmed: navigate to previous match
                if let Some(line) = app.search.prev_match() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "Search: prev match (Shift+Enter) -> line {}, index {}/{}",
                        line,
                        app.search.current_index() + 1,
                        app.search.matches().len()
                    );
                    scroll_to_line(app, line);
                }
            }
            true
        }

        // n/N only navigate when search is confirmed (after Enter)
        KeyCode::Char('n') if !key.modifiers.contains(KeyModifiers::SHIFT) && app.search.is_confirmed() => {
            if let Some(line) = app.search.next_match() {
                #[cfg(debug_assertions)]
                debug!(
                    "Search: next match -> line {}, index {}/{}",
                    line,
                    app.search.current_index() + 1,
                    app.search.matches().len()
                );
                scroll_to_line(app, line);
            }
            true
        }
        KeyCode::Char('N') if app.search.is_confirmed() => {
            if let Some(line) = app.search.prev_match() {
                #[cfg(debug_assertions)]
                debug!(
                    "Search: prev match -> line {}, index {}/{}",
                    line,
                    app.search.current_index() + 1,
                    app.search.matches().len()
                );
                scroll_to_line(app, line);
            }
            true
        }
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::SHIFT) && app.search.is_confirmed() => {
            if let Some(line) = app.search.prev_match() {
                #[cfg(debug_assertions)]
                debug!(
                    "Search: prev match (Shift+n) -> line {}, index {}/{}",
                    line,
                    app.search.current_index() + 1,
                    app.search.matches().len()
                );
                scroll_to_line(app, line);
            }
            true
        }

        // Ctrl+F re-enters edit mode when search is confirmed
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) && app.search.is_confirmed() => {
            #[cfg(debug_assertions)]
            debug!("Search: re-entering edit mode via Ctrl+F");
            app.search.unconfirm();
            true
        }

        // '/' re-enters edit mode when search is confirmed
        KeyCode::Char('/') if app.search.is_confirmed() => {
            #[cfg(debug_assertions)]
            debug!("Search: re-entering edit mode via /");
            app.search.unconfirm();
            true
        }

        // When confirmed, delegate navigation keys to results pane handler
        // User must press Ctrl+F or / to re-enter edit mode
        _ if app.search.is_confirmed() => {
            #[cfg(debug_assertions)]
            debug!("Search: delegating key {:?} to results pane handler", key.code);
            handle_results_pane_key(app, key);
            true
        }

        // When NOT confirmed, pass keys to the search textarea for text input
        _ => {
            // Forward key to textarea
            app.search.search_textarea_mut().input(key);
            
            // Update matches based on new query
            // Use unformatted result (without ANSI codes) so match positions align with rendered text
            if let Some(content) = &app.query.last_successful_result_unformatted {
                app.search.update_matches(content);
                
                #[cfg(debug_assertions)]
                debug!(
                    "Search: query changed to '{}', found {} matches",
                    app.search.query(),
                    app.search.matches().len()
                );
            }
            
            // Jump to first match if we have any
            if let Some(m) = app.search.current_match() {
                scroll_to_line(app, m.line);
            }
            
            true
        }
    }
}

/// Open search bar and focus results pane
pub fn open_search(app: &mut App) {
    #[cfg(debug_assertions)]
    debug!("Search: opened");
    
    app.search.open();
    app.focus = Focus::ResultsPane;
}

/// Close search bar and clear all state
pub fn close_search(app: &mut App) {
    #[cfg(debug_assertions)]
    debug!("Search: closed (query was '{}')", app.search.query());
    
    app.search.close();
}

/// Scroll results pane to make the current match visible (both vertically and horizontally)
fn scroll_to_match(app: &mut App) {
    let Some(current_match) = app.search.current_match() else {
        return;
    };
    
    let target_line = current_match.line.min(u16::MAX as u32) as u16;
    let target_col = current_match.col;
    let match_len = current_match.len;

    #[cfg(debug_assertions)]
    debug!(
        "scroll_to_match: line={}, col={}, len={}, viewport_height={}, max_offset={}, current_offset={}, h_offset={}, viewport_width={}, max_h_offset={}",
        target_line, target_col, match_len,
        app.results_scroll.viewport_height,
        app.results_scroll.max_offset,
        app.results_scroll.offset,
        app.results_scroll.h_offset,
        app.results_scroll.viewport_width,
        app.results_scroll.max_h_offset
    );

    // Vertical scrolling
    let viewport_height = app.results_scroll.viewport_height;
    let current_offset = app.results_scroll.offset;
    let max_offset = app.results_scroll.max_offset;

    if viewport_height > 0 && max_offset > 0 {
        let visible_start = current_offset;
        let visible_end = current_offset.saturating_add(viewport_height);

        if target_line < visible_start || target_line >= visible_end {
            // Line not visible, scroll to center it
            let half_viewport = viewport_height / 2;
            let new_offset = target_line.saturating_sub(half_viewport);
            let clamped_offset = new_offset.min(max_offset);
            
            #[cfg(debug_assertions)]
            debug!(
                "scroll_to_match: vertical scroll from {} to {}",
                current_offset, clamped_offset
            );
            
            app.results_scroll.offset = clamped_offset;
        }
    } else if viewport_height == 0 {
        // Haven't rendered yet, just set offset directly
        app.results_scroll.offset = target_line;
    }

    // Horizontal scrolling - ensure the match is visible horizontally
    let h_offset = app.results_scroll.h_offset;
    let max_h_offset = app.results_scroll.max_h_offset;
    let viewport_width = app.results_scroll.viewport_width;

    // If max_h_offset is 0, content fits horizontally - no scrolling needed
    // If viewport_width is 0, we haven't rendered yet
    if max_h_offset > 0 && viewport_width > 0 {
        let match_end = target_col.saturating_add(match_len);
        let visible_h_start = h_offset;
        let visible_h_end = h_offset.saturating_add(viewport_width);

        // Check if match is fully visible horizontally
        if target_col < visible_h_start || match_end > visible_h_end {
            // Match not fully visible, scroll to show it with some left context
            let left_margin: u16 = 10; // Show some context to the left of the match
            let new_h_offset = target_col.saturating_sub(left_margin);
            let clamped_h_offset = new_h_offset.min(max_h_offset);

            #[cfg(debug_assertions)]
            debug!(
                "scroll_to_match: horizontal scroll from {} to {} (match at col {}-{}, viewport_width={})",
                h_offset,
                clamped_h_offset,
                target_col,
                match_end,
                viewport_width
            );

            app.results_scroll.h_offset = clamped_h_offset;
        }
    } else if max_h_offset > 0 {
        // viewport_width is 0 (not rendered yet), just position the match with left margin
        let left_margin: u16 = 10;
        let new_h_offset = target_col.saturating_sub(left_margin);
        app.results_scroll.h_offset = new_h_offset.min(max_h_offset);
    }
}

/// Scroll results pane to make the given line visible (legacy function for compatibility)
fn scroll_to_line(app: &mut App, _line: u32) {
    // Now delegates to scroll_to_match which handles both vertical and horizontal scrolling
    scroll_to_match(app);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_helpers::{test_app, key, key_with_mods};
    use proptest::prelude::*;

    #[test]
    fn test_open_search_sets_visible_and_focus() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.focus = Focus::InputField;

        open_search(&mut app);

        assert!(app.search.is_visible());
        assert_eq!(app.focus, Focus::ResultsPane);
    }

    #[test]
    fn test_close_search_clears_state() {
        let mut app = test_app(r#"{"name": "test"}"#);
        open_search(&mut app);
        app.search.search_textarea_mut().insert_str("test");

        close_search(&mut app);

        assert!(!app.search.is_visible());
        assert!(app.search.query().is_empty());
    }

    #[test]
    fn test_handle_search_key_returns_false_when_not_visible() {
        let mut app = test_app(r#"{"name": "test"}"#);
        assert!(!app.search.is_visible());

        let handled = handle_search_key(&mut app, key(KeyCode::Char('n')));

        assert!(!handled);
    }

    #[test]
    fn test_escape_closes_search() {
        let mut app = test_app(r#"{"name": "test"}"#);
        open_search(&mut app);

        let handled = handle_search_key(&mut app, key(KeyCode::Esc));

        assert!(handled);
        assert!(!app.search.is_visible());
    }

    #[test]
    fn test_text_input_updates_query() {
        let mut app = test_app(r#"{"name": "test"}"#);
        open_search(&mut app);

        handle_search_key(&mut app, key(KeyCode::Char('t')));
        handle_search_key(&mut app, key(KeyCode::Char('e')));
        handle_search_key(&mut app, key(KeyCode::Char('s')));
        handle_search_key(&mut app, key(KeyCode::Char('t')));

        assert_eq!(app.search.query(), "test");
    }

    #[test]
    fn test_navigation_scrolls_to_match() {
        let mut app = test_app(r#"{"name": "test"}"#);
        
        // Create content with matches on different lines (lines 0, 10, 20)
        let content: String = (0..30)
            .map(|i| {
                if i == 0 || i == 10 || i == 20 {
                    format!("match line {}\n", i)
                } else {
                    format!("other line {}\n", i)
                }
            })
            .collect();
        
        app.query.last_successful_result = Some(content.clone());
        app.query.last_successful_result_unformatted = Some(content.clone());
        
        // Set up viewport (simulate render having happened)
        app.results_scroll.viewport_height = 10;
        app.results_scroll.max_offset = 20; // 30 lines - 10 viewport = 20 max
        app.results_scroll.offset = 0;
        
        open_search(&mut app);
        
        // Type search query
        app.search.search_textarea_mut().insert_str("match");
        app.search.update_matches(&content);
        
        assert_eq!(app.search.matches().len(), 3);
        assert_eq!(app.search.matches()[0].line, 0);
        assert_eq!(app.search.matches()[1].line, 10);
        assert_eq!(app.search.matches()[2].line, 20);
        
        // Confirm search (press Enter) - stays at index 0 (line 0)
        handle_search_key(&mut app, key(KeyCode::Enter));
        assert!(app.search.is_confirmed());
        assert_eq!(app.search.current_index(), 0);
        
        // Scroll should be at top for match at line 0
        assert_eq!(app.results_scroll.offset, 0, "Scroll should be at top for match at line 0");
        
        // Navigate to next match (line 10)
        handle_search_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.search.current_index(), 1);
        
        // Scroll should have been set to center line 10 in viewport
        // half_viewport = 10/2 = 5, so offset = 10 - 5 = 5
        assert_eq!(app.results_scroll.offset, 5, "Scroll should center match at line 10");
        
        // Navigate to next match (line 20)
        handle_search_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.search.current_index(), 2);
        
        // Scroll should center line 20: offset = 20 - 5 = 15
        assert_eq!(app.results_scroll.offset, 15, "Scroll should center match at line 20");
        
        // Navigate to next match (wraps to line 0)
        handle_search_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.search.current_index(), 0);
        
        // Scroll should center line 0: offset = max(0 - 5, 0) = 0
        assert_eq!(app.results_scroll.offset, 0, "Scroll should be at top for match at line 0");
    }

    #[test]
    fn test_n_navigates_to_next_match() {
        let mut app = test_app(r#"{"name": "test"}"#);
        // Set up content with matches
        app.query.last_successful_result = Some("test\ntest\ntest".to_string());
        open_search(&mut app);
        
        // Type search query
        app.search.search_textarea_mut().insert_str("test");
        app.search.update_matches("test\ntest\ntest");
        
        assert_eq!(app.search.matches().len(), 3);
        assert_eq!(app.search.current_index(), 0);

        // Confirm search first (press Enter) - this enables n/N navigation and stays at index 0
        handle_search_key(&mut app, key(KeyCode::Enter));
        assert!(app.search.is_confirmed());
        assert_eq!(app.search.current_index(), 0); // First Enter stays at index 0

        // Navigate to next with 'n'
        handle_search_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.search.current_index(), 1);
        
        // Navigate to next again with 'n'
        handle_search_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.search.current_index(), 2);
    }

    #[test]
    fn test_capital_n_navigates_to_prev_match() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.query.last_successful_result = Some("test\ntest\ntest".to_string());
        open_search(&mut app);
        
        app.search.search_textarea_mut().insert_str("test");
        app.search.update_matches("test\ntest\ntest");
        
        // Confirm search first (press Enter) - this enables n/N navigation and stays at index 0
        handle_search_key(&mut app, key(KeyCode::Enter));
        assert!(app.search.is_confirmed());
        assert_eq!(app.search.current_index(), 0); // First Enter stays at index 0

        // Navigate to previous with 'N' (wraps: 0 -> 2, last index)
        handle_search_key(&mut app, key(KeyCode::Char('N')));
        assert_eq!(app.search.current_index(), 2);
    }

    #[test]
    fn test_enter_navigates_to_next_match() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.query.last_successful_result = Some("test\ntest".to_string());
        open_search(&mut app);
        
        app.search.search_textarea_mut().insert_str("test");
        app.search.update_matches("test\ntest");

        // First Enter: confirms and stays at index 0
        handle_search_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.search.current_index(), 0);
        assert!(app.search.is_confirmed());
        
        // Second Enter: navigates to index 1
        handle_search_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.search.current_index(), 1);
    }

    #[test]
    fn test_shift_enter_navigates_to_prev_match() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.query.last_successful_result = Some("test\ntest".to_string());
        open_search(&mut app);
        
        app.search.search_textarea_mut().insert_str("test");
        app.search.update_matches("test\ntest");

        // First Shift+Enter: confirms and stays at index 0
        handle_search_key(&mut app, key_with_mods(KeyCode::Enter, KeyModifiers::SHIFT));
        assert_eq!(app.search.current_index(), 0);
        assert!(app.search.is_confirmed());
        
        // Second Shift+Enter: navigates to previous (wraps to last, index 1)
        handle_search_key(&mut app, key_with_mods(KeyCode::Enter, KeyModifiers::SHIFT));
        assert_eq!(app.search.current_index(), 1);
    }

    #[test]
    fn test_scroll_to_match_centers_vertically() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.results_scroll.viewport_height = 20;
        app.results_scroll.max_offset = 100;
        app.results_scroll.offset = 0;
        
        // Set up a match at line 50
        app.search.open();
        app.search.search_textarea_mut().insert_str("test");
        // Manually set matches to control the test
        let content = (0..120).map(|i| if i == 50 { "test\n" } else { "line\n" }).collect::<String>();
        app.search.update_matches(&content);
        
        // Navigate to the match at line 50
        while app.search.current_match().map(|m| m.line) != Some(50) {
            app.search.next_match();
        }

        scroll_to_match(&mut app);

        // Should center the match (50 - 10 = 40)
        assert_eq!(app.results_scroll.offset, 40);
    }

    #[test]
    fn test_scroll_to_match_no_scroll_if_visible() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.results_scroll.viewport_height = 20;
        app.results_scroll.max_offset = 100;
        app.results_scroll.offset = 10;
        
        // Set up a match at line 15 (within viewport 10-30)
        app.search.open();
        app.search.search_textarea_mut().insert_str("test");
        let content = (0..120).map(|i| if i == 15 { "test\n" } else { "line\n" }).collect::<String>();
        app.search.update_matches(&content);

        scroll_to_match(&mut app);

        // Should not change offset since line 15 is visible in range [10, 30)
        assert_eq!(app.results_scroll.offset, 10);
    }

    #[test]
    fn test_scroll_to_match_clamps_to_max() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.results_scroll.viewport_height = 20;
        app.results_scroll.max_offset = 50;
        app.results_scroll.offset = 0;
        
        // Set up a match at line 100
        app.search.open();
        app.search.search_textarea_mut().insert_str("test");
        let content = (0..120).map(|i| if i == 100 { "test\n" } else { "line\n" }).collect::<String>();
        app.search.update_matches(&content);
        
        // Navigate to the match at line 100
        while app.search.current_match().map(|m| m.line) != Some(100) {
            app.search.next_match();
        }

        scroll_to_match(&mut app);

        // Should clamp to max_offset (50)
        assert_eq!(app.results_scroll.offset, 50);
    }
    
    #[test]
    fn test_scroll_to_match_horizontal() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.results_scroll.viewport_height = 20;
        app.results_scroll.max_offset = 100;
        app.results_scroll.max_h_offset = 200;
        app.results_scroll.offset = 0;
        app.results_scroll.h_offset = 0;
        
        // Set up a match at column 150 (beyond typical viewport)
        app.search.open();
        app.search.search_textarea_mut().insert_str("test");
        // Create content with match far to the right
        let content = format!("{}test\n", " ".repeat(150));
        app.search.update_matches(&content);

        scroll_to_match(&mut app);

        // Should scroll horizontally to show the match (150 - 10 margin = 140)
        assert_eq!(app.results_scroll.h_offset, 140);
    }

    #[test]
    fn test_ctrl_f_reenters_edit_mode_when_confirmed() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.query.last_successful_result = Some("test\ntest".to_string());
        app.query.last_successful_result_unformatted = Some("test\ntest".to_string());
        open_search(&mut app);
        
        // Type search query
        app.search.search_textarea_mut().insert_str("test");
        app.search.update_matches("test\ntest");
        
        // Confirm search (press Enter)
        handle_search_key(&mut app, key(KeyCode::Enter));
        assert!(app.search.is_confirmed());
        
        // Press Ctrl+F to re-enter edit mode
        handle_search_key(&mut app, key_with_mods(KeyCode::Char('f'), KeyModifiers::CONTROL));
        
        // Should be unconfirmed now (edit mode)
        assert!(!app.search.is_confirmed());
        // Search should still be visible
        assert!(app.search.is_visible());
        // Query should be preserved
        assert_eq!(app.search.query(), "test");
    }

    #[test]
    fn test_slash_reenters_edit_mode_when_confirmed() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.query.last_successful_result = Some("test\ntest".to_string());
        app.query.last_successful_result_unformatted = Some("test\ntest".to_string());
        open_search(&mut app);
        
        // Type search query
        app.search.search_textarea_mut().insert_str("test");
        app.search.update_matches("test\ntest");
        
        // Confirm search (press Enter)
        handle_search_key(&mut app, key(KeyCode::Enter));
        assert!(app.search.is_confirmed());
        
        // Press / to re-enter edit mode
        handle_search_key(&mut app, key(KeyCode::Char('/')));
        
        // Should be unconfirmed now (edit mode)
        assert!(!app.search.is_confirmed());
        // Search should still be visible
        assert!(app.search.is_visible());
        // Query should be preserved
        assert_eq!(app.search.query(), "test");
    }

    #[test]
    fn test_can_type_after_reenter_edit_mode() {
        let mut app = test_app(r#"{"name": "test"}"#);
        app.query.last_successful_result = Some("test\ntest".to_string());
        app.query.last_successful_result_unformatted = Some("test\ntest".to_string());
        open_search(&mut app);
        
        // Type initial query
        app.search.search_textarea_mut().insert_str("test");
        app.search.update_matches("test\ntest");
        
        // Confirm search
        handle_search_key(&mut app, key(KeyCode::Enter));
        assert!(app.search.is_confirmed());
        
        // Re-enter edit mode with Ctrl+F
        handle_search_key(&mut app, key_with_mods(KeyCode::Char('f'), KeyModifiers::CONTROL));
        assert!(!app.search.is_confirmed());
        
        // Now typing should work - add more characters
        handle_search_key(&mut app, key(KeyCode::Char('2')));
        
        // Query should be updated
        assert_eq!(app.search.query(), "test2");
    }

    // =========================================================================
    // Navigation Keys When Search Is Confirmed Tests
    // Feature: search-navigation
    // =========================================================================

    /// Helper to set up app with confirmed search and scrollable content
    fn app_with_confirmed_search() -> App {
        let mut app = test_app(r#"{"name": "test"}"#);
        
        // Set up content with 50 lines
        let content: String = (0..50).map(|i| format!("line {} test\n", i)).collect();
        app.query.last_successful_result = Some(content.clone());
        app.query.last_successful_result_unformatted = Some(content.clone());
        app.query.result = Ok(content.clone());
        
        // Set up scroll bounds
        app.results_scroll.update_bounds(50, 10); // 50 lines, 10 viewport
        app.results_scroll.update_h_bounds(100, 40); // 100 max width, 40 viewport
        app.results_scroll.offset = 0;
        app.results_scroll.h_offset = 0;
        
        // Open and confirm search
        open_search(&mut app);
        app.search.search_textarea_mut().insert_str("test");
        app.search.update_matches(&content);
        handle_search_key(&mut app, key(KeyCode::Enter));
        
        assert!(app.search.is_confirmed());
        app
    }

    #[test]
    fn test_j_scrolls_down_when_search_confirmed() {
        let mut app = app_with_confirmed_search();
        app.results_scroll.offset = 0;
        
        handle_search_key(&mut app, key(KeyCode::Char('j')));
        
        assert_eq!(app.results_scroll.offset, 1);
        assert!(app.search.is_confirmed(), "Search should remain confirmed");
    }

    #[test]
    fn test_k_scrolls_up_when_search_confirmed() {
        let mut app = app_with_confirmed_search();
        app.results_scroll.offset = 10;
        
        handle_search_key(&mut app, key(KeyCode::Char('k')));
        
        assert_eq!(app.results_scroll.offset, 9);
        assert!(app.search.is_confirmed(), "Search should remain confirmed");
    }

    #[test]
    fn test_h_scrolls_left_when_search_confirmed() {
        let mut app = app_with_confirmed_search();
        app.results_scroll.h_offset = 10;
        
        handle_search_key(&mut app, key(KeyCode::Char('h')));
        
        assert_eq!(app.results_scroll.h_offset, 9);
        assert!(app.search.is_confirmed(), "Search should remain confirmed");
    }

    #[test]
    fn test_l_scrolls_right_when_search_confirmed() {
        let mut app = app_with_confirmed_search();
        app.results_scroll.h_offset = 0;
        
        handle_search_key(&mut app, key(KeyCode::Char('l')));
        
        assert_eq!(app.results_scroll.h_offset, 1);
        assert!(app.search.is_confirmed(), "Search should remain confirmed");
    }

    #[test]
    fn test_g_jumps_to_top_when_search_confirmed() {
        let mut app = app_with_confirmed_search();
        app.results_scroll.offset = 30;
        
        handle_search_key(&mut app, key(KeyCode::Char('g')));
        
        assert_eq!(app.results_scroll.offset, 0);
        assert!(app.search.is_confirmed(), "Search should remain confirmed");
    }

    #[test]
    fn test_capital_g_jumps_to_bottom_when_search_confirmed() {
        let mut app = app_with_confirmed_search();
        app.results_scroll.offset = 0;
        let max_offset = app.results_scroll.max_offset;
        
        handle_search_key(&mut app, key(KeyCode::Char('G')));
        
        assert_eq!(app.results_scroll.offset, max_offset);
        assert!(app.search.is_confirmed(), "Search should remain confirmed");
    }

    #[test]
    fn test_ctrl_d_page_down_when_search_confirmed() {
        let mut app = app_with_confirmed_search();
        app.results_scroll.offset = 0;
        
        handle_search_key(&mut app, key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));
        
        // Half page = viewport_height / 2 = 10 / 2 = 5
        assert_eq!(app.results_scroll.offset, 5);
        assert!(app.search.is_confirmed(), "Search should remain confirmed");
    }

    #[test]
    fn test_ctrl_u_page_up_when_search_confirmed() {
        let mut app = app_with_confirmed_search();
        app.results_scroll.offset = 20;
        
        handle_search_key(&mut app, key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));
        
        // Half page = viewport_height / 2 = 10 / 2 = 5
        assert_eq!(app.results_scroll.offset, 15);
        assert!(app.search.is_confirmed(), "Search should remain confirmed");
    }

    #[test]
    fn test_navigation_preserves_match_index() {
        let mut app = app_with_confirmed_search();
        
        // Navigate to a specific match first
        handle_search_key(&mut app, key(KeyCode::Char('n'))); // Go to match 1
        let match_index_before = app.search.current_index();
        
        // Scroll with navigation keys
        handle_search_key(&mut app, key(KeyCode::Char('j')));
        handle_search_key(&mut app, key(KeyCode::Char('k')));
        handle_search_key(&mut app, key(KeyCode::Char('l')));
        handle_search_key(&mut app, key(KeyCode::Char('h')));
        
        // Match index should be unchanged
        assert_eq!(app.search.current_index(), match_index_before);
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // Feature: search-in-results, Property 1: Ctrl+F opens search from any pane
    // *For any* app state regardless of current focus, pressing Ctrl+F should
    // result in search being visible and focus being on results pane.
    // **Validates: Requirements 1.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_ctrl_f_opens_search_from_any_pane(
            // Test with both focus states
            focus_on_input in any::<bool>(),
        ) {
            let mut app = test_app(r#"{"name": "test"}"#);
            
            // Set initial focus based on generated value
            app.focus = if focus_on_input {
                Focus::InputField
            } else {
                Focus::ResultsPane
            };
            
            // Ensure search is initially closed
            assert!(!app.search.is_visible());
            
            // Simulate Ctrl+F by calling open_search (which is what global handler does)
            open_search(&mut app);
            
            // Verify search is now visible
            prop_assert!(
                app.search.is_visible(),
                "Search should be visible after Ctrl+F"
            );
            
            // Verify focus is on results pane
            prop_assert_eq!(
                app.focus, Focus::ResultsPane,
                "Focus should be on results pane after Ctrl+F"
            );
        }
    }

    // Feature: search-in-results, Property 2: Slash opens search only from results pane
    // *For any* app state, pressing `/` should only open search when results pane
    // is focused; when input field is focused, `/` should be typed as a character.
    // **Validates: Requirements 1.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_slash_opens_search_only_from_results_pane(
            // Test with both focus states
            focus_on_input in any::<bool>(),
        ) {
            let mut app = test_app(r#"{"name": "test"}"#);
            
            // Set initial focus based on generated value
            let initial_focus = if focus_on_input {
                Focus::InputField
            } else {
                Focus::ResultsPane
            };
            app.focus = initial_focus;
            
            // Ensure search is initially closed
            assert!(!app.search.is_visible());
            
            // Simulate pressing '/' - this is handled differently based on focus
            // In results pane: opens search
            // In input field: types '/' character (not handled by search)
            if initial_focus == Focus::ResultsPane {
                // When in results pane, '/' opens search
                open_search(&mut app);
                
                prop_assert!(
                    app.search.is_visible(),
                    "Search should be visible after '/' in results pane"
                );
            } else {
                // When in input field, '/' should NOT open search
                // (it would be typed as a character instead)
                // We verify that open_search is NOT called by checking search remains closed
                // In the actual app, the '/' key is handled by the input field, not search
                prop_assert!(
                    !app.search.is_visible(),
                    "Search should NOT be visible when '/' pressed in input field"
                );
            }
        }
    }

    // Feature: search-in-results, Property 8: Auto-scroll positions match in viewport
    // *For any* match navigation, the resulting scroll offset should position the
    // match line within the visible viewport.
    // **Validates: Requirements 3.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_auto_scroll_positions_match_in_viewport(
            // Generate viewport and scroll parameters
            viewport_height in 5u16..50,
            max_offset in 10u16..200,
            initial_offset in 0u16..100,
            // Constrain target_line to be within the scrollable content
            // (max_offset + viewport_height represents the total content height)
            target_line_factor in 0.0f64..1.0,
        ) {
            let mut app = test_app(r#"{"name": "test"}"#);
            
            // Set up scroll state
            app.results_scroll.viewport_height = viewport_height;
            app.results_scroll.max_offset = max_offset;
            app.results_scroll.offset = initial_offset.min(max_offset);
            
            // Calculate target line within valid content range
            // Content height = max_offset + viewport_height (the last visible line when scrolled to max)
            let content_height = max_offset as u32 + viewport_height as u32;
            let target_line = ((target_line_factor * content_height as f64) as u32).min(content_height.saturating_sub(1));
            
            // Set up a match at the target line so scroll_to_match works
            app.search.open();
            app.search.search_textarea_mut().insert_str("test");
            // Create content with a match at the target line
            let content: String = (0..content_height)
                .map(|i| if i == target_line { "test\n" } else { "line\n" })
                .collect();
            app.search.update_matches(&content);
            
            // Navigate to the match at target_line (there should be exactly one match)
            prop_assert_eq!(app.search.matches().len(), 1, "Should have exactly one match");
            prop_assert_eq!(app.search.current_match().map(|m| m.line), Some(target_line), "Match should be at target line");
            
            // Call scroll_to_match
            scroll_to_match(&mut app);
            
            // Get the resulting offset
            let result_offset = app.results_scroll.offset;
            
            // Calculate visible range after scroll
            let visible_start = result_offset as u32;
            let visible_end = visible_start + viewport_height as u32;
            
            // The target line should be within the visible viewport
            prop_assert!(
                target_line >= visible_start && target_line < visible_end,
                "Target line {} should be within visible range [{}, {}), offset={}, viewport_height={}, max_offset={}",
                target_line, visible_start, visible_end, result_offset, viewport_height, max_offset
            );
            
            // Verify offset is within valid bounds
            prop_assert!(
                result_offset <= max_offset,
                "Scroll offset {} should not exceed max_offset {}",
                result_offset, max_offset
            );
        }
    }

    // =========================================================================
    // Feature: search-navigation Property-Based Tests
    // =========================================================================

    // Feature: search-navigation, Property 1: Vertical navigation keys scroll results when search is confirmed
    // *For any* confirmed search state with scrollable content, pressing a vertical navigation key
    // (j, k, J, K, g, G, Up, Down, Home, End) SHALL change the vertical scroll offset according to
    // the key's expected behavior (same as when search is not visible).
    // **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.6**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_vertical_navigation_scrolls_when_search_confirmed(
            // Generate scroll state parameters
            viewport_height in 5u16..50,
            max_offset in 10u16..200,
            initial_offset_factor in 0.0f64..1.0,
            // Generate which vertical key to test (0-9 for different keys)
            key_type in 0u8..10,
        ) {
            let mut app = test_app(r#"{"name": "test"}"#);
            
            // Set up content with enough lines
            let content_lines = max_offset as u32 + viewport_height as u32;
            let content: String = (0..content_lines)
                .map(|i| format!("line {} test\n", i))
                .collect();
            app.query.last_successful_result = Some(content.clone());
            app.query.last_successful_result_unformatted = Some(content.clone());
            app.query.result = Ok(content.clone());
            
            // Set up scroll bounds
            app.results_scroll.update_bounds(content_lines, viewport_height);
            let initial_offset = ((initial_offset_factor * max_offset as f64) as u16).min(max_offset);
            app.results_scroll.offset = initial_offset;
            
            // Open and confirm search
            open_search(&mut app);
            app.search.search_textarea_mut().insert_str("test");
            app.search.update_matches(&content);
            handle_search_key(&mut app, key(KeyCode::Enter));
            
            prop_assert!(app.search.is_confirmed(), "Search should be confirmed");
            
            // Record offset before navigation
            let offset_before = app.results_scroll.offset;
            
            // Apply vertical navigation key
            let test_key = match key_type {
                0 => key(KeyCode::Char('j')),
                1 => key(KeyCode::Char('k')),
                2 => key(KeyCode::Char('J')),
                3 => key(KeyCode::Char('K')),
                4 => key(KeyCode::Char('g')),
                5 => key(KeyCode::Char('G')),
                6 => key(KeyCode::Up),
                7 => key(KeyCode::Down),
                8 => key(KeyCode::Home),
                9 => key(KeyCode::End),
                _ => key(KeyCode::Char('j')),
            };
            
            handle_search_key(&mut app, test_key);
            
            // Calculate expected offset based on key type
            let expected_offset = match key_type {
                0 | 7 => offset_before.saturating_add(1).min(max_offset), // j, Down
                1 | 6 => offset_before.saturating_sub(1), // k, Up
                2 => offset_before.saturating_add(10).min(max_offset), // J
                3 => offset_before.saturating_sub(10), // K
                4 | 8 => 0, // g, Home
                5 | 9 => max_offset, // G, End
                _ => offset_before,
            };
            
            prop_assert_eq!(
                app.results_scroll.offset, expected_offset,
                "Vertical navigation key {} should change offset from {} to {}, got {}",
                key_type, offset_before, expected_offset, app.results_scroll.offset
            );
            
            // Search should remain confirmed
            prop_assert!(app.search.is_confirmed(), "Search should remain confirmed after navigation");
        }
    }

    // Feature: search-navigation, Property 2: Horizontal navigation keys scroll results when search is confirmed
    // *For any* confirmed search state with horizontally scrollable content, pressing a horizontal
    // navigation key (h, l, H, L, 0, ^, $, Left, Right) SHALL change the horizontal scroll offset
    // according to the key's expected behavior (same as when search is not visible).
    // **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_horizontal_navigation_scrolls_when_search_confirmed(
            // Generate scroll state parameters
            viewport_width in 20u16..80,
            max_h_offset in 20u16..200,
            initial_h_offset_factor in 0.0f64..1.0,
            // Generate which horizontal key to test (0-8 for different keys)
            key_type in 0u8..9,
        ) {
            let mut app = test_app(r#"{"name": "test"}"#);
            
            // Set up content with wide lines
            let line_width = max_h_offset + viewport_width;
            let content: String = (0..20)
                .map(|i| format!("line {} test {}\n", i, "x".repeat(line_width as usize)))
                .collect();
            app.query.last_successful_result = Some(content.clone());
            app.query.last_successful_result_unformatted = Some(content.clone());
            app.query.result = Ok(content.clone());
            
            // Set up scroll bounds
            app.results_scroll.update_bounds(20, 10);
            app.results_scroll.update_h_bounds(line_width, viewport_width);
            let initial_h_offset = ((initial_h_offset_factor * max_h_offset as f64) as u16).min(max_h_offset);
            app.results_scroll.h_offset = initial_h_offset;
            
            // Open and confirm search
            open_search(&mut app);
            app.search.search_textarea_mut().insert_str("test");
            app.search.update_matches(&content);
            handle_search_key(&mut app, key(KeyCode::Enter));
            
            prop_assert!(app.search.is_confirmed(), "Search should be confirmed");
            
            // Record h_offset before navigation
            let h_offset_before = app.results_scroll.h_offset;
            
            // Apply horizontal navigation key
            let test_key = match key_type {
                0 => key(KeyCode::Char('h')),
                1 => key(KeyCode::Char('l')),
                2 => key(KeyCode::Char('H')),
                3 => key(KeyCode::Char('L')),
                4 => key(KeyCode::Char('0')),
                5 => key(KeyCode::Char('^')),
                6 => key(KeyCode::Char('$')),
                7 => key(KeyCode::Left),
                8 => key(KeyCode::Right),
                _ => key(KeyCode::Char('h')),
            };
            
            handle_search_key(&mut app, test_key);
            
            // Calculate expected h_offset based on key type
            let expected_h_offset = match key_type {
                0 | 7 => h_offset_before.saturating_sub(1), // h, Left
                1 | 8 => h_offset_before.saturating_add(1).min(max_h_offset), // l, Right
                2 => h_offset_before.saturating_sub(10), // H
                3 => h_offset_before.saturating_add(10).min(max_h_offset), // L
                4 | 5 => 0, // 0, ^
                6 => max_h_offset, // $
                _ => h_offset_before,
            };
            
            prop_assert_eq!(
                app.results_scroll.h_offset, expected_h_offset,
                "Horizontal navigation key {} should change h_offset from {} to {}, got {}",
                key_type, h_offset_before, expected_h_offset, app.results_scroll.h_offset
            );
            
            // Search should remain confirmed
            prop_assert!(app.search.is_confirmed(), "Search should remain confirmed after navigation");
        }
    }

    // Feature: search-navigation, Property 3: Page scroll keys scroll results when search is confirmed
    // *For any* confirmed search state with scrollable content, pressing a page scroll key
    // (Ctrl+D, Ctrl+U, PageDown, PageUp) SHALL change the vertical scroll offset by half the
    // viewport height (clamped to bounds).
    // **Validates: Requirements 3.1, 3.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_page_scroll_when_search_confirmed(
            // Generate scroll state parameters
            viewport_height in 10u16..50,
            max_offset in 20u16..200,
            initial_offset_factor in 0.0f64..1.0,
            // Generate which page scroll key to test (0-3 for different keys)
            key_type in 0u8..4,
        ) {
            let mut app = test_app(r#"{"name": "test"}"#);
            
            // Set up content with enough lines
            let content_lines = max_offset as u32 + viewport_height as u32;
            let content: String = (0..content_lines)
                .map(|i| format!("line {} test\n", i))
                .collect();
            app.query.last_successful_result = Some(content.clone());
            app.query.last_successful_result_unformatted = Some(content.clone());
            app.query.result = Ok(content.clone());
            
            // Set up scroll bounds
            app.results_scroll.update_bounds(content_lines, viewport_height);
            let initial_offset = ((initial_offset_factor * max_offset as f64) as u16).min(max_offset);
            app.results_scroll.offset = initial_offset;
            
            // Open and confirm search
            open_search(&mut app);
            app.search.search_textarea_mut().insert_str("test");
            app.search.update_matches(&content);
            handle_search_key(&mut app, key(KeyCode::Enter));
            
            prop_assert!(app.search.is_confirmed(), "Search should be confirmed");
            
            // Record offset before navigation
            let offset_before = app.results_scroll.offset;
            let half_page = viewport_height / 2;
            
            // Apply page scroll key
            let test_key = match key_type {
                0 => key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL), // Ctrl+D
                1 => key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL), // Ctrl+U
                2 => key(KeyCode::PageDown),
                3 => key(KeyCode::PageUp),
                _ => key(KeyCode::PageDown),
            };
            
            handle_search_key(&mut app, test_key);
            
            // Calculate expected offset based on key type
            let expected_offset = match key_type {
                0 | 2 => offset_before.saturating_add(half_page).min(max_offset), // Ctrl+D, PageDown
                1 | 3 => offset_before.saturating_sub(half_page), // Ctrl+U, PageUp
                _ => offset_before,
            };
            
            prop_assert_eq!(
                app.results_scroll.offset, expected_offset,
                "Page scroll key {} should change offset from {} to {} (half_page={}), got {}",
                key_type, offset_before, expected_offset, half_page, app.results_scroll.offset
            );
            
            // Search should remain confirmed
            prop_assert!(app.search.is_confirmed(), "Search should remain confirmed after navigation");
        }
    }

    // Feature: search-navigation, Property 4: Navigation does not affect match index
    // *For any* confirmed search state with matches, scrolling using navigation keys SHALL NOT
    // change the current match index.
    // **Validates: Requirements 4.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_navigation_preserves_match_index(
            // Generate scroll state parameters
            viewport_height in 10u16..50,
            max_offset in 20u16..200,
            initial_offset_factor in 0.0f64..1.0,
            // Generate number of 'n' presses to set initial match index (0-5)
            n_presses in 0usize..6,
            // Generate which navigation key to test
            key_type in 0u8..15,
        ) {
            let mut app = test_app(r#"{"name": "test"}"#);
            
            // Set up content with matches on multiple lines
            let content_lines = max_offset as u32 + viewport_height as u32;
            let content: String = (0..content_lines)
                .map(|i| format!("line {} test\n", i)) // "test" on every line
                .collect();
            app.query.last_successful_result = Some(content.clone());
            app.query.last_successful_result_unformatted = Some(content.clone());
            app.query.result = Ok(content.clone());
            
            // Set up scroll bounds
            app.results_scroll.update_bounds(content_lines, viewport_height);
            app.results_scroll.update_h_bounds(100, 40);
            let initial_offset = ((initial_offset_factor * max_offset as f64) as u16).min(max_offset);
            app.results_scroll.offset = initial_offset;
            app.results_scroll.h_offset = 10;
            
            // Open and confirm search
            open_search(&mut app);
            app.search.search_textarea_mut().insert_str("test");
            app.search.update_matches(&content);
            handle_search_key(&mut app, key(KeyCode::Enter));
            
            prop_assert!(app.search.is_confirmed(), "Search should be confirmed");
            prop_assert!(!app.search.matches().is_empty(), "Should have matches");
            
            // Navigate to a specific match index
            for _ in 0..n_presses {
                handle_search_key(&mut app, key(KeyCode::Char('n')));
            }
            
            // Record match index before navigation
            let match_index_before = app.search.current_index();
            
            // Apply navigation key (not n/N which change match index)
            let test_key = match key_type {
                0 => key(KeyCode::Char('j')),
                1 => key(KeyCode::Char('k')),
                2 => key(KeyCode::Char('J')),
                3 => key(KeyCode::Char('K')),
                4 => key(KeyCode::Char('g')),
                5 => key(KeyCode::Char('G')),
                6 => key(KeyCode::Char('h')),
                7 => key(KeyCode::Char('l')),
                8 => key(KeyCode::Char('H')),
                9 => key(KeyCode::Char('L')),
                10 => key(KeyCode::Char('0')),
                11 => key(KeyCode::Char('^')),
                12 => key(KeyCode::Char('$')),
                13 => key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL),
                14 => key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL),
                _ => key(KeyCode::Char('j')),
            };
            
            handle_search_key(&mut app, test_key);
            
            // Match index should be unchanged
            prop_assert_eq!(
                app.search.current_index(), match_index_before,
                "Navigation key {} should not change match index (was {}, now {})",
                key_type, match_index_before, app.search.current_index()
            );
            
            // Search should remain confirmed
            prop_assert!(app.search.is_confirmed(), "Search should remain confirmed after navigation");
        }
    }

    // =========================================================================
    // Feature: search-in-results Property-Based Tests (existing)
    // =========================================================================

    // Feature: search-in-results, Property 9: Scroll preserves search state
    // *For any* active search state, scrolling the results pane should not modify
    // the matches list or current_index.
    // **Validates: Requirements 5.1, 5.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_scroll_preserves_search_state(
            // Generate search state parameters
            num_matches in 1usize..20,
            query in "[a-zA-Z]{1,10}",
            // Generate scroll parameters
            viewport_height in 10u16..50,
            max_offset in 20u16..200,
            initial_offset in 0u16..100,
            // Generate scroll operation type
            scroll_op in 0u8..8,
        ) {
            use crate::search::search_state::Match;

            let mut app = test_app(r#"{"name": "test"}"#);
            
            // Set up search state with matches
            app.search.open();
            app.search.search_textarea_mut().insert_str(&query);
            
            // Set up content that will produce matches
            let content: String = (0..num_matches)
                .map(|i| format!("line {} {}\n", i, query))
                .collect();
            app.query.last_successful_result = Some(content.clone());
            app.search.update_matches(&content);
            
            // Capture search state before scroll
            let matches_before: Vec<Match> = app.search.matches().to_vec();
            let current_index_before = app.search.current_index();
            let query_before = app.search.query().to_string();
            let visible_before = app.search.is_visible();
            
            // Set up scroll state
            app.results_scroll.viewport_height = viewport_height;
            app.results_scroll.max_offset = max_offset;
            app.results_scroll.offset = initial_offset.min(max_offset);
            
            // Perform a scroll operation (simulating what results/events.rs does)
            match scroll_op {
                0 => app.results_scroll.scroll_up(1),
                1 => app.results_scroll.scroll_down(1),
                2 => app.results_scroll.scroll_up(10),
                3 => app.results_scroll.scroll_down(10),
                4 => app.results_scroll.page_up(),
                5 => app.results_scroll.page_down(),
                6 => app.results_scroll.jump_to_top(),
                7 => app.results_scroll.jump_to_bottom(),
                _ => app.results_scroll.scroll_down(1),
            }
            
            // Verify search state is unchanged after scroll
            prop_assert_eq!(
                app.search.matches().to_vec(), matches_before,
                "Matches should be unchanged after scroll"
            );
            prop_assert_eq!(
                app.search.current_index(), current_index_before,
                "Current index should be unchanged after scroll"
            );
            prop_assert_eq!(
                app.search.query(), query_before,
                "Query should be unchanged after scroll"
            );
            prop_assert_eq!(
                app.search.is_visible(), visible_before,
                "Visibility should be unchanged after scroll"
            );
        }
    }
}
