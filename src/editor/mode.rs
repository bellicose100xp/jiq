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
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        assert_eq!(EditorMode::default(), EditorMode::Insert);
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(EditorMode::Insert.display(), "INSERT");
        assert_eq!(EditorMode::Normal.display(), "NORMAL");
        assert_eq!(EditorMode::Operator('d').display(), "OPERATOR(d)");
        assert_eq!(EditorMode::Operator('c').display(), "OPERATOR(c)");
    }
}
