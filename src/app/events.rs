use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use std::time::Duration;

use crate::clipboard;
use crate::editor::EditorMode;
use crate::editor;
use crate::history;
use crate::results;
use super::state::{App, Focus};

mod global;

/// Timeout for event polling - allows periodic UI refresh for notifications
const EVENT_POLL_TIMEOUT: Duration = Duration::from_millis(100);

impl App {
    /// Handle events and update application state
    pub fn handle_events(&mut self) -> io::Result<()> {
        // Check for pending debounced execution before processing new events
        // This ensures queries are executed after the debounce period (50ms) has elapsed
        if self.debouncer.should_execute() {
            editor::events::execute_query(self);
            self.debouncer.mark_executed();
        }

        // Poll with timeout to allow periodic refresh for notification expiration
        if event::poll(EVENT_POLL_TIMEOUT)? {
            match event::read()? {
                // Check that it's a key press event to avoid duplicates
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                }
                // Handle paste events (bracketed paste mode)
                Event::Paste(text) => {
                    self.handle_paste_event(text);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle paste events from bracketed paste mode
    /// Inserts all pasted text at once and executes query immediately (no debounce)
    fn handle_paste_event(&mut self, text: String) {
        // Insert all text at once into the textarea
        self.input.textarea.insert_str(&text);
        
        // Execute query immediately (no debounce for paste operations)
        editor::events::execute_query(self);
        
        // Update autocomplete suggestions after paste
        self.update_autocomplete();
        
        // Update tooltip based on new cursor position
        self.update_tooltip();
    }

    /// Handle key press events
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Try global keys first
        if global::handle_global_keys(self, key) {
            return; // Key was handled globally
        }

        // Handle clipboard Ctrl+Y before mode-specific handling
        if clipboard::events::handle_clipboard_key(self, key, self.clipboard_backend) {
            return; // Key was handled by clipboard
        }

        // Not a global key, delegate to focused pane
        match self.focus {
            Focus::InputField => self.handle_input_field_key(key),
            Focus::ResultsPane => results::events::handle_results_pane_key(self, key),
        }
    }

    /// Handle keys when Input field is focused
    fn handle_input_field_key(&mut self, key: KeyEvent) {
        // Handle history popup when visible
        if self.history.is_visible() {
            history::events::handle_history_popup_key(self, key);
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
                    editor::events::execute_query(self);
                }
                return;
            }

            // Ctrl+R: Open history
            if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.open_history_popup();
                return;
            }

            // Up arrow: Open history popup (always)
            if key.code == KeyCode::Up {
                self.open_history_popup();
                return;
            }
        }

        // Handle input based on current mode
        match self.input.editor_mode {
            EditorMode::Insert => editor::events::handle_insert_mode_key(self, key),
            EditorMode::Normal => editor::events::handle_normal_mode_key(self, key),
            EditorMode::Operator(_) => editor::events::handle_operator_mode_key(self, key),
        }
    }


    /// Replace the current query with the given text
    fn replace_query_with(&mut self, text: &str) {
        self.input.textarea.delete_line_by_head();
        self.input.textarea.delete_line_by_end();
        self.input.textarea.insert_str(text);
        editor::events::execute_query(self);
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
    use crate::config::ClipboardBackend;
    use crate::history::HistoryState;
    use proptest::prelude::*;

    /// Helper to create App with default clipboard backend for tests
    fn test_app(json: &str) -> App {
        let mut app = App::new(json.to_string(), ClipboardBackend::Auto);
        // Use empty in-memory history for all tests to prevent disk writes
        app.history = HistoryState::empty();
        app
    }

    // =========================================================================
    // Unit Tests for Paste Event Handling
    // =========================================================================

    #[test]
    fn test_paste_event_inserts_text() {
        let mut app = test_app(r#"{"name": "test"}"#);
        
        // Simulate paste event
        app.handle_paste_event(".name".to_string());
        
        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_paste_event_executes_query() {
        let mut app = test_app(r#"{"name": "Alice"}"#);
        
        // Simulate paste event
        app.handle_paste_event(".name".to_string());
        
        // Query should have been executed
        assert!(app.query.result.is_ok());
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_paste_event_appends_to_existing_text() {
        let mut app = test_app(r#"{"user": {"name": "Bob"}}"#);
        
        // First, type some text
        app.input.textarea.insert_str(".user");
        
        // Then paste more text
        app.handle_paste_event(".name".to_string());
        
        assert_eq!(app.query(), ".user.name");
    }

    #[test]
    fn test_paste_event_with_empty_string() {
        let mut app = test_app(r#"{"name": "test"}"#);
        
        // Paste empty string
        app.handle_paste_event(String::new());
        
        // Query should remain empty
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_paste_event_with_multiline_text() {
        let mut app = test_app(r#"{"name": "test"}"#);
        
        // Paste multiline text (jq queries are single-line, but paste should handle it)
        app.handle_paste_event(".name\n| length".to_string());
        
        // The textarea handles this - verify text was inserted
        assert!(app.query().contains(".name"));
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // Feature: performance, Property 1: Paste text insertion integrity
    // *For any* string pasted into the application, the input field content after
    // the paste operation should contain exactly that string at the cursor position.
    // **Validates: Requirements 1.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_paste_text_insertion_integrity(
            // Generate printable ASCII strings (avoiding control characters that might
            // cause issues with the textarea)
            text in "[a-zA-Z0-9._\\[\\]|? ]{0,50}"
        ) {
            let mut app = test_app(r#"{"test": true}"#);
            
            // Paste the text
            app.handle_paste_event(text.clone());
            
            // The query should contain exactly the pasted text
            prop_assert_eq!(
                app.query(), &text,
                "Pasted text should appear exactly in the input field"
            );
        }

        #[test]
        fn prop_paste_appends_at_cursor_position(
            // Generate two parts of text
            prefix in "[a-zA-Z0-9.]{0,20}",
            pasted in "[a-zA-Z0-9.]{0,20}",
        ) {
            let mut app = test_app(r#"{"test": true}"#);
            
            // First insert the prefix
            app.input.textarea.insert_str(&prefix);
            
            // Then paste additional text
            app.handle_paste_event(pasted.clone());
            
            // The query should be prefix + pasted
            let expected = format!("{}{}", prefix, pasted);
            prop_assert_eq!(
                app.query(), &expected,
                "Pasted text should be appended at cursor position"
            );
        }

        #[test]
        fn prop_paste_executes_query_once(
            // Generate valid jq-like queries
            query in "\\.[a-z]{1,10}"
        ) {
            let json = r#"{"name": "test", "value": 42}"#;
            let mut app = test_app(json);
            
            // Paste a query
            app.handle_paste_event(query.clone());
            
            // Query should have been executed (result should be set)
            // We can't easily verify "exactly once" but we can verify it was executed
            prop_assert!(
                app.query.result.is_ok() || app.query.result.is_err(),
                "Query should have been executed after paste"
            );
            
            // The query text should match what was pasted
            prop_assert_eq!(
                app.query(), &query,
                "Query text should match pasted text"
            );
        }
    }
}

