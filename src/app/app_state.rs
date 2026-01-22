use crate::ai::AiState;
use crate::autocomplete::{self, AutocompleteState};
use crate::config::{ClipboardBackend, Config};
use crate::help::HelpPopupState;
use crate::history::HistoryState;
use crate::input::{FileLoader, InputState};
use crate::notification::NotificationState;
use crate::query::{Debouncer, QueryState};
use crate::scroll::ScrollState;
use crate::search::SearchState;
use crate::snippets::SnippetState;
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
    pub query: Option<QueryState>,
    pub file_loader: Option<FileLoader>,
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
    pub snippets: SnippetState,
    pub ai: AiState,
    pub saved_tooltip_visibility: bool,
    pub saved_ai_visibility_for_search: bool,
    pub saved_tooltip_visibility_for_search: bool,
    pub saved_focus_for_search: Focus,
    pub saved_ai_visibility_for_results: bool,
    pub saved_tooltip_visibility_for_results: bool,
    pub input_json_schema: Option<String>,
    pub frame_count: u64,
    pub needs_render: bool,
}

impl App {
    /// Create App with deferred file loading
    pub fn new_with_loader(loader: FileLoader, config: &Config) -> Self {
        let anthropic_configured =
            config.ai.anthropic.api_key.is_some() && config.ai.anthropic.model.is_some();
        let bedrock_configured =
            config.ai.bedrock.region.is_some() && config.ai.bedrock.model.is_some();
        let openai_configured =
            config.ai.openai.api_key.is_some() && config.ai.openai.model.is_some();
        let gemini_configured =
            config.ai.gemini.api_key.is_some() && config.ai.gemini.model.is_some();

        let provider_name = match config.ai.provider {
            Some(crate::config::ai_types::AiProviderType::Anthropic) => "Anthropic",
            Some(crate::config::ai_types::AiProviderType::Bedrock) => "Bedrock",
            Some(crate::config::ai_types::AiProviderType::Openai) => {
                // Check if using custom OpenAI-compatible endpoint
                let is_custom = config
                    .ai
                    .openai
                    .base_url
                    .as_ref()
                    .map(|url| !url.contains("api.openai.com"))
                    .unwrap_or(false);
                if is_custom {
                    "OpenAI-compatible"
                } else {
                    "OpenAI"
                }
            }
            Some(crate::config::ai_types::AiProviderType::Gemini) => "Gemini",
            None => "Not Configured",
        }
        .to_string();

        let ai_configured = config.ai.provider.is_some()
            && (anthropic_configured
                || bedrock_configured
                || openai_configured
                || gemini_configured);

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

        let ai_state = AiState::new_with_config(
            config.ai.enabled,
            ai_configured,
            provider_name,
            model_name,
            config.ai.max_context_length as usize,
        );

        let tooltip_enabled = if ai_state.visible {
            false
        } else {
            config.tooltip.auto_show
        };

        Self {
            input: InputState::new(),
            query: None,
            file_loader: Some(loader),
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
            snippets: SnippetState::new(),
            ai: ai_state,
            saved_tooltip_visibility: config.tooltip.auto_show,
            saved_ai_visibility_for_search: false,
            saved_tooltip_visibility_for_search: false,
            saved_focus_for_search: Focus::InputField,
            saved_ai_visibility_for_results: false,
            saved_tooltip_visibility_for_results: false,
            input_json_schema: None,
            frame_count: 0,
            needs_render: true,
        }
    }

    /// Poll file loader and initialize QueryState when complete
    pub fn poll_file_loader(&mut self) {
        if let Some(loader) = &mut self.file_loader
            && let Some(result) = loader.poll()
        {
            self.mark_dirty();
            match result {
                Ok(json_input) => {
                    self.query = Some(QueryState::new(json_input.clone()));

                    self.input_json_schema = crate::json::extract_json_schema_dynamic(&json_input)
                        .map(|s| {
                            crate::ai::context::prepare_schema_for_context(
                                &s,
                                self.ai.max_context_length,
                            )
                        });

                    // Initialize stats for initial result
                    self.update_stats();

                    self.file_loader = None;

                    // Ensure AI works on launch with deferred file loading
                    if self.ai.visible && self.ai.enabled && self.ai.configured {
                        self.trigger_ai_request();
                    }
                }
                Err(_e) => {
                    // Keep loader for state tracking
                    // Show brief notification - full error details in results area
                    self.notification.show_error("Failed to load file");
                }
            }
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
        self.query.as_ref().map_or(0, |q| q.line_count())
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
    pub fn trigger_ai_request(&mut self) {
        if !self.ai.configured {
            return;
        }

        let query_state = match &self.query {
            Some(q) => q,
            None => return,
        };

        let query = self.input.query().to_string();
        let cursor_pos = self.input.textarea.cursor().1;

        let ai_result: Result<String, String> = match &query_state.result {
            Ok(_) => query_state
                .last_successful_result_unformatted
                .as_ref()
                .map(|s| Ok(s.as_ref().clone()))
                .unwrap_or_else(|| Ok(String::new())),
            Err(e) => Err(e.clone()),
        };

        crate::ai::ai_events::handle_execution_result(
            &mut self.ai,
            &ai_result,
            &query,
            cursor_pos,
            crate::ai::context::ContextParams {
                input_schema: self.input_json_schema.as_deref(),
                base_query: query_state.base_query_for_suggestions.as_deref(),
                base_query_result: query_state
                    .last_successful_result_for_context
                    .as_deref()
                    .map(|s| s.as_ref()),
                is_empty_result: query_state.is_empty_result,
            },
        );
    }

    pub fn mark_dirty(&mut self) {
        self.needs_render = true;
    }

    pub fn clear_dirty(&mut self) {
        self.needs_render = false;
    }

    /// Returns true if continuous rendering is needed for animations
    fn needs_animation(&self) -> bool {
        // Query execution spinner
        if let Some(ref query) = self.query
            && query.is_pending()
        {
            return true;
        }
        // AI loading spinner
        if self.ai.loading {
            return true;
        }
        // File loading spinner
        if self.file_loader.as_ref().is_some_and(|l| l.is_loading()) {
            return true;
        }
        // Notification timer expiry check
        if self.notification.current().is_some() {
            return true;
        }
        false
    }

    pub fn should_render(&self) -> bool {
        self.needs_render || self.needs_animation()
    }
}

#[cfg(test)]
#[path = "app_state_tests.rs"]
mod app_state_tests;
