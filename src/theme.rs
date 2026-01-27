//! Centralized theme configuration for all UI components.
//!
//! All colors and styles are defined here. When adding or modifying UI components:
//! - Add new colors to the appropriate module
//! - Use `theme::module::CONSTANT` in render files
//! - Do NOT hardcode `Color::*` values directly in render files
//!
//! Theme: Galaxy - Purple/pink accents with deep space blue background

use ratatui::style::{Color, Modifier, Style};

/// Core color palette - shared base colors.
/// Only use these directly when a component truly shares the same color.
/// Otherwise, define component-specific constants that reference these.
pub mod palette {
    use super::*;

    // Text colors - softer than pure white
    pub const TEXT: Color = Color::Rgb(236, 236, 244);
    pub const TEXT_DIM: Color = Color::Rgb(90, 92, 119);
    pub const TEXT_MUTED: Color = Color::Rgb(130, 133, 158);

    // Background colors - deep space blue tints
    pub const BG_DARK: Color = Color::Rgb(26, 26, 46);
    pub const BG_SURFACE: Color = Color::Rgb(35, 35, 58);
    pub const BG_HOVER: Color = Color::Rgb(45, 45, 72);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(55, 55, 85);

    // Semantic colors - vibrant Galaxy palette
    pub const SUCCESS: Color = Color::Rgb(107, 203, 119);
    pub const WARNING: Color = Color::Rgb(255, 217, 61);
    pub const ERROR: Color = Color::Rgb(224, 108, 117);
    pub const INFO: Color = Color::Rgb(0, 217, 255);

    // Accent colors
    pub const CYAN: Color = Color::Rgb(0, 217, 255);
    pub const YELLOW: Color = Color::Rgb(255, 217, 61);
    pub const GREEN: Color = Color::Rgb(107, 203, 119);
    pub const MAGENTA: Color = Color::Rgb(198, 120, 221);
    pub const PINK: Color = Color::Rgb(255, 107, 157);
    pub const RED: Color = Color::Rgb(224, 108, 117);
    pub const ORANGE: Color = Color::Rgb(255, 184, 108);
    pub const PURPLE: Color = Color::Rgb(189, 147, 249);

    // Shared cursor style (used by textarea widgets in history, search, snippets, input)
    pub const CURSOR: Style = Style::new().add_modifier(Modifier::REVERSED);
}

/// Input field styles
pub mod input {
    use super::*;

    // Mode indicator colors - vibrant and distinct
    pub const MODE_INSERT: Color = Color::Rgb(0, 217, 255); // Electric cyan
    pub const MODE_NORMAL: Color = Color::Rgb(255, 217, 61); // Golden yellow
    pub const MODE_OPERATOR: Color = Color::Rgb(107, 203, 119); // Fresh green
    pub const MODE_CHAR_SEARCH: Color = Color::Rgb(255, 107, 157); // Hot pink

    // Border colors (focused border uses mode color)
    pub const BORDER_UNFOCUSED: Color = Color::Rgb(90, 92, 119);
    pub const BORDER_ERROR: Color = Color::Rgb(224, 108, 117);

    // Title hints
    pub const SYNTAX_ERROR_WARNING: Color = Color::Rgb(255, 217, 61);
    pub const TOOLTIP_HINT: Color = Color::Rgb(198, 120, 221); // Vibrant purple
    pub const UNFOCUSED_HINT: Color = Color::Rgb(90, 92, 119);

    // Unfocused query text
    pub const QUERY_UNFOCUSED: Color = Color::Rgb(90, 92, 119);

    pub const CURSOR: Style = Style::new().add_modifier(Modifier::REVERSED);
}

/// Results pane styles
pub mod results {
    use super::*;

    // Border colors
    pub const BORDER_FOCUSED: Color = Color::Rgb(0, 217, 255); // Electric cyan
    pub const BORDER_UNFOCUSED: Color = Color::Rgb(90, 92, 119);
    pub const BORDER_WARNING: Color = Color::Rgb(255, 217, 61);
    pub const BORDER_ERROR: Color = Color::Rgb(224, 108, 117);
    pub const BACKGROUND: Color = Color::Rgb(26, 26, 46);

    // Search mode text colors (in title)
    pub const SEARCH_ACTIVE: Color = Color::Rgb(255, 107, 157); // Hot pink
    pub const SEARCH_INACTIVE: Color = Color::Rgb(90, 92, 119);

    // Query timing indicator colors
    pub const TIMING_NORMAL: Color = Color::Rgb(0, 217, 255);
    pub const TIMING_SLOW: Color = Color::Rgb(255, 217, 61);
    pub const TIMING_VERY_SLOW: Color = Color::Rgb(224, 108, 117);

    // Query state indicators
    pub const RESULT_OK: Color = Color::Rgb(107, 203, 119);
    pub const RESULT_WARNING: Color = Color::Rgb(255, 217, 61);
    pub const RESULT_ERROR: Color = Color::Rgb(224, 108, 117);
    pub const RESULT_PENDING: Color = Color::Rgb(130, 133, 158);

    // Search match highlighting
    pub const MATCH_HIGHLIGHT_BG: Color = Color::Rgb(85, 85, 115);
    pub const MATCH_HIGHLIGHT_FG: Color = Color::Rgb(236, 236, 244);
    pub const CURRENT_MATCH_BG: Color = Color::Rgb(255, 184, 108); // Orange
    pub const CURRENT_MATCH_FG: Color = Color::Rgb(26, 26, 46);

    // Cursor and selection
    pub const CURSOR_LINE_BG: Color = Color::Rgb(45, 45, 72);
    pub const HOVERED_LINE_BG: Color = Color::Rgb(40, 40, 65);
    pub const VISUAL_SELECTION_BG: Color = Color::Rgb(60, 60, 95);
    pub const CURSOR_INDICATOR_FG: Color = Color::Rgb(255, 107, 157);

    // Stale state
    pub const STALE_MODIFIER: Modifier = Modifier::DIM;

    // Hints (bottom of results pane)
    pub const HINT_KEY: Color = Color::Rgb(0, 217, 255);
    pub const HINT_DESCRIPTION: Style = Style::new()
        .fg(Color::Rgb(0, 217, 255))
        .add_modifier(Modifier::DIM);

    // Spinner animation colors (galaxy rainbow)
    pub const SPINNER_COLORS: &[Color] = &[
        Color::Rgb(255, 107, 157), // Pink
        Color::Rgb(255, 184, 108), // Orange
        Color::Rgb(255, 217, 61),  // Yellow
        Color::Rgb(107, 203, 119), // Green
        Color::Rgb(0, 217, 255),   // Cyan
        Color::Rgb(189, 147, 249), // Purple
        Color::Rgb(198, 120, 221), // Magenta
        Color::Rgb(224, 108, 117), // Red
    ];
}

/// Search bar styles
pub mod search {
    use super::*;

    pub const BORDER_ACTIVE: Color = Color::Rgb(255, 107, 157); // Hot pink
    pub const BORDER_INACTIVE: Color = Color::Rgb(90, 92, 119);
    pub const BACKGROUND: Color = Color::Rgb(26, 26, 46);

    // Text colors
    pub const TEXT_ACTIVE: Color = Color::Rgb(236, 236, 244);
    pub const TEXT_INACTIVE: Color = Color::Rgb(90, 92, 119);

    // Match count display
    pub const NO_MATCHES: Color = Color::Rgb(224, 108, 117);
    pub const MATCH_COUNT: Color = Color::Rgb(130, 133, 158);
    pub const MATCH_COUNT_CONFIRMED: Color = Color::Rgb(90, 92, 119);

    // Hints at bottom
    pub const HINTS: Color = Color::Rgb(255, 107, 157);
}

/// Help popup styles
pub mod help {
    use super::*;

    // Border and title
    pub const BORDER: Color = Color::Rgb(0, 217, 255);
    pub const BACKGROUND: Color = Color::Rgb(26, 26, 46);
    pub const SCROLLBAR: Color = Color::Rgb(0, 217, 255);
    pub const TITLE: Style = Style::new()
        .fg(Color::Rgb(0, 217, 255))
        .add_modifier(Modifier::BOLD);

    // Tab bar
    pub const TAB_ACTIVE: Style = Style::new()
        .fg(Color::Rgb(0, 217, 255))
        .add_modifier(Modifier::BOLD);
    pub const TAB_INACTIVE: Style = Style::new()
        .fg(Color::Rgb(0, 217, 255))
        .add_modifier(Modifier::DIM);
    pub const TAB_HOVER_FG: Color = Color::Rgb(0, 217, 255);
    pub const TAB_HOVER_BG: Color = Color::Rgb(35, 35, 58);

    // Content
    pub const SECTION_HEADER: Style = Style::new()
        .fg(Color::Rgb(0, 217, 255))
        .add_modifier(Modifier::BOLD);
    pub const KEY: Style = Style::new()
        .fg(Color::Rgb(255, 217, 61))
        .add_modifier(Modifier::BOLD);
    pub const DESCRIPTION: Color = Color::Rgb(236, 236, 244);

    // Footer
    pub const FOOTER: Color = Color::Rgb(90, 92, 119);
}

/// History popup styles
pub mod history {
    use super::*;

    // Border and scrollbar
    pub const BORDER: Color = Color::Rgb(0, 217, 255);
    pub const SCROLLBAR: Color = Color::Rgb(0, 217, 255);
    pub const BACKGROUND: Color = Color::Rgb(26, 26, 46);

    // Selected item - clear highlight with accent indicator
    pub const ITEM_SELECTED_BG: Color = Color::Rgb(45, 45, 72);
    pub const ITEM_SELECTED_INDICATOR: Color = Color::Rgb(0, 217, 255);

    // Normal items - clean, readable with uniform background
    pub const ITEM_NORMAL_BG: Color = Color::Rgb(26, 26, 46);
    pub const ITEM_NORMAL_FG: Color = Color::Rgb(180, 182, 200);

    // Empty state
    pub const NO_MATCHES: Color = Color::Rgb(90, 92, 119);

    // Search textarea
    pub const SEARCH_TEXT: Color = Color::Rgb(236, 236, 244);
    pub const SEARCH_BG: Color = Color::Rgb(26, 26, 46);
}

/// Snippets popup styles
pub mod snippets {
    use super::*;

    // Border (distinct green color)
    pub const BORDER: Color = Color::Rgb(107, 203, 119);
    pub const SCROLLBAR: Color = Color::Rgb(107, 203, 119);
    pub const BACKGROUND: Color = Color::Rgb(26, 26, 46);

    // List items
    pub const ITEM_NORMAL_FG: Color = Color::Rgb(236, 236, 244);
    pub const ITEM_NORMAL_BG: Color = Color::Rgb(26, 26, 46);
    pub const ITEM_SELECTED_FG: Color = Color::Rgb(26, 26, 46);
    pub const ITEM_SELECTED_BG: Color = Color::Rgb(45, 45, 72);
    pub const ITEM_SELECTED_INDICATOR: Color = Color::Rgb(107, 203, 119);
    pub const ITEM_SELECTED_MODIFIER: Modifier = Modifier::BOLD;
    pub const ITEM_HOVERED_FG: Color = Color::Rgb(236, 236, 244);
    pub const ITEM_HOVERED_BG: Color = Color::Rgb(40, 40, 65);

    // Content
    pub const NAME: Color = Color::Rgb(236, 236, 244);
    pub const DESCRIPTION: Color = Color::Rgb(90, 92, 119);
    pub const QUERY_PREVIEW: Color = Color::Rgb(255, 217, 61);
    pub const CATEGORY: Color = Color::Rgb(107, 203, 119);

    // Edit/Create mode
    pub const FIELD_ACTIVE_BORDER: Color = Color::Rgb(255, 217, 61);
    pub const FIELD_INACTIVE_BORDER: Color = Color::Rgb(107, 203, 119);
    pub const FIELD_TEXT: Color = Color::Rgb(236, 236, 244);
    pub const FIELD_BG: Color = Color::Rgb(26, 26, 46);

    // Delete confirmation
    pub const DELETE_BORDER: Color = Color::Rgb(224, 108, 117);

    // Keyboard hints
    pub const HINT_KEY: Color = Color::Rgb(255, 217, 61);
    pub const HINT_TEXT: Color = Color::Rgb(236, 236, 244);

    // Search
    pub const SEARCH_TEXT: Color = Color::Rgb(236, 236, 244);
    pub const SEARCH_BG: Color = Color::Rgb(26, 26, 46);
}

/// AI assistant styles
pub mod ai {
    use super::*;

    // Border and title
    pub const BORDER: Color = Color::Rgb(0, 217, 255);
    pub const BACKGROUND: Color = Color::Rgb(26, 26, 46);
    pub const SCROLLBAR: Color = Color::Rgb(0, 217, 255);
    pub const TITLE: Style = Style::new()
        .fg(Color::Rgb(0, 217, 255))
        .add_modifier(Modifier::BOLD);

    // Model display in title bar
    pub const MODEL_DISPLAY: Color = Color::Rgb(189, 147, 249); // Purple

    // Selection counter in title
    pub const COUNTER: Color = Color::Rgb(255, 217, 61);

    // Config not set state
    pub const CONFIG_ICON: Color = Color::Rgb(255, 217, 61);
    pub const CONFIG_TITLE: Style = Style::new()
        .fg(Color::Rgb(255, 217, 61))
        .add_modifier(Modifier::BOLD);
    pub const CONFIG_DESC: Color = Color::Rgb(130, 133, 158);
    pub const CONFIG_CODE: Color = Color::Rgb(0, 217, 255);
    pub const CONFIG_LINK: Style = Style::new()
        .fg(Color::Rgb(189, 147, 249))
        .add_modifier(Modifier::UNDERLINED);

    // Thinking state
    pub const THINKING_ICON: Color = Color::Rgb(255, 217, 61);
    pub const THINKING_TEXT: Style = Style::new()
        .fg(Color::Rgb(255, 217, 61))
        .add_modifier(Modifier::ITALIC);

    // Error state
    pub const ERROR_ICON: Color = Color::Rgb(224, 108, 117);
    pub const ERROR_TITLE: Style = Style::new()
        .fg(Color::Rgb(224, 108, 117))
        .add_modifier(Modifier::BOLD);
    pub const ERROR_MESSAGE: Color = Color::Rgb(224, 108, 117);

    // Content text
    pub const QUERY_TEXT: Color = Color::Rgb(0, 217, 255);
    pub const RESULT_TEXT: Color = Color::Rgb(236, 236, 244);
    pub const PREVIOUS_RESPONSE: Color = Color::Rgb(90, 92, 119);

    // Suggestion list
    pub const SUGGESTION_SELECTED_BG: Color = Color::Rgb(55, 55, 85);
    pub const SUGGESTION_HOVERED_BG: Color = Color::Rgb(45, 45, 72);
    pub const SUGGESTION_TEXT_SELECTED: Color = Color::Rgb(26, 26, 46);
    pub const SUGGESTION_TEXT_NORMAL: Color = Color::Rgb(130, 133, 158);
    pub const SUGGESTION_DESC_NORMAL: Color = Color::Rgb(90, 92, 119);
    pub const SUGGESTION_DESC_MUTED: Color = Color::Rgb(130, 133, 158);

    // Suggestion type colors
    pub const SUGGESTION_FIX: Color = Color::Rgb(224, 108, 117);
    pub const SUGGESTION_OPTIMIZE: Color = Color::Rgb(255, 217, 61);
    pub const SUGGESTION_NEXT: Color = Color::Rgb(107, 203, 119);

    // Hints
    pub const HINT: Color = Color::Rgb(90, 92, 119);
}

/// Autocomplete dropdown styles
pub mod autocomplete {
    use super::*;

    // Border and scrollbar
    pub const BORDER: Color = Color::Rgb(0, 217, 255);
    pub const SCROLLBAR: Color = Color::Rgb(0, 217, 255);
    pub const BACKGROUND: Color = Color::Rgb(26, 26, 46);

    // List items
    pub const ITEM_NORMAL_FG: Color = Color::Rgb(236, 236, 244);
    pub const ITEM_NORMAL_BG: Color = Color::Rgb(26, 26, 46);
    pub const ITEM_SELECTED_FG: Color = Color::Rgb(26, 26, 46);
    pub const ITEM_SELECTED_BG: Color = Color::Rgb(0, 217, 255);
    pub const ITEM_SELECTED_MODIFIER: Modifier = Modifier::BOLD;

    // Completion type colors
    pub const TYPE_FUNCTION: Color = Color::Rgb(255, 217, 61);
    pub const TYPE_FIELD: Color = Color::Rgb(0, 217, 255);
    pub const TYPE_OPERATOR: Color = Color::Rgb(198, 120, 221);
    pub const TYPE_PATTERN: Color = Color::Rgb(107, 203, 119);
    pub const TYPE_VARIABLE: Color = Color::Rgb(224, 108, 117);
}

/// Tooltip styles
pub mod tooltip {
    use super::*;

    // Border and title (distinct magenta/purple)
    pub const BORDER: Color = Color::Rgb(198, 120, 221);
    pub const BACKGROUND: Color = Color::Rgb(26, 26, 46);
    pub const TITLE: Style = Style::new()
        .fg(Color::Rgb(198, 120, 221))
        .add_modifier(Modifier::BOLD);

    // Content
    pub const DESCRIPTION: Color = Color::Rgb(236, 236, 244);
    pub const EXAMPLE: Color = Color::Rgb(0, 217, 255);
    pub const EXAMPLE_DESC: Color = Color::Rgb(130, 133, 158);
    pub const TIP: Color = Color::Rgb(255, 217, 61);
    pub const SEPARATOR: Color = Color::Rgb(90, 92, 119);
    pub const DISMISS_HINT: Color = Color::Rgb(90, 92, 119);
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
        fg: Color::Rgb(236, 236, 244),
        bg: Color::Rgb(55, 55, 85),
        border: Color::Rgb(130, 133, 158),
    };

    pub const WARNING: NotificationColors = NotificationColors {
        fg: Color::Rgb(26, 26, 46),
        bg: Color::Rgb(255, 217, 61),
        border: Color::Rgb(255, 217, 61),
    };

    pub const ERROR: NotificationColors = NotificationColors {
        fg: Color::Rgb(236, 236, 244),
        bg: Color::Rgb(224, 108, 117),
        border: Color::Rgb(255, 135, 145),
    };
}

/// Help line (bottom status bar) styles
pub mod help_line {
    use super::*;

    pub const KEY: Color = Color::Rgb(130, 133, 158);
    pub const DESCRIPTION: Color = Color::Rgb(90, 92, 119);
    pub const SEPARATOR: Color = Color::Rgb(90, 92, 119);
}

/// Border hint utilities - for building styled keyboard shortcuts on borders
pub mod border_hints {
    use super::*;
    use ratatui::text::{Line, Span};

    /// Build a single hint with key in full color and description dimmed
    pub fn hint(key: &'static str, desc: &'static str, color: Color) -> Vec<Span<'static>> {
        vec![
            Span::styled(key, Style::new().fg(color)),
            Span::styled(
                format!(" {} ", desc),
                Style::new().fg(color).add_modifier(Modifier::DIM),
            ),
        ]
    }

    /// Build a separator dot in dimmed color
    pub fn separator(color: Color) -> Span<'static> {
        Span::styled("• ", Style::new().fg(color).add_modifier(Modifier::DIM))
    }

    /// Build a line with multiple hints separated by dots
    pub fn build_hints(hints: &[(&'static str, &'static str)], color: Color) -> Line<'static> {
        let mut spans = vec![Span::raw(" ")];
        for (i, (key, desc)) in hints.iter().enumerate() {
            if i > 0 {
                spans.push(separator(color));
            }
            spans.extend(hint(key, desc, color));
        }
        Line::from(spans)
    }
}

/// Scrollbar styles (for components that share scrollbar appearance)
pub mod scrollbar {
    use super::*;

    pub const DEFAULT: Color = Color::Rgb(0, 217, 255);
    pub const TRACK: Color = Color::Rgb(55, 55, 85);
}

/// Syntax highlighting styles (for jq query input)
pub mod syntax {
    use super::*;

    pub const KEYWORD: Color = Color::Rgb(255, 107, 157); // Hot pink keywords
    pub const FUNCTION: Color = Color::Rgb(0, 217, 255); // Electric cyan functions
    pub const STRING: Color = Color::Rgb(107, 203, 119); // Fresh green strings
    pub const NUMBER: Color = Color::Rgb(189, 147, 249); // Purple numbers
    pub const OPERATOR: Color = Color::Rgb(198, 120, 221); // Magenta operators
    pub const VARIABLE: Color = Color::Rgb(255, 184, 108); // Orange variables
    pub const FIELD: Color = Color::Rgb(0, 217, 255); // Cyan fields

    /// Bracket pair matching style (color + bold + underlined)
    /// Applied to matching brackets when cursor is on a bracket
    pub mod bracket_match {
        use super::*;

        pub const COLOR: Color = Color::Rgb(255, 217, 61);
        pub const STYLE: Style = Style::new()
            .fg(Color::Rgb(255, 217, 61))
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED);
    }
}
