use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use tui_textarea::CursorMove;

use crate::editor::EditorMode;
use super::state::{App, Focus, OutputMode};

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
            Focus::ResultsPane => self.handle_results_pane_key(key),
        }
    }

    /// Handle global keys that work regardless of focus
    /// Returns true if key was handled, false otherwise
    fn handle_global_keys(&mut self, key: KeyEvent) -> bool {
        // Ctrl+C: Exit application
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return true;
        }

        // Ctrl+E: Toggle error overlay (only if error exists)
        if key.code == KeyCode::Char('e') && key.modifiers.contains(KeyModifiers::CONTROL) {
            if self.query_result.is_err() {
                self.error_overlay_visible = !self.error_overlay_visible;
            }
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

        // q (without Ctrl): Exit application without output (not in insert mode)
        if key.code == KeyCode::Char('q')
            && !key.modifiers.contains(KeyModifiers::CONTROL)
            && self.editor_mode != EditorMode::Insert
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
            if self.query_result.is_ok() && !self.query().is_empty() {
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
            self.handle_history_popup_key(key);
            return;
        }

        // Handle ESC - close autocomplete or switch to Normal mode
        if key.code == KeyCode::Esc {
            if self.autocomplete.is_visible() {
                self.autocomplete.hide();
                return;
            }
            self.editor_mode = EditorMode::Normal;
            return;
        }

        // Handle autocomplete navigation (in Insert mode only)
        if self.editor_mode == EditorMode::Insert && self.autocomplete.is_visible() {
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
        if self.editor_mode == EditorMode::Insert {
            let cursor_col = self.textarea.cursor().1;
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
                    self.textarea.delete_line_by_head();
                    self.textarea.delete_line_by_end();
                    self.execute_query();
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
        match self.editor_mode {
            EditorMode::Insert => self.handle_insert_mode_key(key),
            EditorMode::Normal => self.handle_normal_mode_key(key),
            EditorMode::Operator(_) => self.handle_operator_mode_key(key),
        }
    }

    /// Handle keys in Insert mode
    fn handle_insert_mode_key(&mut self, key: KeyEvent) {
        // Use textarea's built-in input handling
        let content_changed = self.textarea.input(key);

        // Execute query on every keystroke that changes content
        if content_changed {
            // Reset history cycling when user types
            self.history.reset_cycling();

            let query = self.textarea.lines()[0].as_ref();
            self.query_result = self.executor.execute(query);

            // Cache successful results
            if let Ok(result) = &self.query_result {
                self.last_successful_result = Some(result.clone());
            }

            // Reset scroll when query changes
            self.results_scroll = 0;
            self.error_overlay_visible = false; // Auto-hide error overlay on query change
        }

        // Update autocomplete suggestions after any input
        self.update_autocomplete();
    }

    /// Handle keys in Normal mode (VIM navigation and commands)
    fn handle_normal_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            // Basic cursor movement (h/l)
            KeyCode::Char('h') | KeyCode::Left => {
                self.textarea.move_cursor(CursorMove::Back);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.textarea.move_cursor(CursorMove::Forward);
            }

            // Line extent movement (0/$)
            KeyCode::Char('0') | KeyCode::Home => {
                self.textarea.move_cursor(CursorMove::Head);
            }
            KeyCode::Char('$') | KeyCode::End => {
                self.textarea.move_cursor(CursorMove::End);
            }

            // Word movement (w/b/e)
            KeyCode::Char('w') => {
                self.textarea.move_cursor(CursorMove::WordForward);
            }
            KeyCode::Char('b') => {
                self.textarea.move_cursor(CursorMove::WordBack);
            }
            KeyCode::Char('e') => {
                self.textarea.move_cursor(CursorMove::WordEnd);
            }

            // Enter Insert mode commands
            KeyCode::Char('i') => {
                // i - Insert at cursor
                self.editor_mode = EditorMode::Insert;
            }
            KeyCode::Char('a') => {
                // a - Append (insert after cursor)
                self.textarea.move_cursor(CursorMove::Forward);
                self.editor_mode = EditorMode::Insert;
            }
            KeyCode::Char('I') => {
                // I - Insert at line start
                self.textarea.move_cursor(CursorMove::Head);
                self.editor_mode = EditorMode::Insert;
            }
            KeyCode::Char('A') => {
                // A - Append at line end
                self.textarea.move_cursor(CursorMove::End);
                self.editor_mode = EditorMode::Insert;
            }

            // Simple delete operations
            KeyCode::Char('x') => {
                // x - Delete character at cursor
                self.textarea.delete_next_char();
                self.execute_query();
            }
            KeyCode::Char('X') => {
                // X - Delete character before cursor
                self.textarea.delete_char();
                self.execute_query();
            }

            // Delete/Change to end of line
            KeyCode::Char('D') => {
                // D - Delete to end of line (like d$)
                self.textarea.delete_line_by_end();
                self.execute_query();
            }
            KeyCode::Char('C') => {
                // C - Change to end of line (like c$)
                self.textarea.delete_line_by_end();
                self.textarea.cancel_selection();
                self.editor_mode = EditorMode::Insert;
                self.execute_query();
            }

            // Operators - enter Operator mode
            KeyCode::Char('d') => {
                // d - Delete operator (wait for motion)
                self.editor_mode = EditorMode::Operator('d');
                self.textarea.start_selection();
            }
            KeyCode::Char('c') => {
                // c - Change operator (delete then insert)
                self.editor_mode = EditorMode::Operator('c');
                self.textarea.start_selection();
            }

            // Undo/Redo
            KeyCode::Char('u') => {
                // u - Undo
                self.textarea.undo();
                self.execute_query();
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+r - Redo
                self.textarea.redo();
                self.execute_query();
            }

            _ => {
                // Other VIM commands not yet implemented
            }
        }
    }

    /// Handle keys in Operator mode (waiting for motion after d/c)
    fn handle_operator_mode_key(&mut self, key: KeyEvent) {
        let operator = match self.editor_mode {
            EditorMode::Operator(op) => op,
            _ => return, // Should never happen
        };

        // Check for double operator (dd, cc)
        if key.code == KeyCode::Char(operator) {
            // dd or cc - delete entire line
            self.textarea.delete_line_by_head();
            self.textarea.delete_line_by_end();
            self.editor_mode = if operator == 'c' {
                EditorMode::Insert
            } else {
                EditorMode::Normal
            };
            self.execute_query();
            return;
        }

        // Apply operator with motion
        let motion_applied = match key.code {
            // Word motions
            KeyCode::Char('w') => {
                self.textarea.move_cursor(CursorMove::WordForward);
                true
            }
            KeyCode::Char('b') => {
                self.textarea.move_cursor(CursorMove::WordBack);
                true
            }
            KeyCode::Char('e') => {
                self.textarea.move_cursor(CursorMove::WordEnd);
                self.textarea.move_cursor(CursorMove::Forward); // Include char at cursor
                true
            }

            // Line extent motions
            KeyCode::Char('0') | KeyCode::Home => {
                self.textarea.move_cursor(CursorMove::Head);
                true
            }
            KeyCode::Char('$') | KeyCode::End => {
                self.textarea.move_cursor(CursorMove::End);
                true
            }

            // Character motions
            KeyCode::Char('h') | KeyCode::Left => {
                self.textarea.move_cursor(CursorMove::Back);
                true
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.textarea.move_cursor(CursorMove::Forward);
                true
            }

            _ => false,
        };

        if motion_applied {
            // Execute the operator
            match operator {
                'd' => {
                    // Delete - cut and stay in Normal mode
                    self.textarea.cut();
                    self.editor_mode = EditorMode::Normal;
                }
                'c' => {
                    // Change - cut and enter Insert mode
                    self.textarea.cut();
                    self.editor_mode = EditorMode::Insert;
                }
                _ => {
                    self.textarea.cancel_selection();
                    self.editor_mode = EditorMode::Normal;
                }
            }
            self.execute_query();
        } else {
            // Invalid motion or ESC - cancel operator
            self.textarea.cancel_selection();
            self.editor_mode = EditorMode::Normal;
        }
    }

    /// Execute current query and update results
    fn execute_query(&mut self) {
        let query = self.textarea.lines()[0].as_ref();
        self.query_result = self.executor.execute(query);

        // Cache successful results
        if let Ok(result) = &self.query_result {
            self.last_successful_result = Some(result.clone());
        }

        self.results_scroll = 0;
        self.error_overlay_visible = false; // Auto-hide error overlay on query change
    }

    /// Replace the current query with the given text
    fn replace_query_with(&mut self, text: &str) {
        self.textarea.delete_line_by_head();
        self.textarea.delete_line_by_end();
        self.textarea.insert_str(text);
        self.execute_query();
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

    /// Handle keys when history popup is visible
    fn handle_history_popup_key(&mut self, key: KeyEvent) {
        match key.code {
            // Navigation (reversed because display is reversed - most recent at bottom)
            KeyCode::Up => {
                self.history.select_next(); // Move to older entries (visually up)
            }
            KeyCode::Down => {
                self.history.select_previous(); // Move to newer entries (visually down)
            }

            // Select and close
            KeyCode::Enter | KeyCode::Tab => {
                if let Some(entry) = self.history.selected_entry() {
                    let entry = entry.to_string();
                    self.replace_query_with(&entry);
                }
                self.history.close();
            }

            // Cancel
            KeyCode::Esc => {
                self.history.close();
            }

            // Search input
            KeyCode::Backspace => {
                self.history.pop_search_char();
            }
            KeyCode::Char(c) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
                {
                    self.history.push_search_char(c);
                }
            }

            _ => {}
        }
    }

    /// Handle keys when Results pane is focused
    fn handle_results_pane_key(&mut self, key: KeyEvent) {
        match key.code {
            // Basic line scrolling (1 line)
            KeyCode::Up | KeyCode::Char('k') => {
                self.results_scroll = self.results_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.results_scroll = self.results_scroll.saturating_add(1);
            }

            // 10 line scrolling
            KeyCode::Char('K') => {
                self.results_scroll = self.results_scroll.saturating_sub(10);
            }
            KeyCode::Char('J') => {
                self.results_scroll = self.results_scroll.saturating_add(10);
            }

            // Jump to top
            KeyCode::Home | KeyCode::Char('g') => {
                self.results_scroll = 0;
            }

            // Jump to bottom
            KeyCode::Char('G') => {
                self.results_scroll = self.max_scroll();
            }

            // Half page scrolling up
            KeyCode::PageUp => {
                let half_page = self.results_viewport_height / 2;
                self.results_scroll = self.results_scroll.saturating_sub(half_page);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let half_page = self.results_viewport_height / 2;
                self.results_scroll = self.results_scroll.saturating_sub(half_page);
            }

            // Half page scrolling down
            KeyCode::PageDown => {
                let half_page = self.results_viewport_height / 2;
                self.results_scroll = self.results_scroll.saturating_add(half_page);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let half_page = self.results_viewport_height / 2;
                self.results_scroll = self.results_scroll.saturating_add(half_page);
            }

            _ => {
                // Ignore other keys in Results pane
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocomplete::{Suggestion, SuggestionType};

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
        app.textarea.insert_str(query);
        // Use empty in-memory history for all tests to prevent disk writes
        app.history = HistoryState::empty();
        app
    }

    // Helper to move cursor to specific position by text content
    fn move_cursor_to_position(app: &mut App, target_pos: usize) {
        app.textarea.move_cursor(CursorMove::Head);
        for _ in 0..target_pos {
            app.textarea.move_cursor(CursorMove::Forward);
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
        app.editor_mode = EditorMode::Insert;

        // Type an invalid query (| is invalid jq syntax)
        app.handle_key_event(key(KeyCode::Char('|')));

        // Should have an error now
        assert!(app.query_result.is_err());
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
        assert!(app.query_result.is_ok());
        assert!(!app.error_overlay_visible);

        // Press Ctrl+E (should do nothing since no error)
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(!app.error_overlay_visible); // Should remain hidden
    }

    #[test]
    fn test_error_overlay_hides_on_query_change() {
        let mut app = App::new(TEST_JSON.to_string());
        app.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        assert!(app.query_result.is_err());

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
        app.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        assert!(app.query_result.is_err());

        // Show error overlay
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);

        // Switch to Normal mode and delete the character
        app.handle_key_event(key(KeyCode::Esc));
        app.textarea.move_cursor(CursorMove::Head);
        app.handle_key_event(key(KeyCode::Char('x')));

        // Overlay should auto-hide after query change
        assert!(!app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_works_in_normal_mode() {
        let mut app = App::new(TEST_JSON.to_string());
        app.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        assert!(app.query_result.is_err());

        // Switch to Normal mode
        app.handle_key_event(key(KeyCode::Esc));
        assert_eq!(app.editor_mode, EditorMode::Normal);

        // Press Ctrl+E in Normal mode
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);
    }

    #[test]
    fn test_ctrl_e_works_when_results_pane_focused() {
        let mut app = App::new(TEST_JSON.to_string());
        app.editor_mode = EditorMode::Insert;

        // Type invalid query
        app.handle_key_event(key(KeyCode::Char('|')));
        assert!(app.query_result.is_err());

        // Switch focus to results pane
        app.handle_key_event(key(KeyCode::BackTab));
        assert_eq!(app.focus, Focus::ResultsPane);

        // Press Ctrl+E while results pane is focused
        app.handle_key_event(key_with_mods(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert!(app.error_overlay_visible);
    }

    // ========== VIM Operator Tests ==========

    #[test]
    fn test_operator_dw_deletes_word_from_start() {
        let mut app = app_with_query(".name.first");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;

        // Type 'd' to enter Operator mode
        app.handle_key_event(key(KeyCode::Char('d')));
        assert!(matches!(app.editor_mode, EditorMode::Operator('d')));

        // Type 'w' to delete word
        app.handle_key_event(key(KeyCode::Char('w')));
        // The selection behavior deletes from cursor to end of word motion
        assert!(app.query().contains("first"));
        assert_eq!(app.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_operator_dw_deletes_word_from_middle() {
        let mut app = app_with_query(".name.first");
        // Move to position 5 (at the dot before "first")
        move_cursor_to_position(&mut app, 5);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('w')));
        // Verify something was deleted
        assert!(app.query().len() < ".name.first".len());
        assert!(app.query().starts_with(".name"));
    }

    #[test]
    fn test_operator_db_deletes_word_backward() {
        let mut app = app_with_query(".name.first");
        app.textarea.move_cursor(CursorMove::End);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('b')));

        // Should delete ".first" backwards
        assert!(app.query().starts_with(".name"));
    }

    #[test]
    fn test_operator_de_deletes_to_word_end() {
        let mut app = app_with_query(".name.first");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('e')));

        // Should delete to end of first word (including the character at cursor)
        assert!(app.query().contains("first"));
    }

    #[test]
    fn test_operator_d_dollar_deletes_to_end_of_line() {
        let mut app = app_with_query(".name.first");
        // Move to position 5 (after ".name")
        move_cursor_to_position(&mut app, 5);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('$')));

        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_operator_d0_deletes_to_start_of_line() {
        let mut app = app_with_query(".name.first");
        // Move to middle of text
        move_cursor_to_position(&mut app, 6);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('0')));

        assert!(app.query().ends_with("first"));
    }

    #[test]
    fn test_operator_dd_deletes_entire_line() {
        let mut app = app_with_query(".name.first");
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('d')));

        assert_eq!(app.query(), "");
        assert_eq!(app.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_operator_cw_changes_word() {
        let mut app = app_with_query(".name.first");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('c')));
        app.handle_key_event(key(KeyCode::Char('w')));

        // Should delete word and enter Insert mode
        assert!(app.query().contains("first"));
        assert_eq!(app.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn test_operator_cc_changes_entire_line() {
        let mut app = app_with_query(".name.first");
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('c')));
        app.handle_key_event(key(KeyCode::Char('c')));

        assert_eq!(app.query(), "");
        assert_eq!(app.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn test_operator_invalid_motion_cancels() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Normal;
        let original_query = app.query().to_string();

        app.handle_key_event(key(KeyCode::Char('d')));
        assert!(matches!(app.editor_mode, EditorMode::Operator('d')));

        // Press invalid motion key (z is not a valid motion)
        app.handle_key_event(key(KeyCode::Char('z')));

        // Should cancel operator and return to Normal mode without changing text
        assert_eq!(app.editor_mode, EditorMode::Normal);
        assert_eq!(app.query(), original_query);
    }

    #[test]
    fn test_escape_in_operator_mode_cancels_operator() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Normal;
        let original_query = app.query().to_string();

        // Enter operator mode
        app.handle_key_event(key(KeyCode::Char('d')));
        assert!(matches!(app.editor_mode, EditorMode::Operator('d')));

        // Press Escape - should NOT go to Insert mode, should cancel operator
        app.handle_key_event(key(KeyCode::Esc));

        // Should return to Normal mode and preserve text
        assert_eq!(app.editor_mode, EditorMode::Normal);
        assert_eq!(app.query(), original_query);
    }

    #[test]
    fn test_operator_dh_deletes_character_backward() {
        let mut app = app_with_query(".name");
        app.textarea.move_cursor(CursorMove::End);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('h')));

        // Should delete one character backward
        assert!(app.query().len() < 5);
        assert_eq!(app.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_operator_dl_deletes_character_forward() {
        let mut app = app_with_query(".name");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));
        app.handle_key_event(key(KeyCode::Char('l')));

        // Should delete one character forward
        assert!(app.query().len() < 5);
        assert_eq!(app.editor_mode, EditorMode::Normal);
    }

    // ========== Mode Transition Tests ==========

    #[test]
    fn test_escape_from_insert_to_normal() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Insert;

        app.handle_key_event(key(KeyCode::Esc));

        assert_eq!(app.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_i_enters_insert_mode_at_cursor() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Normal;
        app.textarea.move_cursor(CursorMove::Head);
        let cursor_before = app.textarea.cursor();

        app.handle_key_event(key(KeyCode::Char('i')));

        assert_eq!(app.editor_mode, EditorMode::Insert);
        // Cursor should remain at same position
        assert_eq!(app.textarea.cursor(), cursor_before);
    }

    #[test]
    fn test_a_enters_insert_mode_after_cursor() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Normal;
        app.textarea.move_cursor(CursorMove::Head);
        let cursor_col_before = app.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('a')));

        assert_eq!(app.editor_mode, EditorMode::Insert);
        // Cursor should move forward by one
        assert_eq!(app.textarea.cursor().1, cursor_col_before + 1);
    }

    #[test]
    fn test_capital_i_enters_insert_at_line_start() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Normal;
        app.textarea.move_cursor(CursorMove::End);

        app.handle_key_event(key(KeyCode::Char('I')));

        assert_eq!(app.editor_mode, EditorMode::Insert);
        assert_eq!(app.textarea.cursor().1, 0);
    }

    #[test]
    fn test_capital_a_enters_insert_at_line_end() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Normal;
        app.textarea.move_cursor(CursorMove::Head);

        app.handle_key_event(key(KeyCode::Char('A')));

        assert_eq!(app.editor_mode, EditorMode::Insert);
        assert_eq!(app.textarea.cursor().1, 5); // Should be at end of ".name"
    }

    #[test]
    fn test_d_enters_operator_mode() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('d')));

        assert!(matches!(app.editor_mode, EditorMode::Operator('d')));
    }

    #[test]
    fn test_c_enters_operator_mode() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('c')));

        assert!(matches!(app.editor_mode, EditorMode::Operator('c')));
    }

    // ========== Simple VIM Commands ==========

    #[test]
    fn test_x_deletes_character_at_cursor() {
        let mut app = app_with_query(".name");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('x')));

        assert_eq!(app.query(), "name");
    }

    #[test]
    fn test_capital_x_deletes_character_before_cursor() {
        let mut app = app_with_query(".name");
        app.textarea.move_cursor(CursorMove::Head);
        app.textarea.move_cursor(CursorMove::Forward); // Move to 'n'
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('X')));

        assert_eq!(app.query(), "name");
    }

    #[test]
    fn test_capital_d_deletes_to_end_of_line() {
        let mut app = app_with_query(".name.first");
        move_cursor_to_position(&mut app, 5);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('D')));

        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_capital_c_changes_to_end_of_line() {
        let mut app = app_with_query(".name.first");
        move_cursor_to_position(&mut app, 5);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('C')));

        assert_eq!(app.query(), ".name");
        assert_eq!(app.editor_mode, EditorMode::Insert);
    }

    #[test]
    fn test_u_triggers_undo() {
        let mut app = app_with_query("");
        app.editor_mode = EditorMode::Insert;
        app.textarea.insert_str(".name");

        app.editor_mode = EditorMode::Normal;
        app.handle_key_event(key(KeyCode::Char('u')));

        // After undo, query should be empty
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_ctrl_r_triggers_redo() {
        let mut app = app_with_query("");
        app.editor_mode = EditorMode::Insert;
        app.textarea.insert_str(".name");

        app.editor_mode = EditorMode::Normal;
        app.textarea.undo(); // Undo the insert
        assert_eq!(app.query(), "");

        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // After redo, query should be back
        assert_eq!(app.query(), ".name");
    }

    // ========== VIM Navigation Tests ==========

    #[test]
    fn test_h_moves_cursor_left() {
        let mut app = app_with_query(".name");
        app.textarea.move_cursor(CursorMove::End);
        app.editor_mode = EditorMode::Normal;
        let cursor_before = app.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('h')));

        assert_eq!(app.textarea.cursor().1, cursor_before - 1);
    }

    #[test]
    fn test_l_moves_cursor_right() {
        let mut app = app_with_query(".name");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('l')));

        assert_eq!(app.textarea.cursor().1, 1);
    }

    #[test]
    fn test_0_moves_to_line_start() {
        let mut app = app_with_query(".name");
        app.textarea.move_cursor(CursorMove::End);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('0')));

        assert_eq!(app.textarea.cursor().1, 0);
    }

    #[test]
    fn test_dollar_moves_to_line_end() {
        let mut app = app_with_query(".name");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('$')));

        assert_eq!(app.textarea.cursor().1, 5);
    }

    #[test]
    fn test_w_moves_word_forward() {
        let mut app = app_with_query(".name.first");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;
        let cursor_before = app.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('w')));

        // Should move forward by at least one position
        assert!(app.textarea.cursor().1 > cursor_before);
    }

    #[test]
    fn test_b_moves_word_backward() {
        let mut app = app_with_query(".name.first");
        app.textarea.move_cursor(CursorMove::End);
        app.editor_mode = EditorMode::Normal;
        let cursor_before = app.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('b')));

        // Should move backward
        assert!(app.textarea.cursor().1 < cursor_before);
    }

    #[test]
    fn test_e_moves_to_word_end() {
        let mut app = app_with_query(".name.first");
        app.textarea.move_cursor(CursorMove::Head);
        app.editor_mode = EditorMode::Normal;
        let cursor_before = app.textarea.cursor().1;

        app.handle_key_event(key(KeyCode::Char('e')));

        // Should move forward
        assert!(app.textarea.cursor().1 > cursor_before);
    }

    // ========== Autocomplete Interaction Tests ==========

    #[test]
    fn test_escape_closes_autocomplete() {
        let mut app = app_with_query(".na");
        app.editor_mode = EditorMode::Insert;

        // Manually set autocomplete as visible with suggestions
        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);
        assert!(app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Esc));

        assert!(!app.autocomplete.is_visible());
        assert_eq!(app.query(), ".na"); // Query unchanged
        assert_eq!(app.editor_mode, EditorMode::Insert); // Still in insert mode
    }

    #[test]
    fn test_escape_without_autocomplete_switches_to_normal() {
        let mut app = app_with_query(".name");
        app.editor_mode = EditorMode::Insert;
        assert!(!app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Esc));

        assert_eq!(app.editor_mode, EditorMode::Normal);
    }

    #[test]
    fn test_down_arrow_selects_next_suggestion() {
        let mut app = app_with_query(".na");
        app.editor_mode = EditorMode::Insert;

        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
            Suggestion::new(".nested", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);

        app.handle_key_event(key(KeyCode::Down));

        // Should select second suggestion
        assert_eq!(app.autocomplete.selected().unwrap().text, ".nested");
    }

    #[test]
    fn test_up_arrow_selects_previous_suggestion() {
        let mut app = app_with_query(".na");
        app.editor_mode = EditorMode::Insert;

        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
            Suggestion::new(".nested", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);

        // Move to second suggestion
        app.autocomplete.select_next();

        app.handle_key_event(key(KeyCode::Up));

        // Should select first suggestion
        assert_eq!(app.autocomplete.selected().unwrap().text, ".name");
    }

    #[test]
    fn test_tab_accepts_autocomplete_suggestion() {
        let mut app = app_with_query(".na");
        app.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);

        app.handle_key_event(key(KeyCode::Tab));

        assert_eq!(app.query(), ".name");
        assert!(!app.autocomplete.is_visible());
    }

    #[test]
    fn test_tab_without_autocomplete_stays_in_consistent_state() {
        let mut app = app_with_query("x");  // Use a query that won't trigger autocomplete
        app.editor_mode = EditorMode::Insert;
        app.focus = Focus::InputField;

        // Ensure autocomplete is not visible
        app.autocomplete.hide();
        assert!(!app.autocomplete.is_visible());

        app.handle_key_event(key(KeyCode::Tab));

        // Tab without autocomplete gets passed through to textarea
        // Verify the app remains in a consistent state (doesn't crash, mode unchanged)
        assert_eq!(app.editor_mode, EditorMode::Insert);
        assert_eq!(app.focus, Focus::InputField);
    }

    #[test]
    fn test_autocomplete_navigation_only_works_in_insert_mode() {
        let mut app = app_with_query(".na");
        app.editor_mode = EditorMode::Normal;
        app.focus = Focus::InputField;

        let suggestions = vec![
            Suggestion::new(".name", SuggestionType::Field),
        ];
        app.autocomplete.update_suggestions(suggestions);

        // Down arrow in Normal mode should NOT navigate autocomplete (it's not handled)
        let selected_before = app.autocomplete.selected().unwrap().text.clone();
        app.handle_key_event(key(KeyCode::Down));
        let selected_after = app.autocomplete.selected().unwrap().text.clone();

        // Autocomplete selection should remain unchanged in Normal mode
        assert_eq!(selected_before, selected_after);
    }

    // ========== Results Scrolling Tests ==========

    #[test]
    fn test_j_scrolls_down_one_line() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 0;

        app.handle_key_event(key(KeyCode::Char('j')));

        assert_eq!(app.results_scroll, 1);
    }

    #[test]
    fn test_k_scrolls_up_one_line() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 5;

        app.handle_key_event(key(KeyCode::Char('k')));

        assert_eq!(app.results_scroll, 4);
    }

    #[test]
    fn test_k_at_top_stays_at_zero() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 0;

        app.handle_key_event(key(KeyCode::Char('k')));

        // Should saturate at 0, not go negative
        assert_eq!(app.results_scroll, 0);
    }

    #[test]
    fn test_capital_j_scrolls_down_ten_lines() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 5;

        app.handle_key_event(key(KeyCode::Char('J')));

        assert_eq!(app.results_scroll, 15);
    }

    #[test]
    fn test_capital_k_scrolls_up_ten_lines() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 20;

        app.handle_key_event(key(KeyCode::Char('K')));

        assert_eq!(app.results_scroll, 10);
    }

    #[test]
    fn test_g_jumps_to_top() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 50;

        app.handle_key_event(key(KeyCode::Char('g')));

        assert_eq!(app.results_scroll, 0);
    }

    #[test]
    fn test_capital_g_jumps_to_bottom() {
        let json = r#"{"line1": 1, "line2": 2, "line3": 3}"#;
        let mut app = App::new(json.to_string());
        app.textarea.insert_str(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 0;
        app.results_viewport_height = 2; // Small viewport to ensure max_scroll > 0

        let max_scroll = app.max_scroll();
        app.handle_key_event(key(KeyCode::Char('G')));

        // Should jump to max_scroll position
        assert_eq!(app.results_scroll, max_scroll);
    }

    #[test]
    fn test_page_up_scrolls_half_page() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 20;
        app.results_viewport_height = 20;

        app.handle_key_event(key(KeyCode::PageUp));

        // Should scroll up by half viewport (10 lines)
        assert_eq!(app.results_scroll, 10);
    }

    #[test]
    fn test_page_down_scrolls_half_page() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 0;
        app.results_viewport_height = 20;

        app.handle_key_event(key(KeyCode::PageDown));

        // Should scroll down by half viewport (10 lines)
        assert_eq!(app.results_scroll, 10);
    }

    #[test]
    fn test_ctrl_u_scrolls_half_page_up() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 20;
        app.results_viewport_height = 20;

        app.handle_key_event(key_with_mods(KeyCode::Char('u'), KeyModifiers::CONTROL));

        assert_eq!(app.results_scroll, 10);
    }

    #[test]
    fn test_ctrl_d_scrolls_half_page_down() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 0;
        app.results_viewport_height = 20;

        app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

        assert_eq!(app.results_scroll, 10);
    }

    #[test]
    fn test_up_arrow_scrolls_in_results_pane() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 5;

        app.handle_key_event(key(KeyCode::Up));

        assert_eq!(app.results_scroll, 4);
    }

    #[test]
    fn test_down_arrow_scrolls_in_results_pane() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 0;

        app.handle_key_event(key(KeyCode::Down));

        assert_eq!(app.results_scroll, 1);
    }

    #[test]
    fn test_home_jumps_to_top() {
        let mut app = app_with_query(".");
        app.focus = Focus::ResultsPane;
        app.results_scroll = 50;

        app.handle_key_event(key(KeyCode::Home));

        assert_eq!(app.results_scroll, 0);
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
        app.editor_mode = EditorMode::Normal;

        app.handle_key_event(key(KeyCode::Char('q')));

        assert!(app.should_quit);
    }

    #[test]
    fn test_q_does_not_quit_in_insert_mode() {
        let mut app = app_with_query(".");
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

        // Simulate typing a character
        app.handle_key_event(key(KeyCode::Char('.')));

        assert_eq!(app.query(), ".");
    }

    #[test]
    fn test_query_execution_resets_scroll() {
        let mut app = app_with_query("");
        app.editor_mode = EditorMode::Insert;
        app.results_scroll = 50;

        // Insert text which should trigger query execution
        app.handle_key_event(key(KeyCode::Char('.')));

        // Scroll should be reset when query changes
        assert_eq!(app.results_scroll, 0);
    }

    // ========== History Popup Tests ==========

    #[test]
    fn test_history_popup_does_not_open_when_empty() {
        let mut app = app_with_query("");
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;
        app.history.add_entry_in_memory(".history_item");

        // Move cursor to start
        app.textarea.move_cursor(tui_textarea::CursorMove::Head);
        assert_eq!(app.textarea.cursor().1, 0);

        // Press Up arrow
        app.handle_key_event(key(KeyCode::Up));

        // History should open
        assert!(app.history.is_visible());
    }

    #[test]
    fn test_up_arrow_opens_history_when_input_empty() {
        let mut app = app_with_query("");
        app.editor_mode = EditorMode::Insert;
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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

        // History is empty from app_with_query helper
        assert_eq!(app.history.total_count(), 0);

        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));

        // Query should be unchanged
        assert_eq!(app.query(), ".existing");
    }

    // ========== UTF-8 Edge Case Tests ==========

    #[test]
    fn test_history_with_emoji() {
        let mut app = app_with_query("");
        app.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".emoji_field 🚀");

        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".emoji_field 🚀");
    }

    #[test]
    fn test_history_with_multibyte_chars() {
        let mut app = app_with_query("");
        app.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".café | .naïve");

        app.handle_key_event(key_with_mods(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(app.query(), ".café | .naïve");
    }

    #[test]
    fn test_history_search_with_unicode() {
        let mut app = app_with_query("");
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

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
        app.editor_mode = EditorMode::Insert;

        app.history.add_entry_in_memory(".test");

        app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

        // Search is empty initially
        assert_eq!(app.history.search_query(), "");

        // Press backspace - should not crash
        app.handle_key_event(key(KeyCode::Backspace));
        assert_eq!(app.history.search_query(), "");
    }
}
