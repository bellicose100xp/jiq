#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanState {
    #[default]
    Normal,
    InString,
    InStringEscape,
}

impl ScanState {
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
            ScanState::InStringEscape => ScanState::InString,
        }
    }

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
        let mut state = ScanState::Normal;
        for ch in "\"hello\"".chars() {
            state = state.advance(ch);
        }
        assert_eq!(state, ScanState::Normal);
    }

    #[test]
    fn test_escaped_quote_in_string() {
        let mut state = ScanState::Normal;
        for ch in "\"say \\\"hi\\\"\"".chars() {
            state = state.advance(ch);
        }
        assert_eq!(state, ScanState::Normal);
    }
}
