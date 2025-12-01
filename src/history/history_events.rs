use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::Input;

use crate::app::App;

/// Handle keys when history popup is visible
pub fn handle_history_popup_key(app: &mut App, key: KeyEvent) {
    match key.code {
        // Navigation (reversed because display is reversed - most recent at bottom)
        KeyCode::Up => {
            app.history.select_next(); // Move to older entries (visually up)
        }
        KeyCode::Down => {
            app.history.select_previous(); // Move to newer entries (visually down)
        }

        // Select and close
        KeyCode::Enter | KeyCode::Tab => {
            if let Some(entry) = app.history.selected_entry() {
                let entry = entry.to_string();
                replace_query_with(app, &entry);
            }
            app.history.close();
        }

        // Cancel
        KeyCode::Esc => {
            app.history.close();
        }

        // Let TextArea handle all other input (chars, backspace, left/right arrows, etc.)
        _ => {
            let input = Input::from(key);
            if app.history.search_textarea_mut().input(input) {
                // Input was consumed, update filter
                app.history.on_search_input_changed();
            }
        }
    }
}

/// Replace the current query with the given text (helper for history)
fn replace_query_with(app: &mut App, text: &str) {
    app.input.textarea.delete_line_by_head();
    app.input.textarea.delete_line_by_end();
    app.input.textarea.insert_str(text);

    // Execute the new query
    let query = app.input.textarea.lines()[0].as_ref();
    // Use QueryState::execute() which handles non-null result caching
    app.query.execute(query);

    app.results_scroll.reset();
    app.error_overlay_visible = false;
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, Focus};
    use crate::config::Config;
    use crate::editor::EditorMode;
    use crate::history::HistoryState;
    use ratatui::crossterm::event::KeyModifiers;

    // Test fixture data
    const TEST_JSON: &str = r#"{"name": "test", "age": 30, "city": "NYC"}"#;

    /// Helper to create App with default config for tests
    fn test_app(json: &str) -> App {
        App::new(json.to_string(), &Config::default())
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

    // ========== History Popup Tests ==========

    #[test]
    fn test_history_popup_does_not_open_when_empty() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        // app_with_query helper creates empty in-memory history
        assert_eq!(app.history.total_count(), 0);

        // Try to open with Ctrl+R
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Should NOT open because history is empty
        assert!(!app.history.is_visible());
    }

    #[test]
    fn test_history_popup_navigation() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        // Add entries to in-memory history only (doesn't write to disk)
        // Most recent first: .baz, .bar, .foo (displays bottom to top)
        app.history.add_entry_in_memory(".foo");
        app.history.add_entry_in_memory(".bar");
        app.history.add_entry_in_memory(".baz");

        // Open history - .baz (most recent) should be selected at bottom
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert!(app.history.is_visible());
        assert_eq!(app.history.selected_index(), 0); // .baz at bottom

        // Press Up - should go to older entry (visually up)
        app.handle_key_event(key(KeyCode::Up));
        assert_eq!(app.history.selected_index(), 1); // .bar in middle

        // Press Down - should go to newer entry (visually down)
        app.handle_key_event(key(KeyCode::Down));
        assert_eq!(app.history.selected_index(), 0); // Back to .baz at bottom
    }

    #[test]
    fn test_history_popup_escape_closes() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".test");

        // Open history
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert!(app.history.is_visible());

        // Press Escape
        app.handle_key_event(key(KeyCode::Esc));
        assert!(!app.history.is_visible());

        // Query should be unchanged
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_history_popup_enter_selects() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".selected_query");

        // Open history
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Press Enter to select
        app.handle_key_event(key(KeyCode::Enter));

        assert!(!app.history.is_visible());
        assert_eq!(app.query(), ".selected_query");
    }

    #[test]
    fn test_history_popup_tab_selects() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".tab_selected");

        // Open history
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Press Tab to select
        app.handle_key_event(key(KeyCode::Tab));

        assert!(!app.history.is_visible());
        assert_eq!(app.query(), ".tab_selected");
    }

    #[test]
    fn test_history_popup_search_filters() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".apple");
        app.history.add_entry_in_memory(".banana");
        app.history.add_entry_in_memory(".apricot");

        // Open history
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Type 'ap' to filter
        app.handle_key_event(key(KeyCode::Char('a')));
        app.handle_key_event(key(KeyCode::Char('p')));

        // Should filter to entries containing 'ap'
        assert_eq!(app.history.search_query(), "ap");
        // Filtered count should be less than total (banana filtered out)
        assert!(app.history.filtered_count() < app.history.total_count());
    }

    #[test]
    fn test_history_popup_backspace_removes_search_char() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".test");

        // Open history
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Type something
        app.handle_key_event(key(KeyCode::Char('a')));
        app.handle_key_event(key(KeyCode::Char('b')));
        assert_eq!(app.history.search_query(), "ab");

        // Backspace
        app.handle_key_event(key(KeyCode::Backspace));
        assert_eq!(app.history.search_query(), "a");
    }

    #[test]
    fn test_shift_tab_closes_history_popup() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".test");

        // Open history
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert!(app.history.is_visible());

        // Press Shift+Tab to switch focus
        app.handle_key_event(key(KeyCode::BackTab));

        // History should be closed
        assert!(!app.history.is_visible());
        assert_eq!(app.focus, Focus::ResultsPane);
    }

    #[test]
    fn test_up_arrow_opens_history_when_cursor_at_start() {
        let mut app = app_with_query(".existing");
        app.input.editor_mode = EditorMode::Insert;
        app.history.add_entry_in_memory(".history_item");

        // Move cursor to start
        app.input.textarea.move_cursor(tui_textarea::CursorMove::Head);
        assert_eq!(app.input.textarea.cursor().1, 0);

        // Press Up arrow
        app.handle_key_event(key(KeyCode::Up));

        // History should open
        assert!(app.history.is_visible());
    }

    #[test]
    fn test_up_arrow_opens_history_when_input_empty() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;
        app.history.add_entry_in_memory(".history_item");

        // Press Up arrow
        app.handle_key_event(key(KeyCode::Up));

        // History should open
        assert!(app.history.is_visible());
    }

    // ========== History Cycling Tests (Ctrl+P/Ctrl+N) ==========

    #[test]
    fn test_ctrl_p_cycles_to_previous_history() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".first");
        app.history.add_entry_in_memory(".second");
        app.history.add_entry_in_memory(".third");

        // Press Ctrl+P - should load most recent (.third)
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".third");

        // Press Ctrl+P again - should load .second
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".second");

        // Press Ctrl+P again - should load .first
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".first");
    }

    #[test]
    fn test_ctrl_n_cycles_to_next_history() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".first");
        app.history.add_entry_in_memory(".second");
        app.history.add_entry_in_memory(".third");

        // Cycle back to .first
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".first");

        // Press Ctrl+N - should go forward to .second
        app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".second");

        // Press Ctrl+N again - should go to .third
        app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".third");
    }

    #[test]
    fn test_ctrl_n_at_most_recent_clears_input() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".test");

        // Cycle to history
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".test");

        // Press Ctrl+N at most recent entry - should clear
        app.handle_key_event(key_with_mods(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_typing_resets_history_cycling() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".first");
        app.history.add_entry_in_memory(".second");

        // Cycle to .second
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".second");

        // Type a character - should reset cycling
        app.handle_key_event(key(KeyCode::Char('x')));

        // Now Ctrl+P should start from beginning again
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        // Should get most recent (.second), not continue from where we were
        assert_eq!(app.query(), ".second");
    }

    #[test]
    fn test_ctrl_p_with_empty_history_does_nothing() {
        let mut app = app_with_query(".existing");
        app.input.editor_mode = EditorMode::Insert;

        // History is empty from app_with_query helper
        assert_eq!(app.history.total_count(), 0);

        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));

        // Query should be unchanged
        assert_eq!(app.query(), ".existing");
    }

    #[test]
    fn test_filter_with_no_matches() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".apple");
        app.history.add_entry_in_memory(".banana");

        // Open history
        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Type something that matches nothing
        app.handle_key_event(key(KeyCode::Char('x')));
        app.handle_key_event(key(KeyCode::Char('y')));
        app.handle_key_event(key(KeyCode::Char('z')));

        // Should have zero matches
        assert_eq!(app.history.filtered_count(), 0);
    }

    #[test]
    fn test_backspace_on_empty_search() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".test");

        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Search is empty initially
        assert_eq!(app.history.search_query(), "");

        // Press backspace - should not crash
        app.handle_key_event(key(KeyCode::Backspace));
        assert_eq!(app.history.search_query(), "");
    }
}
