//! Lexer-aware scanner state for jq queries.
//!
//! This module provides a state machine for tracking whether the scanner
//! is inside a string literal, handling escape sequences correctly.
//! This enables other components to skip syntactic characters inside strings.

/// Scanner state for lexer-aware parsing of jq queries.
///
/// This state machine tracks whether we're inside a string literal,
/// allowing components to correctly ignore syntactic characters
/// (like braces, brackets, operators) that appear inside strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanState {
    /// Normal code context - syntactic characters are meaningful
    #[default]
    Normal,
    /// Inside a string literal `"..."`
    InString,
    /// After a backslash inside a string (escape sequence)
    InStringEscape,
}

impl ScanState {
    /// Advance the scanner state based on the next character.
    ///
    /// Returns the new state after processing the character.
    ///
    /// # Arguments
    /// * `ch` - The character to process
    ///
    /// # Returns
    /// The new `ScanState` after processing the character
    pub fn advance(self, ch: char) -> Self {
        match self {
            ScanState::Normal => match ch {
                '"' => ScanState::InString,
                _ => ScanState::Normal,
            },
            ScanState::InString => match ch {
                '\\' => ScanState::InStringEscape,
                '"' => ScanState::Normal,
                _ => ScanState::InString,
            },
            ScanState::InStringEscape => {
                // After escape, return to string state regardless of character
                ScanState::InString
            }
        }
    }

    /// Check if currently inside a string literal.
    pub fn is_in_string(self) -> bool {
        matches!(self, ScanState::InString | ScanState::InStringEscape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_normal() {
        assert_eq!(ScanState::default(), ScanState::Normal);
    }

    #[test]
    fn test_enter_string() {
        let state = ScanState::Normal.advance('"');
        assert_eq!(state, ScanState::InString);
        assert!(state.is_in_string());
    }

    #[test]
    fn test_exit_string() {
        let state = ScanState::InString.advance('"');
        assert_eq!(state, ScanState::Normal);
        assert!(!state.is_in_string());
    }

    #[test]
    fn test_escape_in_string() {
        let state = ScanState::InString.advance('\\');
        assert_eq!(state, ScanState::InStringEscape);
        assert!(state.is_in_string());
    }

    #[test]
    fn test_escaped_quote() {
        let state = ScanState::InStringEscape.advance('"');
        assert_eq!(state, ScanState::InString);
        assert!(state.is_in_string());
    }

    #[test]
    fn test_escaped_backslash() {
        let state = ScanState::InStringEscape.advance('\\');
        assert_eq!(state, ScanState::InString);
    }

    #[test]
    fn test_normal_chars_stay_normal() {
        let state = ScanState::Normal.advance('a');
        assert_eq!(state, ScanState::Normal);
    }

    #[test]
    fn test_string_chars_stay_in_string() {
        let state = ScanState::InString.advance('a');
        assert_eq!(state, ScanState::InString);
    }

    #[test]
    fn test_full_string_scan() {
        // Scan through: "hello"
        let mut state = ScanState::Normal;
        for ch in "\"hello\"".chars() {
            state = state.advance(ch);
        }
        assert_eq!(state, ScanState::Normal);
    }

    #[test]
    fn test_escaped_quote_in_string() {
        // Scan through: "say \"hi\""
        let mut state = ScanState::Normal;
        for ch in "\"say \\\"hi\\\"\"".chars() {
            state = state.advance(ch);
        }
        assert_eq!(state, ScanState::Normal);
    }
}
