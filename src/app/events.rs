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
        // Poll with timeout to allow periodic refresh for notification expiration
        if event::poll(EVENT_POLL_TIMEOUT)? {
            match event::read()? {
                // Check that it's a key press event to avoid duplicates
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                }
                _ => {}
            }
        }
        Ok(())
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

// No tests in this file - all tests are in submodules

