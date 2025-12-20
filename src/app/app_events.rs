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

        // Poll for query responses
        self.poll_query_response();

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

    /// Poll for query responses and update state
    ///
    /// Checks for completed async queries and triggers AI updates when needed.
    fn poll_query_response(&mut self) {
        if let Some(query_state) = &mut self.query {
            if query_state.poll_response() {
                // State changed - trigger AI update if visible
                if self.ai.visible {
                    let query = self.input.query().to_string();
                    let cursor_pos = self.input.textarea.cursor().1;
                    crate::ai::ai_events::handle_query_result(
                        &mut self.ai,
                        &query_state.result,
                        &query,
                        cursor_pos,
                        query_state.executor.json_input(),
                        crate::ai::context::ContextParams {
                            input_schema: self.input_json_schema.as_deref(),
                            base_query: query_state.base_query_for_suggestions.as_deref(),
                            base_query_result: query_state
                                .last_successful_result
                                .as_deref()
                                .map(|s| s.as_ref()),
                        },
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "app_events_tests.rs"]
mod app_events_tests;
