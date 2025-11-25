use tui_textarea::CursorMove;

use super::help_state::HelpPopupState;
use super::input_state::InputState;
use super::query_state::QueryState;
use crate::autocomplete::{AutocompleteState, get_suggestions};
use crate::autocomplete::json_analyzer::JsonAnalyzer;
use crate::history::HistoryState;
use crate::scroll::ScrollState;

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
    pub input: InputState,
    pub query: QueryState,
    pub focus: Focus,
    pub results_scroll: ScrollState,
    pub output_mode: Option<OutputMode>,
    pub should_quit: bool,
    pub autocomplete: AutocompleteState,
    pub json_analyzer: JsonAnalyzer,
    pub error_overlay_visible: bool,
    pub history: HistoryState,
    pub help: HelpPopupState,
}

impl App {
    /// Create a new App instance with JSON input
    pub fn new(json_input: String) -> Self {
        // Initialize JSON analyzer with the input JSON
        let mut json_analyzer = JsonAnalyzer::new();
        let _ = json_analyzer.analyze(&json_input);

        Self {
            input: InputState::new(),
            query: QueryState::new(json_input),
            focus: Focus::InputField,
            results_scroll: ScrollState::new(),
            output_mode: None,
            should_quit: false,
            autocomplete: AutocompleteState::new(),
            json_analyzer,
            error_overlay_visible: false,
            history: HistoryState::new(),
            help: HelpPopupState::new(),
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
        self.input.query()
    }

    /// Get the total number of lines in the current results
    /// Note: Returns u32 to handle large files (>65K lines) correctly
    /// When there's an error, uses last_successful_result since that's what gets rendered
    pub fn results_line_count_u32(&self) -> u32 {
        self.query.line_count()
    }


    /// Update autocomplete suggestions based on current query and cursor position
    pub fn update_autocomplete(&mut self) {
        let query = self.query();
        let cursor_pos = self.input.textarea.cursor().1; // Column position

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
        let cursor_pos = self.input.textarea.cursor().1;
        let before_cursor = &query[..cursor_pos.min(query.len())];

        // Find the start position to replace from
        let replace_start = if suggestion.starts_with('[') {
            // Array access suggestion like [] or [].field
            // Two scenarios:
            // 1. ".services" + "[].name" = ".services[].name" (append - no partial)
            // 2. ".services.s" + "[].serviceArn" = ".services[].serviceArn" (replace partial ".s")

            // Check if there's a dot followed by a short partial text (1-3 chars) before cursor
            // This indicates user is filtering suggestions, not at a complete field
            if let Some(last_dot_pos) = before_cursor.rfind('.') {
                let after_dot = &before_cursor[last_dot_pos + 1..];
                // If there's 1-3 chars after last dot, it's likely a partial being filtered
                if !after_dot.is_empty() && after_dot.len() <= 3 && !after_dot.contains('[') {
                    // Short partial text - replace from the dot
                    last_dot_pos
                } else {
                    // Complete field name or no partial - append at cursor
                    cursor_pos
                }
            } else {
                // No dot found - append at cursor
                cursor_pos
            }
        } else if suggestion.starts_with('.') {
            // Field suggestion - find the last dot to replace from there
            // This handles: .field suggestions (e.g., .name)
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
        self.input.textarea.delete_line_by_head();
        self.input.textarea.insert_str(&new_query);

        // Move cursor to end of inserted suggestion
        let target_pos = replace_start + suggestion.len();
        self.move_cursor_to_column(target_pos);

        // Hide autocomplete and execute query
        self.autocomplete.hide();
        self.execute_query_and_update();
    }

    /// Move cursor to a specific column position (helper method)
    fn move_cursor_to_column(&mut self, target_col: usize) {
        let current_col = self.input.textarea.cursor().1;

        match target_col.cmp(&current_col) {
            std::cmp::Ordering::Less => {
                // Move backward
                for _ in 0..(current_col - target_col) {
                    self.input.textarea.move_cursor(CursorMove::Back);
                }
            }
            std::cmp::Ordering::Greater => {
                // Move forward
                for _ in 0..(target_col - current_col) {
                    self.input.textarea.move_cursor(CursorMove::Forward);
                }
            }
            std::cmp::Ordering::Equal => {
                // Already at target position
            }
        }
    }

    /// Execute query and update results (helper method)
    fn execute_query_and_update(&mut self) {
        let query_text = self.query().to_string();
        self.query.execute(&query_text);
        self.results_scroll.reset();
        self.error_overlay_visible = false; // Auto-hide error overlay on query change
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
        assert_eq!(app.results_scroll.offset, 0);
        assert_eq!(app.output_mode, None);
        assert!(!app.should_quit);
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_initial_query_result() {
        let json = r#"{"name": "Bob"}"#;
        let app = App::new(json.to_string());

        // Initial query should execute identity filter "."
        assert!(app.query.result.is_ok());
        let result = app.query.result.as_ref().unwrap();
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

        assert!(app.query.result.is_ok());
    }

    #[test]
    fn test_app_with_json_array() {
        let json = r#"[1, 2, 3]"#;
        let app = App::new(json.to_string());

        assert!(app.query.result.is_ok());
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    // Tests for large file handling (>65K lines)
    #[test]
    fn test_max_scroll_large_content() {
        let json = r#"{"test": true}"#;
        let mut app = App::new(json.to_string());

        // Simulate large content result
        let large_result: String = (0..70000).map(|i| format!("line {}\n", i)).collect();
        app.query.result = Ok(large_result);

        // Should handle >65K lines without overflow
        let line_count = app.results_line_count_u32();
        assert!(line_count > 65535);

        // Update scroll bounds
        app.results_scroll.update_bounds(line_count, 20);

        // max_offset should be clamped to u16::MAX
        assert_eq!(app.results_scroll.max_offset, u16::MAX);
    }

    #[test]
    fn test_results_line_count_large_file() {
        let json = r#"{"test": true}"#;
        let mut app = App::new(json.to_string());

        // Simulate result with exactly u16::MAX lines
        let result: String = (0..65535).map(|_| "x\n").collect();
        app.query.result = Ok(result);

        // Verify line count is correct (using internal method)
        assert_eq!(app.results_line_count_u32(), 65535);

        // Update scroll bounds
        app.results_scroll.update_bounds(65535, 10);

        // Verify max_offset handles it correctly
        assert_eq!(app.results_scroll.max_offset, 65525); // 65535 - 10
    }

    #[test]
    fn test_line_count_uses_last_result_on_error() {
        let json = r#"{"test": true}"#;
        let mut app = App::new(json.to_string());

        // Execute a valid query first to cache result
        let valid_result: String = (0..50).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(valid_result.clone());
        app.query.last_successful_result = Some(valid_result);

        // Verify line count with valid result
        assert_eq!(app.results_line_count_u32(), 50);

        // Now simulate an error (short error message)
        app.query.result = Err("syntax error\nline 2\nline 3".to_string());

        // Line count should use last_successful_result (50 lines), not error (3 lines)
        assert_eq!(app.results_line_count_u32(), 50);

        // Update scroll bounds and verify max_offset is calculated correctly
        app.results_scroll.update_bounds(50, 10);
        assert_eq!(app.results_scroll.max_offset, 40); // 50 - 10 = 40
    }

    #[test]
    fn test_line_count_with_error_no_cached_result() {
        let json = r#"{"test": true}"#;
        let mut app = App::new(json.to_string());

        // Set error without any cached result
        app.query.last_successful_result = None;
        app.query.result = Err("error message".to_string());

        // Should return 0 when no cached result available
        assert_eq!(app.results_line_count_u32(), 0);

        // Update scroll bounds
        app.results_scroll.update_bounds(0, 10);
        assert_eq!(app.results_scroll.max_offset, 0);
    }

    #[test]
    fn test_array_suggestion_appends_to_path() {
        // When accepting [].field suggestion for .services, should produce .services[].field
        let json = r#"{"services": [{"name": "test"}]}"#;
        let mut app = App::new(json.to_string());

        // Simulate: user typed ".services" and cursor is at end (no partial)
        app.input.textarea.insert_str(".services");

        // Accept autocomplete suggestion "[].name"
        app.insert_autocomplete_suggestion("[].name");

        // Should produce .services[].name (append, not replace)
        assert_eq!(app.query(), ".services[].name");
    }

    #[test]
    fn test_array_suggestion_replaces_partial_field() {
        // When user types partial field after array name, accepting [] suggestion should replace partial
        let json = r#"{"services": [{"serviceArn": "test"}]}"#;
        let mut app = App::new(json.to_string());

        // Simulate: user typed ".services.s" (partial match for serviceArn)
        app.input.textarea.insert_str(".services.s");

        // Accept autocomplete suggestion "[].serviceArn"
        app.insert_autocomplete_suggestion("[].serviceArn");

        // Should produce .services[].serviceArn (replace ".s" with "[].serviceArn")
        assert_eq!(app.query(), ".services[].serviceArn");
    }

    #[test]
    fn test_field_suggestion_replaces_from_dot() {
        // When accepting .field suggestion, should replace from last dot
        let json = r#"{"name": "test", "age": 30}"#;
        let mut app = App::new(json.to_string());

        // Simulate: user typed ".na" and cursor is at end
        app.input.textarea.insert_str(".na");

        // Accept autocomplete suggestion ".name"
        app.insert_autocomplete_suggestion(".name");

        // Should produce .name (replace from the dot)
        assert_eq!(app.query(), ".name");
    }
}
