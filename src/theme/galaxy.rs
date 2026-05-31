//! Built-in Galaxy themes: the original dark variant and a WCAG-AA light variant.
//!
//! `galaxy_dark` reproduces the historical compile-time palette verbatim, so the
//! dark experience is unchanged. `galaxy_light` is a light-background companion
//! tuned for AA contrast against its pure-white background `Rgb(255, 255, 255)`.

use super::*;
use ratatui::style::{Color, Modifier, Style};

/// The original Galaxy theme - purple/pink accents on deep space blue.
pub fn galaxy_dark() -> Theme {
    Theme {
        palette: PaletteTheme {
            text: Color::Rgb(236, 236, 244),
            text_dim: Color::Rgb(90, 92, 119),
            text_muted: Color::Rgb(130, 133, 158),
            bg_dark: Color::Rgb(26, 26, 46),
            bg_surface: Color::Rgb(35, 35, 58),
            bg_hover: Color::Rgb(45, 45, 72),
            bg_highlight: Color::Rgb(55, 55, 85),
            success: Color::Rgb(107, 203, 119),
            warning: Color::Rgb(255, 217, 61),
            error: Color::Rgb(224, 108, 117),
            info: Color::Rgb(0, 217, 255),
            cyan: Color::Rgb(0, 217, 255),
            yellow: Color::Rgb(255, 217, 61),
            green: Color::Rgb(107, 203, 119),
            magenta: Color::Rgb(198, 120, 221),
            pink: Color::Rgb(255, 107, 157),
            red: Color::Rgb(224, 108, 117),
            orange: Color::Rgb(255, 184, 108),
            purple: Color::Rgb(189, 147, 249),
            cursor: Style::new().add_modifier(Modifier::REVERSED),
        },
        input: InputTheme {
            mode_insert: Color::Rgb(0, 217, 255),
            mode_normal: Color::Rgb(255, 217, 61),
            mode_operator: Color::Rgb(107, 203, 119),
            mode_char_search: Color::Rgb(255, 107, 157),
            border_unfocused: Color::Rgb(90, 92, 119),
            border_error: Color::Rgb(224, 108, 117),
            syntax_error_warning: Color::Rgb(255, 217, 61),
            tooltip_hint: Color::Rgb(198, 120, 221),
            unfocused_hint: Color::Rgb(90, 92, 119),
            query_unfocused: Color::Rgb(90, 92, 119),
            cursor: Style::new().add_modifier(Modifier::REVERSED),
        },
        results: ResultsTheme {
            border_focused: Color::Rgb(0, 217, 255),
            border_unfocused: Color::Rgb(90, 92, 119),
            border_warning: Color::Rgb(255, 217, 61),
            border_error: Color::Rgb(224, 108, 117),
            background: Color::Rgb(26, 26, 46),
            search_active: Color::Rgb(255, 107, 157),
            search_inactive: Color::Rgb(90, 92, 119),
            timing_normal: Color::Rgb(0, 217, 255),
            timing_slow: Color::Rgb(255, 217, 61),
            timing_very_slow: Color::Rgb(224, 108, 117),
            result_ok: Color::Rgb(107, 203, 119),
            result_warning: Color::Rgb(255, 217, 61),
            result_error: Color::Rgb(224, 108, 117),
            result_pending: Color::Rgb(130, 133, 158),
            badge_syntax_error: Style::new()
                .fg(Color::Rgb(35, 30, 10))
                .bg(Color::Rgb(255, 217, 61)),
            badge_empty_result: Style::new()
                .fg(Color::Rgb(20, 25, 40))
                .bg(Color::Rgb(130, 140, 170)),
            badge_back: Style::new()
                .fg(Color::Rgb(15, 20, 35))
                .bg(Color::Rgb(0, 217, 255)),
            badge_back_hover: Style::new()
                .fg(Color::Rgb(15, 20, 35))
                .bg(Color::Rgb(189, 147, 249))
                .add_modifier(Modifier::BOLD),
            match_highlight_bg: Color::Rgb(85, 85, 115),
            match_highlight_fg: Color::Rgb(236, 236, 244),
            current_match_bg: Color::Rgb(255, 184, 108),
            current_match_fg: Color::Rgb(26, 26, 46),
            cursor_line_bg: Color::Rgb(45, 45, 72),
            hovered_line_bg: Color::Rgb(40, 40, 65),
            visual_selection_bg: Color::Rgb(60, 60, 95),
            cursor_indicator_fg: Color::Rgb(255, 107, 157),
            stale_modifier: Modifier::DIM,
            path_at_cursor_separator: Color::Rgb(90, 92, 119),
            path_at_cursor: Color::Rgb(189, 147, 249),
            hint_key: Color::Rgb(0, 217, 255),
            hint_description: Style::new()
                .fg(Color::Rgb(0, 217, 255))
                .add_modifier(Modifier::DIM),
            spinner_colors: vec![
                Color::Rgb(255, 107, 157),
                Color::Rgb(255, 184, 108),
                Color::Rgb(255, 217, 61),
                Color::Rgb(107, 203, 119),
                Color::Rgb(0, 217, 255),
                Color::Rgb(189, 147, 249),
                Color::Rgb(198, 120, 221),
                Color::Rgb(224, 108, 117),
            ],
            jq_colors: [
                Color::Rgb(130, 133, 158), // null - muted gray
                Color::Rgb(224, 108, 117), // false - soft red
                Color::Rgb(107, 203, 119), // true - fresh green
                Color::Rgb(189, 147, 249), // numbers - purple
                Color::Rgb(107, 203, 119), // strings - fresh green
                Color::Rgb(0, 217, 255),   // arrays - electric cyan
                Color::Rgb(0, 217, 255),   // objects - electric cyan
                Color::Rgb(255, 217, 61),  // keys - golden yellow
            ],
        },
        search: SearchTheme {
            border_active: Color::Rgb(255, 107, 157),
            border_inactive: Color::Rgb(90, 92, 119),
            background: Color::Rgb(26, 26, 46),
            text_active: Color::Rgb(236, 236, 244),
            text_inactive: Color::Rgb(90, 92, 119),
            no_matches: Color::Rgb(224, 108, 117),
            match_count: Color::Rgb(130, 133, 158),
            match_count_confirmed: Color::Rgb(90, 92, 119),
            badge_no_matches: Style::new()
                .fg(Color::Rgb(45, 15, 20))
                .bg(Color::Rgb(224, 108, 117)),
            badge_match_count: Style::new()
                .fg(Color::Rgb(35, 15, 30))
                .bg(Color::Rgb(255, 107, 157)),
            badge_match_count_confirmed: Style::new()
                .fg(Color::Rgb(200, 205, 220))
                .bg(Color::Rgb(70, 72, 95)),
            hints: Color::Rgb(255, 107, 157),
        },
        help: HelpTheme {
            border: Color::Rgb(0, 217, 255),
            background: Color::Rgb(26, 26, 46),
            scrollbar: Color::Rgb(0, 217, 255),
            title: Style::new()
                .fg(Color::Rgb(0, 217, 255))
                .add_modifier(Modifier::BOLD),
            tab_active: Style::new()
                .fg(Color::Rgb(0, 217, 255))
                .add_modifier(Modifier::BOLD),
            tab_inactive: Style::new()
                .fg(Color::Rgb(0, 217, 255))
                .add_modifier(Modifier::DIM),
            tab_hover_fg: Color::Rgb(0, 217, 255),
            tab_hover_bg: Color::Rgb(35, 35, 58),
            section_header: Style::new()
                .fg(Color::Rgb(0, 217, 255))
                .add_modifier(Modifier::BOLD),
            key: Style::new()
                .fg(Color::Rgb(255, 217, 61))
                .add_modifier(Modifier::BOLD),
            description: Color::Rgb(236, 236, 244),
            footer: Color::Rgb(90, 92, 119),
        },
        history: HistoryTheme {
            border: Color::Rgb(0, 217, 255),
            scrollbar: Color::Rgb(0, 217, 255),
            background: Color::Rgb(26, 26, 46),
            item_selected_bg: Color::Rgb(45, 45, 72),
            item_selected_indicator: Color::Rgb(0, 217, 255),
            item_normal_bg: Color::Rgb(26, 26, 46),
            item_normal_fg: Color::Rgb(180, 182, 200),
            no_matches: Color::Rgb(90, 92, 119),
            search_text: Color::Rgb(236, 236, 244),
            search_bg: Color::Rgb(26, 26, 46),
            delete_button: Color::Rgb(130, 133, 158),
            delete_button_hover: Color::Rgb(255, 107, 107),
        },
        snippets: SnippetsTheme {
            border: Color::Rgb(107, 203, 119),
            scrollbar: Color::Rgb(107, 203, 119),
            background: Color::Rgb(26, 26, 46),
            item_normal_fg: Color::Rgb(236, 236, 244),
            item_normal_bg: Color::Rgb(26, 26, 46),
            item_selected_fg: Color::Rgb(26, 26, 46),
            item_selected_bg: Color::Rgb(45, 45, 72),
            item_selected_indicator: Color::Rgb(107, 203, 119),
            item_selected_modifier: Modifier::BOLD,
            item_hovered_fg: Color::Rgb(236, 236, 244),
            item_hovered_bg: Color::Rgb(40, 40, 65),
            name: Color::Rgb(236, 236, 244),
            description: Color::Rgb(90, 92, 119),
            query_preview: Color::Rgb(255, 217, 61),
            category: Color::Rgb(107, 203, 119),
            field_active_border: Color::Rgb(255, 217, 61),
            field_inactive_border: Color::Rgb(107, 203, 119),
            field_text: Color::Rgb(236, 236, 244),
            field_bg: Color::Rgb(26, 26, 46),
            delete_border: Color::Rgb(224, 108, 117),
            hint_key: Color::Rgb(255, 217, 61),
            hint_text: Color::Rgb(236, 236, 244),
            search_text: Color::Rgb(236, 236, 244),
            search_bg: Color::Rgb(26, 26, 46),
        },
        save: SaveTheme {
            title: Color::Rgb(255, 184, 108),
            border: Color::Rgb(255, 184, 108),
            input_border: Color::Rgb(255, 217, 61),
            input_fg: Color::Rgb(236, 236, 244),
            input_bg: Color::Rgb(26, 26, 46),
            hint_key: Color::Rgb(255, 217, 61),
            hint_text: Color::Rgb(236, 236, 244),
            preview_ok: Color::Rgb(152, 195, 121),
            preview_warn: Color::Rgb(224, 108, 117),
            error: Color::Rgb(224, 108, 117),
        },
        ai: AiTheme {
            border: Color::Rgb(0, 217, 255),
            background: Color::Rgb(26, 26, 46),
            scrollbar: Color::Rgb(0, 217, 255),
            title: Style::new()
                .fg(Color::Rgb(0, 217, 255))
                .add_modifier(Modifier::BOLD),
            model_display: Color::Rgb(189, 147, 249),
            counter: Color::Rgb(255, 217, 61),
            config_icon: Color::Rgb(255, 217, 61),
            config_title: Style::new()
                .fg(Color::Rgb(255, 217, 61))
                .add_modifier(Modifier::BOLD),
            config_desc: Color::Rgb(130, 133, 158),
            config_code: Color::Rgb(0, 217, 255),
            config_link: Style::new()
                .fg(Color::Rgb(189, 147, 249))
                .add_modifier(Modifier::UNDERLINED),
            thinking_icon: Color::Rgb(255, 217, 61),
            thinking_text: Style::new()
                .fg(Color::Rgb(255, 217, 61))
                .add_modifier(Modifier::ITALIC),
            error_icon: Color::Rgb(224, 108, 117),
            error_title: Style::new()
                .fg(Color::Rgb(224, 108, 117))
                .add_modifier(Modifier::BOLD),
            error_message: Color::Rgb(224, 108, 117),
            empty_icon: Color::Rgb(130, 133, 158),
            empty_title: Style::new()
                .fg(Color::Rgb(0, 217, 255))
                .add_modifier(Modifier::BOLD),
            empty_message: Color::Rgb(130, 133, 158),
            query_text: Color::Rgb(0, 217, 255),
            result_text: Color::Rgb(236, 236, 244),
            previous_response: Color::Rgb(90, 92, 119),
            suggestion_selected_bg: Color::Rgb(55, 55, 85),
            suggestion_hovered_bg: Color::Rgb(45, 45, 72),
            suggestion_text_selected: Color::Rgb(26, 26, 46),
            suggestion_text_normal: Color::Rgb(130, 133, 158),
            suggestion_desc_normal: Color::Rgb(90, 92, 119),
            suggestion_desc_muted: Color::Rgb(130, 133, 158),
            suggestion_fix: Color::Rgb(224, 108, 117),
            suggestion_optimize: Color::Rgb(255, 217, 61),
            suggestion_next: Color::Rgb(107, 203, 119),
            hint: Color::Rgb(90, 92, 119),
        },
        autocomplete: AutocompleteTheme {
            border: Color::Rgb(0, 217, 255),
            scrollbar: Color::Rgb(0, 217, 255),
            background: Color::Rgb(26, 26, 46),
            item_normal_fg: Color::Rgb(236, 236, 244),
            item_normal_bg: Color::Rgb(26, 26, 46),
            item_selected_fg: Color::Rgb(26, 26, 46),
            item_selected_bg: Color::Rgb(0, 217, 255),
            item_selected_modifier: Modifier::BOLD,
            type_function: Color::Rgb(255, 217, 61),
            type_field: Color::Rgb(0, 217, 255),
            type_operator: Color::Rgb(198, 120, 221),
            type_pattern: Color::Rgb(107, 203, 119),
            type_variable: Color::Rgb(224, 108, 117),
            type_value: Color::Rgb(232, 165, 90),
        },
        tooltip: TooltipTheme {
            border: Color::Rgb(198, 120, 221),
            background: Color::Rgb(26, 26, 46),
            title: Style::new()
                .fg(Color::Rgb(198, 120, 221))
                .add_modifier(Modifier::BOLD),
            description: Color::Rgb(236, 236, 244),
            example: Color::Rgb(0, 217, 255),
            example_desc: Color::Rgb(130, 133, 158),
            tip: Color::Rgb(255, 217, 61),
            separator: Color::Rgb(90, 92, 119),
        },
        notification: NotificationTheme {
            info: NotificationColors {
                fg: Color::Rgb(236, 236, 244),
                bg: Color::Rgb(55, 55, 85),
                border: Color::Rgb(130, 133, 158),
            },
            warning: NotificationColors {
                fg: Color::Rgb(26, 26, 46),
                bg: Color::Rgb(255, 217, 61),
                border: Color::Rgb(255, 217, 61),
            },
            error: NotificationColors {
                fg: Color::Rgb(236, 236, 244),
                bg: Color::Rgb(224, 108, 117),
                border: Color::Rgb(255, 135, 145),
            },
        },
        help_line: HelpLineTheme {
            key: Color::Rgb(130, 133, 158),
            description: Color::Rgb(90, 92, 119),
            separator: Color::Rgb(90, 92, 119),
            hint_modifier: Modifier::DIM,
        },
        scrollbar: ScrollbarTheme {
            default: Color::Rgb(0, 217, 255),
            track: Color::Rgb(55, 55, 85),
        },
        syntax: SyntaxTheme {
            keyword: Color::Rgb(255, 107, 157),
            function: Color::Rgb(0, 217, 255),
            string: Color::Rgb(107, 203, 119),
            number: Color::Rgb(189, 147, 249),
            operator: Color::Rgb(198, 120, 221),
            variable: Color::Rgb(255, 184, 108),
            field: Color::Rgb(0, 217, 255),
            bracket_match_color: Color::Rgb(255, 217, 61),
            bracket_match_style: Style::new()
                .fg(Color::Rgb(255, 217, 61))
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        },
    }
}

/// Light Galaxy theme - WCAG-AA tuned against an off-white background.
pub fn galaxy_light() -> Theme {
    Theme {
        palette: PaletteTheme {
            text: Color::Rgb(40, 40, 60),
            text_dim: Color::Rgb(89, 99, 110),
            text_muted: Color::Rgb(96, 99, 128),
            bg_dark: Color::Rgb(255, 255, 255),
            bg_surface: Color::Rgb(234, 232, 243),
            bg_hover: Color::Rgb(228, 224, 242),
            bg_highlight: Color::Rgb(216, 210, 236),
            success: Color::Rgb(26, 127, 55),
            warning: Color::Rgb(180, 69, 0),
            error: Color::Rgb(210, 15, 57),
            info: Color::Rgb(14, 116, 144),
            cyan: Color::Rgb(14, 116, 144),
            yellow: Color::Rgb(146, 98, 0),
            green: Color::Rgb(26, 127, 55),
            magenta: Color::Rgb(162, 28, 175),
            pink: Color::Rgb(209, 33, 109),
            red: Color::Rgb(210, 15, 57),
            orange: Color::Rgb(180, 69, 0),
            purple: Color::Rgb(136, 57, 239),
            cursor: Style::new().add_modifier(Modifier::REVERSED),
        },
        input: InputTheme {
            mode_insert: Color::Rgb(14, 116, 144),
            mode_normal: Color::Rgb(146, 98, 0),
            mode_operator: Color::Rgb(26, 127, 55),
            mode_char_search: Color::Rgb(209, 33, 109),
            border_unfocused: Color::Rgb(118, 119, 140),
            border_error: Color::Rgb(210, 15, 57),
            syntax_error_warning: Color::Rgb(146, 98, 0),
            tooltip_hint: Color::Rgb(162, 28, 175),
            unfocused_hint: Color::Rgb(118, 119, 140),
            query_unfocused: Color::Rgb(118, 119, 140),
            cursor: Style::new().add_modifier(Modifier::REVERSED),
        },
        results: ResultsTheme {
            border_focused: Color::Rgb(14, 116, 144),
            border_unfocused: Color::Rgb(118, 119, 140),
            border_warning: Color::Rgb(180, 69, 0),
            border_error: Color::Rgb(210, 15, 57),
            background: Color::Rgb(255, 255, 255),
            search_active: Color::Rgb(209, 33, 109),
            search_inactive: Color::Rgb(118, 119, 140),
            timing_normal: Color::Rgb(14, 116, 144),
            timing_slow: Color::Rgb(146, 98, 0),
            timing_very_slow: Color::Rgb(210, 15, 57),
            result_ok: Color::Rgb(26, 127, 55),
            result_warning: Color::Rgb(146, 98, 0),
            result_error: Color::Rgb(210, 15, 57),
            result_pending: Color::Rgb(120, 122, 145),
            badge_syntax_error: Style::new()
                .fg(Color::Rgb(60, 45, 0))
                .bg(Color::Rgb(214, 160, 0)),
            badge_empty_result: Style::new()
                .fg(Color::Rgb(245, 247, 252))
                .bg(Color::Rgb(96, 104, 130)),
            badge_back: Style::new()
                .fg(Color::Rgb(245, 250, 255))
                .bg(Color::Rgb(14, 116, 144)),
            badge_back_hover: Style::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(136, 57, 239))
                .add_modifier(Modifier::BOLD),
            match_highlight_bg: Color::Rgb(255, 235, 150),
            match_highlight_fg: Color::Rgb(40, 40, 60),
            current_match_bg: Color::Rgb(255, 196, 120),
            current_match_fg: Color::Rgb(40, 30, 0),
            cursor_line_bg: Color::Rgb(214, 208, 234),
            hovered_line_bg: Color::Rgb(224, 219, 240),
            visual_selection_bg: Color::Rgb(210, 202, 236),
            cursor_indicator_fg: Color::Rgb(209, 33, 109),
            stale_modifier: Modifier::empty(),
            path_at_cursor_separator: Color::Rgb(118, 119, 140),
            path_at_cursor: Color::Rgb(136, 57, 239),
            hint_key: Color::Rgb(14, 116, 144),
            hint_description: Style::new().fg(Color::Rgb(89, 99, 110)),
            spinner_colors: vec![
                Color::Rgb(209, 33, 109),
                Color::Rgb(180, 69, 0),
                Color::Rgb(150, 104, 0),
                Color::Rgb(26, 127, 55),
                Color::Rgb(14, 116, 144),
                Color::Rgb(136, 57, 239),
                Color::Rgb(162, 28, 175),
                Color::Rgb(210, 15, 57),
            ],
            jq_colors: [
                Color::Rgb(96, 99, 128),  // null - muted gray
                Color::Rgb(210, 15, 57),  // false - red
                Color::Rgb(26, 127, 55),  // true - green
                Color::Rgb(136, 57, 239), // numbers - purple
                Color::Rgb(26, 127, 55),  // strings - green
                Color::Rgb(14, 116, 144),  // arrays - cyan
                Color::Rgb(14, 116, 144),  // objects - cyan
                Color::Rgb(146, 98, 0),  // keys - amber
            ],
        },
        search: SearchTheme {
            border_active: Color::Rgb(209, 33, 109),
            border_inactive: Color::Rgb(118, 119, 140),
            background: Color::Rgb(255, 255, 255),
            text_active: Color::Rgb(40, 40, 60),
            text_inactive: Color::Rgb(118, 119, 140),
            no_matches: Color::Rgb(210, 15, 57),
            match_count: Color::Rgb(96, 99, 128),
            match_count_confirmed: Color::Rgb(89, 99, 110),
            badge_no_matches: Style::new()
                .fg(Color::Rgb(255, 238, 240))
                .bg(Color::Rgb(210, 15, 57)),
            badge_match_count: Style::new()
                .fg(Color::Rgb(255, 240, 247))
                .bg(Color::Rgb(209, 33, 109)),
            badge_match_count_confirmed: Style::new()
                .fg(Color::Rgb(60, 62, 85))
                .bg(Color::Rgb(225, 228, 236)),
            hints: Color::Rgb(209, 33, 109),
        },
        help: HelpTheme {
            border: Color::Rgb(14, 116, 144),
            background: Color::Rgb(255, 255, 255),
            scrollbar: Color::Rgb(14, 116, 144),
            title: Style::new()
                .fg(Color::Rgb(14, 116, 144))
                .add_modifier(Modifier::BOLD),
            tab_active: Style::new()
                .fg(Color::Rgb(14, 116, 144))
                .add_modifier(Modifier::BOLD),
            tab_inactive: Style::new().fg(Color::Rgb(89, 99, 110)),
            tab_hover_fg: Color::Rgb(14, 116, 144),
            tab_hover_bg: Color::Rgb(234, 232, 243),
            section_header: Style::new()
                .fg(Color::Rgb(14, 116, 144))
                .add_modifier(Modifier::BOLD),
            key: Style::new()
                .fg(Color::Rgb(146, 98, 0))
                .add_modifier(Modifier::BOLD),
            description: Color::Rgb(40, 40, 60),
            footer: Color::Rgb(89, 99, 110),
        },
        history: HistoryTheme {
            border: Color::Rgb(14, 116, 144),
            scrollbar: Color::Rgb(14, 116, 144),
            background: Color::Rgb(255, 255, 255),
            item_selected_bg: Color::Rgb(224, 220, 240),
            item_selected_indicator: Color::Rgb(14, 116, 144),
            item_normal_bg: Color::Rgb(255, 255, 255),
            item_normal_fg: Color::Rgb(80, 82, 108),
            no_matches: Color::Rgb(118, 119, 140),
            search_text: Color::Rgb(40, 40, 60),
            search_bg: Color::Rgb(255, 255, 255),
            delete_button: Color::Rgb(96, 99, 128),
            delete_button_hover: Color::Rgb(210, 15, 57),
        },
        snippets: SnippetsTheme {
            border: Color::Rgb(26, 127, 55),
            scrollbar: Color::Rgb(26, 127, 55),
            background: Color::Rgb(255, 255, 255),
            item_normal_fg: Color::Rgb(40, 40, 60),
            item_normal_bg: Color::Rgb(255, 255, 255),
            item_selected_fg: Color::Rgb(40, 40, 60),
            item_selected_bg: Color::Rgb(224, 220, 240),
            item_selected_indicator: Color::Rgb(26, 127, 55),
            item_selected_modifier: Modifier::BOLD,
            item_hovered_fg: Color::Rgb(40, 40, 60),
            item_hovered_bg: Color::Rgb(232, 229, 245),
            name: Color::Rgb(40, 40, 60),
            description: Color::Rgb(89, 99, 110),
            query_preview: Color::Rgb(146, 98, 0),
            category: Color::Rgb(26, 127, 55),
            field_active_border: Color::Rgb(146, 98, 0),
            field_inactive_border: Color::Rgb(26, 127, 55),
            field_text: Color::Rgb(40, 40, 60),
            field_bg: Color::Rgb(255, 255, 255),
            delete_border: Color::Rgb(210, 15, 57),
            hint_key: Color::Rgb(146, 98, 0),
            hint_text: Color::Rgb(40, 40, 60),
            search_text: Color::Rgb(40, 40, 60),
            search_bg: Color::Rgb(255, 255, 255),
        },
        save: SaveTheme {
            title: Color::Rgb(180, 69, 0),
            border: Color::Rgb(180, 69, 0),
            input_border: Color::Rgb(146, 98, 0),
            input_fg: Color::Rgb(40, 40, 60),
            input_bg: Color::Rgb(255, 255, 255),
            hint_key: Color::Rgb(146, 98, 0),
            hint_text: Color::Rgb(40, 40, 60),
            preview_ok: Color::Rgb(26, 127, 55),
            preview_warn: Color::Rgb(210, 15, 57),
            error: Color::Rgb(210, 15, 57),
        },
        ai: AiTheme {
            border: Color::Rgb(14, 116, 144),
            background: Color::Rgb(255, 255, 255),
            scrollbar: Color::Rgb(14, 116, 144),
            title: Style::new()
                .fg(Color::Rgb(14, 116, 144))
                .add_modifier(Modifier::BOLD),
            model_display: Color::Rgb(136, 57, 239),
            counter: Color::Rgb(146, 98, 0),
            config_icon: Color::Rgb(146, 98, 0),
            config_title: Style::new()
                .fg(Color::Rgb(146, 98, 0))
                .add_modifier(Modifier::BOLD),
            config_desc: Color::Rgb(96, 99, 128),
            config_code: Color::Rgb(14, 116, 144),
            config_link: Style::new()
                .fg(Color::Rgb(136, 57, 239))
                .add_modifier(Modifier::UNDERLINED),
            thinking_icon: Color::Rgb(146, 98, 0),
            thinking_text: Style::new()
                .fg(Color::Rgb(146, 98, 0))
                .add_modifier(Modifier::ITALIC),
            error_icon: Color::Rgb(210, 15, 57),
            error_title: Style::new()
                .fg(Color::Rgb(210, 15, 57))
                .add_modifier(Modifier::BOLD),
            error_message: Color::Rgb(210, 15, 57),
            empty_icon: Color::Rgb(89, 99, 110),
            empty_title: Style::new()
                .fg(Color::Rgb(14, 116, 144))
                .add_modifier(Modifier::BOLD),
            empty_message: Color::Rgb(89, 99, 110),
            query_text: Color::Rgb(14, 116, 144),
            result_text: Color::Rgb(40, 40, 60),
            previous_response: Color::Rgb(89, 99, 110),
            suggestion_selected_bg: Color::Rgb(216, 210, 236),
            suggestion_hovered_bg: Color::Rgb(224, 220, 240),
            suggestion_text_selected: Color::Rgb(40, 40, 60),
            suggestion_text_normal: Color::Rgb(96, 99, 128),
            suggestion_desc_normal: Color::Rgb(89, 99, 110),
            suggestion_desc_muted: Color::Rgb(96, 99, 128),
            suggestion_fix: Color::Rgb(210, 15, 57),
            suggestion_optimize: Color::Rgb(146, 98, 0),
            suggestion_next: Color::Rgb(26, 127, 55),
            hint: Color::Rgb(89, 99, 110),
        },
        autocomplete: AutocompleteTheme {
            border: Color::Rgb(14, 116, 144),
            scrollbar: Color::Rgb(14, 116, 144),
            background: Color::Rgb(255, 255, 255),
            item_normal_fg: Color::Rgb(40, 40, 60),
            item_normal_bg: Color::Rgb(255, 255, 255),
            item_selected_fg: Color::Rgb(255, 255, 255),
            item_selected_bg: Color::Rgb(14, 116, 144),
            item_selected_modifier: Modifier::BOLD,
            type_function: Color::Rgb(146, 98, 0),
            type_field: Color::Rgb(14, 116, 144),
            type_operator: Color::Rgb(162, 28, 175),
            type_pattern: Color::Rgb(26, 127, 55),
            type_variable: Color::Rgb(210, 15, 57),
            type_value: Color::Rgb(180, 69, 0),
        },
        tooltip: TooltipTheme {
            border: Color::Rgb(162, 28, 175),
            background: Color::Rgb(255, 255, 255),
            title: Style::new()
                .fg(Color::Rgb(162, 28, 175))
                .add_modifier(Modifier::BOLD),
            description: Color::Rgb(40, 40, 60),
            example: Color::Rgb(14, 116, 144),
            example_desc: Color::Rgb(96, 99, 128),
            tip: Color::Rgb(146, 98, 0),
            separator: Color::Rgb(89, 99, 110),
        },
        notification: NotificationTheme {
            info: NotificationColors {
                fg: Color::Rgb(40, 40, 60),
                bg: Color::Rgb(220, 224, 240),
                border: Color::Rgb(89, 99, 110),
            },
            warning: NotificationColors {
                fg: Color::Rgb(40, 30, 0),
                bg: Color::Rgb(255, 213, 120),
                border: Color::Rgb(180, 69, 0),
            },
            error: NotificationColors {
                fg: Color::Rgb(255, 238, 240),
                bg: Color::Rgb(210, 15, 57),
                border: Color::Rgb(150, 36, 44),
            },
        },
        help_line: HelpLineTheme {
            key: Color::Rgb(96, 99, 128),
            description: Color::Rgb(89, 99, 110),
            separator: Color::Rgb(89, 99, 110),
            hint_modifier: Modifier::empty(),
        },
        scrollbar: ScrollbarTheme {
            default: Color::Rgb(14, 116, 144),
            track: Color::Rgb(214, 210, 232),
        },
        syntax: SyntaxTheme {
            keyword: Color::Rgb(209, 33, 109),
            function: Color::Rgb(14, 116, 144),
            string: Color::Rgb(26, 127, 55),
            number: Color::Rgb(136, 57, 239),
            operator: Color::Rgb(162, 28, 175),
            variable: Color::Rgb(180, 69, 0),
            field: Color::Rgb(14, 116, 144),
            bracket_match_color: Color::Rgb(146, 98, 0),
            bracket_match_style: Style::new()
                .fg(Color::Rgb(146, 98, 0))
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        },
    }
}
