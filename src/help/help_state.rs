use crate::scroll::ScrollState;

pub struct HelpPopupState {
    pub visible: bool,
    pub scroll: ScrollState,
}

impl HelpPopupState {
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
#[path = "help_state_tests.rs"]
mod help_state_tests;
