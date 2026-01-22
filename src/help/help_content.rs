use super::help_state::HelpTab;

pub struct HelpSection {
    pub title: Option<&'static str>,
    pub entries: &'static [(&'static str, &'static str)],
}

pub struct HelpCategory {
    pub tab: HelpTab,
    pub sections: &'static [HelpSection],
}

pub const HELP_CATEGORIES: &[HelpCategory] = &[
    // Global tab
    HelpCategory {
        tab: HelpTab::Global,
        sections: &[HelpSection {
            title: None,
            entries: &[
                ("F1 or ?", "Toggle this help"),
                ("Ctrl+A", "Toggle AI assistant"),
                ("Ctrl+S", "Open snippets manager"),
                ("Ctrl+C", "Quit without output"),
                ("Enter", "Output filtered JSON and exit"),
                ("Ctrl+Q", "Output query string only and exit"),
                ("Shift+Tab", "Switch focus (Input / Results)"),
                ("q", "Quit (in Normal mode or Results pane)"),
            ],
        }],
    },
    // Input tab (combines Insert + Normal modes)
    HelpCategory {
        tab: HelpTab::Input,
        sections: &[
            HelpSection {
                title: Some("INSERT MODE"),
                entries: &[
                    ("Esc", "Switch to Normal mode"),
                    ("↑/Ctrl+R", "Open history popup"),
                    ("Ctrl+P/N", "Previous/Next query in history"),
                    ("Ctrl+D/U", "Scroll results half page down/up"),
                ],
            },
            HelpSection {
                title: Some("NORMAL MODE"),
                entries: &[
                    ("i/a/I/A", "Enter Insert mode"),
                    ("h/l", "Move cursor left/right"),
                    ("0/^/$", "Jump to start/end of line"),
                    ("w/b/e", "Word navigation"),
                    ("f/F/t/T", "Find/till char forward/backward"),
                    (";/,", "Repeat/reverse last char search"),
                    ("x/X", "Delete character"),
                    ("dd/D", "Delete line/to end"),
                    ("dw/cw/ciw", "Delete/change word (operators)"),
                    ("df/dt/cf/ct", "Delete/change to/till char"),
                    ("di\"/ci\"/etc", "Delete/change inside quotes/parens"),
                    ("u", "Undo"),
                    ("Ctrl+R", "Redo"),
                    ("Ctrl+D/U", "Scroll results half page down/up"),
                ],
            },
        ],
    },
    // Results tab
    HelpCategory {
        tab: HelpTab::Results,
        sections: &[HelpSection {
            title: None,
            entries: &[
                ("j/k/↑/↓", "Scroll line by line"),
                ("J/K", "Scroll 10 lines"),
                ("h/l/←/→", "Scroll column by column"),
                ("H/L", "Scroll 10 columns"),
                ("0/^", "Jump to left edge"),
                ("$", "Jump to right edge"),
                ("g/Home", "Jump to top"),
                ("G/End", "Jump to bottom"),
                ("Ctrl+D/U", "Half page down/up"),
                ("PageDown/Up", "Half page down/up"),
            ],
        }],
    },
    // Search tab
    HelpCategory {
        tab: HelpTab::Search,
        sections: &[HelpSection {
            title: None,
            entries: &[
                ("Ctrl+F", "Open search (from any pane)"),
                ("/", "Open search (from Results pane)"),
                ("Enter", "Confirm search"),
                ("n/Enter", "Next match"),
                ("N/Shift+Enter", "Previous match"),
                ("Ctrl+F or /", "Re-enter edit mode"),
                ("Esc", "Close search"),
            ],
        }],
    },
    // Popups tab (History, Autocomplete, Snippets, Error)
    HelpCategory {
        tab: HelpTab::Popups,
        sections: &[
            HelpSection {
                title: Some("HISTORY POPUP"),
                entries: &[
                    ("↑/↓", "Navigate entries"),
                    ("Type", "Fuzzy search filter"),
                    ("Enter/Tab", "Select entry and close"),
                    ("Esc", "Close without selecting"),
                ],
            },
            HelpSection {
                title: Some("AUTOCOMPLETE"),
                entries: &[
                    ("↑/↓", "Navigate suggestions"),
                    ("Tab", "Accept suggestion"),
                    ("Esc", "Dismiss"),
                ],
            },
            HelpSection {
                title: Some("ERROR OVERLAY"),
                entries: &[("Ctrl+E", "Toggle error details")],
            },
        ],
    },
    // AI tab
    HelpCategory {
        tab: HelpTab::AI,
        sections: &[HelpSection {
            title: None,
            entries: &[
                ("Ctrl+A", "Toggle AI assistant"),
                ("Alt+1-5", "Apply AI suggestion (direct)"),
                ("Alt+↑↓/j/k", "Navigate suggestions"),
                ("Enter", "Apply selected suggestion"),
            ],
        }],
    },
];

pub const HELP_FOOTER: &str = "h/l: tab | j/k: scroll | g/G: top/bottom | q: close";

pub fn get_tab_content(tab: HelpTab) -> &'static HelpCategory {
    HELP_CATEGORIES
        .iter()
        .find(|c| c.tab == tab)
        .expect("All tabs should have content")
}

#[cfg(test)]
#[path = "help_content_tests.rs"]
mod help_content_tests;
