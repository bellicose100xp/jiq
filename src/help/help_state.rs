use crate::scroll::ScrollState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HelpTab {
    #[default]
    Global,
    Input,
    Result,
    History,
    AI,
    Search,
    Snippet,
}

impl HelpTab {
    pub const COUNT: usize = 7;

    pub fn all() -> &'static [HelpTab] {
        &[
            HelpTab::Global,
            HelpTab::Input,
            HelpTab::Result,
            HelpTab::History,
            HelpTab::AI,
            HelpTab::Search,
            HelpTab::Snippet,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            HelpTab::Global => 0,
            HelpTab::Input => 1,
            HelpTab::Result => 2,
            HelpTab::History => 3,
            HelpTab::AI => 4,
            HelpTab::Search => 5,
            HelpTab::Snippet => 6,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => HelpTab::Global,
            1 => HelpTab::Input,
            2 => HelpTab::Result,
            3 => HelpTab::History,
            4 => HelpTab::AI,
            5 => HelpTab::Search,
            6 => HelpTab::Snippet,
            _ => HelpTab::Global,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            HelpTab::Global => "Global",
            HelpTab::Input => "Input",
            HelpTab::Result => "Result",
            HelpTab::History => "History",
            HelpTab::AI => "AI",
            HelpTab::Search => "Search",
            HelpTab::Snippet => "Snippet",
        }
    }

    pub fn next(&self) -> Self {
        Self::from_index((self.index() + 1) % Self::COUNT)
    }

    pub fn prev(&self) -> Self {
        Self::from_index((self.index() + Self::COUNT - 1) % Self::COUNT)
    }
}

pub struct HelpPopupState {
    pub visible: bool,
    pub active_tab: HelpTab,
    scroll_per_tab: [ScrollState; HelpTab::COUNT],
}

impl HelpPopupState {
    pub fn new() -> Self {
        Self {
            visible: false,
            active_tab: HelpTab::Global,
            scroll_per_tab: [
                ScrollState::new(),
                ScrollState::new(),
                ScrollState::new(),
                ScrollState::new(),
                ScrollState::new(),
                ScrollState::new(),
                ScrollState::new(),
            ],
        }
    }

    pub fn current_scroll(&self) -> &ScrollState {
        &self.scroll_per_tab[self.active_tab.index()]
    }

    pub fn current_scroll_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll_per_tab[self.active_tab.index()]
    }

    pub fn reset(&mut self) {
        self.visible = false;
        self.active_tab = HelpTab::Global;
        for scroll in &mut self.scroll_per_tab {
            scroll.reset();
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
