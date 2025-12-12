use crate::help::HelpPopupState;
use crate::input::InputState;
use crate::query::{Debouncer, QueryState};
use crate::autocomplete::{self, AutocompleteState};
use crate::config::{ClipboardBackend, Config};
use crate::history::HistoryState;
use crate::notification::NotificationState;
use crate::scroll::ScrollState;
use crate::search::SearchState;
use crate::stats::{self, StatsState};
use crate::tooltip::{self, TooltipState};

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
    pub error_overlay_visible: bool,
    pub history: HistoryState,
    pub help: HelpPopupState,
    pub notification: NotificationState,
    pub clipboard_backend: ClipboardBackend,
    pub tooltip: TooltipState,
    pub stats: StatsState,
    pub debouncer: Debouncer,
    pub search: SearchState,
}

impl App {
    /// Create a new App instance with JSON input and configuration
    ///
    /// # Arguments
    /// * `json_input` - The JSON data to explore
    /// * `config` - Application configuration
    pub fn new(json_input: String, config: &Config) -> Self {
        Self {
            input: InputState::new(),
            query: QueryState::new(json_input),
            focus: Focus::InputField,
            results_scroll: ScrollState::new(),
            output_mode: None,
            should_quit: false,
            autocomplete: AutocompleteState::new(),
            error_overlay_visible: false,
            history: HistoryState::new(),
            help: HelpPopupState::new(),
            notification: NotificationState::new(),
            clipboard_backend: config.clipboard.backend,
            tooltip: TooltipState::new(config.tooltip.auto_show),
            stats: StatsState::default(),
            debouncer: Debouncer::new(),
            search: SearchState::new(),
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
    /// Delegates to the autocomplete module for the actual logic
    pub fn update_autocomplete(&mut self) {
        autocomplete::update_suggestions_from_app(self);
    }

    /// Update tooltip state based on current cursor position
    /// Delegates to the tooltip module for the actual logic
    pub fn update_tooltip(&mut self) {
        tooltip::update_tooltip_from_app(self);
    }

    /// Update stats based on the last successful result
    /// Delegates to the stats module for the actual logic
    pub fn update_stats(&mut self) {
        stats::update_stats_from_app(self);
    }

    /// Insert an autocomplete suggestion at the current cursor position
    /// Delegates to the autocomplete insertion module
    pub fn insert_autocomplete_suggestion(&mut self, suggestion: &autocomplete::autocomplete_state::Suggestion) {
        autocomplete::insert_suggestion_from_app(self, suggestion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_helpers::test_app;

    #[test]
    fn test_app_initialization() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let app = test_app(json);

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
        let app = test_app(json);

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
        let mut app = test_app(json);

        assert!(!app.should_quit());

        app.should_quit = true;
        assert!(app.should_quit());
    }

    #[test]
    fn test_output_mode_getter() {
        let json = r#"{}"#;
        let mut app = test_app(json);

        assert_eq!(app.output_mode(), None);

        app.output_mode = Some(OutputMode::Results);
        assert_eq!(app.output_mode(), Some(OutputMode::Results));

        app.output_mode = Some(OutputMode::Query);
        assert_eq!(app.output_mode(), Some(OutputMode::Query));
    }

    #[test]
    fn test_query_getter_empty() {
        let json = r#"{"test": true}"#;
        let app = test_app(json);

        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_app_with_empty_json_object() {
        let json = "{}";
        let app = test_app(json);

        assert!(app.query.result.is_ok());
    }

    #[test]
    fn test_app_with_json_array() {
        let json = r#"[1, 2, 3]"#;
        let app = test_app(json);

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
        let mut app = test_app(json);

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
        let mut app = test_app(json);

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
        let mut app = test_app(json);

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
        let mut app = test_app(json);

        // Set error without any cached result
        app.query.last_successful_result = None;
        app.query.result = Err("error message".to_string());

        // Should return 0 when no cached result available
        assert_eq!(app.results_line_count_u32(), 0);

        // Update scroll bounds
        app.results_scroll.update_bounds(0, 10);
        assert_eq!(app.results_scroll.max_offset, 0);
    }

    // Autocomplete insertion tests have been moved to src/autocomplete/insertion.rs
    // Tooltip detection tests have been moved to src/tooltip/tooltip_state.rs

    // ========== Tooltip Integration Tests ==========
    // Only cross-feature integration tests remain here

    #[test]
    fn test_tooltip_initialized_enabled() {
        let json = r#"{"name": "test"}"#;
        let app = test_app(json);
        
        // Tooltip should be enabled by default
        assert!(app.tooltip.enabled);
        assert!(app.tooltip.current_function.is_none());
    }
}
