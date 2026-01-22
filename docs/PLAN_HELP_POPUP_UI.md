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

## Tab Organization

**7 Tabs with numbered labels for quick access:**

```
[1:Global] [2:Input] [3:Result] [4:History] [5:AI] [6:Search] [7:Snippet]
```

### Tab Navigation
- **Number keys 1-7**: Jump directly to tab
- **Tab key**: Cycle to next tab
- **Shift+Tab**: Cycle to previous tab
- **h/l or ←/→**: Previous/next tab

### Context-Aware Auto-Selection

When help is opened, it automatically selects the relevant tab based on current focus:

| Current Focus | Opens Tab |
|--------------|-----------|
| Input box | **[2:Input]** |
| Results box | **[3:Result]** |
| Search box | **[6:Search]** |
| Snippet manager | **[7:Snippet]** |
| Anywhere else | **[1:Global]** |

**Note:** History and AI tabs never auto-focus - users navigate to them manually.

---

## UI Mockups

### Popup Dimensions

The help popup should use more screen real estate:
- **Width:** 80% of terminal width (min 70, max 90 chars)
- **Height:** 80% of terminal height (min 20, max 30 lines)

### Global Tab (Default/Fallback)

```
┌─────────────────────────────────── Keyboard Shortcuts ────────────────────────────────────┐
│                                                                                           │
│  [1:Global]  2:Input   3:Result   4:History   5:AI   6:Search   7:Snippet                 │
│  ─────────────────────────────────────────────────────────────────────────────────────    │
│                                                                                           │
│  F1 or ?        Toggle this help                                                          │
│  Ctrl+A         Toggle AI assistant                                                       │
│  Ctrl+S         Open snippets manager                                                     │
│  Ctrl+C         Quit without output                                                       │
│  Enter          Output filtered JSON and exit                                             │
│  Ctrl+Q         Output query string only and exit                                         │
│  Shift+Tab      Switch focus (Input ↔ Results)                                            │
│  q              Quit (in Normal mode or Results pane)                                     │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│  ──────────────────────────────────────────────────────────────────────────────────────── │
│  1-7: jump to tab | Tab: next tab | h/l: switch tab | j/k: scroll | q: close              │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Input Tab (Auto-selected when Input field focused)

```
┌─────────────────────────────────── Keyboard Shortcuts ────────────────────────────────────┐
│                                                                                           │
│   1:Global  [2:Input]  3:Result   4:History   5:AI   6:Search   7:Snippet                 │
│  ─────────────────────────────────────────────────────────────────────────────────────    │
│                                                                                           │
│  ── INSERT MODE ──                                                                        │
│  Esc             Switch to Normal mode                                                    │
│  ↑ or Ctrl+R     Open history popup                                                       │
│  Ctrl+P/N        Cycle history (prev/next)                                                │
│  Ctrl+D/U        Scroll results half page                                                 │
│                                                                                           │
│  ── NORMAL MODE ──                                                                        │
│  i/a/I/A         Enter Insert mode                                                        │
│  h/l             Move cursor left/right                                                   │
│  0/^/$           Jump to start/first char/end                                             │
│  w/b/e           Word motions                                                             │
│  f/F/t/T         Find char forward/backward                                               │
│  ;/,             Repeat find motion                                                       │
│  x/X             Delete char under/before cursor                                          │
│  dd/D            Delete line/to end                                                       │
│  ──────────────────────────────────────────────────────────────────────────────────────── │
│  1-7: jump to tab | Tab: next tab | h/l: switch tab | j/k: scroll | q: close              │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Result Tab (Auto-selected when Results pane focused)

```
┌─────────────────────────────────── Keyboard Shortcuts ────────────────────────────────────┐
│                                                                                           │
│   1:Global   2:Input  [3:Result]  4:History   5:AI   6:Search   7:Snippet                 │
│  ─────────────────────────────────────────────────────────────────────────────────────    │
│                                                                                           │
│  j/k/↑/↓        Scroll line by line                                                       │
│  J/K            Scroll 10 lines                                                           │
│  h/l/←/→        Scroll column by column                                                   │
│  H/L            Scroll 10 columns                                                         │
│  0/^            Jump to left edge                                                         │
│  $              Jump to right edge                                                        │
│  g/Home         Jump to top                                                               │
│  G/End          Jump to bottom                                                            │
│  Ctrl+D/U       Half page down/up                                                         │
│  PageDown/Up    Half page down/up                                                         │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│  ──────────────────────────────────────────────────────────────────────────────────────── │
│  1-7: jump to tab | Tab: next tab | h/l: switch tab | j/k: scroll | q: close              │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### History Tab (Never auto-focused)

```
┌─────────────────────────────────── Keyboard Shortcuts ────────────────────────────────────┐
│                                                                                           │
│   1:Global   2:Input   3:Result  [4:History]  5:AI   6:Search   7:Snippet                 │
│  ─────────────────────────────────────────────────────────────────────────────────────    │
│                                                                                           │
│  ↑ or Ctrl+R    Open history popup                                                        │
│  ↑/↓            Navigate history entries                                                  │
│  Type           Filter history                                                            │
│  Enter/Tab      Select entry                                                              │
│  Esc            Close popup                                                               │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│  ──────────────────────────────────────────────────────────────────────────────────────── │
│  1-7: jump to tab | Tab: next tab | h/l: switch tab | j/k: scroll | q: close              │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### AI Tab (Never auto-focused)

```
┌─────────────────────────────────── Keyboard Shortcuts ────────────────────────────────────┐
│                                                                                           │
│   1:Global   2:Input   3:Result   4:History  [5:AI]  6:Search   7:Snippet                 │
│  ─────────────────────────────────────────────────────────────────────────────────────    │
│                                                                                           │
│  Ctrl+A         Toggle AI assistant                                                       │
│  Alt+1-5        Apply AI suggestion directly                                              │
│  Alt+↑↓/j/k     Navigate AI suggestions                                                   │
│  Enter          Apply selected suggestion                                                 │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│  ──────────────────────────────────────────────────────────────────────────────────────── │
│  1-7: jump to tab | Tab: next tab | h/l: switch tab | j/k: scroll | q: close              │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Search Tab (Auto-selected when Search mode active)

```
┌─────────────────────────────────── Keyboard Shortcuts ────────────────────────────────────┐
│                                                                                           │
│   1:Global   2:Input   3:Result   4:History   5:AI  [6:Search]  7:Snippet                 │
│  ─────────────────────────────────────────────────────────────────────────────────────    │
│                                                                                           │
│  Ctrl+F         Open search                                                               │
│  /              Open search (from Results pane)                                           │
│  Enter          Confirm search pattern                                                    │
│  n              Jump to next match                                                        │
│  N              Jump to previous match                                                    │
│  /              Edit search (after confirming)                                            │
│  Esc            Close search                                                              │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│  ──────────────────────────────────────────────────────────────────────────────────────── │
│  1-7: jump to tab | Tab: next tab | h/l: switch tab | j/k: scroll | q: close              │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Snippet Tab (Auto-selected when Snippet manager focused)

```
┌─────────────────────────────────── Keyboard Shortcuts ────────────────────────────────────┐
│                                                                                           │
│   1:Global   2:Input   3:Result   4:History   5:AI   6:Search  [7:Snippet]                │
│  ─────────────────────────────────────────────────────────────────────────────────────    │
│                                                                                           │
│  Ctrl+S         Open snippets manager                                                     │
│  ↑/↓ or j/k     Navigate snippets                                                         │
│  Enter          Apply selected snippet                                                    │
│  a              Add new snippet                                                           │
│  e              Edit selected snippet                                                     │
│  d              Delete selected snippet                                                   │
│  /              Filter snippets                                                           │
│  Esc            Close snippets manager                                                    │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│                                                                                           │
│  ──────────────────────────────────────────────────────────────────────────────────────── │
│  1-7: jump to tab | Tab: next tab | h/l: switch tab | j/k: scroll | q: close              │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Tab Styling Detail

```
Tab bar format:  [N:Name] for active,  N:Name  for inactive

Inactive tabs:  DarkGray text
Active tab:     Cyan text, Bold, with [brackets]

Example styling:

  1:Global  [2:Input]  3:Result   4:History   5:AI   6:Search   7:Snippet
  ~~~~~~~~   ~~~~~~~~   ~~~~~~~~   ~~~~~~~~~   ~~~~   ~~~~~~~~   ~~~~~~~~~
  gray       CYAN/BG    gray       gray        gray   gray       gray
```

---

## Tab Structure

| Tab | Number | Auto-Selected When | Contents |
|-----|--------|-------------------|----------|
| **Global** | 1 | Default fallback | F1, Ctrl+A/S/C/Q, Enter, Shift+Tab, q |
| **Input** | 2 | Input field focused | Insert mode + Normal mode shortcuts |
| **Result** | 3 | Results pane focused | Navigation, scrolling shortcuts |
| **History** | 4 | Never (manual only) | History popup navigation |
| **AI** | 5 | Never (manual only) | AI assistant shortcuts |
| **Search** | 6 | Search mode active | Ctrl+F, /, Enter, n/N, Esc |
| **Snippet** | 7 | Snippet manager visible | Snippet manager shortcuts |

### Context-to-Tab Mapping

```rust
fn get_default_tab(app: &App) -> HelpTab {
    // Priority order matters - more specific contexts first

    // Snippet manager visible
    if app.snippets.visible {
        return HelpTab::Snippet;
    }

    // Search mode active
    if app.search.visible {
        return HelpTab::Search;
    }

    // Results pane focused
    if app.focus == Focus::Results {
        return HelpTab::Result;
    }

    // Input field focused (covers Insert and Normal modes)
    if app.focus == Focus::Input {
        return HelpTab::Input;
    }

    // Fallback - Global tab
    // Note: History and AI tabs never auto-focus
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

    /// Returns the tab name for display (without number prefix)
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

    /// Returns the display label with number prefix (e.g., "1:Global")
    pub fn label(&self) -> String {
        format!("{}:{}", self.index() + 1, self.name())
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
    pub scroll_per_tab: [ScrollState; HelpTab::COUNT],  // Independent scroll per tab (7 tabs)
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
    // 1: Global tab
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
                    ("Ctrl+E", "Toggle error overlay"),
                ],
            },
        ],
    },

    // 2: Input tab (combines Insert + Normal modes)
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
            HelpSection {
                title: Some("AUTOCOMPLETE"),
                entries: &[
                    ("↑/↓", "Navigate suggestions"),
                    ("Tab", "Accept suggestion"),
                    ("Esc", "Dismiss"),
                ],
            },
        ],
    },

    // 3: Result tab
    HelpCategory {
        tab: HelpTab::Result,
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

    // 4: History tab
    HelpCategory {
        tab: HelpTab::History,
        sections: &[
            HelpSection {
                title: None,
                entries: &[
                    ("↑ or Ctrl+R", "Open history popup"),
                    ("↑/↓", "Navigate history entries"),
                    ("Type", "Filter history"),
                    ("Enter/Tab", "Select entry"),
                    ("Esc", "Close popup"),
                ],
            },
        ],
    },

    // 5: AI tab
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

    // 6: Search tab
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

    // 7: Snippet tab
    HelpCategory {
        tab: HelpTab::Snippet,
        sections: &[
            HelpSection {
                title: None,
                entries: &[
                    ("Ctrl+S", "Open snippets manager"),
                    ("↑/↓ or j/k", "Navigate snippets"),
                    ("Enter", "Apply selected snippet"),
                    ("a", "Add new snippet"),
                    ("e", "Edit selected snippet"),
                    ("d", "Delete selected snippet"),
                    ("/", "Filter snippets"),
                    ("Esc", "Close snippets manager"),
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
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `h` / `←` | Previous tab |
| `l` / `→` | Next tab |
| `1`-`7` | Jump to tab by number |
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

        // Tab navigation with Tab key
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.help.active_tab = app.help.active_tab.prev();
            true
        }
        KeyCode::Tab => {
            app.help.active_tab = app.help.active_tab.next();
            true
        }
        KeyCode::BackTab => {
            app.help.active_tab = app.help.active_tab.prev();
            true
        }

        // Tab navigation with h/l keys
        KeyCode::Char('h') | KeyCode::Left => {
            app.help.active_tab = app.help.active_tab.prev();
            true
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.help.active_tab = app.help.active_tab.next();
            true
        }

        // Jump to tab by number (1-7)
        KeyCode::Char(c) if ('1'..='7').contains(&c) => {
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
            let label = tab.label();  // e.g., "1:Global"
            if *tab == active_tab {
                Line::styled(
                    format!("[{}]", label),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Line::styled(
                    format!(" {} ", label),
                    Style::default().fg(Color::DarkGray),
                )
            }
        })
        .collect();

    Tabs::new(titles)
        .divider(Span::raw(" "))
}
```

### Full Popup Render

```rust
pub fn render_popup(app: &mut App, frame: &mut Frame) {
    let frame_area = frame.area();

    // Popup dimensions - use more screen real estate
    // Width: 80% of terminal width (min 70, max 90 chars)
    let popup_width = ((frame_area.width as f32 * 0.8) as u16)
        .clamp(70, 90)
        .min(frame_area.width);

    // Height: 80% of terminal height (min 20, max 30 lines)
    let popup_height = ((frame_area.height as f32 * 0.8) as u16)
        .clamp(20, 30)
        .min(frame_area.height);

    if frame_area.width < 40 || frame_area.height < 15 {
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

    // Render footer - emphasize number keys for tab switching
    let footer = Line::from(vec![
        Span::styled("1-7", Style::default().fg(Color::Yellow)),
        Span::raw(": jump to tab | "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(": next tab | "),
        Span::styled("h/l", Style::default().fg(Color::Yellow)),
        Span::raw(": switch tab | "),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(": scroll | "),
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
1. If Snippet manager visible → opens to **[7:Snippet]** tab
2. If Search mode active → opens to **[6:Search]** tab
3. If Results pane focused → opens to **[3:Result]** tab
4. If Input field focused → opens to **[2:Input]** tab
5. Otherwise → opens to **[1:Global]** tab

**Note:** History and AI tabs never auto-focus - users navigate to them manually with keys 4 or 5.

---

## Implementation Order

### Phase 1: Core Tab Infrastructure
1. Add `HelpTab` enum to `help_state.rs` with 7 variants
2. Update `HelpPopupState` with `active_tab` and per-tab scroll array (size 7)
3. Refactor `help_content.rs` into `HelpCategory` / `HelpSection` structure with 7 tabs
4. Update key handler with `h/l`, `Tab/Shift+Tab`, and `1-7` tab navigation

### Phase 2: Context-Aware Auto-Selection
1. Implement `get_default_tab()` function (only auto-selects Input, Result, Search, Snippet)
2. Update help toggle (F1/?) to set initial tab based on context
3. Test all context → tab mappings

### Phase 3: Rendering
1. Add tab bar rendering with numbered labels (e.g., "1:Global")
2. Update popup dimensions to use 80% of screen (min 70x20, max 100x30)
3. Style active tab with brackets and cyan/bold
4. Update footer to show "1-7: jump to tab | Tab: next tab | ..."

### Phase 4: Polish & Testing
1. Ensure popup width accommodates all 7 tab names (min 70, max 90 chars)
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
