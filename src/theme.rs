//! Centralized theme configuration for all UI components.
//!
//! Colors and styles are resolved at runtime from a single [`Theme`] held in a
//! global [`OnceLock`]. Call sites use lowercase accessor functions
//! (`theme::module::name()`); the active theme is chosen once at startup via
//! [`init`]. When the theme is read before [`init`] runs, it falls back to
//! [`galaxy_dark`] so tests stay deterministic.
//!
//! When adding or modifying UI components:
//! - Add new fields to the appropriate sub-struct and accessor module
//! - Use `theme::module::name()` in render files
//! - Do NOT hardcode `Color::*` values directly in render files

use ratatui::style::{Color, Modifier, Style};
use std::sync::OnceLock;

pub mod detect;

mod galaxy;
pub use galaxy::{galaxy_dark, galaxy_light};

mod structs;
pub use structs::*;

static THEME: OnceLock<Theme> = OnceLock::new();

/// Install the active theme. The first call wins; later calls are ignored.
pub fn init(t: Theme) {
    let _ = THEME.set(t);
}

fn theme() -> &'static Theme {
    THEME.get_or_init(galaxy_dark)
}

/// Core color palette - shared base colors.
pub mod palette {
    use super::*;

    pub fn text() -> Color {
        super::theme().palette.text
    }
    pub fn text_dim() -> Color {
        super::theme().palette.text_dim
    }
    pub fn text_muted() -> Color {
        super::theme().palette.text_muted
    }
    pub fn bg_dark() -> Color {
        super::theme().palette.bg_dark
    }
    pub fn bg_surface() -> Color {
        super::theme().palette.bg_surface
    }
    pub fn bg_hover() -> Color {
        super::theme().palette.bg_hover
    }
    pub fn bg_highlight() -> Color {
        super::theme().palette.bg_highlight
    }
    pub fn success() -> Color {
        super::theme().palette.success
    }
    pub fn warning() -> Color {
        super::theme().palette.warning
    }
    pub fn error() -> Color {
        super::theme().palette.error
    }
    pub fn info() -> Color {
        super::theme().palette.info
    }
    pub fn cyan() -> Color {
        super::theme().palette.cyan
    }
    pub fn yellow() -> Color {
        super::theme().palette.yellow
    }
    pub fn green() -> Color {
        super::theme().palette.green
    }
    pub fn magenta() -> Color {
        super::theme().palette.magenta
    }
    pub fn pink() -> Color {
        super::theme().palette.pink
    }
    pub fn red() -> Color {
        super::theme().palette.red
    }
    pub fn orange() -> Color {
        super::theme().palette.orange
    }
    pub fn purple() -> Color {
        super::theme().palette.purple
    }
    pub fn cursor() -> Style {
        super::theme().palette.cursor
    }
}

/// Input field styles.
pub mod input {
    use super::*;

    pub fn mode_insert() -> Color {
        super::theme().input.mode_insert
    }
    pub fn mode_normal() -> Color {
        super::theme().input.mode_normal
    }
    pub fn mode_operator() -> Color {
        super::theme().input.mode_operator
    }
    pub fn mode_char_search() -> Color {
        super::theme().input.mode_char_search
    }
    pub fn border_unfocused() -> Color {
        super::theme().input.border_unfocused
    }
    pub fn border_error() -> Color {
        super::theme().input.border_error
    }
    pub fn syntax_error_warning() -> Color {
        super::theme().input.syntax_error_warning
    }
    pub fn tooltip_hint() -> Color {
        super::theme().input.tooltip_hint
    }
    pub fn unfocused_hint() -> Color {
        super::theme().input.unfocused_hint
    }
    pub fn query_unfocused() -> Color {
        super::theme().input.query_unfocused
    }
    pub fn cursor() -> Style {
        super::theme().input.cursor
    }
}

/// Results pane styles.
pub mod results {
    use super::*;

    pub fn border_focused() -> Color {
        super::theme().results.border_focused
    }
    pub fn border_unfocused() -> Color {
        super::theme().results.border_unfocused
    }
    pub fn border_warning() -> Color {
        super::theme().results.border_warning
    }
    pub fn border_error() -> Color {
        super::theme().results.border_error
    }
    pub fn background() -> Color {
        super::theme().results.background
    }
    pub fn search_active() -> Color {
        super::theme().results.search_active
    }
    pub fn search_inactive() -> Color {
        super::theme().results.search_inactive
    }
    pub fn timing_normal() -> Color {
        super::theme().results.timing_normal
    }
    pub fn timing_slow() -> Color {
        super::theme().results.timing_slow
    }
    pub fn timing_very_slow() -> Color {
        super::theme().results.timing_very_slow
    }
    pub fn result_ok() -> Color {
        super::theme().results.result_ok
    }
    pub fn result_warning() -> Color {
        super::theme().results.result_warning
    }
    pub fn result_error() -> Color {
        super::theme().results.result_error
    }
    pub fn result_pending() -> Color {
        super::theme().results.result_pending
    }
    pub fn badge_syntax_error() -> Style {
        super::theme().results.badge_syntax_error
    }
    pub fn badge_empty_result() -> Style {
        super::theme().results.badge_empty_result
    }
    pub fn badge_back() -> Style {
        super::theme().results.badge_back
    }
    pub fn badge_back_hover() -> Style {
        super::theme().results.badge_back_hover
    }
    pub fn match_highlight_bg() -> Color {
        super::theme().results.match_highlight_bg
    }
    pub fn match_highlight_fg() -> Color {
        super::theme().results.match_highlight_fg
    }
    pub fn current_match_bg() -> Color {
        super::theme().results.current_match_bg
    }
    pub fn current_match_fg() -> Color {
        super::theme().results.current_match_fg
    }
    pub fn cursor_line_bg() -> Color {
        super::theme().results.cursor_line_bg
    }
    pub fn hovered_line_bg() -> Color {
        super::theme().results.hovered_line_bg
    }
    pub fn visual_selection_bg() -> Color {
        super::theme().results.visual_selection_bg
    }
    pub fn cursor_indicator_fg() -> Color {
        super::theme().results.cursor_indicator_fg
    }
    pub fn stale_modifier() -> Modifier {
        super::theme().results.stale_modifier
    }
    pub fn path_at_cursor_separator() -> Color {
        super::theme().results.path_at_cursor_separator
    }
    pub fn path_at_cursor() -> Color {
        super::theme().results.path_at_cursor
    }
    pub fn hint_key() -> Color {
        super::theme().results.hint_key
    }
    pub fn hint_description() -> Style {
        super::theme().results.hint_description
    }
    pub fn spinner_colors() -> &'static [Color] {
        &super::theme().results.spinner_colors
    }
    pub fn jq_colors() -> [Color; 8] {
        super::theme().results.jq_colors
    }
}

/// Search bar styles.
pub mod search {
    use super::*;

    pub fn border_active() -> Color {
        super::theme().search.border_active
    }
    pub fn border_inactive() -> Color {
        super::theme().search.border_inactive
    }
    pub fn background() -> Color {
        super::theme().search.background
    }
    pub fn text_active() -> Color {
        super::theme().search.text_active
    }
    pub fn text_inactive() -> Color {
        super::theme().search.text_inactive
    }
    pub fn no_matches() -> Color {
        super::theme().search.no_matches
    }
    pub fn match_count() -> Color {
        super::theme().search.match_count
    }
    pub fn match_count_confirmed() -> Color {
        super::theme().search.match_count_confirmed
    }
    pub fn badge_no_matches() -> Style {
        super::theme().search.badge_no_matches
    }
    pub fn badge_match_count() -> Style {
        super::theme().search.badge_match_count
    }
    pub fn badge_match_count_confirmed() -> Style {
        super::theme().search.badge_match_count_confirmed
    }
    pub fn hints() -> Color {
        super::theme().search.hints
    }
}

/// Help popup styles.
pub mod help {
    use super::*;

    pub fn border() -> Color {
        super::theme().help.border
    }
    pub fn background() -> Color {
        super::theme().help.background
    }
    pub fn scrollbar() -> Color {
        super::theme().help.scrollbar
    }
    pub fn title() -> Style {
        super::theme().help.title
    }
    pub fn tab_active() -> Style {
        super::theme().help.tab_active
    }
    pub fn tab_inactive() -> Style {
        super::theme().help.tab_inactive
    }
    pub fn tab_hover_fg() -> Color {
        super::theme().help.tab_hover_fg
    }
    pub fn tab_hover_bg() -> Color {
        super::theme().help.tab_hover_bg
    }
    pub fn section_header() -> Style {
        super::theme().help.section_header
    }
    pub fn key() -> Style {
        super::theme().help.key
    }
    pub fn description() -> Color {
        super::theme().help.description
    }
    pub fn footer() -> Color {
        super::theme().help.footer
    }
}

/// History popup styles.
pub mod history {
    use super::*;

    pub fn border() -> Color {
        super::theme().history.border
    }
    pub fn scrollbar() -> Color {
        super::theme().history.scrollbar
    }
    pub fn background() -> Color {
        super::theme().history.background
    }
    pub fn item_selected_bg() -> Color {
        super::theme().history.item_selected_bg
    }
    pub fn item_selected_indicator() -> Color {
        super::theme().history.item_selected_indicator
    }
    pub fn item_normal_bg() -> Color {
        super::theme().history.item_normal_bg
    }
    pub fn item_normal_fg() -> Color {
        super::theme().history.item_normal_fg
    }
    pub fn no_matches() -> Color {
        super::theme().history.no_matches
    }
    pub fn search_text() -> Color {
        super::theme().history.search_text
    }
    pub fn search_bg() -> Color {
        super::theme().history.search_bg
    }
    pub fn delete_button() -> Color {
        super::theme().history.delete_button
    }
    pub fn delete_button_hover() -> Color {
        super::theme().history.delete_button_hover
    }
}

/// Snippets popup styles.
pub mod snippets {
    use super::*;

    pub fn border() -> Color {
        super::theme().snippets.border
    }
    pub fn scrollbar() -> Color {
        super::theme().snippets.scrollbar
    }
    pub fn background() -> Color {
        super::theme().snippets.background
    }
    pub fn item_normal_fg() -> Color {
        super::theme().snippets.item_normal_fg
    }
    pub fn item_normal_bg() -> Color {
        super::theme().snippets.item_normal_bg
    }
    pub fn item_selected_fg() -> Color {
        super::theme().snippets.item_selected_fg
    }
    pub fn item_selected_bg() -> Color {
        super::theme().snippets.item_selected_bg
    }
    pub fn item_selected_indicator() -> Color {
        super::theme().snippets.item_selected_indicator
    }
    pub fn item_selected_modifier() -> Modifier {
        super::theme().snippets.item_selected_modifier
    }
    pub fn item_hovered_fg() -> Color {
        super::theme().snippets.item_hovered_fg
    }
    pub fn item_hovered_bg() -> Color {
        super::theme().snippets.item_hovered_bg
    }
    pub fn name() -> Color {
        super::theme().snippets.name
    }
    pub fn description() -> Color {
        super::theme().snippets.description
    }
    pub fn query_preview() -> Color {
        super::theme().snippets.query_preview
    }
    pub fn category() -> Color {
        super::theme().snippets.category
    }
    pub fn field_active_border() -> Color {
        super::theme().snippets.field_active_border
    }
    pub fn field_inactive_border() -> Color {
        super::theme().snippets.field_inactive_border
    }
    pub fn field_text() -> Color {
        super::theme().snippets.field_text
    }
    pub fn field_bg() -> Color {
        super::theme().snippets.field_bg
    }
    pub fn delete_border() -> Color {
        super::theme().snippets.delete_border
    }
    pub fn hint_key() -> Color {
        super::theme().snippets.hint_key
    }
    pub fn hint_text() -> Color {
        super::theme().snippets.hint_text
    }
    pub fn search_text() -> Color {
        super::theme().snippets.search_text
    }
    pub fn search_bg() -> Color {
        super::theme().snippets.search_bg
    }
}

/// Save-to-file popup styles.
pub mod save {
    use super::*;

    pub fn title() -> Color {
        super::theme().save.title
    }
    pub fn border() -> Color {
        super::theme().save.border
    }
    pub fn input_border() -> Color {
        super::theme().save.input_border
    }
    pub fn input_fg() -> Color {
        super::theme().save.input_fg
    }
    pub fn input_bg() -> Color {
        super::theme().save.input_bg
    }
    pub fn hint_key() -> Color {
        super::theme().save.hint_key
    }
    pub fn hint_text() -> Color {
        super::theme().save.hint_text
    }
    pub fn preview_ok() -> Color {
        super::theme().save.preview_ok
    }
    pub fn preview_warn() -> Color {
        super::theme().save.preview_warn
    }
    pub fn error() -> Color {
        super::theme().save.error
    }
}

/// AI assistant styles.
pub mod ai {
    use super::*;

    pub fn border() -> Color {
        super::theme().ai.border
    }
    pub fn background() -> Color {
        super::theme().ai.background
    }
    pub fn scrollbar() -> Color {
        super::theme().ai.scrollbar
    }
    pub fn title() -> Style {
        super::theme().ai.title
    }
    pub fn model_display() -> Color {
        super::theme().ai.model_display
    }
    pub fn counter() -> Color {
        super::theme().ai.counter
    }
    pub fn config_icon() -> Color {
        super::theme().ai.config_icon
    }
    pub fn config_title() -> Style {
        super::theme().ai.config_title
    }
    pub fn config_desc() -> Color {
        super::theme().ai.config_desc
    }
    pub fn config_code() -> Color {
        super::theme().ai.config_code
    }
    pub fn config_link() -> Style {
        super::theme().ai.config_link
    }
    pub fn thinking_icon() -> Color {
        super::theme().ai.thinking_icon
    }
    pub fn thinking_text() -> Style {
        super::theme().ai.thinking_text
    }
    pub fn error_icon() -> Color {
        super::theme().ai.error_icon
    }
    pub fn error_title() -> Style {
        super::theme().ai.error_title
    }
    pub fn error_message() -> Color {
        super::theme().ai.error_message
    }
    pub fn query_text() -> Color {
        super::theme().ai.query_text
    }
    pub fn result_text() -> Color {
        super::theme().ai.result_text
    }
    pub fn previous_response() -> Color {
        super::theme().ai.previous_response
    }
    pub fn suggestion_selected_bg() -> Color {
        super::theme().ai.suggestion_selected_bg
    }
    pub fn suggestion_hovered_bg() -> Color {
        super::theme().ai.suggestion_hovered_bg
    }
    pub fn suggestion_text_selected() -> Color {
        super::theme().ai.suggestion_text_selected
    }
    pub fn suggestion_text_normal() -> Color {
        super::theme().ai.suggestion_text_normal
    }
    pub fn suggestion_desc_normal() -> Color {
        super::theme().ai.suggestion_desc_normal
    }
    pub fn suggestion_desc_muted() -> Color {
        super::theme().ai.suggestion_desc_muted
    }
    pub fn suggestion_fix() -> Color {
        super::theme().ai.suggestion_fix
    }
    pub fn suggestion_optimize() -> Color {
        super::theme().ai.suggestion_optimize
    }
    pub fn suggestion_next() -> Color {
        super::theme().ai.suggestion_next
    }
    pub fn hint() -> Color {
        super::theme().ai.hint
    }
}

/// Autocomplete dropdown styles.
pub mod autocomplete {
    use super::*;

    pub fn border() -> Color {
        super::theme().autocomplete.border
    }
    pub fn scrollbar() -> Color {
        super::theme().autocomplete.scrollbar
    }
    pub fn background() -> Color {
        super::theme().autocomplete.background
    }
    pub fn item_normal_fg() -> Color {
        super::theme().autocomplete.item_normal_fg
    }
    pub fn item_normal_bg() -> Color {
        super::theme().autocomplete.item_normal_bg
    }
    pub fn item_selected_fg() -> Color {
        super::theme().autocomplete.item_selected_fg
    }
    pub fn item_selected_bg() -> Color {
        super::theme().autocomplete.item_selected_bg
    }
    pub fn item_selected_modifier() -> Modifier {
        super::theme().autocomplete.item_selected_modifier
    }
    pub fn type_function() -> Color {
        super::theme().autocomplete.type_function
    }
    pub fn type_field() -> Color {
        super::theme().autocomplete.type_field
    }
    pub fn type_operator() -> Color {
        super::theme().autocomplete.type_operator
    }
    pub fn type_pattern() -> Color {
        super::theme().autocomplete.type_pattern
    }
    pub fn type_variable() -> Color {
        super::theme().autocomplete.type_variable
    }
    pub fn type_value() -> Color {
        super::theme().autocomplete.type_value
    }
}

/// Tooltip styles.
pub mod tooltip {
    use super::*;

    pub fn border() -> Color {
        super::theme().tooltip.border
    }
    pub fn background() -> Color {
        super::theme().tooltip.background
    }
    pub fn title() -> Style {
        super::theme().tooltip.title
    }
    pub fn description() -> Color {
        super::theme().tooltip.description
    }
    pub fn example() -> Color {
        super::theme().tooltip.example
    }
    pub fn example_desc() -> Color {
        super::theme().tooltip.example_desc
    }
    pub fn tip() -> Color {
        super::theme().tooltip.tip
    }
    pub fn separator() -> Color {
        super::theme().tooltip.separator
    }
}

/// Notification styles.
pub mod notification {
    pub use super::NotificationColors;

    pub fn info() -> NotificationColors {
        super::theme().notification.info
    }
    pub fn warning() -> NotificationColors {
        super::theme().notification.warning
    }
    pub fn error() -> NotificationColors {
        super::theme().notification.error
    }
}

/// Help line (bottom status bar) styles.
pub mod help_line {
    use super::*;

    pub fn key() -> Color {
        super::theme().help_line.key
    }
    pub fn description() -> Color {
        super::theme().help_line.description
    }
    pub fn separator() -> Color {
        super::theme().help_line.separator
    }
}

/// Border hint utilities - for building styled keyboard shortcuts on borders.
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

/// Scrollbar styles (for components that share scrollbar appearance).
pub mod scrollbar {
    use super::*;

    pub fn default() -> Color {
        super::theme().scrollbar.default
    }
    pub fn track() -> Color {
        super::theme().scrollbar.track
    }
}

/// Syntax highlighting styles (for jq query input).
pub mod syntax {
    use super::*;

    pub fn keyword() -> Color {
        super::theme().syntax.keyword
    }
    pub fn function() -> Color {
        super::theme().syntax.function
    }
    pub fn string() -> Color {
        super::theme().syntax.string
    }
    pub fn number() -> Color {
        super::theme().syntax.number
    }
    pub fn operator() -> Color {
        super::theme().syntax.operator
    }
    pub fn variable() -> Color {
        super::theme().syntax.variable
    }
    pub fn field() -> Color {
        super::theme().syntax.field
    }

    /// Bracket pair matching style (color + bold + underlined).
    /// Applied to matching brackets when cursor is on a bracket.
    pub mod bracket_match {
        use super::*;

        pub fn color() -> Color {
            super::super::theme().syntax.bracket_match_color
        }
        pub fn style() -> Style {
            super::super::theme().syntax.bracket_match_style
        }
    }
}
