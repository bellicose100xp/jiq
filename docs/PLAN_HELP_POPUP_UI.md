# Help Popup UI Improvements Plan

## Decision Summary

**Chosen Approach:** Tabbed Navigation with Context-Aware Auto-Selection

**Core UX Principle:** When users launch help from a specific context (e.g., search mode, snippet manager, results pane), they're looking for help with *that* context. The help popup should automatically open to the relevant tab while still allowing navigation to other tabs.

---

## Current State

The help popup displays **77 keyboard shortcuts** in a single scrollable list across 9 sections:
- GLOBAL (8), INPUT: INSERT MODE (4), INPUT: NORMAL MODE (16), AUTOCOMPLETE (3)
- RESULTS PANE (10), SEARCH IN RESULTS (7), HISTORY POPUP (4), ERROR OVERLAY (1), AI ASSISTANT (4)

**Problems:**
- Long vertical scroll to find relevant shortcuts
- No way to jump to sections
- Shows everything regardless of what user is doing
- Cognitive overload from too many options at once

---

## UI Mockups

### Global Tab (Default/Fallback)

```
┌──────────────────── Keyboard Shortcuts ─────────────────────┐
│                                                             │
│  [Global]  Input   Results   Search   Popups   AI           │
│  ───────────────────────────────────────────────────────    │
│                                                             │
│  F1 or ?        Toggle this help                            │
│  Ctrl+A         Toggle AI assistant                         │
│  Ctrl+S         Open snippets manager                       │
│  Ctrl+C         Quit without output                         │
│  Enter          Output filtered JSON and exit               │
│  Ctrl+Q         Output query string only and exit           │
│  Shift+Tab      Switch focus (Input ↔ Results)              │
│  q              Quit (in Normal mode or Results pane)       │
│                                                             │
│  ────────────────────────────────────────────────────────── │
│  h/l: switch tab | j/k: scroll | g/G: top/bottom | q: close │
└─────────────────────────────────────────────────────────────┘
```

### Input Tab (Auto-selected when Input field focused)

```
┌──────────────────── Keyboard Shortcuts ─────────────────────┐
│                                                             │
│   Global  [Input]  Results   Search   Popups   AI           │
│  ───────────────────────────────────────────────────────    │
│                                                             │
│  ── INSERT MODE ──                                          │
│  Esc             Switch to Normal mode                      │
│  ↑ or Ctrl+R     Open history popup                         │
│  Ctrl+P/N        Cycle history (prev/next)                  │
│  Ctrl+D/U        Scroll results half page                   │
│                                                             │
│  ── NORMAL MODE ──                                          │
│  i/a/I/A         Enter Insert mode                          │
│  h/l             Move cursor left/right                     │
│  0/^/$           Jump to start/first char/end               │
│  w/b/e           Word motions                               │
│  f/F/t/T         Find char forward/backward                 │
│  ;/,             Repeat find motion                         │
│  x/X             Delete char under/before cursor            │
│  dd/D            Delete line/to end                         │
│  ────────────────────────────────────────────────────────── │
│  h/l: switch tab | j/k: scroll | g/G: top/bottom | q: close │
└─────────────────────────────────────────────────────────────┘
```

### Results Tab (Auto-selected when Results pane focused)

```
┌──────────────────── Keyboard Shortcuts ─────────────────────┐
│                                                             │
│   Global   Input  [Results]  Search   Popups   AI           │
│  ───────────────────────────────────────────────────────    │
│                                                             │
│  j/k/↑/↓        Scroll line by line                         │
│  J/K            Scroll 10 lines                             │
│  h/l/←/→        Scroll column by column                     │
│  H/L            Scroll 10 columns                           │
│  0/^            Jump to left edge                           │
│  $              Jump to right edge                          │
│  g/Home         Jump to top                                 │
│  G/End          Jump to bottom                              │
│  Ctrl+D/U       Half page down/up                           │
│  PageDown/Up    Half page down/up                           │
│                                                             │
│  ────────────────────────────────────────────────────────── │
│  h/l: switch tab | j/k: scroll | g/G: top/bottom | q: close │
└─────────────────────────────────────────────────────────────┘
```

### Search Tab (Auto-selected when Search mode active)

```
┌──────────────────── Keyboard Shortcuts ─────────────────────┐
│                                                             │
│   Global   Input   Results  [Search]  Popups   AI           │
│  ───────────────────────────────────────────────────────    │
│                                                             │
│  Ctrl+F         Open search                                 │
│  /              Open search (from Results pane)             │
│  Enter          Confirm search pattern                      │
│  n              Jump to next match                          │
│  N              Jump to previous match                      │
│  /              Edit search (after confirming)              │
│  Esc            Close search                                │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│  ────────────────────────────────────────────────────────── │
│  h/l: switch tab | j/k: scroll | g/G: top/bottom | q: close │
└─────────────────────────────────────────────────────────────┘
```

### Popups Tab (Auto-selected when History/Autocomplete/Snippets/Error visible)

```
┌──────────────────── Keyboard Shortcuts ─────────────────────┐
│                                                             │
│   Global   Input   Results   Search  [Popups]  AI           │
│  ───────────────────────────────────────────────────────    │
│                                                             │
│  ── HISTORY POPUP ──                                        │
│  ↑/↓            Navigate history                            │
│  Type           Filter history                              │
│  Enter/Tab      Select entry                                │
│  Esc            Close popup                                 │
│                                                             │
│  ── AUTOCOMPLETE ──                                         │
│  ↑/↓            Navigate suggestions                        │
│  Tab            Accept suggestion                           │
│  Esc            Dismiss                                     │
│                                                             │
│  ── ERROR OVERLAY ──                                        │
│  Ctrl+E         Toggle error overlay                        │
│                                                             │
│  ────────────────────────────────────────────────────────── │
│  h/l: switch tab | j/k: scroll | g/G: top/bottom | q: close │
└─────────────────────────────────────────────────────────────┘
```

### AI Tab (Auto-selected when AI assistant visible)

```
┌──────────────────── Keyboard Shortcuts ─────────────────────┐
│                                                             │
│   Global   Input   Results   Search   Popups  [AI]          │
│  ───────────────────────────────────────────────────────    │
│                                                             │
│  Ctrl+A         Toggle AI assistant                         │
│  Alt+1-5        Apply AI suggestion directly                │
│  Alt+↑↓/j/k     Navigate AI suggestions                     │
│  Enter          Apply selected suggestion                   │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│                                                             │
│  ────────────────────────────────────────────────────────── │
│  h/l: switch tab | j/k: scroll | g/G: top/bottom | q: close │
└─────────────────────────────────────────────────────────────┘
```

### Tab Styling Detail

```
Inactive tabs:  DarkGray text
Active tab:     Cyan text, Bold, with [brackets] or reversed background

Example styling:

   Global   Input  [Results]  Search   Popups   AI
   ~~~~~~   ~~~~~   ~~~~~~~   ~~~~~~   ~~~~~~   ~~
   gray     gray    CYAN/BG   gray     gray     gray
```

---

## Tab Structure

| Tab | Auto-Selected When | Contents |
|-----|-------------------|----------|
| **Global** | Default fallback | F1, Ctrl+A/S/C/Q, Enter, Shift+Tab, q |
| **Input** | Input field focused (Insert or Normal mode) | Insert mode + Normal mode shortcuts |
| **Results** | Results pane focused | Navigation, scrolling shortcuts |
| **Search** | Search mode active | Ctrl+F, /, Enter, n/N, Esc |
| **Popups** | History, Autocomplete, Snippets, or Error overlay visible | History nav, Autocomplete, Error toggle |
| **AI** | AI assistant visible | Ctrl+A, Alt+1-5, Alt+↑↓, Enter |

### Context-to-Tab Mapping

```rust
fn get_default_tab(app: &App) -> HelpTab {
    // Priority order matters - more specific contexts first

    // AI assistant visible
    if app.ai_visible() {
        return HelpTab::AI;
    }

    // Search mode active
    if app.search.visible {
        return HelpTab::Search;
    }

    // Popups (History, Autocomplete, Snippets, Error)
    if app.history_popup.visible
        || app.autocomplete.visible()
        || app.snippets.visible
        || app.error_overlay_visible {
        return HelpTab::Popups;
    }

    // Results pane focused
    if app.focus == Focus::Results {
        return HelpTab::Results;
    }

    // Input field focused (covers Insert and Normal modes)
    if app.focus == Focus::Input {
        return HelpTab::Input;
    }

    // Fallback
    HelpTab::Global
}
```

---

## State Management

### New Types

```rust
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
```

### Updated HelpPopupState

```rust
pub struct HelpPopupState {
    pub visible: bool,
    pub active_tab: HelpTab,
    pub scroll_per_tab: [ScrollState; HelpTab::COUNT],  // Independent scroll per tab
}

impl HelpPopupState {
    pub fn new() -> Self {
        Self {
            visible: false,
            active_tab: HelpTab::Global,
            scroll_per_tab: Default::default(),
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
```

---

## Content Organization

### Refactored help_content.rs

```rust
pub struct HelpCategory {
    pub tab: HelpTab,
    pub sections: &'static [HelpSection],
}

pub struct HelpSection {
    pub title: Option<&'static str>,  // None for no header
    pub entries: &'static [(&'static str, &'static str)],
}

pub const HELP_CATEGORIES: &[HelpCategory] = &[
    // Global tab
    HelpCategory {
        tab: HelpTab::Global,
        sections: &[
            HelpSection {
                title: None,
                entries: &[
                    ("F1 or ?", "Toggle this help"),
                    ("Ctrl+A", "Toggle AI assistant"),
                    ("Ctrl+S", "Open snippets manager"),
                    ("Ctrl+C", "Quit without output"),
                    ("Enter", "Output filtered JSON and exit"),
                    ("Ctrl+Q", "Output query string only and exit"),
                    ("Shift+Tab", "Switch focus (Input ↔ Results)"),
                    ("q", "Quit (in Normal mode or Results pane)"),
                ],
            },
        ],
    },

    // Input tab (combines Insert + Normal modes)
    HelpCategory {
        tab: HelpTab::Input,
        sections: &[
            HelpSection {
                title: Some("INSERT MODE"),
                entries: &[
                    ("Esc", "Switch to Normal mode"),
                    ("↑ or Ctrl+R", "Open history popup"),
                    ("Ctrl+P/N", "Cycle history (prev/next)"),
                    ("Ctrl+D/U", "Scroll results half page"),
                ],
            },
            HelpSection {
                title: Some("NORMAL MODE"),
                entries: &[
                    ("i/a/I/A", "Enter Insert mode"),
                    ("h/l", "Move cursor left/right"),
                    ("0/^/$", "Jump to start/first char/end"),
                    ("w/b/e", "Word motions"),
                    ("f/F/t/T", "Find char forward/backward"),
                    (";/,", "Repeat find motion"),
                    ("x/X", "Delete char under/before cursor"),
                    ("dd/D", "Delete line/to end"),
                    ("dw/cw/ciw", "Delete/change word/inner word"),
                    ("df/dt/cf/ct", "Delete/change to char"),
                    ("di\"/ci\"/etc", "Delete/change inside quotes/parens"),
                    ("u", "Undo"),
                    ("Ctrl+R", "Redo"),
                    ("Ctrl+D/U", "Scroll results half page"),
                ],
            },
        ],
    },

    // Results tab
    HelpCategory {
        tab: HelpTab::Results,
        sections: &[
            HelpSection {
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
            },
        ],
    },

    // Search tab
    HelpCategory {
        tab: HelpTab::Search,
        sections: &[
            HelpSection {
                title: None,
                entries: &[
                    ("Ctrl+F", "Open search"),
                    ("/", "Open search (from Results pane)"),
                    ("Enter", "Confirm search pattern"),
                    ("n", "Jump to next match"),
                    ("N", "Jump to previous match"),
                    ("/", "Edit search (after confirming)"),
                    ("Esc", "Close search"),
                ],
            },
        ],
    },

    // Popups tab (History, Autocomplete, Snippets, Error)
    HelpCategory {
        tab: HelpTab::Popups,
        sections: &[
            HelpSection {
                title: Some("HISTORY POPUP"),
                entries: &[
                    ("↑/↓", "Navigate history"),
                    ("Type", "Filter history"),
                    ("Enter/Tab", "Select entry"),
                    ("Esc", "Close popup"),
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
                entries: &[
                    ("Ctrl+E", "Toggle error overlay"),
                ],
            },
        ],
    },

    // AI tab
    HelpCategory {
        tab: HelpTab::AI,
        sections: &[
            HelpSection {
                title: None,
                entries: &[
                    ("Ctrl+A", "Toggle AI assistant"),
                    ("Alt+1-5", "Apply AI suggestion directly"),
                    ("Alt+↑↓/j/k", "Navigate AI suggestions"),
                    ("Enter", "Apply selected suggestion"),
                ],
            },
        ],
    },
];

/// Get entries for a specific tab
pub fn get_tab_content(tab: HelpTab) -> &'static HelpCategory {
    HELP_CATEGORIES
        .iter()
        .find(|c| c.tab == tab)
        .expect("All tabs should have content")
}
```

---

## Key Bindings

### Help Popup Navigation

| Key | Action |
|-----|--------|
| `h` / `←` | Previous tab |
| `l` / `→` | Next tab |
| `1`-`6` | Jump to tab by number |
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `J` | Scroll down 10 lines |
| `K` | Scroll up 10 lines |
| `Ctrl+D` | Half page down |
| `Ctrl+U` | Half page up |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |
| `q` / `?` / `F1` / `Esc` | Close help |

### Event Handler Update

```rust
fn handle_help_keys(app: &mut App, key: KeyEvent) -> bool {
    if !app.help.visible {
        return false;
    }

    match key.code {
        // Close help
        KeyCode::F(1) | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            app.help.reset();
            true
        }

        // Tab navigation
        KeyCode::Char('h') | KeyCode::Left => {
            app.help.active_tab = app.help.active_tab.prev();
            true
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.help.active_tab = app.help.active_tab.next();
            true
        }
        KeyCode::Char(c) if ('1'..='6').contains(&c) => {
            let index = (c as usize) - ('1' as usize);
            app.help.active_tab = HelpTab::from_index(index);
            true
        }

        // Scrolling (using current_scroll_mut() for per-tab scroll)
        KeyCode::Char('j') | KeyCode::Down => {
            app.help.current_scroll_mut().down(1);
            true
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.help.current_scroll_mut().up(1);
            true
        }
        KeyCode::Char('J') => {
            app.help.current_scroll_mut().down(10);
            true
        }
        KeyCode::Char('K') => {
            app.help.current_scroll_mut().up(10);
            true
        }
        KeyCode::Char('g') | KeyCode::Home => {
            app.help.current_scroll_mut().to_top();
            true
        }
        KeyCode::Char('G') | KeyCode::End => {
            app.help.current_scroll_mut().to_bottom();
            true
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.help.current_scroll_mut().down(10);
            true
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.help.current_scroll_mut().up(10);
            true
        }
        KeyCode::PageDown => {
            app.help.current_scroll_mut().down(10);
            true
        }
        KeyCode::PageUp => {
            app.help.current_scroll_mut().up(10);
            true
        }

        _ => true  // Consume all keys when help is visible
    }
}
```

---

## Rendering

### Tab Bar Rendering

```rust
use ratatui::widgets::Tabs;

fn render_tab_bar(active_tab: HelpTab) -> Tabs<'static> {
    let titles: Vec<Line> = HelpTab::all()
        .iter()
        .map(|tab| {
            if *tab == active_tab {
                Line::styled(
                    format!("[{}]", tab.name()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Line::styled(
                    format!(" {} ", tab.name()),
                    Style::default().fg(Color::DarkGray),
                )
            }
        })
        .collect();

    Tabs::new(titles)
        .divider(Span::raw("  "))
}
```

### Full Popup Render

```rust
pub fn render_popup(app: &mut App, frame: &mut Frame) {
    let frame_area = frame.area();

    // Popup dimensions - wider to accommodate tab bar
    let popup_width = 65.min(frame_area.width);
    let popup_height = 20.min(frame_area.height);

    if frame_area.width < 20 || frame_area.height < 10 {
        return;
    }

    let popup_area = popup::centered_popup(frame_area, popup_width, popup_height);
    popup::clear_area(frame, popup_area);

    // Outer block with title and border
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" Keyboard Shortcuts ")
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(popup_area);
    frame.render_widget(outer_block, popup_area);

    // Split inner area: tab bar, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Tab bar + separator
            Constraint::Min(1),     // Content
            Constraint::Length(1),  // Footer
        ])
        .split(inner_area);

    // Render tab bar
    let tabs = render_tab_bar(app.help.active_tab);
    frame.render_widget(tabs, chunks[0]);

    // Render content for active tab
    let content = get_tab_content(app.help.active_tab);
    let lines = render_help_sections(content.sections);

    // Update scroll bounds for current tab
    let content_height = lines.len() as u32;
    let visible_height = chunks[1].height as u32;
    app.help
        .current_scroll_mut()
        .update_bounds(content_height, visible_height);

    let paragraph = Paragraph::new(Text::from(lines))
        .scroll((app.help.current_scroll().offset, 0));
    frame.render_widget(paragraph, chunks[1]);

    // Render footer
    let footer = Line::from(vec![
        Span::styled("h/l", Style::default().fg(Color::Yellow)),
        Span::raw(": tab | "),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(": scroll | "),
        Span::styled("g/G", Style::default().fg(Color::Yellow)),
        Span::raw(": top/bottom | "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(": close"),
    ]);
    frame.render_widget(
        Paragraph::new(footer)
            .style(Style::default().fg(Color::DarkGray))
            .centered(),
        chunks[2],
    );
}

fn render_help_sections(sections: &[HelpSection]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for section in sections {
        // Add section header if present
        if let Some(title) = section.title {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("── {} ──", title),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        // Add entries
        for (key, desc) in section.entries {
            let key_span = Span::styled(
                format!("  {:<15}", key),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            let desc_span = Span::styled(*desc, Style::default().fg(Color::White));
            lines.push(Line::from(vec![key_span, desc_span]));
        }
    }

    lines
}
```

---

## Opening Help with Context

### Toggle Help Handler

```rust
// In handle_truly_global_keys or wherever F1/? is handled
KeyCode::F(1) => {
    if app.help.visible {
        app.help.reset();
    } else {
        // Auto-select tab based on current context
        app.help.active_tab = get_default_tab(app);
        app.help.visible = true;
    }
    true
}
```

This ensures that when the user presses F1:
1. If in Results pane → opens to **Results** tab
2. If in Search mode → opens to **Search** tab
3. If Snippets manager open → opens to **Popups** tab
4. If AI assistant visible → opens to **AI** tab
5. If Input field focused → opens to **Input** tab
6. Otherwise → opens to **Global** tab

---

## Implementation Order

### Phase 1: Core Tab Infrastructure
1. Add `HelpTab` enum to `help_state.rs`
2. Update `HelpPopupState` with `active_tab` and per-tab scroll array
3. Refactor `help_content.rs` into `HelpCategory` / `HelpSection` structure
4. Update key handler with `h/l` and `1-6` tab navigation

### Phase 2: Context-Aware Auto-Selection
1. Implement `get_default_tab()` function
2. Update help toggle (F1/?) to set initial tab based on context
3. Test all context → tab mappings

### Phase 3: Rendering
1. Add tab bar rendering with `ratatui::widgets::Tabs`
2. Update layout to accommodate tab bar (3-part vertical split)
3. Style active tab with brackets and cyan/bold
4. Update footer with tab navigation hint

### Phase 4: Polish & Testing
1. Ensure popup width accommodates all tab names (need ~65 chars)
2. Verify per-tab scroll state is independent
3. Update snapshot tests for new layout
4. Manual testing: open help from each context, verify correct tab selected

---

## Research References

- [Lazygit](https://github.com/jesseduffield/lazygit) - Context-sensitive keybinding help
- [which-key.nvim](https://github.com/folke/which-key.nvim) - Progressive disclosure popup
- [Helix Editor](https://helix-editor.com/) - Hierarchical command menus
- [btop](https://github.com/aristocratos/btop) - Menu-based help system
- [Ratatui Tabs Widget](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Tabs.html) - Tab bar implementation
