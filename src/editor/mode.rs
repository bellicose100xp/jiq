use crate::editor::char_search::{SearchDirection, SearchType};

/// Scope for text object operations (inner vs around)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextObjectScope {
    /// Inner - select content inside delimiters (ci", di(, etc.)
    Inner,
    /// Around - select content including delimiters (ca", da(, etc.)
    Around,
}

/// VIM editing modes for the input field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorMode {
    /// Insert mode - typing inserts characters
    #[default]
    Insert,
    /// Normal mode - VIM navigation and commands
    Normal,
    /// Operator mode - waiting for motion after operator (d or c)
    Operator(char),
    /// CharSearch mode - waiting for target character after f/F/t/T
    CharSearch(SearchDirection, SearchType),
    /// TextObject mode - waiting for text object target after operator + i/a
    TextObject(char, TextObjectScope),
}

impl EditorMode {
    /// Get the display string for the mode indicator
    pub fn display(&self) -> String {
        match self {
            EditorMode::Insert => "INSERT".to_string(),
            EditorMode::Normal => "NORMAL".to_string(),
            EditorMode::Operator(op) => format!("OPERATOR({})", op),
            EditorMode::CharSearch(dir, st) => {
                let dir_char = match dir {
                    SearchDirection::Forward => match st {
                        SearchType::Find => 'f',
                        SearchType::Till => 't',
                    },
                    SearchDirection::Backward => match st {
                        SearchType::Find => 'F',
                        SearchType::Till => 'T',
                    },
                };
                format!("CHAR({})", dir_char)
            }
            EditorMode::TextObject(op, scope) => {
                let scope_char = match scope {
                    TextObjectScope::Inner => 'i',
                    TextObjectScope::Around => 'a',
                };
                format!("{}{}…", op, scope_char)
            }
        }
    }
}

#[cfg(test)]
#[path = "mode_tests.rs"]
mod mode_tests;
