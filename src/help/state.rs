use crate::scroll::ScrollState;

/// Help popup state
pub struct HelpPopupState {
    pub visible: bool,
    pub scroll: ScrollState,
}

impl HelpPopupState {
    /// Create a new HelpPopupState (initially hidden)
    pub fn new() -> Self {
        Self {
            visible: false,
            scroll: ScrollState::new(),
        }
    }
}

impl Default for HelpPopupState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_help_state() {
        let state = HelpPopupState::new();
        assert!(!state.visible);
        assert_eq!(state.scroll.offset, 0);
    }
}
