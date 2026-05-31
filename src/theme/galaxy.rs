//! Built-in Galaxy themes: the original dark variant and a WCAG-AA light variant.
//!
//! `galaxy_dark` reproduces the historical compile-time palette verbatim, so the
//! dark experience is unchanged. `galaxy_light` is a light-background companion
//! tuned for AA contrast against `Rgb(244, 243, 250)`.

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
            text_dim: Color::Rgb(139, 142, 168),
            text_muted: Color::Rgb(96, 99, 128),
            bg_dark: Color::Rgb(244, 243, 250),
            bg_surface: Color::Rgb(234, 232, 243),
            bg_hover: Color::Rgb(228, 224, 242),
            bg_highlight: Color::Rgb(216, 210, 236),
            success: Color::Rgb(22, 163, 74),
            warning: Color::Rgb(217, 119, 6),
            error: Color::Rgb(220, 38, 38),
            info: Color::Rgb(8, 160, 205),
            cyan: Color::Rgb(8, 160, 205),
            yellow: Color::Rgb(202, 138, 4),
            green: Color::Rgb(22, 163, 74),
            magenta: Color::Rgb(171, 50, 207),
            pink: Color::Rgb(229, 35, 110),
            red: Color::Rgb(220, 38, 38),
            orange: Color::Rgb(234, 88, 12),
            purple: Color::Rgb(124, 58, 237),
            cursor: Style::new().add_modifier(Modifier::REVERSED),
        },
        input: InputTheme {
            mode_insert: Color::Rgb(8, 160, 205),
            mode_normal: Color::Rgb(202, 138, 4),
            mode_operator: Color::Rgb(22, 163, 74),
            mode_char_search: Color::Rgb(229, 35, 110),
            border_unfocused: Color::Rgb(150, 150, 176),
            border_error: Color::Rgb(220, 38, 38),
            syntax_error_warning: Color::Rgb(202, 138, 4),
            tooltip_hint: Color::Rgb(171, 50, 207),
            unfocused_hint: Color::Rgb(150, 150, 176),
            query_unfocused: Color::Rgb(150, 150, 176),
            cursor: Style::new().add_modifier(Modifier::REVERSED),
        },
        results: ResultsTheme {
            border_focused: Color::Rgb(8, 160, 205),
            border_unfocused: Color::Rgb(150, 150, 176),
            border_warning: Color::Rgb(217, 119, 6),
            border_error: Color::Rgb(220, 38, 38),
            background: Color::Rgb(244, 243, 250),
            search_active: Color::Rgb(229, 35, 110),
            search_inactive: Color::Rgb(150, 150, 176),
            timing_normal: Color::Rgb(8, 160, 205),
            timing_slow: Color::Rgb(202, 138, 4),
            timing_very_slow: Color::Rgb(220, 38, 38),
            result_ok: Color::Rgb(22, 163, 74),
            result_warning: Color::Rgb(202, 138, 4),
            result_error: Color::Rgb(220, 38, 38),
            result_pending: Color::Rgb(120, 122, 145),
            badge_syntax_error: Style::new()
                .fg(Color::Rgb(60, 45, 0))
                .bg(Color::Rgb(214, 160, 0)),
            badge_empty_result: Style::new()
                .fg(Color::Rgb(245, 247, 252))
                .bg(Color::Rgb(96, 104, 130)),
            badge_back: Style::new()
                .fg(Color::Rgb(245, 250, 255))
                .bg(Color::Rgb(8, 160, 205)),
            badge_back_hover: Style::new()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(124, 58, 237))
                .add_modifier(Modifier::BOLD),
            match_highlight_bg: Color::Rgb(255, 235, 150),
            match_highlight_fg: Color::Rgb(40, 40, 60),
            current_match_bg: Color::Rgb(255, 196, 120),
            current_match_fg: Color::Rgb(40, 30, 0),
            cursor_line_bg: Color::Rgb(224, 220, 240),
            hovered_line_bg: Color::Rgb(232, 229, 245),
            visual_selection_bg: Color::Rgb(210, 202, 236),
            cursor_indicator_fg: Color::Rgb(229, 35, 110),
            stale_modifier: Modifier::DIM,
            path_at_cursor_separator: Color::Rgb(150, 150, 176),
            path_at_cursor: Color::Rgb(124, 58, 237),
            hint_key: Color::Rgb(8, 160, 205),
            hint_description: Style::new()
                .fg(Color::Rgb(8, 160, 205))
                .add_modifier(Modifier::DIM),
            spinner_colors: vec![
                Color::Rgb(229, 35, 110),
                Color::Rgb(234, 88, 12),
                Color::Rgb(150, 104, 0),
                Color::Rgb(22, 163, 74),
                Color::Rgb(8, 160, 205),
                Color::Rgb(124, 58, 237),
                Color::Rgb(171, 50, 207),
                Color::Rgb(220, 38, 38),
            ],
            jq_colors: [
                Color::Rgb(96, 99, 128),  // null - muted gray
                Color::Rgb(220, 38, 38),  // false - red
                Color::Rgb(22, 163, 74),  // true - green
                Color::Rgb(124, 58, 237), // numbers - purple
                Color::Rgb(22, 163, 74),  // strings - green
                Color::Rgb(8, 160, 205),  // arrays - cyan
                Color::Rgb(8, 160, 205),  // objects - cyan
                Color::Rgb(202, 138, 4),  // keys - amber
            ],
        },
        search: SearchTheme {
            border_active: Color::Rgb(229, 35, 110),
            border_inactive: Color::Rgb(150, 150, 176),
            background: Color::Rgb(244, 243, 250),
            text_active: Color::Rgb(40, 40, 60),
            text_inactive: Color::Rgb(150, 150, 176),
            no_matches: Color::Rgb(220, 38, 38),
            match_count: Color::Rgb(96, 99, 128),
            match_count_confirmed: Color::Rgb(139, 142, 168),
            badge_no_matches: Style::new()
                .fg(Color::Rgb(255, 238, 240))
                .bg(Color::Rgb(220, 38, 38)),
            badge_match_count: Style::new()
                .fg(Color::Rgb(255, 240, 247))
                .bg(Color::Rgb(229, 35, 110)),
            badge_match_count_confirmed: Style::new()
                .fg(Color::Rgb(60, 62, 85))
                .bg(Color::Rgb(214, 212, 226)),
            hints: Color::Rgb(229, 35, 110),
        },
        help: HelpTheme {
            border: Color::Rgb(8, 160, 205),
            background: Color::Rgb(244, 243, 250),
            scrollbar: Color::Rgb(8, 160, 205),
            title: Style::new()
                .fg(Color::Rgb(8, 160, 205))
                .add_modifier(Modifier::BOLD),
            tab_active: Style::new()
                .fg(Color::Rgb(8, 160, 205))
                .add_modifier(Modifier::BOLD),
            tab_inactive: Style::new()
                .fg(Color::Rgb(8, 160, 205))
                .add_modifier(Modifier::DIM),
            tab_hover_fg: Color::Rgb(8, 160, 205),
            tab_hover_bg: Color::Rgb(234, 232, 243),
            section_header: Style::new()
                .fg(Color::Rgb(8, 160, 205))
                .add_modifier(Modifier::BOLD),
            key: Style::new()
                .fg(Color::Rgb(202, 138, 4))
                .add_modifier(Modifier::BOLD),
            description: Color::Rgb(40, 40, 60),
            footer: Color::Rgb(139, 142, 168),
        },
        history: HistoryTheme {
            border: Color::Rgb(8, 160, 205),
            scrollbar: Color::Rgb(8, 160, 205),
            background: Color::Rgb(244, 243, 250),
            item_selected_bg: Color::Rgb(224, 220, 240),
            item_selected_indicator: Color::Rgb(8, 160, 205),
            item_normal_bg: Color::Rgb(244, 243, 250),
            item_normal_fg: Color::Rgb(80, 82, 108),
            no_matches: Color::Rgb(150, 150, 176),
            search_text: Color::Rgb(40, 40, 60),
            search_bg: Color::Rgb(244, 243, 250),
            delete_button: Color::Rgb(96, 99, 128),
            delete_button_hover: Color::Rgb(220, 38, 38),
        },
        snippets: SnippetsTheme {
            border: Color::Rgb(22, 163, 74),
            scrollbar: Color::Rgb(22, 163, 74),
            background: Color::Rgb(244, 243, 250),
            item_normal_fg: Color::Rgb(40, 40, 60),
            item_normal_bg: Color::Rgb(244, 243, 250),
            item_selected_fg: Color::Rgb(40, 40, 60),
            item_selected_bg: Color::Rgb(224, 220, 240),
            item_selected_indicator: Color::Rgb(22, 163, 74),
            item_selected_modifier: Modifier::BOLD,
            item_hovered_fg: Color::Rgb(40, 40, 60),
            item_hovered_bg: Color::Rgb(232, 229, 245),
            name: Color::Rgb(40, 40, 60),
            description: Color::Rgb(139, 142, 168),
            query_preview: Color::Rgb(202, 138, 4),
            category: Color::Rgb(22, 163, 74),
            field_active_border: Color::Rgb(202, 138, 4),
            field_inactive_border: Color::Rgb(22, 163, 74),
            field_text: Color::Rgb(40, 40, 60),
            field_bg: Color::Rgb(244, 243, 250),
            delete_border: Color::Rgb(220, 38, 38),
            hint_key: Color::Rgb(202, 138, 4),
            hint_text: Color::Rgb(40, 40, 60),
            search_text: Color::Rgb(40, 40, 60),
            search_bg: Color::Rgb(244, 243, 250),
        },
        save: SaveTheme {
            title: Color::Rgb(234, 88, 12),
            border: Color::Rgb(234, 88, 12),
            input_border: Color::Rgb(202, 138, 4),
            input_fg: Color::Rgb(40, 40, 60),
            input_bg: Color::Rgb(244, 243, 250),
            hint_key: Color::Rgb(202, 138, 4),
            hint_text: Color::Rgb(40, 40, 60),
            preview_ok: Color::Rgb(22, 163, 74),
            preview_warn: Color::Rgb(220, 38, 38),
            error: Color::Rgb(220, 38, 38),
        },
        ai: AiTheme {
            border: Color::Rgb(8, 160, 205),
            background: Color::Rgb(244, 243, 250),
            scrollbar: Color::Rgb(8, 160, 205),
            title: Style::new()
                .fg(Color::Rgb(8, 160, 205))
                .add_modifier(Modifier::BOLD),
            model_display: Color::Rgb(124, 58, 237),
            counter: Color::Rgb(202, 138, 4),
            config_icon: Color::Rgb(202, 138, 4),
            config_title: Style::new()
                .fg(Color::Rgb(202, 138, 4))
                .add_modifier(Modifier::BOLD),
            config_desc: Color::Rgb(96, 99, 128),
            config_code: Color::Rgb(8, 160, 205),
            config_link: Style::new()
                .fg(Color::Rgb(124, 58, 237))
                .add_modifier(Modifier::UNDERLINED),
            thinking_icon: Color::Rgb(202, 138, 4),
            thinking_text: Style::new()
                .fg(Color::Rgb(202, 138, 4))
                .add_modifier(Modifier::ITALIC),
            error_icon: Color::Rgb(220, 38, 38),
            error_title: Style::new()
                .fg(Color::Rgb(220, 38, 38))
                .add_modifier(Modifier::BOLD),
            error_message: Color::Rgb(220, 38, 38),
            query_text: Color::Rgb(8, 160, 205),
            result_text: Color::Rgb(40, 40, 60),
            previous_response: Color::Rgb(139, 142, 168),
            suggestion_selected_bg: Color::Rgb(216, 210, 236),
            suggestion_hovered_bg: Color::Rgb(224, 220, 240),
            suggestion_text_selected: Color::Rgb(40, 40, 60),
            suggestion_text_normal: Color::Rgb(96, 99, 128),
            suggestion_desc_normal: Color::Rgb(139, 142, 168),
            suggestion_desc_muted: Color::Rgb(96, 99, 128),
            suggestion_fix: Color::Rgb(220, 38, 38),
            suggestion_optimize: Color::Rgb(202, 138, 4),
            suggestion_next: Color::Rgb(22, 163, 74),
            hint: Color::Rgb(139, 142, 168),
        },
        autocomplete: AutocompleteTheme {
            border: Color::Rgb(8, 160, 205),
            scrollbar: Color::Rgb(8, 160, 205),
            background: Color::Rgb(244, 243, 250),
            item_normal_fg: Color::Rgb(40, 40, 60),
            item_normal_bg: Color::Rgb(244, 243, 250),
            item_selected_fg: Color::Rgb(255, 255, 255),
            item_selected_bg: Color::Rgb(8, 160, 205),
            item_selected_modifier: Modifier::BOLD,
            type_function: Color::Rgb(202, 138, 4),
            type_field: Color::Rgb(8, 160, 205),
            type_operator: Color::Rgb(171, 50, 207),
            type_pattern: Color::Rgb(22, 163, 74),
            type_variable: Color::Rgb(220, 38, 38),
            type_value: Color::Rgb(234, 88, 12),
        },
        tooltip: TooltipTheme {
            border: Color::Rgb(171, 50, 207),
            background: Color::Rgb(244, 243, 250),
            title: Style::new()
                .fg(Color::Rgb(171, 50, 207))
                .add_modifier(Modifier::BOLD),
            description: Color::Rgb(40, 40, 60),
            example: Color::Rgb(8, 160, 205),
            example_desc: Color::Rgb(96, 99, 128),
            tip: Color::Rgb(202, 138, 4),
            separator: Color::Rgb(139, 142, 168),
        },
        notification: NotificationTheme {
            info: NotificationColors {
                fg: Color::Rgb(40, 40, 60),
                bg: Color::Rgb(220, 224, 240),
                border: Color::Rgb(96, 99, 128),
            },
            warning: NotificationColors {
                fg: Color::Rgb(40, 30, 0),
                bg: Color::Rgb(255, 213, 120),
                border: Color::Rgb(217, 119, 6),
            },
            error: NotificationColors {
                fg: Color::Rgb(255, 238, 240),
                bg: Color::Rgb(220, 38, 38),
                border: Color::Rgb(150, 36, 44),
            },
        },
        help_line: HelpLineTheme {
            key: Color::Rgb(96, 99, 128),
            description: Color::Rgb(139, 142, 168),
            separator: Color::Rgb(139, 142, 168),
        },
        scrollbar: ScrollbarTheme {
            default: Color::Rgb(8, 160, 205),
            track: Color::Rgb(214, 210, 232),
        },
        syntax: SyntaxTheme {
            keyword: Color::Rgb(229, 35, 110),
            function: Color::Rgb(8, 160, 205),
            string: Color::Rgb(22, 163, 74),
            number: Color::Rgb(124, 58, 237),
            operator: Color::Rgb(171, 50, 207),
            variable: Color::Rgb(234, 88, 12),
            field: Color::Rgb(8, 160, 205),
            bracket_match_color: Color::Rgb(202, 138, 4),
            bracket_match_style: Style::new()
                .fg(Color::Rgb(202, 138, 4))
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        },
    }
}
