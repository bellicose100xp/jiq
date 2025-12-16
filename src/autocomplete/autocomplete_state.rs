use std::fmt;

use crate::app::App;
use crate::autocomplete::update_suggestions;

pub fn update_suggestions_from_app(app: &mut App) {
    let query = app.input.query().to_string();
    let cursor_pos = app.input.textarea.cursor().1; // Column position
    let result = app.query.last_successful_result_unformatted.clone();
    let result_type = app.query.base_type_for_suggestions.clone();

    update_suggestions(
        &mut app.autocomplete,
        &query,
        cursor_pos,
        result.as_deref(),
        result_type,
        &app.input.brace_tracker,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionType {
    Function,
    Field,
    Operator,
    Pattern,
}

impl fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuggestionType::Function => write!(f, "function"),
            SuggestionType::Field => write!(f, "field"),
            SuggestionType::Operator => write!(f, "operator"),
            SuggestionType::Pattern => write!(f, "iterator"),
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
            is_visible: false,
        }
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        self.suggestions = suggestions;
        self.selected_index = 0;
        self.is_visible = !self.suggestions.is_empty();
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.suggestions.clear();
        self.selected_index = 0;
    }

    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.suggestions.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.suggestions.len() - 1;
            } else {
                self.selected_index -= 1;
            }
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
}
