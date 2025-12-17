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
#[path = "scan_state_tests.rs"]
mod scan_state_tests;
