use crate::scroll::ScrollState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HelpTab {
    #[default]
    Global,
    Input,
    Results,
    Search,
    Popups,
    AI,
}

impl HelpTab {
    pub const COUNT: usize = 6;

    pub fn all() -> &'static [HelpTab] {
        &[
            HelpTab::Global,
            HelpTab::Input,
            HelpTab::Results,
            HelpTab::Search,
            HelpTab::Popups,
            HelpTab::AI,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            HelpTab::Global => 0,
            HelpTab::Input => 1,
            HelpTab::Results => 2,
            HelpTab::Search => 3,
            HelpTab::Popups => 4,
            HelpTab::AI => 5,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => HelpTab::Global,
            1 => HelpTab::Input,
            2 => HelpTab::Results,
            3 => HelpTab::Search,
            4 => HelpTab::Popups,
            5 => HelpTab::AI,
            _ => HelpTab::Global,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            HelpTab::Global => "Global",
            HelpTab::Input => "Input",
            HelpTab::Results => "Results",
            HelpTab::Search => "Search",
            HelpTab::Popups => "Popups",
            HelpTab::AI => "AI",
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
