use std::fmt;
use std::sync::Arc;

use crate::app::App;
use crate::autocomplete::json_navigator::navigate_multi;
use crate::autocomplete::path_parser::parse_path;
use crate::autocomplete::update_suggestions;
use crate::autocomplete::value_collector::collect_distinct_strings;
use crate::autocomplete::value_trigger::{TriggerKind, ValueTrigger, classify};
use crate::scroll::Scrollable;

pub const MAX_VISIBLE_SUGGESTIONS: usize = 10;

/// One-slot memo for value suggestions. Caches the most recent walk so that
/// typing additional partial characters at the same trigger site filters the
/// already-collected list instead of re-walking the JSON.
#[derive(Debug, Default)]
pub struct ValueMemo {
    cache_key: Option<String>,
    values: Arc<Vec<String>>,
}

impl ValueMemo {
    pub fn new() -> Self {
        Self::default()
    }

    fn matches(&self, key: &str) -> bool {
        self.cache_key.as_deref() == Some(key)
    }

    fn put(&mut self, key: String, values: Vec<String>) {
        self.cache_key = Some(key);
        self.values = Arc::new(values);
    }

    fn values(&self) -> Arc<Vec<String>> {
        self.values.clone()
    }
}

pub fn update_suggestions_from_app(app: &mut App) {
    let query_state = match &app.query {
        Some(q) => q,
        None => {
            app.autocomplete.hide();
            return;
        }
    };

    let query = app.input.query().to_string();
    let cursor_char = app.input.textarea.cursor().1;
    let cursor_pos = crate::str_utils::char_pos_to_byte_pos(&query, cursor_char);
    let original_json = query_state.executor.json_input_parsed();

    if let Some(trigger) = classify(&query, cursor_pos) {
        let all_string_values = query_state.executor.all_string_values();

        let handled = update_value_suggestions(
            &mut app.autocomplete,
            &mut app.value_memo,
            &trigger,
            original_json.clone(),
            all_string_values,
        );
        if handled {
            return;
        }
    }

    let result_parsed = query_state.last_successful_result_parsed.clone();
    let result_type = query_state.base_type_for_suggestions.clone();
    let all_field_names = query_state.executor.all_field_names();

    update_suggestions(
        &mut app.autocomplete,
        &query,
        cursor_pos,
        result_parsed,
        result_type,
        original_json,
        all_field_names,
        &app.input.brace_tracker,
        app.array_sample_size,
    );
}

/// Returns `true` when value autocomplete handled this keystroke. Returns
/// `false` only for `has`/`in` calls whose LHS resolves to an object, where
/// the caller should fall through to `ObjectKeyContext` dispatch.
fn update_value_suggestions(
    autocomplete: &mut AutocompleteState,
    memo: &mut ValueMemo,
    trigger: &ValueTrigger,
    original_json: Option<Arc<serde_json::Value>>,
    all_string_values: Arc<Vec<String>>,
) -> bool {
    let json = match original_json {
        Some(j) => j,
        None => {
            autocomplete.hide();
            return true;
        }
    };

    if trigger.kind == TriggerKind::HasOrIn && lhs_resolves_to_object(&json, trigger) {
        return false;
    }

    let cache_key = build_memo_key(trigger);
    let collected = if memo.matches(&cache_key) {
        memo.values()
    } else {
        let values = collect_for_trigger(trigger, &json, &all_string_values);
        memo.put(cache_key, values);
        memo.values()
    };

    let suggestions = build_value_suggestions(&collected, &trigger.partial);
    autocomplete.update_suggestions(suggestions);
    true
}

/// Memo key derived from the trigger's path. Two consecutive keystrokes at
/// the same trigger site (e.g. `"a` → `"ac`) share a key, so the second
/// re-uses the first walk.
fn build_memo_key(trigger: &ValueTrigger) -> String {
    match &trigger.lhs_path {
        Some(p) => format!("path:{p}"),
        None => "global".to_string(),
    }
}

/// Walk `original_json` with the trigger's folded absolute path and collect
/// distinct string values. If folding gave up (`lhs_path` is None) or the
/// walk yields no strings, fall back to the executor's precomputed
/// `all_string_values` (the global firehose).
///
/// Mid-query and end-of-query both use the same source — `original_json` —
/// because the result_parsed cache is shape-unreliable for streaming queries
/// (synthetic merges) and the folded path is rooted at the JSON root anyway.
fn collect_for_trigger(
    trigger: &ValueTrigger,
    original_json: &serde_json::Value,
    all_string_values: &[String],
) -> Vec<String> {
    if let Some(path) = trigger.lhs_path.as_deref() {
        // The folded path is COMPLETE — every identifier is a final field
        // name, including any trailing one that `parse_path` would otherwise
        // treat as the user's in-progress input.
        let parsed = parse_path(path);
        let mut segments = parsed.segments;
        if !parsed.partial.is_empty() {
            segments.push(crate::autocomplete::path_parser::PathSegment::Field(
                parsed.partial,
            ));
        }
        if !segments.is_empty() {
            let navigated = navigate_multi(original_json, &segments, VALUE_SAMPLE_SIZE);
            if !navigated.is_empty() {
                let strings = collect_distinct_strings(&navigated);
                if !strings.is_empty() {
                    return strings;
                }
            }
        }
    }
    all_string_values.to_vec()
}

/// Sample size used when fanning out arrays during value collection. Higher
/// than the field-name `DEFAULT_ARRAY_SAMPLE_SIZE = 10` so we don't miss
/// values when distinct values cluster in non-uniform array slices.
const VALUE_SAMPLE_SIZE: usize = 100;

fn lhs_resolves_to_object(json: &serde_json::Value, trigger: &ValueTrigger) -> bool {
    let path = match &trigger.lhs_path {
        Some(p) => p,
        None => return false,
    };
    let parsed = parse_path(path);
    let mut segments = parsed.segments;
    if !parsed.partial.is_empty() {
        segments.push(crate::autocomplete::path_parser::PathSegment::Field(
            parsed.partial,
        ));
    }
    let values = navigate_multi(json, &segments, 1);
    values
        .into_iter()
        .any(|v| matches!(v, serde_json::Value::Object(_)))
}

fn build_value_suggestions(values: &[String], partial: &str) -> Vec<Suggestion> {
    let lower = partial.to_lowercase();
    values
        .iter()
        .filter(|v| {
            if lower.is_empty() {
                true
            } else {
                v.to_lowercase().contains(&lower)
            }
        })
        .take(MAX_VISIBLE_SUGGESTIONS * 4)
        .map(|v| {
            Suggestion::new_with_type(
                v.clone(),
                SuggestionType::Value,
                Some(JsonFieldType::String),
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionType {
    Function,
    Field,
    Operator,
    Pattern,
    Variable,
    Value,
}

impl fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuggestionType::Function => write!(f, "function"),
            SuggestionType::Field => write!(f, "field"),
            SuggestionType::Operator => write!(f, "operator"),
            SuggestionType::Pattern => write!(f, "iterator"),
            SuggestionType::Variable => write!(f, "variable"),
            SuggestionType::Value => write!(f, "value"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonFieldType {
    String,
    Number,
    Boolean,
    Null,
    Object,
    Array,
    ArrayOf(Box<JsonFieldType>),
}

impl fmt::Display for JsonFieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonFieldType::String => write!(f, "String"),
            JsonFieldType::Number => write!(f, "Number"),
            JsonFieldType::Boolean => write!(f, "Boolean"),
            JsonFieldType::Null => write!(f, "Null"),
            JsonFieldType::Object => write!(f, "Object"),
            JsonFieldType::Array => write!(f, "Array"),
            JsonFieldType::ArrayOf(inner) => write!(f, "Array[{}]", inner),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub suggestion_type: SuggestionType,
    pub description: Option<String>,
    pub field_type: Option<JsonFieldType>,
    pub signature: Option<String>,
    pub needs_parens: bool,
}

impl Suggestion {
    pub fn new(text: impl Into<String>, suggestion_type: SuggestionType) -> Self {
        Self {
            text: text.into(),
            suggestion_type,
            description: None,
            field_type: None,
            signature: None,
            needs_parens: false,
        }
    }

    pub fn new_with_type(
        text: impl Into<String>,
        suggestion_type: SuggestionType,
        field_type: Option<JsonFieldType>,
    ) -> Self {
        Self {
            text: text.into(),
            suggestion_type,
            description: None,
            field_type,
            signature: None,
            needs_parens: false,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_signature(mut self, sig: impl Into<String>) -> Self {
        self.signature = Some(sig.into());
        self
    }

    pub fn with_needs_parens(mut self, needs_parens: bool) -> Self {
        self.needs_parens = needs_parens;
        self
    }
}

#[derive(Debug, Clone)]
pub struct AutocompleteState {
    suggestions: Vec<Suggestion>,
    selected_index: usize,
    scroll_offset: usize,
    is_visible: bool,
}

impl Default for AutocompleteState {
    fn default() -> Self {
        Self::new()
    }
}

impl AutocompleteState {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            is_visible: false,
        }
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        self.suggestions = suggestions;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.is_visible = !self.suggestions.is_empty();
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.suggestions.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() && self.selected_index < self.suggestions.len() - 1 {
            self.selected_index += 1;
            self.adjust_scroll_to_selection();
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll_to_selection();
        }
    }

    fn adjust_scroll_to_selection(&mut self) {
        if self.selected_index >= self.scroll_offset + MAX_VISIBLE_SUGGESTIONS {
            self.scroll_offset = self.selected_index - MAX_VISIBLE_SUGGESTIONS + 1;
        } else if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    pub fn selected(&self) -> Option<&Suggestion> {
        if self.is_visible && self.selected_index < self.suggestions.len() {
            Some(&self.suggestions[self.selected_index])
        } else {
            None
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Highlight a specific suggestion, clamped to the available list.
    /// Used by mouse interactions where the click target is computed from
    /// the rendered viewport.
    pub fn set_selected_index(&mut self, index: usize) {
        if self.suggestions.is_empty() {
            return;
        }
        let clamped = index.min(self.suggestions.len() - 1);
        self.selected_index = clamped;
        self.adjust_scroll_to_selection();
    }

    #[allow(dead_code)]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn visible_suggestions(&self) -> impl Iterator<Item = (usize, &Suggestion)> {
        self.suggestions
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(MAX_VISIBLE_SUGGESTIONS)
    }
}

impl Scrollable for AutocompleteState {
    fn scroll_view_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    fn scroll_view_down(&mut self, lines: usize) {
        let max = self.max_scroll();
        self.scroll_offset = (self.scroll_offset + lines).min(max);
    }

    fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    fn max_scroll(&self) -> usize {
        self.suggestions
            .len()
            .saturating_sub(MAX_VISIBLE_SUGGESTIONS)
    }

    fn viewport_size(&self) -> usize {
        MAX_VISIBLE_SUGGESTIONS
    }
}

#[cfg(test)]
#[path = "autocomplete_state_tests.rs"]
mod autocomplete_state_tests;
