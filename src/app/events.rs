use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;

use crate::editor::EditorMode;
use super::state::{App, Focus, OutputMode};

mod history;
mod results;
mod vim;

impl App {
    /// Handle events and update application state
    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // Check that it's a key press event to avoid duplicates
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle key press events
    fn handle_key_event(&mut self, key: KeyEvent) {
        // Try global keys first
        if self.handle_global_keys(key) {
            return; // Key was handled globally
        }

        // Not a global key, delegate to focused pane
        match self.focus {
            Focus::InputField => self.handle_input_field_key(key),
            Focus::ResultsPane => results::handle_results_pane_key(self, key),
        }
    }

    /// Handle global keys that work regardless of focus
    /// Returns true if key was handled, false otherwise
    fn handle_global_keys(&mut self, key: KeyEvent) -> bool {
        // Handle help popup when visible (must be first to block other keys)
        if self.help.visible {
            match key.code {
                // Close help
                KeyCode::Esc | KeyCode::F(1) => {
                    self.help.visible = false;
                    self.help.scroll.reset();
                    return true;
                }
                KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.help.visible = false;
                    self.help.scroll.reset();
                    return true;
                }
                KeyCode::Char('?') => {
                    self.help.visible = false;
                    self.help.scroll.reset();
                    return true;
                }
                // Scroll down (j, J, Down, Ctrl+D)
                KeyCode::Char('j') | KeyCode::Down => {
                    self.help.scroll.scroll_down(1);
                    return true;
                }
                KeyCode::Char('J') => {
                    self.help.scroll.scroll_down(10);
                    return true;
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.help.scroll.scroll_down(10);
                    return true;
                }
                KeyCode::PageDown => {
                    self.help.scroll.scroll_down(10);
                    return true;
                }
                // Scroll up (k, K, Up, Ctrl+U, PageUp)
                KeyCode::Char('k') | KeyCode::Up => {
                    self.help.scroll.scroll_up(1);
                    return true;
                }
                KeyCode::Char('K') => {
                    self.help.scroll.scroll_up(10);
                    return true;
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.help.scroll.scroll_up(10);
                    return true;
                }
                KeyCode::PageUp => {
                    self.help.scroll.scroll_up(10);
                    return true;
                }
                // Jump to top/bottom
                KeyCode::Char('g') | KeyCode::Home => {
                    self.help.scroll.jump_to_top();
                    return true;
                }
                KeyCode::Char('G') | KeyCode::End => {
                    self.help.scroll.jump_to_bottom();
                    return true;
                }
                _ => return true, // Block all other keys when help is visible
            }
        }

        // Ctrl+C: Exit application
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return true;
        }

        // Ctrl+E: Toggle error overlay (only if error exists)
        if key.code == KeyCode::Char('e') && key.modifiers.contains(KeyModifiers::CONTROL) {
            if self.query.result.is_err() {
                self.error_overlay_visible = !self.error_overlay_visible;
            }
            return true;
        }

        // F1: Toggle help popup (works in all modes)
        if key.code == KeyCode::F(1) {
            self.help.visible = !self.help.visible;
            return true;
        }

        // Tab: Accept autocomplete suggestion (if visible in input field)
        if key.code == KeyCode::Tab && !key.modifiers.contains(KeyModifiers::CONTROL) {
            // Check if autocomplete is visible and we're in input field
            if self.focus == Focus::InputField && self.autocomplete.is_visible() {
                // Accept the selected suggestion
                if let Some(suggestion) = self.autocomplete.selected() {
                    let text = suggestion.text.clone();
                    self.insert_autocomplete_suggestion(&text);
                }
                return true;
            }
            // Tab without autocomplete does nothing (don't interfere with textarea)
            return false;
        }

        // Shift+Tab: Switch focus between panes
        if key.code == KeyCode::BackTab {
            // Close any open popups when switching focus
            self.history.close();
            self.autocomplete.hide();
            self.focus = match self.focus {
                Focus::InputField => Focus::ResultsPane,
                Focus::ResultsPane => Focus::InputField,
            };
            return true;
        }

        // q (without Ctrl): Exit application without output
        // - In Normal/Operator mode: always quit (VIM behavior)
        // - In Insert mode: only quit if focus is on ResultsPane (not editing text)
        if key.code == KeyCode::Char('q')
            && !key.modifiers.contains(KeyModifiers::CONTROL)
            && (self.input.editor_mode != EditorMode::Insert || self.focus == Focus::ResultsPane)
        {
            self.should_quit = true;
            return true;
        }

        // Shift+Enter / Alt+Enter / Ctrl+Q: Exit and output query only
        // Note: Some terminals (e.g., macOS Terminal.app) don't properly send
        // Shift+Enter or Alt+Enter, so Ctrl+Q is provided as a universal fallback.
        if (key.code == KeyCode::Enter
            && (key.modifiers.contains(KeyModifiers::SHIFT)
                || key.modifiers.contains(KeyModifiers::ALT)))
            || (key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.output_mode = Some(OutputMode::Query);
            self.should_quit = true;
            return true;
        }

        // Enter: Exit and output filtered results (but not when history popup is open)
        if key.code == KeyCode::Enter && !self.history.is_visible() {
            // Save successful queries to history
            if self.query.result.is_ok() && !self.query().is_empty() {
                let query = self.query().to_string();
                self.history.add_entry(&query);
            }
            self.output_mode = Some(OutputMode::Results);
            self.should_quit = true;
            return true;
        }

        false // Key not handled
    }

    /// Handle keys when Input field is focused
    fn handle_input_field_key(&mut self, key: KeyEvent) {
        // Handle history popup when visible
        if self.history.is_visible() {
            history::handle_history_popup_key(self, key);
            return;
        }

        // Handle ESC - close autocomplete or switch to Normal mode
        if key.code == KeyCode::Esc {
            if self.autocomplete.is_visible() {
                self.autocomplete.hide();
                return;
            }
            self.input.editor_mode = EditorMode::Normal;
            return;
        }

        // Handle autocomplete navigation (in Insert mode only)
        if self.input.editor_mode == EditorMode::Insert && self.autocomplete.is_visible() {
            match key.code {
                KeyCode::Down => {
                    self.autocomplete.select_next();
                    return;
                }
                KeyCode::Up => {
                    self.autocomplete.select_previous();
                    return;
                }
                _ => {}
            }
        }

        // Handle history trigger (in Insert mode only)
        if self.input.editor_mode == EditorMode::Insert {
            let cursor_col = self.input.textarea.cursor().1;
            let query_empty = self.query().is_empty();

            // Ctrl+P: Cycle to previous (older) history entry
            if key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
                if let Some(entry) = self.history.cycle_previous() {
                    self.replace_query_with(&entry);
                }
                return;
            }

            // Ctrl+N: Cycle to next (newer) history entry
            if key.code == KeyCode::Char('n') && key.modifiers.contains(KeyModifiers::CONTROL) {
                if let Some(entry) = self.history.cycle_next() {
                    self.replace_query_with(&entry);
                } else {
                    // At most recent, clear the input
                    self.input.textarea.delete_line_by_head();
                    self.input.textarea.delete_line_by_end();
                    vim::execute_query(self);
                }
                return;
            }

            // Ctrl+R: Open history
            if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.open_history_popup();
                return;
            }

            // Up arrow: Open history if input empty or cursor at start
            if key.code == KeyCode::Up && (query_empty || cursor_col == 0) {
                self.open_history_popup();
                return;
            }
        }

        // Handle input based on current mode
        match self.input.editor_mode {
            EditorMode::Insert => vim::handle_insert_mode_key(self, key),
            EditorMode::Normal => vim::handle_normal_mode_key(self, key),
            EditorMode::Operator(_) => vim::handle_operator_mode_key(self, key),
        }
    }


    /// Replace the current query with the given text
    fn replace_query_with(&mut self, text: &str) {
        self.input.textarea.delete_line_by_head();
        self.input.textarea.delete_line_by_end();
        self.input.textarea.insert_str(text);
        vim::execute_query(self);
    }

    /// Open the history popup with current query as initial search
    fn open_history_popup(&mut self) {
        // Don't open if history is empty
        if self.history.total_count() == 0 {
            return;
        }

        let query = self.query().to_string();
        let initial_query = if query.is_empty() {
            None
        } else {
            Some(query.as_str())
        };
        self.history.open(initial_query);
        self.autocomplete.hide();
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tui_textarea::CursorMove;

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
        use crate::history::HistoryState;

        let mut app = App::new(TEST_JSON.to_string());
        app.input.textarea.insert_str(query);
        // Use empty in-memory history for all tests to prevent disk writes
        app.history = HistoryState::empty();
        app
    }

    // Helper to move cursor to specific position by text content
    fn move_cursor_to_position(app: &mut App, target_pos: usize) {
        app.input.textarea.move_cursor(CursorMove::Head);
        for _ in 0..target_pos {
            app.input.textarea.move_cursor(CursorMove::Forward);
        }
    }

    // ========== Error Overlay Tests ==========

    #[test]
    fn test_error_overlay_initializes_hidden() {
        let app = App::new(TEST_JSON.to_string());
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_toggles_error_overlay_when_error_exists() {
        let mut app = App::new(TEST_JSON.to_string());
        app.input.editor_mode = EditorMode::Insert;

        // Type an invalid query (| is invalid jq syntax)
        app.handle_key_event(key(KeyCode::Char('|')));

        // Should have an error now
        assert!(app.query.result.is_err());
        assert!(!app.error_overlay_visible); // Initially hidden

        // Press Ctrl+E to show overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);

        // Press Ctrl+E again to hide overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_does_nothing_when_no_error() {
        let mut app = App::new(TEST_JSON.to_string());
        // Initial query "." should succeed
        assert!(app.query.result.is_ok());
        assert!(!app.error_overlay_visible);

        // Press Ctrl+E (should do nothing since no error)
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(!app.error_overlay_visible); // Should remain hidden
    }

    #[test]
    fn test_error_overlay_hides_on_query_change() {
        let mut app = App::new(TEST_JSON.to_string());
        app.input.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        assert!(app.query.result.is_err());

        // Show error overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);

        // Change query by pressing backspace to delete the invalid character
        app.handle_key_event(key(KeyCode::Backspace));

        // Overlay should auto-hide after query change
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_error_overlay_hides_on_query_change_in_normal_mode() {
        let mut app = App::new(TEST_JSON.to_string());
        app.input.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        assert!(app.query.result.is_err());

        // Show error overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);

        // Switch to Normal mode and delete the character
        app.handle_key_event(key(KeyCode::Esc));
        app.input.textarea.move_cursor(CursorMove::Head);
        app.handle_key_event(key(KeyCode::Char('x')));

        // Overlay should auto-hide after query change
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_works_in_normal_mode() {
        let mut app = App::new(TEST_JSON.to_string());
        app.input.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        assert!(app.query.result.is_err());

        // Switch to Normal mode
        app.handle_key_event(key(KeyCode::Esc));
        assert_eq!(app.input.editor_mode, EditorMode::Normal);

        // Press Ctrl+E in Normal mode
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_works_when_results_pane_focused() {
        let mut app = App::new(TEST_JSON.to_string());
        app.input.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        assert!(app.query.result.is_err());

        // Switch focus to results pane
        app.handle_key_event(key(KeyCode::BackTab));
        assert_eq!(app.focus, Focus::ResultsPane);

        // Press Ctrl+E while results pane is focused
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);
    }

    // ========== Global Key Handler Tests ==========

    #[test]
    fn test_ctrl_c_sets_quit_flag() {
        let mut app = app_with_query(".");

        app.handle_key_event(key_with_mods(KeyCode::Char('c'), KeyModifiers::CONTROL));

        assert!(app.should_quit);
    }

    #[test]
    fn test_q_sets_quit_flag_in_normal_mode() {
        let mut app = app_with_query(".");
        app.input.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('q')));

        assert!(app.should_quit);
    }

    #[test]
    fn test_q_does_not_quit_in_insert_mode() {
        let mut app = app_with_query(".");
        app.input.editor_mode = EditorMode::Insert;

        app.handle_key_event(key(KeyCode::Char('q')));

        // Should NOT quit - 'q' should be typed instead
        assert!(!app.should_quit);
        assert_eq!(app.query(), ".q");
    }

    #[test]
    fn test_enter_sets_results_output_mode() {
        let mut app = app_with_query(".");

        app.handle_key_event(key(KeyCode::Enter));

        assert_eq!(app.output_mode, Some(OutputMode::Results));
        assert!(app.should_quit);
    }

    #[test]
    fn test_shift_enter_sets_query_output_mode() {
        let mut app = app_with_query(".");

        app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::SHIFT));

        assert_eq!(app.output_mode, Some(OutputMode::Query));
        assert!(app.should_quit);
    }

    #[test]
    fn test_alt_enter_sets_query_output_mode() {
        let mut app = app_with_query(".");

        // Some terminals send Alt+Enter instead of Shift+Enter
        app.handle_key_event(key_with_mods(KeyCode::Enter, KeyModifiers::ALT));

        assert_eq!(app.output_mode, Some(OutputMode::Query));
        assert!(app.should_quit);
    }

    #[test]
    fn test_shift_tab_switches_focus_to_results() {
        let mut app = app_with_query(".");
        app.focus = Focus::InputField;

        app.handle_key_event(key(KeyCode::BackTab));

        assert_eq!(app.focus, Focus::ResultsPane);
    }

    #[test]
    fn test_shift_tab_switches_focus_to_input() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;

        app.handle_key_event(key(KeyCode::BackTab));

        assert_eq!(app.focus, Focus::InputField);
    }

    #[test]
    fn test_global_keys_work_regardless_of_focus() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;

        app.handle_key_event(key_with_mods(KeyCode::Char('c'), KeyModifiers::CONTROL));

        // Ctrl+C should work even when results pane is focused
        assert!(app.should_quit);
    }

    #[test]
    fn test_insert_mode_text_input_updates_query() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        // Simulate typing a character
        app.handle_key_event(key(KeyCode::Char('.')));

        assert_eq!(app.query(), ".");
    }

    #[test]
    fn test_query_execution_resets_scroll() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;
        app.results_scroll.offset =50;

        // Insert text which should trigger query execution
        app.handle_key_event(key(KeyCode::Char('.')));

        // Scroll should be reset when query changes
        assert_eq!(app.results_scroll.offset, 0);
    }

    // ========== UTF-8 Edge Case Tests ==========

    #[test]
    fn test_history_with_emoji() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".emoji_field 🚀");

        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".emoji_field 🚀");
    }

    #[test]
    fn test_history_with_multibyte_chars() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".café | .naïve");

        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".café | .naïve");
    }

    #[test]
    fn test_history_search_with_unicode() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".café");
        app.history.add_entry_in_memory(".coffee");

        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Search for unicode
        app.handle_key_event(key(KeyCode::Char('c')));
        app.handle_key_event(key(KeyCode::Char('a')));
        app.handle_key_event(key(KeyCode::Char('f')));

        // Should filter to .café
        assert_eq!(app.history.filtered_count(), 1);
    }

    // ========== Boundary Condition Tests ==========

    #[test]
    fn test_cycling_stops_at_oldest() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".first");

        // Cycle to first entry
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".first");

        // Spam Ctrl+P - should stay at .first
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".first");
    }

    #[test]
    fn test_history_popup_with_single_entry() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".single");

        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert!(app.history.is_visible());

        // Should wrap on navigation
        app.handle_key_event(key(KeyCode::Up));
        assert_eq!(app.history.selected_index(), 0);

        app.handle_key_event(key(KeyCode::Down));
        assert_eq!(app.history.selected_index(), 0);
    }

    #[test]
    fn test_filter_with_no_matches() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".foo");
        app.history.add_entry_in_memory(".bar");

        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Search for something that doesn't match
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

    // ========== 'q' key behavior tests ==========

    #[test]
    fn test_q_quits_in_results_pane_insert_mode() {
        let mut app = app_with_query("");
        app.focus = Focus::ResultsPane;
        app.input.editor_mode = EditorMode::Insert;

        // 'q' should quit even when editor is in Insert mode
        // because we're in ResultsPane (not editing text)
        app.handle_key_event(key(KeyCode::Char('q')));

        assert!(app.should_quit);
    }

    #[test]
    fn test_q_does_not_quit_in_input_field_insert_mode() {
        let mut app = app_with_query("");
        app.focus = Focus::InputField;
        app.input.editor_mode = EditorMode::Insert;

        // 'q' should NOT quit when in InputField with Insert mode
        // (user is typing)
        app.handle_key_event(key(KeyCode::Char('q')));

        assert!(!app.should_quit);
        // The 'q' should be inserted into the query
        assert!(app.query().contains('q'));
    }

    #[test]
    fn test_q_quits_in_input_field_normal_mode() {
        let mut app = app_with_query("");
        app.focus = Focus::InputField;
        app.input.editor_mode = EditorMode::Normal;

        // 'q' should quit when in Normal mode
        app.handle_key_event(key(KeyCode::Char('q')));

        assert!(app.should_quit);
    }

    #[test]
    fn test_q_quits_in_results_pane_normal_mode() {
        let mut app = app_with_query("");
        app.focus = Focus::ResultsPane;
        app.input.editor_mode = EditorMode::Normal;

        // 'q' should quit when in ResultsPane Normal mode
        app.handle_key_event(key(KeyCode::Char('q')));

        assert!(app.should_quit);
    }

    #[test]
    fn test_focus_switch_preserves_editor_mode() {
        let mut app = app_with_query("");
        app.focus = Focus::InputField;
        app.input.editor_mode = EditorMode::Insert;

        // Switch to ResultsPane
        app.handle_key_event(key(KeyCode::BackTab));

        // Editor mode should still be Insert
        assert_eq!(app.focus, Focus::ResultsPane);
        assert_eq!(app.input.editor_mode, EditorMode::Insert);

        // 'q' should quit in ResultsPane even with Insert mode
        app.handle_key_event(key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    // ========== Help Popup Tests ==========

    #[test]
    fn test_help_popup_initializes_hidden() {
        let app = App::new(TEST_JSON.to_string());
        assert!(!app.help.visible);
    }

    #[test]
    fn test_f1_toggles_help_popup() {
        let mut app = app_with_query(".");
        assert!(!app.help.visible);

        app.handle_key_event(key(KeyCode::F(1)));
        assert!(app.help.visible);

        app.handle_key_event(key(KeyCode::F(1)));
        assert!(!app.help.visible);
    }

    #[test]
    fn test_question_mark_toggles_help_in_normal_mode() {
        let mut app = app_with_query(".");
        app.input.editor_mode = EditorMode::Normal;
        app.focus = Focus::InputField;

        app.handle_key_event(key(KeyCode::Char('?')));
        assert!(app.help.visible);

        app.handle_key_event(key(KeyCode::Char('?')));
        assert!(!app.help.visible);
    }

    #[test]
    fn test_question_mark_does_not_toggle_help_in_insert_mode() {
        let mut app = app_with_query("");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        app.handle_key_event(key(KeyCode::Char('?')));
        // Should type '?' not toggle help
        assert!(!app.help.visible);
        assert!(app.query().contains('?'));
    }

    #[test]
    fn test_esc_closes_help_popup() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        app.handle_key_event(key(KeyCode::Esc));
        assert!(!app.help.visible);
    }

    #[test]
    fn test_q_closes_help_popup() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        app.handle_key_event(key(KeyCode::Char('q')));
        assert!(!app.help.visible);
    }

    #[test]
    fn test_help_popup_blocks_other_keys() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.input.editor_mode = EditorMode::Insert;

        // Try to type - should be blocked
        app.handle_key_event(key(KeyCode::Char('x')));
        assert!(!app.query().contains('x'));
        assert!(app.help.visible);
    }

    #[test]
    fn test_f1_works_in_insert_mode() {
        let mut app = app_with_query(".");
        app.input.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        app.handle_key_event(key(KeyCode::F(1)));
        assert!(app.help.visible);
    }

    #[test]
    fn test_help_popup_scroll_j_scrolls_down() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content (48 lines + padding, viewport 20)
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset =0;

        app.handle_key_event(key(KeyCode::Char('j')));
        assert_eq!(app.help.scroll.offset, 1);
    }

    #[test]
    fn test_help_popup_scroll_k_scrolls_up() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =5;

        app.handle_key_event(key(KeyCode::Char('k')));
        assert_eq!(app.help.scroll.offset, 4);
    }

    #[test]
    fn test_help_popup_scroll_down_arrow() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset =0;

        app.handle_key_event(key(KeyCode::Down));
        assert_eq!(app.help.scroll.offset, 1);
    }

    #[test]
    fn test_help_popup_scroll_up_arrow() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =5;

        app.handle_key_event(key(KeyCode::Up));
        assert_eq!(app.help.scroll.offset, 4);
    }

    #[test]
    fn test_help_popup_scroll_capital_j_scrolls_10() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset =0;

        app.handle_key_event(key(KeyCode::Char('J')));
        assert_eq!(app.help.scroll.offset, 10);
    }

    #[test]
    fn test_help_popup_scroll_capital_k_scrolls_10() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =15;

        app.handle_key_event(key(KeyCode::Char('K')));
        assert_eq!(app.help.scroll.offset, 5);
    }

    #[test]
    fn test_help_popup_scroll_ctrl_d() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset =0;

        app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.help.scroll.offset, 10);
    }

    #[test]
    fn test_help_popup_scroll_ctrl_u() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =15;

        app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(app.help.scroll.offset, 5);
    }

    #[test]
    fn test_help_popup_scroll_g_jumps_to_top() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =20;

        app.handle_key_event(key(KeyCode::Char('g')));
        assert_eq!(app.help.scroll.offset, 0);
    }

    #[test]
    fn test_help_popup_scroll_capital_g_jumps_to_bottom() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset =0;

        app.handle_key_event(key(KeyCode::Char('G')));
        assert_eq!(app.help.scroll.offset, app.help.scroll.max_offset);
    }

    #[test]
    fn test_help_popup_scroll_k_saturates_at_zero() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =0;

        app.handle_key_event(key(KeyCode::Char('k')));
        assert_eq!(app.help.scroll.offset, 0);
    }

    #[test]
    fn test_help_popup_close_resets_scroll() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =10;

        app.handle_key_event(key(KeyCode::Esc));
        assert!(!app.help.visible);
        assert_eq!(app.help.scroll.offset, 0);
    }

    #[test]
    fn test_help_popup_scroll_page_down() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset =0;

        app.handle_key_event(key(KeyCode::PageDown));
        assert_eq!(app.help.scroll.offset, 10);
    }

    #[test]
    fn test_help_popup_scroll_page_up() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =15;

        app.handle_key_event(key(KeyCode::PageUp));
        assert_eq!(app.help.scroll.offset, 5);
    }

    #[test]
    fn test_help_popup_scroll_home_jumps_to_top() {
        let mut app = app_with_query(".");
        app.help.visible = true;
        app.help.scroll.offset =20;

        app.handle_key_event(key(KeyCode::Home));
        assert_eq!(app.help.scroll.offset, 0);
    }

    #[test]
    fn test_help_popup_scroll_end_jumps_to_bottom() {
        let mut app = app_with_query(".");
        app.help.visible = true;

        // Set up bounds for help content
        app.help.scroll.update_bounds(60, 20);
        app.help.scroll.offset =0;

        app.handle_key_event(key(KeyCode::End));
        assert_eq!(app.help.scroll.offset, app.help.scroll.max_offset);
    }
}
