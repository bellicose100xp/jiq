use std::fmt;

use crate::app::App;
use crate::autocomplete::update_suggestions;
use crate::scroll::Scrollable;

pub const MAX_VISIBLE_SUGGESTIONS: usize = 10;

pub fn update_suggestions_from_app(app: &mut App) {
    // Only update if query state is available
    let query_state = match &app.query {
        Some(q) => q,
        None => {
            app.autocomplete.hide();
            return;
        }
    };

    let query = app.input.query().to_string();
    let cursor_pos = app.input.textarea.cursor().1; // Column position
    let result_parsed = query_state.last_successful_result_parsed.clone();
    let result_type = query_state.base_type_for_suggestions.clone();
    let original_json = query_state.executor.json_input_parsed();
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
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionType {
    Function,
    Field,
    Operator,
    Pattern,
    Variable,
}

impl fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuggestionType::Function => write!(f, "function"),
            SuggestionType::Field => write!(f, "field"),
            SuggestionType::Operator => write!(f, "operator"),
            SuggestionType::Pattern => write!(f, "iterator"),
            SuggestionType::Variable => write!(f, "variable"),
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
