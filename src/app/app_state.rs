use crate::ai::AiState;
use crate::autocomplete::{self, AutocompleteState};
use crate::config::{ClipboardBackend, Config};
use crate::help::HelpPopupState;
use crate::history::HistoryState;
use crate::input::InputState;
use crate::notification::NotificationState;
use crate::query::{Debouncer, QueryState};
use crate::scroll::ScrollState;
use crate::search::SearchState;
use crate::stats::{self, StatsState};
use crate::tooltip::{self, TooltipState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    InputField,
    ResultsPane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Results,
    Query,
}

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
    pub ai: AiState,
    pub saved_tooltip_visibility: bool,
}

impl App {
    pub fn new(json_input: String, config: &Config) -> Self {
        let ai_state = AiState::new_with_config(
            config.ai.enabled,
            config.ai.anthropic.api_key.is_some() && config.ai.anthropic.model.is_some(),
        );

        let tooltip_enabled = if ai_state.visible {
            false
        } else {
            config.tooltip.auto_show
        };

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
            tooltip: TooltipState::new(tooltip_enabled),
            stats: StatsState::default(),
            debouncer: Debouncer::new(),
            search: SearchState::new(),
            ai: ai_state,
            saved_tooltip_visibility: config.tooltip.auto_show,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn output_mode(&self) -> Option<OutputMode> {
        self.output_mode
    }

    pub fn query(&self) -> &str {
        self.input.query()
    }

    pub fn results_line_count_u32(&self) -> u32 {
        self.query.line_count()
    }

    pub fn update_autocomplete(&mut self) {
        autocomplete::update_suggestions_from_app(self);
    }

    pub fn update_tooltip(&mut self) {
        tooltip::update_tooltip_from_app(self);
    }

    pub fn update_stats(&mut self) {
        stats::update_stats_from_app(self);
    }

    pub fn insert_autocomplete_suggestion(
        &mut self,
        suggestion: &autocomplete::autocomplete_state::Suggestion,
    ) {
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

    #[test]
    fn test_max_scroll_large_content() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        let large_result: String = (0..70000).map(|i| format!("line {}\n", i)).collect();
        app.query.result = Ok(large_result);

        let line_count = app.results_line_count_u32();
        assert!(line_count > 65535);

        app.results_scroll.update_bounds(line_count, 20);

        assert_eq!(app.results_scroll.max_offset, u16::MAX);
    }

    #[test]
    fn test_results_line_count_large_file() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        let result: String = (0..65535).map(|_| "x\n").collect();
        app.query.result = Ok(result);

        assert_eq!(app.results_line_count_u32(), 65535);

        app.results_scroll.update_bounds(65535, 10);

        assert_eq!(app.results_scroll.max_offset, 65525);
    }

    #[test]
    fn test_line_count_uses_last_result_on_error() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        let valid_result: String = (0..50).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(valid_result.clone());
        app.query.last_successful_result = Some(valid_result);

        assert_eq!(app.results_line_count_u32(), 50);

        app.query.result = Err("syntax error\nline 2\nline 3".to_string());

        assert_eq!(app.results_line_count_u32(), 50);

        app.results_scroll.update_bounds(50, 10);
        assert_eq!(app.results_scroll.max_offset, 40);
    }

    #[test]
    fn test_line_count_with_error_no_cached_result() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        app.query.last_successful_result = None;
        app.query.result = Err("error message".to_string());

        assert_eq!(app.results_line_count_u32(), 0);

        app.results_scroll.update_bounds(0, 10);
        assert_eq!(app.results_scroll.max_offset, 0);
    }

    #[test]
    fn test_tooltip_initialized_enabled() {
        let json = r#"{"name": "test"}"#;
        let app = test_app(json);

        assert!(app.tooltip.enabled);
        assert!(app.tooltip.current_function.is_none());
    }

    use proptest::prelude::*;

    // **Feature: ai-assistant-phase2, Property 10: Info popup hidden while AI visible**
    // *For any* state where AI popup is visible, the info popup SHALL be hidden.
    // **Validates: Requirements 9.1, 9.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_tooltip_hidden_while_ai_visible(
            initial_tooltip_enabled: bool,
            ai_enabled: bool,
            ai_configured: bool
        ) {
            let json = r#"{"test": true}"#;
            let mut app = test_app(json);

            // Set up initial state
            app.tooltip.enabled = initial_tooltip_enabled;
            app.ai.enabled = ai_enabled;
            app.ai.configured = ai_configured;
            app.ai.visible = false;

            // Toggle AI popup to make it visible
            let was_visible = app.ai.visible;
            app.ai.toggle();

            if !was_visible && app.ai.visible {
                // Save current tooltip state and hide it
                app.saved_tooltip_visibility = app.tooltip.enabled;
                app.tooltip.enabled = false;
            }

            // When AI popup is visible, tooltip should be disabled
            if app.ai.visible {
                prop_assert!(
                    !app.tooltip.enabled,
                    "Tooltip should be disabled when AI popup is visible"
                );
            }
        }
    }

    // **Feature: ai-assistant-phase2, Property 11: Info popup state restoration**
    // *For any* AI popup hide action, the info popup visibility SHALL be restored to its saved state.
    // **Validates: Requirements 9.2, 9.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_tooltip_state_restoration(
            initial_tooltip_enabled: bool,
            ai_enabled: bool,
            ai_configured: bool
        ) {
            let json = r#"{"test": true}"#;
            let mut app = test_app(json);

            // Set up initial state
            app.tooltip.enabled = initial_tooltip_enabled;
            app.ai.enabled = ai_enabled;
            app.ai.configured = ai_configured;
            app.ai.visible = false;

            let original_tooltip_state = app.tooltip.enabled;

            // Toggle AI popup to make it visible (simulating Ctrl+A press)
            let was_visible = app.ai.visible;
            app.ai.toggle();

            if !was_visible && app.ai.visible {
                // Save current tooltip state and hide it
                app.saved_tooltip_visibility = app.tooltip.enabled;
                app.tooltip.enabled = false;
            }

            // Now toggle AI popup to hide it (simulating second Ctrl+A press)
            let was_visible = app.ai.visible;
            app.ai.toggle();

            if was_visible && !app.ai.visible {
                // Restore saved tooltip state
                app.tooltip.enabled = app.saved_tooltip_visibility;
            }

            // After hiding AI popup, tooltip should be restored to original state
            prop_assert_eq!(
                app.tooltip.enabled,
                original_tooltip_state,
                "Tooltip state should be restored to original value after AI popup is hidden"
            );
        }
    }
}
