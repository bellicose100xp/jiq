# Centralize Theme Configuration Plan

## Goal

Centralize all theme-related code (colors, styles, modifiers) into a single `theme.rs` module. This is **not** about creating a multi-theme system—it's about having one place to edit the default theme without touching business logic components.

---

## Current State

### Problem: Highly Scattered Theme Code (8/10 Severity)

**26 files** contain hardcoded colors and styles with significant duplication:

| Component | File | Current Pattern |
|-----------|------|-----------------|
| Results Pane | `results/results_render.rs` | Module constants (partial) |
| Input Field | `input/input_render.rs` | Hardcoded inline |
| Search Bar | `search/search_render.rs` | Hardcoded inline |
| Help Popup | `help/help_popup_render.rs` | Hardcoded inline |
| Help Line | `help/help_line_render.rs` | Hardcoded inline |
| AI Window | `ai/ai_render.rs` | Hardcoded inline |
| History | `history/history_render.rs` | Hardcoded inline |
| Snippets | `snippets/snippet_render.rs` | Hardcoded inline |
| Autocomplete | `autocomplete/autocomplete_render.rs` | Hardcoded inline |
| Syntax Highlighting | `syntax_highlight.rs` | Hardcoded inline |
| Notifications | `notification/notification_state.rs` | Type-based method |
| Tooltip | `tooltip/tooltip_render.rs` | Hardcoded inline |
| Scrollbar | `widgets/scrollbar.rs` | Parameterized |

### Color Duplication Examples

| Color | Usage Count | Locations |
|-------|-------------|-----------|
| `Color::Cyan` | 8+ files | Borders, highlights, active tabs, syntax |
| `Color::Yellow` | 7+ files | Keywords, warnings, keys in help |
| `Color::DarkGray` | 7+ files | Inactive states, hints, backgrounds |
| `Color::White` | 10+ files | Normal text content |
| `Color::Indexed(236)` | 3 files | Hover backgrounds (inconsistent) |

### Existing Partial Centralization

1. **`results_render.rs`** - Has module-level constants:
   ```rust
   const MATCH_HIGHLIGHT_BG: Color = Color::Rgb(128, 128, 128);
   const CURSOR_LINE_BG: Color = Color::Rgb(50, 55, 65);
   ```

2. **`notification_state.rs`** - Type-based styling:
   ```rust
   impl NotificationType {
       fn style(self) -> NotificationStyle { ... }
   }
   ```

3. **`ai/suggestion/parser.rs`** - Color method:
   ```rust
   impl SuggestionType {
       pub fn color(&self) -> Color { ... }
   }
   ```

---

## Proposed Solution

### New File: `src/theme.rs`

A single module containing all theme definitions organized by semantic purpose.

### Architecture

```
src/
├── theme.rs              # NEW: All color/style definitions
├── results/
│   └── results_render.rs # Uses theme::results::*
├── input/
│   └── input_render.rs   # Uses theme::input::*
├── ...
```

### Module Structure

Each component gets its own dedicated submodule with all its specific colors/styles. No generic "popup" or "list" modules—each popup has distinct styling.

```rust
// src/theme.rs

use ratatui::style::{Color, Modifier, Style};

/// Core color palette - shared base colors
/// Only use these directly when a component truly shares the same color.
/// Otherwise, define component-specific constants that reference these.
pub mod palette {
    use super::*;

    // Text colors
    pub const TEXT: Color = Color::White;
    pub const TEXT_DIM: Color = Color::DarkGray;
    pub const TEXT_MUTED: Color = Color::Gray;

    // Background colors
    pub const BG_DARK: Color = Color::Black;
    pub const BG_HOVER: Color = Color::Indexed(236);  // Dark gray

    // Semantic colors
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const ERROR: Color = Color::Red;
    pub const INFO: Color = Color::Blue;

    // Shared cursor style (used by textarea widgets in history, search, snippets, input)
    pub const CURSOR: Style = Style::new().add_modifier(Modifier::REVERSED);
}

/// Input field styles
pub mod input {
    use super::*;

    // Mode indicator colors
    pub const MODE_INSERT: Color = Color::Cyan;
    pub const MODE_NORMAL: Color = Color::Yellow;
    pub const MODE_OPERATOR: Color = Color::Green;
    pub const MODE_CHAR_SEARCH: Color = Color::Magenta;

    // Border colors (focused border uses mode color)
    pub const BORDER_UNFOCUSED: Color = Color::DarkGray;

    // Title hints
    pub const SYNTAX_ERROR_WARNING: Color = Color::Yellow;
    pub const TOOLTIP_HINT: Color = Color::Magenta;
    pub const AI_HINT: Color = Color::Cyan;
    pub const UNFOCUSED_HINT: Color = Color::DarkGray;

    // Unfocused query text
    pub const QUERY_UNFOCUSED: Color = Color::DarkGray;

    pub const CURSOR: Style = Style::new()
        .add_modifier(Modifier::REVERSED);
}

/// Results pane styles
pub mod results {
    use super::*;

    // Border colors
    pub const BORDER_FOCUSED: Color = Color::Cyan;
    pub const BORDER_UNFOCUSED: Color = Color::DarkGray;
    pub const BORDER_WARNING: Color = Color::Yellow;  // Partial results
    pub const BORDER_ERROR: Color = Color::Red;       // Error state
    pub const BACKGROUND: Color = Color::Black;

    // Search mode text colors (in title)
    pub const SEARCH_ACTIVE: Color = Color::LightMagenta;
    pub const SEARCH_INACTIVE: Color = Color::DarkGray;

    // Query timing indicator colors
    pub const TIMING_NORMAL: Color = Color::Cyan;     // < 200ms (uses border color)
    pub const TIMING_SLOW: Color = Color::Yellow;     // 200-1000ms
    pub const TIMING_VERY_SLOW: Color = Color::Red;   // > 1000ms

    // Query state indicators
    pub const RESULT_OK: Color = Color::Green;
    pub const RESULT_WARNING: Color = Color::Yellow;
    pub const RESULT_ERROR: Color = Color::Red;
    pub const RESULT_PENDING: Color = Color::Gray;

    // Search match highlighting
    pub const MATCH_HIGHLIGHT_BG: Color = Color::Rgb(128, 128, 128);
    pub const MATCH_HIGHLIGHT_FG: Color = Color::White;
    pub const CURRENT_MATCH_BG: Color = Color::Rgb(255, 165, 0);  // Orange
    pub const CURRENT_MATCH_FG: Color = Color::Black;

    // Cursor and selection
    pub const CURSOR_LINE_BG: Color = Color::Rgb(50, 55, 65);
    pub const HOVERED_LINE_BG: Color = Color::Rgb(45, 50, 60);
    pub const VISUAL_SELECTION_BG: Color = Color::Rgb(70, 80, 100);
    pub const CURSOR_INDICATOR_FG: Color = Color::Rgb(255, 85, 85);  // Red

    // Stale state
    pub const STALE_MODIFIER: Modifier = Modifier::DIM;

    // Spinner animation colors (rainbow)
    pub const SPINNER_COLORS: &[Color] = &[
        Color::Rgb(255, 107, 107), // Red/Coral
        Color::Rgb(255, 159, 67),  // Orange
        Color::Rgb(254, 202, 87),  // Yellow
        Color::Rgb(72, 219, 147),  // Green
        Color::Rgb(69, 170, 242),  // Blue
        Color::Rgb(120, 111, 213), // Indigo
        Color::Rgb(214, 128, 255), // Violet
        Color::Rgb(255, 121, 198), // Pink
    ];
}

/// Search bar styles
pub mod search {
    use super::*;

    pub const BORDER_ACTIVE: Color = Color::LightMagenta;
    pub const BORDER_INACTIVE: Color = Color::DarkGray;
    pub const BACKGROUND: Color = Color::Black;

    // Text colors
    pub const TEXT_ACTIVE: Color = Color::White;
    pub const TEXT_INACTIVE: Color = Color::DarkGray;

    // Match count display
    pub const NO_MATCHES: Color = Color::Red;
    pub const MATCH_COUNT: Color = Color::Gray;
    pub const MATCH_COUNT_CONFIRMED: Color = Color::DarkGray;

    // Hints at bottom
    pub const HINTS: Color = Color::LightMagenta;
}

/// Help popup styles
pub mod help {
    use super::*;

    // Border and title
    pub const BORDER: Color = Color::Cyan;
    pub const BACKGROUND: Color = Color::Black;
    pub const SCROLLBAR: Color = Color::Cyan;
    pub const TITLE: Style = Style::new()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    // Tab bar
    pub const TAB_ACTIVE: Style = Style::new()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    pub const TAB_INACTIVE: Style = Style::new()
        .fg(Color::DarkGray);
    pub const TAB_HOVER_FG: Color = Color::White;
    pub const TAB_HOVER_BG: Color = Color::Indexed(236);

    // Content
    pub const SECTION_HEADER: Style = Style::new()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    pub const KEY: Style = Style::new()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    pub const DESCRIPTION: Color = Color::White;

    // Footer
    pub const FOOTER: Color = Color::DarkGray;
}

/// History popup styles
pub mod history {
    use super::*;

    // Border and scrollbar
    pub const BORDER: Color = Color::Cyan;
    pub const SCROLLBAR: Color = Color::Cyan;
    pub const BACKGROUND: Color = Color::Black;

    // List items
    pub const ITEM_NORMAL_FG: Color = Color::White;
    pub const ITEM_NORMAL_BG: Color = Color::Black;
    pub const ITEM_SELECTED_FG: Color = Color::Black;
    pub const ITEM_SELECTED_BG: Color = Color::Cyan;
    pub const ITEM_SELECTED_MODIFIER: Modifier = Modifier::BOLD;

    // Empty state
    pub const NO_MATCHES: Color = Color::DarkGray;

    // Search textarea
    pub const SEARCH_TEXT: Color = Color::White;
    pub const SEARCH_BG: Color = Color::Black;
}

/// Snippets popup styles
pub mod snippets {
    use super::*;

    // Border (distinct green color)
    pub const BORDER: Color = Color::LightGreen;
    pub const SCROLLBAR: Color = Color::LightGreen;
    pub const BACKGROUND: Color = Color::Black;

    // List items
    pub const ITEM_NORMAL_FG: Color = Color::White;
    pub const ITEM_NORMAL_BG: Color = Color::Black;
    pub const ITEM_SELECTED_FG: Color = Color::Black;
    pub const ITEM_SELECTED_BG: Color = Color::Cyan;
    pub const ITEM_SELECTED_MODIFIER: Modifier = Modifier::BOLD;
    pub const ITEM_HOVERED_FG: Color = Color::White;
    pub const ITEM_HOVERED_BG: Color = Color::Indexed(236);

    // Content
    pub const NAME: Color = Color::White;
    pub const DESCRIPTION: Color = Color::DarkGray;
    pub const QUERY_PREVIEW: Color = Color::Yellow;
    pub const CATEGORY: Color = Color::Green;

    // Edit/Create mode
    pub const FIELD_ACTIVE_BORDER: Color = Color::Yellow;
    pub const FIELD_INACTIVE_BORDER: Color = Color::LightGreen;
    pub const FIELD_TEXT: Color = Color::White;
    pub const FIELD_BG: Color = Color::Black;

    // Delete confirmation
    pub const DELETE_BORDER: Color = Color::Red;

    // Keyboard hints
    pub const HINT_KEY: Color = Color::Yellow;
    pub const HINT_TEXT: Color = Color::White;

    // Search
    pub const SEARCH_TEXT: Color = Color::White;
    pub const SEARCH_BG: Color = Color::Black;
}

/// AI assistant styles
pub mod ai {
    use super::*;

    // Border and title
    pub const BORDER: Color = Color::Cyan;
    pub const BACKGROUND: Color = Color::Black;
    pub const SCROLLBAR: Color = Color::Cyan;
    pub const TITLE: Style = Style::new()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    // Model info (underlined for clickable appearance)
    pub const MODEL_NAME: Style = Style::new()
        .fg(Color::Blue)
        .add_modifier(Modifier::UNDERLINED);

    // Loading state
    pub const LOADING_ICON: Color = Color::Yellow;
    pub const LOADING_TEXT: Style = Style::new()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    pub const TOKEN_COUNT: Color = Color::Gray;

    // Thinking state
    pub const THINKING_ICON: Color = Color::Yellow;
    pub const THINKING_TEXT: Style = Style::new()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::ITALIC);

    // Error state
    pub const ERROR_ICON: Color = Color::Red;
    pub const ERROR_TITLE: Style = Style::new()
        .fg(Color::Red)
        .add_modifier(Modifier::BOLD);
    pub const ERROR_MESSAGE: Color = Color::Red;
    pub const RETRY_HINT: Color = Color::DarkGray;

    // Content text
    pub const QUERY_TEXT: Color = Color::Cyan;
    pub const STATUS_TEXT: Color = Color::Cyan;
    pub const RESULT_TEXT: Color = Color::White;

    // Suggestion list
    pub const SUGGESTION_SELECTED_BG: Color = Color::DarkGray;
    pub const SUGGESTION_HOVERED_BG: Color = Color::Indexed(236);
    pub const SUGGESTION_TEXT_SELECTED: Color = Color::Black;
    pub const SUGGESTION_TEXT_NORMAL: Color = Color::DarkGray;
    pub const SUGGESTION_NAME: Color = Color::Cyan;
    pub const SUGGESTION_DESC_NORMAL: Color = Color::DarkGray;
    pub const SUGGESTION_DESC_MUTED: Color = Color::Gray;

    // Suggestion type colors
    pub const SUGGESTION_FIX: Color = Color::Red;
    pub const SUGGESTION_OPTIMIZE: Color = Color::Yellow;
    pub const SUGGESTION_NEXT: Color = Color::Green;

    // Hints
    pub const HINT: Color = Color::DarkGray;
    pub const KEY_HINT: Color = Color::Yellow;
}

/// Autocomplete dropdown styles
pub mod autocomplete {
    use super::*;

    // Border and scrollbar
    pub const BORDER: Color = Color::Cyan;
    pub const SCROLLBAR: Color = Color::Cyan;
    pub const BACKGROUND: Color = Color::Black;

    // List items
    pub const ITEM_NORMAL_FG: Color = Color::White;
    pub const ITEM_NORMAL_BG: Color = Color::Black;
    pub const ITEM_SELECTED_FG: Color = Color::Black;
    pub const ITEM_SELECTED_BG: Color = Color::Cyan;
    pub const ITEM_SELECTED_MODIFIER: Modifier = Modifier::BOLD;

    // Completion type colors
    pub const TYPE_FUNCTION: Color = Color::Yellow;
    pub const TYPE_FIELD: Color = Color::Cyan;
    pub const TYPE_OPERATOR: Color = Color::Magenta;
    pub const TYPE_PATTERN: Color = Color::Green;
    pub const TYPE_VARIABLE: Color = Color::Red;
}

/// Tooltip styles
pub mod tooltip {
    use super::*;

    // Border and title (distinct magenta)
    pub const BORDER: Color = Color::Magenta;
    pub const BACKGROUND: Color = Color::Black;
    pub const TITLE: Style = Style::new()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD);

    // Content
    pub const DESCRIPTION: Color = Color::White;
    pub const EXAMPLE: Color = Color::Cyan;
    pub const EXAMPLE_DESC: Color = Color::Gray;
    pub const TIP: Color = Color::Yellow;
    pub const SEPARATOR: Color = Color::DarkGray;
    pub const DISMISS_HINT: Color = Color::DarkGray;
}

/// Notification styles
pub mod notification {
    use super::*;

    pub struct NotificationColors {
        pub fg: Color,
        pub bg: Color,
        pub border: Color,
    }

    pub const INFO: NotificationColors = NotificationColors {
        fg: Color::White,
        bg: Color::DarkGray,
        border: Color::Gray,
    };

    pub const WARNING: NotificationColors = NotificationColors {
        fg: Color::Black,
        bg: Color::Yellow,
        border: Color::Yellow,
    };

    pub const ERROR: NotificationColors = NotificationColors {
        fg: Color::White,
        bg: Color::Red,
        border: Color::LightRed,
    };
}

/// Help line (bottom status bar) styles
pub mod help_line {
    use super::*;

    pub const TEXT: Color = Color::DarkGray;
}

/// Scrollbar styles (for components that share scrollbar appearance)
pub mod scrollbar {
    use super::*;

    pub const DEFAULT: Color = Color::Cyan;
    pub const TRACK: Color = Color::DarkGray;
}

/// Syntax highlighting styles (for jq query input)
pub mod syntax {
    use super::*;

    pub const KEYWORD: Color = Color::Yellow;
    pub const FUNCTION: Color = Color::Blue;
    pub const STRING: Color = Color::Green;
    pub const NUMBER: Color = Color::Cyan;
    pub const OPERATOR: Color = Color::Magenta;
    pub const VARIABLE: Color = Color::Red;
    pub const FIELD: Color = Color::Cyan;

    /// Bracket pair matching style (color + bold + underlined)
    /// Applied to matching brackets when cursor is on a bracket
    pub mod bracket_match {
        use super::*;

        pub const COLOR: Color = Color::Yellow;
        pub const STYLE: Style = Style::new()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED);
    }
}
```

---

## Modifier Usage Reference

This section documents all `Modifier::` usages in the codebase. Modifiers are style attributes that affect text rendering beyond just color (bold, italic, underlined, etc.).

### Modifier by Type

| Modifier | Usage | Components |
|----------|-------|------------|
| `BOLD` | Emphasis, headers, selected items | help (keys, headers), history (selected), snippets (selected), autocomplete (selected), ai (title, loading, thinking, error), tooltip (title) |
| `REVERSED` | Cursor display in text areas | input, search, history, snippets (shared via `palette::CURSOR`) |
| `DIM` | Stale/outdated content | results (stale results indicator) |
| `UNDERLINED` | Links, special highlights | syntax (bracket match), ai (model name link) |
| `ITALIC` | Thinking/processing states | ai (thinking state text) |

### Modifier by Component

**Input Field** (`input_render.rs`, `input_state.rs`)
- `REVERSED` - Cursor style

**Results Pane** (`results_render.rs`)
- `DIM` - Stale results indicator

**Search Bar** (`search_render.rs`, `search_state.rs`)
- `REVERSED` - Cursor style

**Help Popup** (`help_popup_render.rs`)
- `BOLD` - Section headers, key bindings

**History Popup** (`history_render.rs`, `history_state.rs`)
- `BOLD` - Selected item
- `REVERSED` - Search cursor

**Snippets Popup** (`snippet_render.rs`, `snippet_state.rs`)
- `BOLD` - Selected item
- `REVERSED` - Field cursor

**Autocomplete** (`autocomplete_render.rs`)
- `BOLD` - Selected item

**AI Window** (`ai_render.rs`, `ai/render/content.rs`)
- `BOLD` - Title, loading text, thinking text, error title
- `UNDERLINED` - Model name (clickable appearance)
- `ITALIC` - Thinking state text

**Tooltip** (`tooltip_render.rs`)
- `BOLD` - Title

**Syntax Highlighting** (`syntax_highlight/overlay.rs`)
- `BOLD` + `UNDERLINED` - Bracket pair matching (combined)

### Theme Module Integration

Modifiers are included in `Style` constants where they're always used together:

```rust
// Examples from theme.rs

// In palette module - shared cursor style
pub const CURSOR: Style = Style::new().add_modifier(Modifier::REVERSED);

// In help module - always bold keys
pub const KEY: Style = Style::new()
    .fg(Color::Yellow)
    .add_modifier(Modifier::BOLD);

// In results module - standalone modifier
pub const STALE_MODIFIER: Modifier = Modifier::DIM;

// In syntax::bracket_match - combined modifiers
pub const STYLE: Style = Style::new()
    .fg(Color::Yellow)
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::UNDERLINED);
```

---

## Migration Strategy

### Phase 1: Create Theme Module
1. Create `src/theme.rs` with all color definitions
2. Add `pub mod theme;` to `lib.rs` or `main.rs`
3. No changes to render files yet

### Phase 2: Migrate Results Pane (Largest File)
1. Replace constants in `results_render.rs` with `theme::results::*`
2. Remove local constant definitions
3. Test thoroughly

### Phase 3: Migrate Component by Component

**Order by complexity (simplest first):**

1. **Scrollbar** (`widgets/scrollbar.rs`) → `theme::scrollbar::*`
2. **Search** (`search/search_render.rs`) → `theme::search::*`
3. **Notification** (`notification/notification_state.rs`) → `theme::notification::*`
4. **Help Line** (`help/help_line_render.rs`) → `theme::help_line::*`
5. **History** (`history/history_render.rs`) → `theme::history::*`
6. **Input** (`input/input_render.rs`, `input/input_state.rs`) → `theme::input::*`
7. **Autocomplete** (`autocomplete/autocomplete_render.rs`) → `theme::autocomplete::*`
8. **Tooltip** (`tooltip/tooltip_render.rs`) → `theme::tooltip::*`
9. **Help Popup** (`help/help_popup_render.rs`) → `theme::help::*`
10. **Snippets** (`snippets/snippet_render.rs`) → `theme::snippets::*`
11. **AI** (`ai/ai_render.rs`, `ai/render/suggestions.rs`, `ai/render/content.rs`) → `theme::ai::*`
12. **Syntax Highlighting** (`syntax_highlight.rs`, `syntax_highlight/overlay.rs`) → `theme::syntax::*`

### Phase 4: Cleanup
1. Remove all inline color definitions from render files
2. Ensure no `Color::` imports remain in render files (only `theme::*`)
3. Update any tests that assert on specific colors

### Phase 5: Update Documentation
1. Update `CLAUDE.md` to document theme.rs usage guidelines
2. Update `CONTRIBUTING.md` (if exists) with theme contribution rules

Add the following guidelines to documentation:

```markdown
## Theme & Styling

All colors and styles are centralized in `src/theme.rs`. When adding or modifying UI components:

- **DO** add new colors to the appropriate module in `theme.rs`
- **DO** use `theme::module::CONSTANT` in render files
- **DON'T** hardcode `Color::*` values directly in render files
- **DON'T** import `ratatui::style::Color` in render files (import from theme instead)

Example:
```rust
// Good
use crate::theme;
let style = Style::default().fg(theme::input::MODE_INSERT);

// Bad
use ratatui::style::Color;
let style = Style::default().fg(Color::Cyan);
```
```

---

## File Changes Summary

| File | Action |
|------|--------|
| `src/theme.rs` | **CREATE** - Central theme module |
| `src/lib.rs` or `src/main.rs` | **EDIT** - Add `pub mod theme;` |
| `src/results/results_render.rs` | **EDIT** - Use `theme::results::*` |
| `src/input/input_render.rs` | **EDIT** - Use `theme::input::*` |
| `src/input/input_state.rs` | **EDIT** - Use `theme::input::*` (initial border) |
| `src/search/search_render.rs` | **EDIT** - Use `theme::search::*` |
| `src/search/search_state.rs` | **EDIT** - Use shared cursor style |
| `src/help/help_popup_render.rs` | **EDIT** - Use `theme::help::*` |
| `src/help/help_line_render.rs` | **EDIT** - Use `theme::help_line::*` |
| `src/ai/ai_render.rs` | **EDIT** - Use `theme::ai::*` |
| `src/ai/render/suggestions.rs` | **EDIT** - Use `theme::ai::*` |
| `src/ai/render/content.rs` | **EDIT** - Use `theme::ai::*` |
| `src/ai/suggestion/parser.rs` | **EDIT** - Use `theme::ai::*` |
| `src/history/history_render.rs` | **EDIT** - Use `theme::history::*` |
| `src/history/history_state.rs` | **EDIT** - Use shared cursor style |
| `src/snippets/snippet_render.rs` | **EDIT** - Use `theme::snippets::*` |
| `src/snippets/snippet_state.rs` | **EDIT** - Use shared cursor style |
| `src/autocomplete/autocomplete_render.rs` | **EDIT** - Use `theme::autocomplete::*` |
| `src/notification/notification_state.rs` | **EDIT** - Use `theme::notification::*` |
| `src/tooltip/tooltip_render.rs` | **EDIT** - Use `theme::tooltip::*` |
| `src/syntax_highlight.rs` | **EDIT** - Use `theme::syntax::*` |
| `src/syntax_highlight/overlay.rs` | **EDIT** - Use `theme::syntax::bracket_match::STYLE` |
| `src/widgets/scrollbar.rs` | **EDIT** - Use `theme::scrollbar::*` |
| `CLAUDE.md` | **EDIT** - Add theme.rs usage guidelines |
| `CONTRIBUTING.md` | **EDIT** - Add theme contribution rules (if file exists) |

---

## Benefits

1. **Single source of truth** - All colors defined in one file
2. **Easy to modify** - Change a color once, updates everywhere
3. **Consistency** - Eliminates duplicate `Color::Indexed(236)` definitions
4. **Semantic naming** - `theme::palette::PRIMARY` vs `Color::Cyan`
5. **Documentation** - Theme file serves as color reference
6. **Future-proof** - Easy to add theme switching later if needed

---

## Non-Goals

- **NOT** implementing theme switching UI
- **NOT** adding user-configurable themes
- **NOT** supporting light/dark mode toggle
- **NOT** reading theme from config file

This is purely a code organization improvement.

---

## Testing Strategy

1. **Visual testing** - Run app after each migration phase, verify colors unchanged
2. **Snapshot tests** - Update any tests that assert on ANSI color codes
3. **Build verification** - `cargo build --release` after each phase
4. **No regressions** - Colors should look identical before and after

---

## Estimated Scope

- **Files to create**: 1 (`theme.rs`)
- **Files to modify**: ~22 source files + test files with color assertions + documentation
- **Lines of theme code**: ~400 lines in `theme.rs`
- **Lines to remove**: ~150 scattered constants/inline colors
