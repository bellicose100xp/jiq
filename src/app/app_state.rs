use crate::ai::AiState;
use crate::autocomplete::autocomplete_state::ValueMemo;
use crate::autocomplete::{self, AutocompleteState};
use crate::config::{ClipboardBackend, Config};
use crate::help::HelpPopupState;
use crate::history::HistoryState;
use crate::input::loader::LoaderSource;
use crate::input::{FileLoader, InputState, PasteRecoveryState, SourcePickerState};
use crate::layout::LayoutRegions;
use crate::notification::NotificationState;
use crate::path_at_cursor::PathAtCursorCache;
use crate::query::{Debouncer, QueryState};
use crate::query_undo::{QueryUndoRing, ViewportState};
use crate::results::cursor_state::CursorState;
use crate::save::SaveState;
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
    pub paste_recovery: Option<PasteRecoveryState>,
    pub source_picker: Option<SourcePickerState>,
    pub focus: Focus,
    pub results_scroll: ScrollState,
    pub results_cursor: CursorState,
    pub output_mode: Option<OutputMode>,
    pub should_quit: bool,
    pub autocomplete: AutocompleteState,
    pub(crate) value_memo: ValueMemo,
    pub error_overlay_visible: bool,
    pub history: HistoryState,
    pub help: HelpPopupState,
    pub notification: NotificationState,
    pub clipboard_backend: ClipboardBackend,
    pub tooltip: TooltipState,
    pub stats: StatsState,
    pub path_at_cursor: PathAtCursorCache,
    pub query_undo: QueryUndoRing,
    /// Viewport state captured by `<` (drill-back) that must be applied
    /// once the worker delivers the restored query's result. The restore
    /// can't run synchronously because the freshly-rewritten input still
    /// shows the *drilled* result's line layout until the async query
    /// completes — restoring against the wrong layout clamps the cursor.
    pub pending_viewport_restore: Option<ViewportState>,
    pub debouncer: Debouncer,
    pub search: SearchState,
    pub snippets: SnippetState,
    pub save: SaveState,
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
    pub layout_regions: LayoutRegions,
    pub array_sample_size: usize,
    pub double_click: super::double_click::DoubleClickTracker,
    /// Whether the mouse is currently hovering the clickable Back badge on
    /// the results-pane top border. Drives the badge's hover styling.
    pub back_button_hovered: bool,
}

impl App {
    /// Create App with deferred file loading.
    pub fn new_with_loader(loader: FileLoader, config: &Config) -> Self {
        Self::new_with_pre_input(Some(loader), None, None, config)
    }

    /// Create App that drops directly into the manual paste-recovery
    /// flow, without first reading the clipboard. Used by `--paste` and
    /// the source picker's Paste option.
    pub fn new_with_paste_recovery(state: PasteRecoveryState, config: &Config) -> Self {
        Self::new_with_pre_input(None, Some(state), None, config)
    }

    /// Create App that opens with the source picker visible. Used on
    /// bare TTY launches (no file argument, no piped stdin, no
    /// `--clipboard` / `--paste`). The picker holds its own
    /// launch-time clipboard snapshot so the post-confirm load doesn't
    /// re-read the clipboard.
    pub fn new_with_source_picker(state: SourcePickerState, config: &Config) -> Self {
        Self::new_with_pre_input(None, None, Some(state), config)
    }

    fn new_with_pre_input(
        loader: Option<FileLoader>,
        paste_recovery: Option<PasteRecoveryState>,
        source_picker: Option<SourcePickerState>,
        config: &Config,
    ) -> Self {
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
            file_loader: loader,
            source_picker,
            paste_recovery,
            focus: Focus::InputField,
            results_scroll: ScrollState::new(),
            results_cursor: CursorState::new(),
            output_mode: None,
            should_quit: false,
            autocomplete: AutocompleteState::new(),
            value_memo: ValueMemo::new(),
            error_overlay_visible: false,
            history: HistoryState::new(),
            help: HelpPopupState::new(),
            notification: NotificationState::new(),
            clipboard_backend: config.clipboard.backend,
            tooltip: TooltipState::new(tooltip_enabled),
            stats: StatsState::default(),
            path_at_cursor: PathAtCursorCache::new(),
            query_undo: QueryUndoRing::new(),
            pending_viewport_restore: None,
            debouncer: Debouncer::new(),
            search: SearchState::new(),
            snippets: SnippetState::new(),
            save: SaveState::new(),
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
            layout_regions: LayoutRegions::new(),
            array_sample_size: config.autocomplete.array_sample_size,
            double_click: super::double_click::DoubleClickTracker::new(),
            back_button_hovered: false,
        }
    }

    /// Commit the source picker's currently-highlighted choice and
    /// transition the app into the matching pre-input state. Called on
    /// Enter / mouse click / `c` / `p`.
    ///
    /// Clipboard branch reuses the cached bytes from the launch peek,
    /// so the system clipboard is never re-read. Paste branch enters
    /// the explicit-paste editor (cyan border, neutral title).
    pub fn confirm_source_picker(&mut self) {
        let Some(picker) = self.source_picker.take() else {
            return;
        };
        match picker.selection {
            crate::input::SourceChoice::Clipboard => {
                if let Some(bytes) = picker.clipboard_cache {
                    self.file_loader = Some(FileLoader::from_clipboard_string(bytes));
                } else {
                    // Picker only allows confirming Clipboard when the
                    // cache is non-empty (peek was Usable). If we ever
                    // arrive here, fall back to a fresh read so the
                    // user isn't left in a stuck state.
                    log::warn!("confirm_source_picker: Clipboard chosen with no cache; re-reading");
                    self.file_loader = Some(FileLoader::load_clipboard_blocking());
                }
            }
            crate::input::SourceChoice::Paste => {
                self.paste_recovery = Some(PasteRecoveryState::new_explicit());
            }
        }
        self.mark_dirty();
        // For the clipboard branch the loader is already in Complete state, so
        // poll right here to initialize QueryState. Otherwise the main loop
        // would block on the next key event before polling, and the user would
        // need to press a second key before the data appears.
        self.poll_file_loader();
    }

    /// Poll file loader and initialize QueryState when complete
    pub fn poll_file_loader(&mut self) {
        let polled = if let Some(loader) = &mut self.file_loader {
            loader.poll().map(|res| (res, loader.source))
        } else {
            None
        };

        if let Some((result, source)) = polled {
            self.mark_dirty();
            match result {
                Ok(json_input) => {
                    self.initialize_from_json(json_input);
                }
                Err(e) => {
                    log::error!("File loader error: {:?}", e);
                    if source == LoaderSource::Clipboard {
                        // Drop into paste recovery: surface only the
                        // single-sentence diagnosis line (the multi-line
                        // "Usage:" block is past at this point). No
                        // notification is shown — the recovery panel
                        // itself carries the same instruction inline.
                        let original = first_line(&e.to_string());
                        log::debug!(
                            "paste-recovery: ENTER recovery, error_message={:?}",
                            original
                        );
                        self.paste_recovery = Some(PasteRecoveryState::new(original));
                        self.file_loader = None;
                    } else {
                        // File / stdin source: existing behavior unchanged.
                        // Keep loader for state tracking; full details in
                        // results area.
                        self.notification.show_error("Failed to load file");
                    }
                }
            }
        }
    }

    /// Build the initial QueryState (and dependent caches) from a JSON
    /// input string. Shared by the loader-success path and the
    /// paste-recovery acceptance path.
    fn initialize_from_json(&mut self, json_input: String) {
        log::debug!("Initialising from JSON: {} bytes", json_input.len());
        self.query = Some(QueryState::new_with_sample_size(
            json_input.clone(),
            self.array_sample_size,
        ));

        let schema_input = crate::json::extract_first_json_value(&json_input)
            .unwrap_or_else(|| json_input.clone());

        self.input_json_schema = crate::json::extract_json_schema_dynamic(&schema_input).map(|s| {
            crate::ai::context::prepare_schema_for_context(&s, self.ai.max_context_length)
        });

        self.update_stats();
        self.file_loader = None;

        if self.ai.visible && self.ai.enabled && self.ai.configured {
            self.trigger_ai_request();
        }
    }

    /// Accept a JSON string from the paste-recovery flow and continue as
    /// if the JSON had been loaded normally.
    pub fn accept_paste_recovery_json(&mut self, json_input: String) {
        let bytes = json_input.len();
        self.initialize_from_json(json_input);
        self.paste_recovery = None;
        self.notification.show(&format!(
            "Loaded {} bytes — type a query, Enter outputs result",
            bytes
        ));
        self.mark_dirty();
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Request a clean shutdown. Used by the bench-script runner once the
    /// script finishes so the perf summary dumps via the normal exit path.
    pub fn request_quit(&mut self) {
        self.should_quit = true;
    }

    /// Clear a pending quit flag. Used by the bench-script runner so that
    /// keystrokes which incidentally set should_quit (Enter, Ctrl-C, q,
    /// Ctrl-Q, Shift-Enter) don't short-circuit the script before all its
    /// directives have run. Also resets `output_mode` because the same
    /// keystrokes set both fields together — leaving output_mode set would
    /// cause `handle_output()` to write the query/result to stdout at
    /// exit and pollute bench harness output.
    pub fn cancel_quit(&mut self) {
        self.should_quit = false;
        self.output_mode = None;
    }

    /// True while a query is in flight on the worker thread or the
    /// debouncer has a pending execution. Used by the bench-script runner
    /// to drain the worker before quitting so worker-thread perf timers
    /// have time to drop and record into the tally.
    pub fn has_pending_query(&self) -> bool {
        self.debouncer.has_pending() || self.query.as_ref().is_some_and(|q| q.is_pending())
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
        self.path_at_cursor.invalidate();
    }

    /// Resolve the jq path of the value pretty-printed on the current
    /// results-pane cursor row, using the parsed-result cache. Returns
    /// `None` when no successful result exists, the result is a synthetic
    /// merge of multiple top-level documents, the result is non-JSON, or
    /// the cursor row maps to no value.
    pub fn current_cursor_path(&mut self) -> Option<crate::json_path::JsonPath> {
        let row = self.results_cursor.cursor_line();
        self.path_at_row(row)
    }

    /// Resolve the jq path of the value pretty-printed on a specific
    /// `row`. Subject to the same gating as [`current_cursor_path`].
    /// Used by the renderer to show the match-row path while search is
    /// active, and by `>` from search mode to drill into the match.
    pub fn path_at_row(&mut self, row: u32) -> Option<crate::json_path::JsonPath> {
        let query_state = self.query.as_ref()?;
        if query_state.is_empty_result {
            return None;
        }
        if query_state.result.is_err() {
            return None;
        }
        if query_state.is_synthetic_merge {
            return None;
        }
        let parsed = query_state.last_successful_result_parsed.as_ref()?;
        self.path_at_cursor.resolve(parsed, row)
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

    /// Switch focus to the results pane, saving and hiding AI/tooltip/autocomplete visibility
    pub fn focus_results_pane(&mut self) {
        if self.focus == Focus::ResultsPane {
            return;
        }
        self.saved_ai_visibility_for_results = self.ai.visible;
        self.saved_tooltip_visibility_for_results = self.tooltip.enabled;
        self.ai.visible = false;
        self.tooltip.enabled = false;
        self.autocomplete.hide();
        self.focus = Focus::ResultsPane;
    }

    /// Switch focus to the input field, restoring AI/tooltip visibility
    pub fn focus_input_field(&mut self) {
        if self.focus == Focus::InputField {
            return;
        }
        self.ai.visible = self.saved_ai_visibility_for_results;
        self.tooltip.enabled = self.saved_tooltip_visibility_for_results;
        self.focus = Focus::InputField;
    }
}

/// Extract the first non-empty line of a multi-line error message and
/// strip our `JiqError` Display prefix. Used to surface only the
/// diagnosis sentence in the paste-recovery view (the loader's full
/// "Usage:" block is irrelevant once we are past it, and the
/// "IO error: " prefix is implementation detail leaking from
/// `thiserror`'s Display impl).
fn first_line(s: &str) -> String {
    let line = s.lines().find(|l| !l.trim().is_empty()).unwrap_or(s);
    line.strip_prefix("IO error: ")
        .or_else(|| line.strip_prefix("Invalid JSON input: "))
        .unwrap_or(line)
        .to_string()
}

#[cfg(test)]
#[path = "app_state_tests.rs"]
mod app_state_tests;
