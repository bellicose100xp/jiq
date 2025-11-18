use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

use crate::query::executor::JqExecutor;

/// Which pane has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    InputField,
    ResultsPane,
}

/// What to output when exiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Results, // Output filtered JSON results (Enter)
    Query,   // Output query string only (Shift+Enter)
}

/// Application state
pub struct App {
    pub json_input: String,
    pub textarea: TextArea<'static>,
    pub executor: JqExecutor,
    pub query_result: Result<String, String>,
    pub focus: Focus,
    pub results_scroll: u16,
    pub output_mode: Option<OutputMode>,
    pub should_quit: bool,
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
            results_scroll: 0,
            output_mode: None, // No output mode set until Enter/Shift+Enter
            should_quit: false,
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get the output mode (if set)
    pub fn output_mode(&self) -> Option<OutputMode> {
        self.output_mode
    }

    /// Get the current query text
    pub fn query(&self) -> &str {
        self.textarea.lines()[0].as_ref()
    }
}
