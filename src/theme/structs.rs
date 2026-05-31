//! Theme data structures: the [`Theme`] aggregate, its per-module
//! sub-structs, the shared [`NotificationColors`] triplet, and the
//! [`ResolvedTheme`] selector enum. Constructors live in `galaxy.rs`;
//! runtime accessors live in `theme.rs`.

use ratatui::style::{Color, Modifier, Style};

/// The two built-in themes, resolved from config or terminal background.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedTheme {
    Light,
    Dark,
}

/// Notification color triplet, shared by info/warning/error variants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NotificationColors {
    pub fg: Color,
    pub bg: Color,
    pub border: Color,
}

/// Core color palette - shared base colors.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteTheme {
    pub text: Color,
    pub text_dim: Color,
    pub text_muted: Color,
    pub bg_dark: Color,
    pub bg_surface: Color,
    pub bg_hover: Color,
    pub bg_highlight: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub cyan: Color,
    pub yellow: Color,
    pub green: Color,
    pub magenta: Color,
    pub pink: Color,
    pub red: Color,
    pub orange: Color,
    pub purple: Color,
    pub cursor: Style,
}

/// Input field styles.
#[derive(Debug, Clone, PartialEq)]
pub struct InputTheme {
    pub mode_insert: Color,
    pub mode_normal: Color,
    pub mode_operator: Color,
    pub mode_char_search: Color,
    pub border_unfocused: Color,
    pub border_error: Color,
    pub syntax_error_warning: Color,
    pub tooltip_hint: Color,
    pub unfocused_hint: Color,
    pub query_unfocused: Color,
    pub cursor: Style,
}

/// Results pane styles.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultsTheme {
    pub border_focused: Color,
    pub border_unfocused: Color,
    pub border_warning: Color,
    pub border_error: Color,
    pub background: Color,
    pub search_active: Color,
    pub search_inactive: Color,
    pub timing_normal: Color,
    pub timing_slow: Color,
    pub timing_very_slow: Color,
    pub result_ok: Color,
    pub result_warning: Color,
    pub result_error: Color,
    pub result_pending: Color,
    pub error_summary: Color,
    pub error_hint_label: Color,
    pub error_hint_text: Color,
    pub error_location: Color,
    pub badge_syntax_error: Style,
    pub badge_empty_result: Style,
    pub badge_back: Style,
    pub badge_back_hover: Style,
    pub match_highlight_bg: Color,
    pub match_highlight_fg: Color,
    pub current_match_bg: Color,
    pub current_match_fg: Color,
    pub cursor_line_bg: Color,
    pub hovered_line_bg: Color,
    pub visual_selection_bg: Color,
    pub cursor_indicator_fg: Color,
    pub stale_modifier: Modifier,
    pub path_at_cursor_separator: Color,
    pub path_at_cursor: Color,
    pub hint_key: Color,
    pub hint_description: Style,
    pub spinner_colors: Vec<Color>,
    /// Colors for jq's --color-output, in jq's order:
    /// null, false, true, numbers, strings, arrays, objects, keys.
    /// arrays/objects/keys are rendered bold by the executor.
    pub jq_colors: [Color; 8],
}

/// Search bar styles.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchTheme {
    pub border_active: Color,
    pub border_inactive: Color,
    pub background: Color,
    pub text_active: Color,
    pub text_inactive: Color,
    pub no_matches: Color,
    pub match_count: Color,
    pub match_count_confirmed: Color,
    pub badge_no_matches: Style,
    pub badge_match_count: Style,
    pub badge_match_count_confirmed: Style,
    pub hints: Color,
}

/// Help popup styles.
#[derive(Debug, Clone, PartialEq)]
pub struct HelpTheme {
    pub border: Color,
    pub background: Color,
    pub scrollbar: Color,
    pub title: Style,
    pub tab_active: Style,
    pub tab_inactive: Style,
    pub tab_hover_fg: Color,
    pub tab_hover_bg: Color,
    pub section_header: Style,
    pub key: Style,
    pub description: Color,
    pub footer: Color,
}

/// History popup styles.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryTheme {
    pub border: Color,
    pub scrollbar: Color,
    pub background: Color,
    pub item_selected_bg: Color,
    pub item_selected_indicator: Color,
    pub item_normal_bg: Color,
    pub item_normal_fg: Color,
    pub no_matches: Color,
    pub search_text: Color,
    pub search_bg: Color,
    pub delete_button: Color,
    pub delete_button_hover: Color,
}

/// Snippets popup styles.
#[derive(Debug, Clone, PartialEq)]
pub struct SnippetsTheme {
    pub border: Color,
    pub scrollbar: Color,
    pub background: Color,
    pub item_normal_fg: Color,
    pub item_normal_bg: Color,
    pub item_selected_fg: Color,
    pub item_selected_bg: Color,
    pub item_selected_indicator: Color,
    pub item_selected_modifier: Modifier,
    pub item_hovered_fg: Color,
    pub item_hovered_bg: Color,
    pub name: Color,
    pub description: Color,
    pub query_preview: Color,
    pub category: Color,
    pub field_active_border: Color,
    pub field_inactive_border: Color,
    pub field_text: Color,
    pub field_bg: Color,
    pub delete_border: Color,
    pub hint_key: Color,
    pub hint_text: Color,
    pub search_text: Color,
    pub search_bg: Color,
}

/// Save-to-file popup styles.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveTheme {
    pub title: Color,
    pub border: Color,
    pub input_border: Color,
    pub input_fg: Color,
    pub input_bg: Color,
    pub hint_key: Color,
    pub hint_text: Color,
    pub preview_ok: Color,
    pub preview_warn: Color,
    pub error: Color,
}

/// AI assistant styles.
#[derive(Debug, Clone, PartialEq)]
pub struct AiTheme {
    pub border: Color,
    pub background: Color,
    pub scrollbar: Color,
    pub title: Style,
    pub model_display: Color,
    pub counter: Color,
    pub config_icon: Color,
    pub config_title: Style,
    pub config_desc: Color,
    pub config_code: Color,
    pub config_link: Style,
    pub thinking_icon: Color,
    pub thinking_text: Style,
    pub error_icon: Color,
    pub error_title: Style,
    pub error_message: Color,
    pub empty_icon: Color,
    pub empty_title: Style,
    pub empty_message: Color,
    pub query_text: Color,
    pub result_text: Color,
    pub previous_response: Color,
    pub suggestion_selected_bg: Color,
    pub suggestion_hovered_bg: Color,
    pub suggestion_text_selected: Color,
    pub suggestion_text_normal: Color,
    pub suggestion_desc_normal: Color,
    pub suggestion_desc_muted: Color,
    pub suggestion_fix: Color,
    pub suggestion_optimize: Color,
    pub suggestion_next: Color,
    pub hint: Color,
}

/// Autocomplete dropdown styles.
#[derive(Debug, Clone, PartialEq)]
pub struct AutocompleteTheme {
    pub border: Color,
    pub scrollbar: Color,
    pub background: Color,
    pub item_normal_fg: Color,
    pub item_normal_bg: Color,
    pub item_selected_fg: Color,
    pub item_selected_bg: Color,
    pub item_selected_modifier: Modifier,
    pub type_function: Color,
    pub type_field: Color,
    pub type_operator: Color,
    pub type_pattern: Color,
    pub type_variable: Color,
    pub type_value: Color,
}

/// Tooltip styles.
#[derive(Debug, Clone, PartialEq)]
pub struct TooltipTheme {
    pub border: Color,
    pub background: Color,
    pub title: Style,
    pub description: Color,
    pub example: Color,
    pub example_desc: Color,
    pub tip: Color,
    pub separator: Color,
}

/// Notification styles.
#[derive(Debug, Clone, PartialEq)]
pub struct NotificationTheme {
    pub info: NotificationColors,
    pub warning: NotificationColors,
    pub error: NotificationColors,
}

/// Help line (bottom status bar) styles.
#[derive(Debug, Clone, PartialEq)]
pub struct HelpLineTheme {
    pub key: Color,
    pub description: Color,
    pub separator: Color,
    /// Modifier applied to border-hint descriptions and separators. `DIM`
    /// de-emphasizes on dark backgrounds; on light backgrounds DIM blends
    /// toward white and bleaches, so light themes use `Modifier::empty()`.
    pub hint_modifier: Modifier,
}

/// Scrollbar styles (for components that share scrollbar appearance).
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollbarTheme {
    pub default: Color,
    pub track: Color,
}

/// Syntax highlighting styles (for jq query input).
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTheme {
    pub keyword: Color,
    pub function: Color,
    pub string: Color,
    pub number: Color,
    pub operator: Color,
    pub variable: Color,
    pub field: Color,
    pub bracket_match_color: Color,
    pub bracket_match_style: Style,
}

/// Full runtime theme - one sub-struct per UI module.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub palette: PaletteTheme,
    pub input: InputTheme,
    pub results: ResultsTheme,
    pub search: SearchTheme,
    pub help: HelpTheme,
    pub history: HistoryTheme,
    pub snippets: SnippetsTheme,
    pub save: SaveTheme,
    pub ai: AiTheme,
    pub autocomplete: AutocompleteTheme,
    pub tooltip: TooltipTheme,
    pub notification: NotificationTheme,
    pub help_line: HelpLineTheme,
    pub scrollbar: ScrollbarTheme,
    pub syntax: SyntaxTheme,
}
