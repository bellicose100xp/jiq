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
            let query = self.textarea.lines()[0].as_ref();
            self.query_result = self.executor.execute(query);

            // Cache successful results
            if let Ok(result) = &self.query_result {
                self.last_successful_result = Some(result.clone());
            }

            // Reset scroll when query changes
            self.results_scroll = 0;
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
