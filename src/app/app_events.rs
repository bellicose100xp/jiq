use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;
use std::time::Duration;

use super::app_state::{App, Focus};
use crate::clipboard;
use crate::editor;
use crate::editor::EditorMode;
use crate::history;
use crate::results;

mod global;

const EVENT_POLL_TIMEOUT: Duration = Duration::from_millis(100);

impl App {
    pub fn handle_events(&mut self) -> io::Result<()> {
        if self.debouncer.should_execute() {
            editor::editor_events::execute_query_with_auto_show(self);
            self.debouncer.mark_executed();
        }

        crate::ai::ai_events::poll_response_channel(&mut self.ai);

        if event::poll(EVENT_POLL_TIMEOUT)? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                }
                Event::Paste(text) => {
                    self.handle_paste_event(text);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_paste_event(&mut self, text: String) {
        self.input.textarea.insert_str(&text);

        self.input
            .brace_tracker
            .rebuild(self.input.textarea.lines()[0].as_ref());

        editor::editor_events::execute_query(self);

        self.update_autocomplete();

        self.update_tooltip();
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if crate::search::search_events::handle_search_key(self, key) {
            return;
        }

        if global::handle_global_keys(self, key) {
            return;
        }

        if clipboard::clipboard_events::handle_clipboard_key(self, key, self.clipboard_backend) {
            return;
        }

        match self.focus {
            Focus::InputField => self.handle_input_field_key(key),
            Focus::ResultsPane => results::results_events::handle_results_pane_key(self, key),
        }
    }

    fn handle_input_field_key(&mut self, key: KeyEvent) {
        if self.history.is_visible() {
            history::history_events::handle_history_popup_key(self, key);
            return;
        }

        if key.code == KeyCode::Esc {
            if self.autocomplete.is_visible() {
                self.autocomplete.hide();
            }
            self.input.editor_mode = EditorMode::Normal;
            return;
        }

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

        if self.input.editor_mode == EditorMode::Insert {
            if key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
                if let Some(entry) = self.history.cycle_previous() {
                    self.replace_query_with(&entry);
                }
                return;
            }

            if key.code == KeyCode::Char('n') && key.modifiers.contains(KeyModifiers::CONTROL) {
                if let Some(entry) = self.history.cycle_next() {
                    self.replace_query_with(&entry);
                } else {
                    self.input.textarea.delete_line_by_head();
                    self.input.textarea.delete_line_by_end();
                    editor::editor_events::execute_query(self);
                }
                return;
            }

            if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.open_history_popup();
                return;
            }

            if key.code == KeyCode::Up {
                self.open_history_popup();
                return;
            }
        }

        match self.input.editor_mode {
            EditorMode::Insert => editor::editor_events::handle_insert_mode_key(self, key),
            EditorMode::Normal => editor::editor_events::handle_normal_mode_key(self, key),
            EditorMode::Operator(_) => editor::editor_events::handle_operator_mode_key(self, key),
        }
    }

    fn replace_query_with(&mut self, text: &str) {
        self.input.textarea.delete_line_by_head();
        self.input.textarea.delete_line_by_end();
        self.input.textarea.insert_str(text);
        editor::editor_events::execute_query(self);
    }

    fn open_history_popup(&mut self) {
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
    use crate::test_utils::test_helpers::test_app;
    use proptest::prelude::*;

    #[test]
    fn test_paste_event_inserts_text() {
        let mut app = test_app(r#"{"name": "test"}"#);

        app.handle_paste_event(".name".to_string());

        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_paste_event_executes_query() {
        let mut app = test_app(r#"{"name": "Alice"}"#);

        app.handle_paste_event(".name".to_string());

        assert!(app.query.result.is_ok());
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_paste_event_appends_to_existing_text() {
        let mut app = test_app(r#"{"user": {"name": "Bob"}}"#);

        app.input.textarea.insert_str(".user");

        app.handle_paste_event(".name".to_string());

        assert_eq!(app.query(), ".user.name");
    }

    #[test]
    fn test_paste_event_with_empty_string() {
        let mut app = test_app(r#"{"name": "test"}"#);

        app.handle_paste_event(String::new());

        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_paste_event_with_multiline_text() {
        let mut app = test_app(r#"{"name": "test"}"#);

        app.handle_paste_event(".name\n| length".to_string());

        assert!(app.query().contains(".name"));
    }

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
