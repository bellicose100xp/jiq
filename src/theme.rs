//! Centralized theme configuration for all UI components.
//!
//! All colors and styles are defined here. When adding or modifying UI components:
//! - Add new colors to the appropriate module
//! - Use `theme::module::CONSTANT` in render files
//! - Do NOT hardcode `Color::*` values directly in render files

use ratatui::style::{Color, Modifier, Style};

/// Core color palette - shared base colors.
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
    pub const BG_HOVER: Color = Color::Indexed(236);

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
    pub const UNFOCUSED_HINT: Color = Color::DarkGray;

    // Unfocused query text
    pub const QUERY_UNFOCUSED: Color = Color::DarkGray;

    pub const CURSOR: Style = Style::new().add_modifier(Modifier::REVERSED);
}

/// Results pane styles
pub mod results {
    use super::*;

    // Border colors
    pub const BORDER_FOCUSED: Color = Color::Cyan;
    pub const BORDER_UNFOCUSED: Color = Color::DarkGray;
    pub const BORDER_WARNING: Color = Color::Yellow;
    pub const BORDER_ERROR: Color = Color::Red;
    pub const BACKGROUND: Color = Color::Black;

    // Search mode text colors (in title)
    pub const SEARCH_ACTIVE: Color = Color::LightMagenta;
    pub const SEARCH_INACTIVE: Color = Color::DarkGray;

    // Query timing indicator colors
    pub const TIMING_NORMAL: Color = Color::Cyan;
    pub const TIMING_SLOW: Color = Color::Yellow;
    pub const TIMING_VERY_SLOW: Color = Color::Red;

    // Query state indicators
    pub const RESULT_OK: Color = Color::Green;
    pub const RESULT_WARNING: Color = Color::Yellow;
    pub const RESULT_ERROR: Color = Color::Red;
    pub const RESULT_PENDING: Color = Color::Gray;

    // Search match highlighting
    pub const MATCH_HIGHLIGHT_BG: Color = Color::Rgb(128, 128, 128);
    pub const MATCH_HIGHLIGHT_FG: Color = Color::White;
    pub const CURRENT_MATCH_BG: Color = Color::Rgb(255, 165, 0);
    pub const CURRENT_MATCH_FG: Color = Color::Black;

    // Cursor and selection
    pub const CURSOR_LINE_BG: Color = Color::Rgb(50, 55, 65);
    pub const HOVERED_LINE_BG: Color = Color::Rgb(45, 50, 60);
    pub const VISUAL_SELECTION_BG: Color = Color::Rgb(70, 80, 100);
    pub const CURSOR_INDICATOR_FG: Color = Color::Rgb(255, 85, 85);

    // Stale state
    pub const STALE_MODIFIER: Modifier = Modifier::DIM;

    // Hints (bottom of results pane)
    pub const HINT_KEY: Color = Color::Cyan;
    pub const HINT_DESCRIPTION: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::DIM);

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
    pub const TITLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    // Tab bar
    pub const TAB_ACTIVE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    pub const TAB_INACTIVE: Style = Style::new().fg(Color::DarkGray);
    pub const TAB_HOVER_FG: Color = Color::White;
    pub const TAB_HOVER_BG: Color = Color::Indexed(236);

    // Content
    pub const SECTION_HEADER: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    pub const KEY: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
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
    pub const ITEM_SELECTED_BG: Color = Color::Rgb(50, 70, 90);
    pub const ITEM_SELECTED_INDICATOR: Color = Color::Cyan;
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
    pub const TITLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    // Model display in title bar
    pub const MODEL_DISPLAY: Color = Color::Blue;

    // Selection counter in title
    pub const COUNTER: Color = Color::Yellow;

    // Config not set state
    pub const CONFIG_ICON: Color = Color::Yellow;
    pub const CONFIG_TITLE: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    pub const CONFIG_DESC: Color = Color::Gray;
    pub const CONFIG_CODE: Color = Color::Cyan;
    pub const CONFIG_LINK: Style = Style::new()
        .fg(Color::Blue)
        .add_modifier(Modifier::UNDERLINED);

    // Thinking state
    pub const THINKING_ICON: Color = Color::Yellow;
    pub const THINKING_TEXT: Style = Style::new()
        .fg(Color::Yellow)
        .add_modifier(Modifier::ITALIC);

    // Error state
    pub const ERROR_ICON: Color = Color::Red;
    pub const ERROR_TITLE: Style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);
    pub const ERROR_MESSAGE: Color = Color::Red;

    // Content text
    pub const QUERY_TEXT: Color = Color::Cyan;
    pub const RESULT_TEXT: Color = Color::White;
    pub const PREVIOUS_RESPONSE: Color = Color::DarkGray;

    // Suggestion list
    pub const SUGGESTION_SELECTED_BG: Color = Color::DarkGray;
    pub const SUGGESTION_HOVERED_BG: Color = Color::Indexed(236);
    pub const SUGGESTION_TEXT_SELECTED: Color = Color::Black;
    pub const SUGGESTION_TEXT_NORMAL: Color = Color::DarkGray;
    pub const SUGGESTION_DESC_NORMAL: Color = Color::DarkGray;
    pub const SUGGESTION_DESC_MUTED: Color = Color::Gray;

    // Suggestion type colors
    pub const SUGGESTION_FIX: Color = Color::Red;
    pub const SUGGESTION_OPTIMIZE: Color = Color::Yellow;
    pub const SUGGESTION_NEXT: Color = Color::Green;

    // Hints
    pub const HINT: Color = Color::DarkGray;
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
    pub const TITLE: Style = Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD);

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

    pub const KEY: Color = Color::Gray;
    pub const DESCRIPTION: Color = Color::DarkGray;
    pub const SEPARATOR: Color = Color::DarkGray;
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
