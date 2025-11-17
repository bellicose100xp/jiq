use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::io;
use tui_textarea::TextArea;

use crate::query::executor::JqExecutor;

/// Which pane has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    InputField,
    ResultsPane,
}

/// Application state
pub struct App {
    json_input: String,
    textarea: TextArea<'static>,
    executor: JqExecutor,
    query_result: Result<String, String>,
    focus: Focus,
    should_quit: bool,
}

impl App {
    /// Create a new App instance with JSON input
    pub fn new(json_input: String) -> Self {
        // Create textarea for query input
        let mut textarea = TextArea::default();

        // Configure for single-line input
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Query ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        // Remove default underline from cursor line
        textarea.set_cursor_line_style(Style::default());

        // Create JQ executor
        let executor = JqExecutor::new(json_input.clone());

        // Initially show original JSON (identity filter)
        let query_result = Ok(json_input.clone());

        Self {
            json_input,
            textarea,
            executor,
            query_result,
            focus: Focus::InputField, // Start with input field focused
            should_quit: false,
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

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
        // Handle Ctrl+C
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        // Handle Tab key to switch focus
        if key.code == KeyCode::Tab {
            self.focus = match self.focus {
                Focus::InputField => Focus::ResultsPane,
                Focus::ResultsPane => Focus::InputField,
            };
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Enter => {
                // Prevent newlines in single-line input (do nothing)
            }
            _ => {
                // Only handle input when Query field is focused
                if self.focus == Focus::InputField {
                    // Pass key to textarea for editing
                    let content_changed = self.textarea.input(key);

                    // Execute query on every keystroke that changes content
                    if content_changed {
                        let query = self.textarea.lines()[0].as_ref();
                        self.query_result = self.executor.execute(query);
                    }
                }
                // Results pane navigation will be handled in v0.6.0
            }
        }
    }

    /// Render the UI
    pub fn render(&mut self, frame: &mut Frame) {
        // Split the terminal into two panes: results (top) and input (bottom)
        let layout = Layout::vertical([
            Constraint::Min(3),      // Results pane takes most of the space
            Constraint::Length(3),   // Input field is fixed 3 lines
        ])
        .split(frame.area());

        let results_area = layout[0];
        let input_area = layout[1];

        // Render results pane
        self.render_results_pane(frame, results_area);

        // Render input field
        self.render_input_field(frame, input_area);
    }

    /// Render the input field (bottom)
    fn render_input_field(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Set border color based on focus
        let border_color = if self.focus == Focus::InputField {
            Color::Cyan // Focused
        } else {
            Color::DarkGray // Unfocused
        };

        // Update textarea block with focus-aware styling
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Query ")
                .border_style(Style::default().fg(border_color)),
        );

        // Render the textarea widget
        frame.render_widget(&self.textarea, area);
    }

    /// Render the results pane (top)
    fn render_results_pane(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Set border color based on focus
        let border_color = if self.focus == Focus::ResultsPane {
            Color::Cyan // Focused
        } else {
            Color::DarkGray // Unfocused
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Results ")
            .border_style(Style::default().fg(border_color));

        // Display query results or error message
        let (text, style) = match &self.query_result {
            Ok(result) => {
                // Use default style to preserve jq's ANSI color codes
                (result.as_str(), Style::default())
            }
            Err(error) => {
                // Use red color for error messages
                (error.as_str(), Style::default().fg(Color::Red))
            }
        };

        let content = Paragraph::new(text).block(block).style(style);

        frame.render_widget(content, area);
    }
}
