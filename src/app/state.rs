use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

use crate::autocomplete::{AutocompleteState, get_suggestions};
use crate::autocomplete::json_analyzer::JsonAnalyzer;
use crate::editor::EditorMode;
use crate::query::executor::JqExecutor;

// Autocomplete performance constants
const MIN_CHARS_FOR_AUTOCOMPLETE: usize = 1;

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
    pub textarea: TextArea<'static>,
    pub executor: JqExecutor,
    pub query_result: Result<String, String>,
    pub last_successful_result: Option<String>,
    pub focus: Focus,
    pub editor_mode: EditorMode,
    pub results_scroll: u16,
    pub results_viewport_height: u16,
    pub output_mode: Option<OutputMode>,
    pub should_quit: bool,
    pub autocomplete: AutocompleteState,
    pub json_analyzer: JsonAnalyzer,
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

        // Initial result text on startup
        let query_result = executor.execute(".");

        // Cache the initial successful result
        let last_successful_result = query_result.as_ref().ok().cloned();

        // Initialize JSON analyzer with the input JSON
        let mut json_analyzer = JsonAnalyzer::new();
        let _ = json_analyzer.analyze(&json_input);

        Self {
            textarea,
            executor,
            query_result,
            last_successful_result,
            focus: Focus::InputField, // Start with input field focused
            editor_mode: EditorMode::default(), // Start in Insert mode
            results_scroll: 0,
            results_viewport_height: 0, // Will be set during first render
            output_mode: None, // No output mode set until Enter/Shift+Enter
            should_quit: false,
            autocomplete: AutocompleteState::new(),
            json_analyzer,
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

    /// Get the total number of lines in the current results
    pub fn results_line_count(&self) -> u16 {
        match &self.query_result {
            Ok(result) => result.lines().count() as u16,
            Err(error) => error.lines().count() as u16,
        }
    }

    /// Get the maximum scroll position based on content and viewport
    pub fn max_scroll(&self) -> u16 {
        let total_lines = self.results_line_count();
        total_lines.saturating_sub(self.results_viewport_height)
    }

    /// Update autocomplete suggestions based on current query and cursor position
    pub fn update_autocomplete(&mut self) {
        let query = self.query();
        let cursor_pos = self.textarea.cursor().1; // Column position

        // Performance optimization: only show autocomplete for non-empty queries
        if query.trim().len() < MIN_CHARS_FOR_AUTOCOMPLETE {
            self.autocomplete.hide();
            return;
        }

        // Get suggestions based on context
        let suggestions = get_suggestions(query, cursor_pos, &self.json_analyzer);

        // Update autocomplete state
        self.autocomplete.update_suggestions(suggestions);
    }

    /// Insert an autocomplete suggestion at the current cursor position
    pub fn insert_autocomplete_suggestion(&mut self, suggestion: &str) {
        let query = self.query().to_string();
        let cursor_pos = self.textarea.cursor().1;
        let before_cursor = &query[..cursor_pos.min(query.len())];

        // Find the start position to replace from
        // For field suggestions (starting with .), find the last dot
        // For other suggestions, find the token start
        let replace_start = if suggestion.starts_with('.') {
            // Field suggestion - find the last dot in before_cursor
            // This handles nested fields like .services.service correctly
            before_cursor.rfind('.').unwrap_or(0)
        } else {
            // Function/operator/pattern suggestion - find token start
            find_token_start(before_cursor)
        };

        // Build the new query with the suggestion
        let new_query = format!(
            "{}{}{}",
            &query[..replace_start],
            suggestion,
            &query[cursor_pos.min(query.len())..]
        );

        // Replace the entire line and set cursor position
        self.textarea.delete_line_by_head();
        self.textarea.insert_str(&new_query);

        // Move cursor to end of inserted suggestion
        let target_pos = replace_start + suggestion.len();
        self.move_cursor_to_column(target_pos);

        // Hide autocomplete and execute query
        self.autocomplete.hide();
        self.execute_query_and_update();
    }

    /// Move cursor to a specific column position (helper method)
    fn move_cursor_to_column(&mut self, target_col: usize) {
        let current_col = self.textarea.cursor().1;

        match target_col.cmp(&current_col) {
            std::cmp::Ordering::Less => {
                // Move backward
                for _ in 0..(current_col - target_col) {
                    self.textarea.move_cursor(tui_textarea::CursorMove::Back);
                }
            }
            std::cmp::Ordering::Greater => {
                // Move forward
                for _ in 0..(target_col - current_col) {
                    self.textarea.move_cursor(tui_textarea::CursorMove::Forward);
                }
            }
            std::cmp::Ordering::Equal => {
                // Already at target position
            }
        }
    }

    /// Execute query and update results (helper method)
    fn execute_query_and_update(&mut self) {
        let query = self.query();
        self.query_result = self.executor.execute(query);
        if let Ok(result) = &self.query_result {
            self.last_successful_result = Some(result.clone());
        }
        self.results_scroll = 0;
    }
}

/// Find the start position of the current token
fn find_token_start(text: &str) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut i = chars.len();

    // Skip trailing whitespace
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }

    // Find the start of the current token
    while i > 0 {
        let ch = chars[i - 1];
        if is_token_delimiter(ch) {
            break;
        }
        i -= 1;
    }

    i
}

/// Check if a character is a token delimiter
fn is_token_delimiter(ch: char) -> bool {
    matches!(
        ch,
        '|' | ';'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | ','
            | ' '
            | '\t'
            | '\n'
            | '\r'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initialization() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let app = App::new(json.to_string());

        // Check default state
        assert_eq!(app.focus, Focus::InputField);
        assert_eq!(app.results_scroll, 0);
        assert_eq!(app.output_mode, None);
        assert!(!app.should_quit);
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_initial_query_result() {
        let json = r#"{"name": "Bob"}"#;
        let app = App::new(json.to_string());

        // Initial query should execute identity filter "."
        assert!(app.query_result.is_ok());
        let result = app.query_result.as_ref().unwrap();
        assert!(result.contains("Bob"));
    }

    #[test]
    fn test_focus_enum() {
        assert_eq!(Focus::InputField, Focus::InputField);
        assert_eq!(Focus::ResultsPane, Focus::ResultsPane);
        assert_ne!(Focus::InputField, Focus::ResultsPane);
    }

    #[test]
    fn test_output_mode_enum() {
        assert_eq!(OutputMode::Results, OutputMode::Results);
        assert_eq!(OutputMode::Query, OutputMode::Query);
        assert_ne!(OutputMode::Results, OutputMode::Query);
    }

    #[test]
    fn test_should_quit_getter() {
        let json = r#"{}"#;
        let mut app = App::new(json.to_string());

        assert!(!app.should_quit());

        app.should_quit = true;
        assert!(app.should_quit());
    }

    #[test]
    fn test_output_mode_getter() {
        let json = r#"{}"#;
        let mut app = App::new(json.to_string());

        assert_eq!(app.output_mode(), None);

        app.output_mode = Some(OutputMode::Results);
        assert_eq!(app.output_mode(), Some(OutputMode::Results));

        app.output_mode = Some(OutputMode::Query);
        assert_eq!(app.output_mode(), Some(OutputMode::Query));
    }

    #[test]
    fn test_query_getter_empty() {
        let json = r#"{"test": true}"#;
        let app = App::new(json.to_string());

        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_app_with_empty_json_object() {
        let json = "{}";
        let app = App::new(json.to_string());

        assert!(app.query_result.is_ok());
    }

    #[test]
    fn test_app_with_json_array() {
        let json = r#"[1, 2, 3]"#;
        let app = App::new(json.to_string());

        assert!(app.query_result.is_ok());
        let result = app.query_result.as_ref().unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }
}
