# Help Popup UI Improvements Plan

## Current State Analysis

The help popup currently displays **77 keyboard shortcuts** in a single scrollable list organized into 9 sections:
- GLOBAL (8 shortcuts)
- INPUT: INSERT MODE (4 shortcuts)
- INPUT: NORMAL MODE (16 shortcuts)
- AUTOCOMPLETE (3 shortcuts)
- RESULTS PANE (10 shortcuts)
- SEARCH IN RESULTS (7 shortcuts)
- HISTORY POPUP (4 shortcuts)
- ERROR OVERLAY (1 shortcut)
- AI ASSISTANT (4 shortcuts)

**Current Issues:**
- Long vertical scroll required to find shortcuts
- No way to jump directly to a section
- No search functionality
- All sections shown regardless of current context
- No visual hierarchy beyond section headers

---

## Research: Highly Praised TUI Help UI Patterns

### 1. Lazygit - Context-Sensitive Panel Help
[GitHub - jesseduffield/lazygit](https://github.com/jesseduffield/lazygit)

**Key Features:**
- Press `?` to show keybindings **relevant to the currently focused panel**
- Footer shows most common actions at all times
- Keybindings organized by panel context (Files, Branches, Commits, etc.)
- Intuitive: users only see shortcuts they can actually use

**Praise:** "Besides navigating through sections and tabs, the only keyboard shortcut you need to remember is `?`, which shows you contextually available operations based on an active panel/tab."

### 2. Which-Key (Neovim/Vim) - Progressive Disclosure
[GitHub - folke/which-key.nvim](https://github.com/folke/which-key.nvim)

**Key Features:**
- Shows available keybindings **as you type a prefix** (e.g., press `g` and see all `g` commands)
- Categorized by action type (motions, text-objects, operators)
- **Hydra mode**: keeps popup open until you press Escape
- Searchable with built-in fuzzy finder
- Works across all modes (normal, insert, visual, etc.)

**Praise:** "WhichKey helps you remember your Neovim keymaps, by showing available keybindings in a popup as you type."

### 3. Helix Editor - Hierarchical Menus
[Julia Evans - Notes on switching to Helix from vim](https://jvns.ca/blog/2025/10/10/notes-on-switching-to-helix-from-vim/)

**Key Features:**
- Pressing prefix keys (like `g`, `<space>`) opens a **mini help popup**
- Nested menus for related commands (e.g., `<space>` -> `w` for Window Management)
- Shows only the next level of available commands
- Extremely intuitive for discoverability

**Praise:** "When you press `g`, you get a little help popup telling you places you can go. This is appreciated because you don't often use features like 'go to definition' and forget the shortcuts."

### 4. btop - Menu-Based Help
[Linux Blog - btop the htop alternative](https://linuxblog.io/btop-the-htop-alternative/)

**Key Features:**
- Press `ESC` to open main menu, then select HELP
- Press `F1` or `h` for direct help access
- **Categorized by function** with clear visual grouping
- Mouse-clickable buttons for key actions

**Praise:** "One of the most striking features of btop is its ease of use. The UI is controlled using a shortlist of keyboard shortcuts."

---

## Proposed Ideas

### Option A: Tabbed Category Navigation (Recommended)

```
┌─────────────────── Keyboard Shortcuts ───────────────────┐
│                                                          │
│  [Global] [Input] [Results] [Search] [Popups] [AI]       │
│  ────────────────────────────────────────────────────    │
│                                                          │
│  ── GLOBAL ──                                            │
│                                                          │
│  F1 or ?        Toggle this help                         │
│  Ctrl+A         Toggle AI assistant                      │
│  Ctrl+S         Open snippets manager                    │
│  Ctrl+C         Quit without output                      │
│  Enter          Output filtered JSON and exit            │
│  Ctrl+Q         Output query string only and exit        │
│  Shift+Tab      Switch focus (Input / Results)           │
│  q              Quit (in Normal mode or Results pane)    │
│                                                          │
│  ─────────────────────────────────────────────────────── │
│  ←/→: switch tab | j/k: scroll | /: search | q: close    │
└──────────────────────────────────────────────────────────┘
```

**Features:**
- Horizontal tabs for each category (6 tabs)
- `h/l` or `←/→` to switch between tabs
- Each tab shows only its relevant shortcuts
- Optional: highlight current tab based on app state

**Pros:**
- Familiar pattern (browser tabs, lazygit panels)
- Reduces cognitive load by showing fewer items at once
- Easy to find what you need

**Cons:**
- Requires multiple keypresses to see all shortcuts
- Implementation complexity for tab state management

---

### Option B: Searchable Help with Fuzzy Filter

```
┌─────────────────── Keyboard Shortcuts ───────────────────┐
│                                                          │
│  Search: scro█                                           │
│  ────────────────────────────────────────────────────    │
│                                                          │
│  Found 8 matches:                                        │
│                                                          │
│  j/k/↑/↓        Scroll line by line         [Results]    │
│  J/K            Scroll 10 lines             [Results]    │
│  Ctrl+D/U       Scroll results half page    [Input]      │
│  Ctrl+D/U       Half page down/up           [Results]    │
│  PageDown/Up    Half page down/up           [Results]    │
│  g/Home         Jump to top                 [Results]    │
│  G/End          Jump to bottom              [Results]    │
│  j/k            scroll                      [Help]       │
│                                                          │
│  ─────────────────────────────────────────────────────── │
│  Type to search | Esc: clear | q: close                  │
└──────────────────────────────────────────────────────────┘
```

**Features:**
- Search box at top (activated by `/` or typing immediately)
- Fuzzy matching on both key and description
- Shows category tag on the right `[Results]`
- Highlights matching text
- Falls back to full list when search is empty

**Pros:**
- Fastest way to find a specific shortcut
- Power-user friendly
- Reuses existing fuzzy search logic from history popup

**Cons:**
- Users need to know what they're looking for
- Less useful for browsing/discovery

---

### Option C: Context-Aware Help (Lazygit-Style)

```
┌─────────────── Help: Results Pane ───────────────┐
│                                                  │
│  j/k/↑/↓        Scroll line by line              │
│  J/K            Scroll 10 lines                  │
│  h/l/←/→        Scroll column by column          │
│  H/L            Scroll 10 columns                │
│  0/^            Jump to left edge                │
│  $              Jump to right edge               │
│  g/Home         Jump to top                      │
│  G/End          Jump to bottom                   │
│  Ctrl+D/U       Half page down/up                │
│  PageDown/Up    Half page down/up                │
│                                                  │
│  ── ALSO AVAILABLE ──                            │
│  Ctrl+F         Open search                      │
│  /              Open search                      │
│  Shift+Tab      Switch to Input                  │
│                                                  │
│  ──────────────────────────────────────────────  │
│  Tab: show all | j/k: scroll | q: close          │
└──────────────────────────────────────────────────┘
```

**Features:**
- Shows shortcuts **relevant to current focus/mode** by default
- "ALSO AVAILABLE" section for cross-cutting shortcuts
- `Tab` to toggle between context-specific and full help
- Title dynamically shows current context

**Context mappings:**
| Focus/Mode | Primary Shortcuts |
|------------|-------------------|
| Input (Insert) | Insert mode shortcuts + Global |
| Input (Normal) | Normal mode shortcuts + Global |
| Results Pane | Results navigation + Search + Global |
| Search Mode | Search shortcuts |
| AI Assistant | AI shortcuts |
| Snippets | Snippet navigation |
| History Popup | History navigation |

**Pros:**
- Shows exactly what you need when you need it
- Reduces information overload significantly
- Matches mental model (what can I do right now?)

**Cons:**
- Users might not realize other shortcuts exist
- Need to maintain context -> shortcuts mapping

---

### Option D: Which-Key Style Progressive Disclosure

```
Normal mode, after pressing 'd':
┌─────────────────────────────────┐
│  d → delete...                  │
│                                 │
│  d    delete line               │
│  w    delete word               │
│  iw   delete inner word         │
│  i"   delete inside quotes      │
│  i(   delete inside parens      │
│  f    delete to char            │
│  t    delete till char          │
│                                 │
│  Esc: cancel                    │
└─────────────────────────────────┘
```

**Features:**
- Small popup appears when you start a command sequence
- Shows available completions for the current prefix
- Disappears after command is executed or cancelled
- Only shown for multi-key sequences (d, c, f, t, etc.)

**Pros:**
- Just-in-time help exactly when needed
- Very intuitive for vim motions
- Non-intrusive

**Cons:**
- Doesn't help with single-key shortcuts
- Requires significant implementation effort
- Only useful for Normal mode operations

---

### Option E: Hybrid Approach (Recommended Combination)

Combine the best of multiple patterns:

1. **Default View: Tabbed Categories** (Option A)
   - Quick navigation between logical groupings
   - Reduces scroll fatigue

2. **Search Overlay** (Option B)
   - Press `/` to activate search within help
   - Fuzzy filter across all categories

3. **Context Indicator** (Option C, partial)
   - Highlight the tab relevant to current mode
   - Show current mode in title: `Keyboard Shortcuts (Results Pane)`

**Implementation Priority:**
1. Tabbed navigation (highest impact)
2. Search functionality
3. Context-aware tab highlighting

---

## Visual Enhancements

### Color Coding by Category
```
Global shortcuts:     Yellow keys
Navigation:           Cyan keys
Editing:              Green keys
Mode switching:       Magenta keys
```

### Key Grouping with Icons (if supported)
```
  ── NAVIGATION ──
  ↕  j/k          Scroll up/down
  ⇅  J/K          Scroll 10 lines
  ⇤  0/^          Jump to start
  ⇥  $            Jump to end
```

### Highlight Frequently Used
Mark the most common shortcuts with a subtle indicator:
```
  F1 or ?        Toggle this help
★ Ctrl+S         Open snippets manager
★ Enter          Output filtered JSON and exit
  Ctrl+Q         Output query string only
```

---

## Proposed Tab Structure

| Tab | Contents | Shortcut Count |
|-----|----------|----------------|
| **Global** | F1, Ctrl+A/S/C/Q, Enter, Shift+Tab, q | 8 |
| **Input** | Insert mode + Normal mode (combined) | 20 |
| **Results** | Results pane navigation | 10 |
| **Search** | Search in results | 7 |
| **Popups** | History + Autocomplete + Error | 8 |
| **AI** | AI assistant shortcuts | 4 |

Total: 6 tabs, ~57 unique shortcuts (some overlap)

---

## Implementation Considerations

### State Management
```rust
pub struct HelpPopupState {
    pub visible: bool,
    pub scroll: ScrollState,
    pub active_tab: HelpTab,      // NEW
    pub search_query: String,      // NEW (if search implemented)
    pub search_active: bool,       // NEW
}

pub enum HelpTab {
    Global,
    Input,
    Results,
    Search,
    Popups,
    AI,
}
```

### Key Bindings for Help Popup
| Key | Action |
|-----|--------|
| `h/←` | Previous tab |
| `l/→` | Next tab |
| `1-6` | Jump to tab by number |
| `/` | Activate search |
| `Esc` | Clear search / close popup |
| `j/k` | Scroll within tab |
| `q/?/F1` | Close popup |

### Content Organization
Refactor `help_content.rs`:
```rust
pub struct HelpCategory {
    pub name: &'static str,
    pub entries: &'static [(&'static str, &'static str)],
}

pub const HELP_CATEGORIES: &[HelpCategory] = &[
    HelpCategory {
        name: "Global",
        entries: &[
            ("F1 or ?", "Toggle this help"),
            // ...
        ],
    },
    // ...
];
```

---

## Recommended Implementation Order

### Phase 1: Tabbed Navigation
1. Add `HelpTab` enum and state
2. Refactor content into categories
3. Render tab bar with ratatui `Tabs` widget
4. Handle `h/l` navigation between tabs
5. Update scroll state per-tab

### Phase 2: Visual Polish
1. Highlight active tab
2. Show context indicator in title
3. Add subtle dividers between sections
4. Consistent key/description alignment

### Phase 3: Search (Optional)
1. Add search input field
2. Implement fuzzy matching
3. Highlight search results
4. Category tags in search results

### Phase 4: Context Awareness (Optional)
1. Map app state to default tab
2. Auto-select relevant tab on open
3. Visual indicator for current context

---

## References

- [Lazygit](https://github.com/jesseduffield/lazygit) - Context-sensitive keybinding help
- [which-key.nvim](https://github.com/folke/which-key.nvim) - Progressive disclosure popup
- [Helix Editor](https://helix-editor.com/) - Hierarchical command menus
- [btop](https://github.com/aristocratos/btop) - Menu-based help system
- [Ratatui Tabs Widget](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Tabs.html) - Tab bar implementation
- [Ratatui Popup Example](https://ratatui.rs/examples/apps/popup/) - Overlay widget pattern

---

## Decision Points for User

1. **Primary Pattern:** Tabs (A), Search (B), Context-aware (C), Which-key (D), or Hybrid (E)?
2. **Tab Count:** 6 tabs as proposed, or consolidate to fewer (e.g., combine Input modes)?
3. **Search Feature:** Include searchable help or defer to future?
4. **Context Awareness:** Auto-select tab based on current mode?
5. **Visual Enhancements:** Color coding, icons, or keep minimal?
