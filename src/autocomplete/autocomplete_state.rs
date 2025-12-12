use std::fmt;

use crate::app::App;
use crate::autocomplete::update_suggestions;

/// Update autocomplete suggestions from App context
///
/// This function extracts the necessary data from the App struct and delegates
/// to the existing `update_suggestions()` function. This pattern allows the App
/// to delegate feature-specific logic to the autocomplete module.
///
/// # Arguments
/// * `app` - Mutable reference to the App struct
pub fn update_suggestions_from_app(app: &mut App) {
    // Clone values to avoid borrow conflicts
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

/// Type of suggestion being offered
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionType {
    /// jq built-in function (e.g., map, select, keys)
    Function,
    /// JSON field name from the input data
    Field,
    /// jq operator (e.g., |, //, and, or)
    Operator,
    /// Common filter pattern (e.g., .[], .[0])
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

/// JSON field type for providing type information in suggestions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonFieldType {
    /// String value
    String,
    /// Numeric value (integer or float)
    Number,
    /// Boolean value (true/false)
    Boolean,
    /// Null value
    Null,
    /// Object (nested fields)
    Object,
    /// Array (list of values) - unknown element type
    Array,
    /// Array with known element type (based on first element)
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

/// A single autocomplete suggestion
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// The text to insert
    pub text: String,
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Optional description/help text
    pub description: Option<String>,
    /// Optional JSON field type (for Field suggestions)
    pub field_type: Option<JsonFieldType>,
    /// Display signature like "select(expr)" for functions
    pub signature: Option<String>,
    /// Whether to append ( on insertion (for functions requiring arguments)
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

    /// Set the display signature for this suggestion
    pub fn with_signature(mut self, sig: impl Into<String>) -> Self {
        self.signature = Some(sig.into());
        self
    }

    /// Set whether this suggestion needs parentheses on insertion
    pub fn with_needs_parens(mut self, needs_parens: bool) -> Self {
        self.needs_parens = needs_parens;
        self
    }
}

/// State for the autocomplete system
#[derive(Debug, Clone)]
pub struct AutocompleteState {
    /// Current list of suggestions
    suggestions: Vec<Suggestion>,
    /// Index of currently selected suggestion
    selected_index: usize,
    /// Whether the autocomplete popup is visible
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

    /// Update suggestions and show the autocomplete popup
    pub fn update_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        self.suggestions = suggestions;
        self.selected_index = 0;
        self.is_visible = !self.suggestions.is_empty();
    }

    /// Hide the autocomplete popup
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.suggestions.clear();
        self.selected_index = 0;
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if !self.suggestions.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.suggestions.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Get the currently selected suggestion
    pub fn selected(&self) -> Option<&Suggestion> {
        if self.is_visible && self.selected_index < self.suggestions.len() {
            Some(&self.suggestions[self.selected_index])
        } else {
            None
        }
    }

    /// Check if autocomplete popup is visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Get reference to current suggestions
    pub fn suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
}
