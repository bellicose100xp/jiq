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
    hovered_tab: Option<HelpTab>,
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
            hovered_tab: None,
        }
    }

    pub fn current_scroll(&self) -> &ScrollState {
        &self.scroll_per_tab[self.active_tab.index()]
    }

    pub fn current_scroll_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll_per_tab[self.active_tab.index()]
    }

    pub fn get_hovered_tab(&self) -> Option<HelpTab> {
        self.hovered_tab
    }

    pub fn set_hovered_tab(&mut self, tab: Option<HelpTab>) {
        self.hovered_tab = tab;
    }

    pub fn clear_hovered_tab(&mut self) {
        self.hovered_tab = None;
    }

    /// Spacing between tabs in the tab bar (must match render)
    const TAB_DIVIDER_WIDTH: u16 = 3;

    /// Find which tab is at the given X coordinate within the tab bar
    ///
    /// Returns the tab if found, None if X is outside all tabs or in divider space.
    pub fn tab_at_x(&self, relative_x: u16) -> Option<HelpTab> {
        let mut x_pos: u16 = 0;

        for tab in HelpTab::all() {
            let label_width = self.tab_label_width(*tab);
            let tab_end = x_pos.saturating_add(label_width);

            if relative_x >= x_pos && relative_x < tab_end {
                return Some(*tab);
            }

            // Move past this tab and the divider space
            x_pos = tab_end.saturating_add(Self::TAB_DIVIDER_WIDTH);
        }

        None
    }

    /// Calculate the display width of a tab label
    fn tab_label_width(&self, tab: HelpTab) -> u16 {
        let name = tab.name();
        // Format: "N:Name" or "[N:Name]" for active tab
        let base_width = 2 + name.len() as u16; // "N:" + name
        if tab == self.active_tab {
            base_width + 2 // Add brackets []
        } else {
            base_width
        }
    }

    pub fn reset(&mut self) {
        self.visible = false;
        self.active_tab = HelpTab::Global;
        self.hovered_tab = None;
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
