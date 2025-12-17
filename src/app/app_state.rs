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
    pub input_json_schema: Option<String>,
}

impl App {
    pub fn new(json_input: String, config: &Config) -> Self {
        // Check if AI is configured for either Anthropic or Bedrock
        let anthropic_configured =
            config.ai.anthropic.api_key.is_some() && config.ai.anthropic.model.is_some();
        let bedrock_configured =
            config.ai.bedrock.region.is_some() && config.ai.bedrock.model.is_some();
        let ai_configured = anthropic_configured || bedrock_configured;

        let ai_state = AiState::new_with_config(config.ai.enabled, ai_configured);

        let tooltip_enabled = if ai_state.visible {
            false
        } else {
            config.tooltip.auto_show
        };

        // Extract JSON schema once at startup for AI context
        let input_json_schema =
            crate::json::extract_json_schema(&json_input, crate::json::DEFAULT_SCHEMA_MAX_DEPTH);

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
            input_json_schema,
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
#[path = "app_state_tests.rs"]
mod app_state_tests;
