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
}

impl EditorMode {
    /// Get the display string for the mode indicator
    pub fn display(&self) -> String {
        match self {
            EditorMode::Insert => "INSERT".to_string(),
            EditorMode::Normal => "NORMAL".to_string(),
            EditorMode::Operator(op) => format!("OPERATOR({})", op),
        }
    }
}

#[cfg(test)]
#[path = "mode_tests.rs"]
mod mode_tests;
