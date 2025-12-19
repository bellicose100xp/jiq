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
        let openai_configured =
            config.ai.openai.api_key.is_some() && config.ai.openai.model.is_some();
        let gemini_configured =
            config.ai.gemini.api_key.is_some() && config.ai.gemini.model.is_some();

        // Determine provider name based on configuration
        let provider_name = match config.ai.provider {
            Some(crate::config::ai_types::AiProviderType::Anthropic) => "Anthropic",
            Some(crate::config::ai_types::AiProviderType::Bedrock) => "Bedrock",
            Some(crate::config::ai_types::AiProviderType::Openai) => "OpenAI",
            Some(crate::config::ai_types::AiProviderType::Gemini) => "Gemini",
            None => "Not Configured",
        }
        .to_string();

        // ai_configured is false when provider is None
        let ai_configured = config.ai.provider.is_some()
            && (anthropic_configured
                || bedrock_configured
                || openai_configured
                || gemini_configured);

        // Get model name based on provider
        let model_name = match config.ai.provider {
            Some(crate::config::ai_types::AiProviderType::Anthropic) => {
                config.ai.anthropic.model.clone().unwrap_or_default()
            }
            Some(crate::config::ai_types::AiProviderType::Bedrock) => {
                config.ai.bedrock.model.clone().unwrap_or_default()
            }
            Some(crate::config::ai_types::AiProviderType::Openai) => {
                config.ai.openai.model.clone().unwrap_or_default()
            }
            Some(crate::config::ai_types::AiProviderType::Gemini) => {
                config.ai.gemini.model.clone().unwrap_or_default()
            }
            None => String::new(),
        };

        let ai_state =
            AiState::new_with_config(config.ai.enabled, ai_configured, provider_name, model_name);

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

    /// Trigger an AI request for the current query context
    ///
    /// This extracts the current query, cursor position, JSON input, and context
    /// parameters from the app state and triggers an AI request via handle_execution_result.
    pub fn trigger_ai_request(&mut self) {
        if !self.ai.configured {
            return;
        }

        let query = self.input.query().to_string();
        let cursor_pos = self.input.textarea.cursor().1;
        let json_input = self.query.executor.json_input().to_string();

        crate::ai::ai_events::handle_execution_result(
            &mut self.ai,
            &self.query.result,
            &query,
            cursor_pos,
            &json_input,
            crate::ai::context::ContextParams {
                input_schema: self.input_json_schema.as_deref(),
                base_query: self.query.base_query_for_suggestions.as_deref(),
                base_query_result: self.query.last_successful_result.as_deref(),
            },
        );
    }
}

#[cfg(test)]
#[path = "app_state_tests.rs"]
mod app_state_tests;
