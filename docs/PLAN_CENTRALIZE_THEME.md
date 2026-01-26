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

```rust
// src/theme.rs

use ratatui::style::{Color, Modifier, Style};

/// Core color palette - the actual color values
pub mod palette {
    use super::*;

    // Primary colors
    pub const PRIMARY: Color = Color::Cyan;
    pub const SECONDARY: Color = Color::Yellow;
    pub const ACCENT: Color = Color::Magenta;

    // Text colors
    pub const TEXT: Color = Color::White;
    pub const TEXT_DIM: Color = Color::DarkGray;
    pub const TEXT_MUTED: Color = Color::Gray;

    // Background colors
    pub const BG_DARK: Color = Color::Black;
    pub const BG_SUBTLE: Color = Color::Indexed(236);  // Dark gray
    pub const BG_HOVER: Color = Color::Indexed(236);
    pub const BG_SELECTED: Color = Color::Cyan;

    // Semantic colors
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const ERROR: Color = Color::Red;
    pub const INFO: Color = Color::Blue;

    // Syntax highlighting colors
    pub const SYNTAX_KEYWORD: Color = Color::Yellow;
    pub const SYNTAX_FUNCTION: Color = Color::Blue;
    pub const SYNTAX_STRING: Color = Color::Green;
    pub const SYNTAX_NUMBER: Color = Color::Cyan;
    pub const SYNTAX_OPERATOR: Color = Color::Magenta;
    pub const SYNTAX_VARIABLE: Color = Color::Red;
    pub const SYNTAX_FIELD: Color = Color::Cyan;
}

/// Border styles for different components
pub mod border {
    use super::*;

    pub const DEFAULT: Color = palette::PRIMARY;
    pub const FOCUSED: Color = palette::PRIMARY;
    pub const UNFOCUSED: Color = Color::DarkGray;
    pub const SEARCH: Color = Color::LightMagenta;
    pub const SNIPPET: Color = Color::LightGreen;
    pub const TOOLTIP: Color = palette::ACCENT;
}

/// Input field styles
pub mod input {
    use super::*;

    pub fn mode_color(mode: &str) -> Color {
        match mode {
            "insert" => palette::PRIMARY,
            "normal" => palette::SECONDARY,
            "operator" => palette::SUCCESS,
            "char_search" => palette::ACCENT,
            _ => palette::TEXT,
        }
    }

    pub const CURSOR: Style = Style::new()
        .add_modifier(Modifier::REVERSED);
}

/// Results pane styles
pub mod results {
    use super::*;

    // Search match highlighting
    pub const MATCH_HIGHLIGHT_BG: Color = Color::Rgb(128, 128, 128);
    pub const MATCH_HIGHLIGHT_FG: Color = Color::White;
    pub const CURRENT_MATCH_BG: Color = Color::Rgb(200, 150, 50);
    pub const CURRENT_MATCH_FG: Color = Color::Black;

    // Cursor and selection
    pub const CURSOR_LINE_BG: Color = Color::Rgb(50, 55, 65);
    pub const HOVERED_LINE_BG: Color = Color::Rgb(40, 44, 52);
    pub const VISUAL_SELECTION_BG: Color = Color::Rgb(68, 68, 102);
    pub const CURSOR_INDICATOR_FG: Color = Color::Rgb(97, 175, 239);

    // Stale state
    pub const STALE_MODIFIER: Modifier = Modifier::DIM;

    // Spinner animation colors
    pub const SPINNER_COLORS: &[Color] = &[
        Color::Rgb(255, 107, 107), // Red/Coral
        Color::Rgb(255, 159, 67),  // Orange
        Color::Rgb(254, 202, 87),  // Yellow
        Color::Rgb(46, 213, 115),  // Green
        Color::Rgb(30, 144, 255),  // Blue
        Color::Rgb(156, 136, 255), // Purple
        Color::Rgb(255, 107, 129), // Pink
        Color::Rgb(116, 185, 255), // Light Blue
    ];
}

/// Popup styles (help, history, etc.)
pub mod popup {
    use super::*;

    pub const BORDER: Color = palette::PRIMARY;
    pub const TITLE: Style = Style::new()
        .fg(palette::PRIMARY)
        .add_modifier(Modifier::BOLD);

    pub const TAB_ACTIVE: Style = Style::new()
        .fg(palette::PRIMARY)
        .add_modifier(Modifier::BOLD);
    pub const TAB_INACTIVE: Style = Style::new()
        .fg(palette::TEXT_DIM);
    pub const TAB_HOVER: Style = Style::new()
        .fg(palette::TEXT)
        .bg(palette::BG_HOVER);

    pub const FOOTER: Style = Style::new()
        .fg(palette::TEXT_DIM);
}

/// List item styles (history, snippets, autocomplete)
pub mod list {
    use super::*;

    pub const ITEM_NORMAL: Style = Style::new()
        .fg(palette::TEXT)
        .bg(palette::BG_DARK);

    pub const ITEM_SELECTED: Style = Style::new()
        .fg(palette::BG_DARK)
        .bg(palette::BG_SELECTED)
        .add_modifier(Modifier::BOLD);

    pub const ITEM_HOVERED: Style = Style::new()
        .fg(palette::TEXT)
        .bg(palette::BG_HOVER);
}

/// Autocomplete-specific styles
pub mod autocomplete {
    use super::*;

    pub fn type_color(completion_type: &str) -> Color {
        match completion_type {
            "function" => palette::SECONDARY,
            "field" => palette::PRIMARY,
            "operator" => palette::ACCENT,
            "pattern" => palette::SUCCESS,
            "variable" => palette::ERROR,
            _ => palette::TEXT,
        }
    }
}

/// Notification styles
pub mod notification {
    use super::*;

    pub struct NotificationColors {
        pub fg: Color,
        pub bg: Color,
        pub border: Color,
    }

    pub fn style_for(notification_type: &str) -> NotificationColors {
        match notification_type {
            "info" => NotificationColors {
                fg: palette::TEXT,
                bg: palette::TEXT_DIM,
                border: palette::TEXT_MUTED,
            },
            "warning" => NotificationColors {
                fg: palette::BG_DARK,
                bg: palette::WARNING,
                border: palette::WARNING,
            },
            "error" => NotificationColors {
                fg: palette::TEXT,
                bg: Color::Rgb(139, 0, 0),
                border: palette::ERROR,
            },
            _ => NotificationColors {
                fg: palette::TEXT,
                bg: palette::TEXT_DIM,
                border: palette::TEXT_MUTED,
            },
        }
    }
}

/// AI assistant styles
pub mod ai {
    use super::*;

    pub const BORDER: Color = palette::PRIMARY;
    pub const MODEL_NAME: Color = palette::INFO;

    pub fn suggestion_color(suggestion_type: &str) -> Color {
        match suggestion_type {
            "fix" => palette::ERROR,
            "optimize" => palette::WARNING,
            "next" => palette::SUCCESS,
            _ => palette::TEXT,
        }
    }
}

/// Tooltip styles
pub mod tooltip {
    use super::*;

    pub const BORDER: Color = palette::ACCENT;
    pub const TITLE: Style = Style::new()
        .fg(palette::ACCENT)
        .add_modifier(Modifier::BOLD);
    pub const DESCRIPTION: Color = palette::TEXT;
    pub const EXAMPLE: Color = palette::PRIMARY;
    pub const TIP: Color = palette::SECONDARY;
    pub const SEPARATOR: Color = palette::TEXT_DIM;
    pub const HINT: Color = palette::TEXT_DIM;
}

/// Search bar styles
pub mod search {
    use super::*;

    pub const BORDER_ACTIVE: Color = Color::LightMagenta;
    pub const BORDER_INACTIVE: Color = palette::TEXT_DIM;
    pub const NO_MATCHES: Color = palette::ERROR;
    pub const MATCH_COUNT: Color = palette::TEXT_MUTED;
}

/// Help popup specific styles
pub mod help {
    use super::*;

    pub const SECTION_HEADER: Style = Style::new()
        .fg(palette::PRIMARY)
        .add_modifier(Modifier::BOLD);
    pub const KEY: Style = Style::new()
        .fg(palette::SECONDARY)
        .add_modifier(Modifier::BOLD);
    pub const DESCRIPTION: Color = palette::TEXT;
}

/// Scrollbar styles
pub mod scrollbar {
    use super::*;

    pub const DEFAULT: Color = palette::PRIMARY;
    pub const TRACK: Color = palette::TEXT_DIM;
}

/// Syntax highlighting styles
pub mod syntax {
    use super::*;

    pub fn style_for(token_type: &str) -> Style {
        let color = match token_type {
            "keyword" => palette::SYNTAX_KEYWORD,
            "function" | "builtin" => palette::SYNTAX_FUNCTION,
            "string" => palette::SYNTAX_STRING,
            "number" => palette::SYNTAX_NUMBER,
            "operator" => palette::SYNTAX_OPERATOR,
            "variable" => palette::SYNTAX_VARIABLE,
            "field" => palette::SYNTAX_FIELD,
            _ => palette::TEXT,
        };
        Style::default().fg(color)
    }
}
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

1. **Scrollbar** (`widgets/scrollbar.rs`) - Already parameterized
2. **Search** (`search/search_render.rs`) - Small, few colors
3. **Notification** (`notification/notification_state.rs`) - Already has structure
4. **History** (`history/history_render.rs`) - Simple list styles
5. **Input** (`input/input_render.rs`) - Mode-based colors
6. **Autocomplete** (`autocomplete/autocomplete_render.rs`) - Type-based colors
7. **Tooltip** (`tooltip/tooltip_render.rs`) - Moderate complexity
8. **Help Popup** (`help/help_popup_render.rs`) - Tab styles, sections
9. **Snippets** (`snippets/snippet_render.rs`) - Large file, list styles
10. **AI** (`ai/ai_render.rs`, `ai/render/*`) - Multiple sub-components
11. **Syntax Highlighting** (`syntax_highlight.rs`) - Token-based colors

### Phase 4: Cleanup
1. Remove all inline color definitions from render files
2. Ensure no `Color::` imports remain in render files (only `theme::*`)
3. Update any tests that assert on specific colors

---

## File Changes Summary

| File | Action |
|------|--------|
| `src/theme.rs` | **CREATE** - Central theme module |
| `src/lib.rs` or `src/main.rs` | **EDIT** - Add `pub mod theme;` |
| `src/results/results_render.rs` | **EDIT** - Use `theme::results::*` |
| `src/input/input_render.rs` | **EDIT** - Use `theme::input::*` |
| `src/search/search_render.rs` | **EDIT** - Use `theme::search::*` |
| `src/help/help_popup_render.rs` | **EDIT** - Use `theme::help::*`, `theme::popup::*` |
| `src/ai/ai_render.rs` | **EDIT** - Use `theme::ai::*` |
| `src/ai/render/suggestions.rs` | **EDIT** - Use `theme::ai::*` |
| `src/ai/suggestion/parser.rs` | **EDIT** - Use `theme::ai::*` |
| `src/history/history_render.rs` | **EDIT** - Use `theme::list::*` |
| `src/snippets/snippet_render.rs` | **EDIT** - Use `theme::list::*` |
| `src/autocomplete/autocomplete_render.rs` | **EDIT** - Use `theme::autocomplete::*` |
| `src/notification/notification_state.rs` | **EDIT** - Use `theme::notification::*` |
| `src/tooltip/tooltip_render.rs` | **EDIT** - Use `theme::tooltip::*` |
| `src/syntax_highlight.rs` | **EDIT** - Use `theme::syntax::*` |
| `src/widgets/scrollbar.rs` | **EDIT** - Use `theme::scrollbar::*` |

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
- **Files to modify**: ~15 render files
- **Lines of theme code**: ~250 lines in `theme.rs`
- **Lines to remove**: ~100 scattered constants/inline colors
