use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io;

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

        // Tab: Switch focus between panes
        if key.code == KeyCode::Tab {
            self.focus = match self.focus {
                Focus::InputField => Focus::ResultsPane,
                Focus::ResultsPane => Focus::InputField,
            };
            return true;
        }

        // q: Exit application
        if key.code == KeyCode::Char('q') {
            self.should_quit = true;
            return true;
        }

        // Shift+Enter (may be sent as Alt+Enter by some terminals): Exit and output query only
        if key.code == KeyCode::Enter
            && (key.modifiers.contains(KeyModifiers::SHIFT) || key.modifiers.contains(KeyModifiers::ALT))
        {
            self.output_mode = Some(OutputMode::Query);
            self.should_quit = true;
            return true;
        }

        // Enter: Exit and output filtered results
        if key.code == KeyCode::Enter {
            self.output_mode = Some(OutputMode::Results);
            self.should_quit = true;
            return true;
        }

        false // Key not handled
    }

    /// Handle keys when Input field is focused
    fn handle_input_field_key(&mut self, key: KeyEvent) {
        // Pass key to textarea for editing
        let content_changed = self.textarea.input(key);

        // Execute query on every keystroke that changes content
        if content_changed {
            let query = self.textarea.lines()[0].as_ref();
            self.query_result = self.executor.execute(query);
            // Reset scroll when query changes
            self.results_scroll = 0;
        }
    }

    /// Handle keys when Results pane is focused
    fn handle_results_pane_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.results_scroll = self.results_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.results_scroll = self.results_scroll.saturating_add(1);
            }
            KeyCode::PageUp | KeyCode::Char('K') => {
                self.results_scroll = self.results_scroll.saturating_sub(10);
            }
            KeyCode::PageDown | KeyCode::Char('J') => {
                self.results_scroll = self.results_scroll.saturating_add(10);
            }
            KeyCode::Home => {
                self.results_scroll = 0;
            }
            _ => {
                // Ignore other keys in Results pane
            }
        }
    }
}
