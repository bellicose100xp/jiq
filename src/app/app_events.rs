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
            self.mark_dirty();
        }

        // Poll for query responses
        if self.poll_query_response() {
            self.mark_dirty();
        }

        if crate::ai::ai_events::poll_response_channel(&mut self.ai) {
            self.mark_dirty();
        }

        // Check notification expiry
        if self.notification.clear_if_expired() {
            self.mark_dirty();
        }

        if event::poll(EVENT_POLL_TIMEOUT)? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                    self.mark_dirty();
                }
                Event::Paste(text) => {
                    self.handle_paste_event(text);
                    self.mark_dirty();
                }
                Event::Resize(_, _) => {
                    self.mark_dirty();
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

        // Execute immediately for instant feedback (like old behavior)
        // Uses async execution to prevent race conditions
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

        if key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.results_scroll.page_down();
            return;
        }

        if key.code == KeyCode::Char('u') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.results_scroll.page_up();
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
            EditorMode::CharSearch(_, _) => {
                editor::editor_events::handle_char_search_mode_key(self, key)
            }
            EditorMode::TextObject(_, _) => {
                editor::editor_events::handle_text_object_mode_key(self, key)
            }
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

    /// Poll for query responses and update state
    ///
    /// Checks for completed async queries and triggers AI updates when needed.
    /// Uses the query returned from poll_response() to ensure AI gets correct context.
    /// Returns true if state changed (query completed).
    fn poll_query_response(&mut self) -> bool {
        let completed_query = if let Some(query_state) = &mut self.query {
            query_state.poll_response()
        } else {
            None
        };

        if let Some(completed_query) = completed_query {
            // Result changed - update stats once (not on every frame)
            self.update_stats();

            // State changed - trigger AI update if visible and query is not empty
            if self.ai.visible && !completed_query.is_empty() {
                let query_state = self.query.as_ref().unwrap();
                let cursor_pos = self.input.textarea.cursor().1;

                let ai_result: Result<String, String> = match &query_state.result {
                    Ok(_) => query_state
                        .last_successful_result_unformatted
                        .as_ref()
                        .map(|s| Ok(s.as_ref().clone()))
                        .unwrap_or_else(|| Ok(String::new())),
                    Err(e) => Err(e.clone()),
                };

                crate::ai::ai_events::handle_query_result(
                    &mut self.ai,
                    &ai_result,
                    &completed_query, // Use query from response, not current input!
                    cursor_pos,
                    crate::ai::context::ContextParams {
                        input_schema: self.input_json_schema.as_deref(),
                        base_query: query_state.base_query_for_suggestions.as_deref(),
                        base_query_result: query_state
                            .last_successful_result_for_context
                            .as_deref()
                            .map(|s| s.as_ref()),
                        is_empty_result: query_state.is_empty_result,
                    },
                );
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
#[path = "app_events_tests.rs"]
mod app_events_tests;
