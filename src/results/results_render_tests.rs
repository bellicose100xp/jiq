use crate::app::App;
use crate::config::Config;
use crate::input::FileLoader;
use proptest::prelude::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;
use std::path::PathBuf;

/// Helper to create a test terminal
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Helper to render app to string
fn render_to_string(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    terminal.draw(|f| app.render(f)).unwrap();
    terminal.backend().to_string()
}

/// Helper to create an app with a loading FileLoader
fn create_app_with_loading_loader() -> App {
    // Create a FileLoader that will be in Loading state
    // Use a path that will take time to load or doesn't exist yet
    let loader = FileLoader::spawn_load(PathBuf::from("/tmp/test_loading_file.json"));
    App::new_with_loader(loader, &Config::default())
}

#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 1: Loading state displays loading indicator
        /// Feature: deferred-file-loading, Property 1: Loading state displays loading indicator
        /// Validates: Requirements 1.2, 2.1
        #[test]
        fn prop_loading_state_shows_indicator(
            width in 40u16..120u16,
            height in 10u16..40u16,
        ) {
            let mut app = create_app_with_loading_loader();

            // Verify preconditions: query is None and file_loader is Loading
            prop_assert!(app.query.is_none(), "Query should be None when loading");
            prop_assert!(app.file_loader.is_some(), "FileLoader should be present");

            if let Some(loader) = &app.file_loader {
                prop_assert!(loader.is_loading(), "FileLoader should be in Loading state");
            }

            // Render the app
            let output = render_to_string(&mut app, width, height);

            // Verify the loading indicator is displayed
            prop_assert!(
                output.contains("Loading file..."),
                "Rendered output should contain 'Loading file...' when file_loader is Loading. Output:\n{}",
                output
            );

            // Verify the loading indicator has the expected styling elements
            prop_assert!(
                output.contains("Loading"),
                "Rendered output should contain 'Loading' title"
            );
        }
    }
}

#[cfg(test)]
mod spinner_tests {
    use super::super::{SPINNER_CHARS, get_spinner};
    use crate::theme;

    #[test]
    fn test_spinner_first_frame() {
        let (char, color) = get_spinner(0);
        assert_eq!(char, SPINNER_CHARS[0]);
        assert_eq!(color, theme::results::SPINNER_COLORS[0]);
    }

    #[test]
    fn test_spinner_second_frame() {
        let (char, color) = get_spinner(8);
        assert_eq!(char, SPINNER_CHARS[1]);
        assert_eq!(color, theme::results::SPINNER_COLORS[1]);
    }

    #[test]
    fn test_spinner_char_cycling() {
        // Test all 10 spinner characters
        for i in 0..10 {
            let (char, _) = get_spinner(i * 8);
            assert_eq!(
                char,
                SPINNER_CHARS[i as usize],
                "Frame {} should have char {}",
                i * 8,
                SPINNER_CHARS[i as usize]
            );
        }
    }

    #[test]
    fn test_spinner_color_cycling() {
        // Test all 8 colors
        for i in 0..8 {
            let (_, color) = get_spinner(i * 8);
            assert_eq!(
                color,
                theme::results::SPINNER_COLORS[i as usize],
                "Frame {} should have color at index {}",
                i * 8,
                i
            );
        }
    }

    #[test]
    fn test_spinner_char_wrapping() {
        // After 10 chars (80 frames), should wrap back to first char
        let (char_start, _) = get_spinner(0);
        let (char_wrap, _) = get_spinner(80);
        assert_eq!(
            char_start, char_wrap,
            "Character should wrap after 10 iterations"
        );
    }

    #[test]
    fn test_spinner_color_wrapping() {
        // After 8 colors (64 frames), should wrap back to first color
        let (_, color_start) = get_spinner(0);
        let (_, color_wrap) = get_spinner(64);
        assert_eq!(
            color_start, color_wrap,
            "Color should wrap after 8 iterations"
        );
    }

    #[test]
    fn test_spinner_independent_cycling() {
        // Chars and colors cycle independently (different lengths: 10 vs 8)
        // At frame 40: char index = 5, color index = 5
        let (char, _) = get_spinner(40);
        assert_eq!(char, SPINNER_CHARS[5]);

        // At frame 48: char index = 6, color index = 6
        let (char, _) = get_spinner(48);
        assert_eq!(char, SPINNER_CHARS[6]);

        // At frame 64: char index = 8, color index = 0 (wrapped)
        let (char, color) = get_spinner(64);
        assert_eq!(char, SPINNER_CHARS[8]);
        assert_eq!(color, theme::results::SPINNER_COLORS[0]);
    }

    #[test]
    fn test_spinner_large_frame_count() {
        // Test with large frame count to ensure no overflow/panic
        let (char, color) = get_spinner(u64::MAX);
        // Should still produce valid char and color
        assert!(SPINNER_CHARS.contains(&char));
        assert!(theme::results::SPINNER_COLORS.contains(&color));
    }

    #[test]
    fn test_spinner_animation_speed() {
        // Verify frames 0-7 all use same char (changes every 8 frames)
        let (char0, _) = get_spinner(0);
        for frame in 1..8 {
            let (char, _) = get_spinner(frame);
            assert_eq!(char, char0, "Frames 0-7 should all use same character");
        }

        // Frame 8 should use different char
        let (char8, _) = get_spinner(8);
        assert_ne!(
            char8, char0,
            "Frame 8 should use different character than frame 0"
        );
    }
}

#[cfg(test)]
mod position_indicator_tests {
    use super::super::format_position_indicator;
    use crate::scroll::ScrollState;

    fn create_scroll_state(offset: u16, viewport_height: u16, max_offset: u16) -> ScrollState {
        ScrollState {
            offset,
            max_offset,
            viewport_height,
            h_offset: 0,
            max_h_offset: 0,
            viewport_width: 80,
        }
    }

    #[test]
    fn test_empty_content_returns_empty_string() {
        let scroll = create_scroll_state(0, 20, 0);
        assert_eq!(format_position_indicator(&scroll, 0), "");
    }

    #[test]
    fn test_single_line() {
        let scroll = create_scroll_state(0, 20, 0);
        assert_eq!(format_position_indicator(&scroll, 1), "L1-1/1 (0%)");
    }

    #[test]
    fn test_at_top() {
        let scroll = create_scroll_state(0, 20, 80);
        assert_eq!(format_position_indicator(&scroll, 100), "L1-20/100 (0%)");
    }

    #[test]
    fn test_at_bottom() {
        let scroll = create_scroll_state(80, 20, 80);
        assert_eq!(format_position_indicator(&scroll, 100), "L81-100/100 (80%)");
    }

    #[test]
    fn test_middle_position() {
        let scroll = create_scroll_state(45, 20, 80);
        assert_eq!(format_position_indicator(&scroll, 100), "L46-65/100 (45%)");
    }

    #[test]
    fn test_viewport_larger_than_content() {
        let scroll = create_scroll_state(0, 50, 0);
        assert_eq!(format_position_indicator(&scroll, 10), "L1-10/10 (0%)");
    }

    #[test]
    fn test_small_file_exact_viewport() {
        let scroll = create_scroll_state(0, 20, 0);
        assert_eq!(format_position_indicator(&scroll, 20), "L1-20/20 (0%)");
    }

    #[test]
    fn test_large_file() {
        let scroll = create_scroll_state(500, 50, 950);
        assert_eq!(
            format_position_indicator(&scroll, 1000),
            "L501-550/1000 (50%)"
        );
    }

    #[test]
    fn test_percentage_rounding() {
        let scroll = create_scroll_state(33, 20, 80);
        assert_eq!(format_position_indicator(&scroll, 100), "L34-53/100 (33%)");
    }

    #[test]
    fn test_near_end_clamping() {
        let scroll = create_scroll_state(95, 20, 80);
        assert_eq!(format_position_indicator(&scroll, 100), "L96-100/100 (95%)");
    }
}

#[cfg(test)]
mod scrollbar_tests {
    use super::super::render_scrollbar;
    use crate::scroll::ScrollState;
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Style;
    use ratatui::widgets::{Block, Borders};

    fn create_scroll_state(offset: u16, viewport_height: u16, max_offset: u16) -> ScrollState {
        ScrollState {
            offset,
            max_offset,
            viewport_height,
            h_offset: 0,
            max_h_offset: 0,
            viewport_width: 80,
        }
    }

    fn render_scrollbar_to_string(
        scroll: &ScrollState,
        line_count: u32,
        width: u16,
        height: u16,
    ) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, width, height);
                render_scrollbar(frame, area, scroll, line_count);
            })
            .unwrap();
        terminal.backend().to_string()
    }

    fn render_scrollbar_with_border_to_string(
        scroll: &ScrollState,
        line_count: u32,
        width: u16,
        height: u16,
    ) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, width, height);
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default());
                frame.render_widget(block, area);
                render_scrollbar(frame, area, scroll, line_count);
            })
            .unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn test_scrollbar_hidden_when_content_fits() {
        let scroll = create_scroll_state(0, 20, 0);
        let output = render_scrollbar_to_string(&scroll, 10, 80, 22);
        // When content fits, no scrollbar characters should appear
        assert!(
            !output.contains('█') && !output.contains('│') && !output.contains('▐'),
            "Scrollbar should not render when content fits viewport"
        );
    }

    #[test]
    fn test_scrollbar_visible_when_content_exceeds_viewport() {
        let scroll = create_scroll_state(0, 20, 80);
        let output = render_scrollbar_to_string(&scroll, 100, 80, 22);
        // When content exceeds viewport, scrollbar should appear
        // ratatui uses '█' for the thumb
        assert!(
            output.contains('█'),
            "Scrollbar thumb should render when content exceeds viewport. Output:\n{}",
            output
        );
    }

    #[test]
    fn test_scrollbar_position_at_top() {
        let scroll = create_scroll_state(0, 20, 80);
        let backend = TestBackend::new(80, 22);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 80, 22);
                render_scrollbar(frame, area, &scroll, 100);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        // Check that thumb ('█') appears near the top of the scrollbar column (rightmost)
        let col = 79;
        let mut thumb_positions: Vec<u16> = Vec::new();
        for row in 0..22 {
            if buffer[(col, row)].symbol() == "█" {
                thumb_positions.push(row);
            }
        }
        assert!(
            !thumb_positions.is_empty(),
            "Scrollbar thumb should be visible"
        );
        // At offset 0, thumb should be at the top
        let avg_position: f32 =
            thumb_positions.iter().map(|&r| r as f32).sum::<f32>() / thumb_positions.len() as f32;
        assert!(
            avg_position < 11.0,
            "Scrollbar thumb should be near top when offset=0. Avg position: {}",
            avg_position
        );
    }

    #[test]
    fn test_scrollbar_position_at_bottom() {
        let scroll = create_scroll_state(80, 20, 80);
        let backend = TestBackend::new(80, 22);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 80, 22);
                render_scrollbar(frame, area, &scroll, 100);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let col = 79;
        let mut thumb_positions: Vec<u16> = Vec::new();
        for row in 0..22 {
            if buffer[(col, row)].symbol() == "█" {
                thumb_positions.push(row);
            }
        }
        assert!(
            !thumb_positions.is_empty(),
            "Scrollbar thumb should be visible"
        );
        // At max offset, thumb should be at the bottom
        let avg_position: f32 =
            thumb_positions.iter().map(|&r| r as f32).sum::<f32>() / thumb_positions.len() as f32;
        assert!(
            avg_position > 11.0,
            "Scrollbar thumb should be near bottom when offset=max. Avg position: {}",
            avg_position
        );
    }

    #[test]
    fn test_scrollbar_position_middle() {
        let scroll = create_scroll_state(40, 20, 80);
        let backend = TestBackend::new(80, 22);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 80, 22);
                render_scrollbar(frame, area, &scroll, 100);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let col = 79;
        let mut thumb_positions: Vec<u16> = Vec::new();
        for row in 0..22 {
            if buffer[(col, row)].symbol() == "█" {
                thumb_positions.push(row);
            }
        }
        assert!(
            !thumb_positions.is_empty(),
            "Scrollbar thumb should be visible"
        );
        // At middle offset, thumb should be roughly in the middle
        let avg_position: f32 =
            thumb_positions.iter().map(|&r| r as f32).sum::<f32>() / thumb_positions.len() as f32;
        // Middle would be around row 11 for a 22-row area
        assert!(
            avg_position > 5.0 && avg_position < 17.0,
            "Scrollbar thumb should be near middle when offset=40. Avg position: {}",
            avg_position
        );
    }

    #[test]
    fn snapshot_scrollbar_with_border_at_top() {
        let scroll = create_scroll_state(0, 10, 90);
        let output = render_scrollbar_with_border_to_string(&scroll, 100, 20, 12);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_scrollbar_with_border_at_middle() {
        let scroll = create_scroll_state(45, 10, 90);
        let output = render_scrollbar_with_border_to_string(&scroll, 100, 20, 12);
        assert_snapshot!(output);
    }

    #[test]
    fn snapshot_scrollbar_with_border_at_bottom() {
        let scroll = create_scroll_state(90, 10, 90);
        let output = render_scrollbar_with_border_to_string(&scroll, 100, 20, 12);
        assert_snapshot!(output);
    }

    #[test]
    fn test_scrollbar_respects_border_corners() {
        let scroll = create_scroll_state(0, 10, 90);
        let backend = TestBackend::new(20, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 20, 12);
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default());
                frame.render_widget(block, area);
                render_scrollbar(frame, area, &scroll, 100);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();

        let col = 19;
        let top_row_symbol = buffer[(col, 0)].symbol();
        let bottom_row_symbol = buffer[(col, 11)].symbol();

        assert_eq!(
            top_row_symbol, "┐",
            "Top-right corner should be preserved (got: {})",
            top_row_symbol
        );
        assert_eq!(
            bottom_row_symbol, "┘",
            "Bottom-right corner should be preserved (got: {})",
            bottom_row_symbol
        );

        let mut has_scrollbar_in_middle = false;
        for row in 1..11 {
            let symbol = buffer[(col, row)].symbol();
            if symbol == "█" || symbol == "│" {
                has_scrollbar_in_middle = true;
                break;
            }
        }
        assert!(
            has_scrollbar_in_middle,
            "Scrollbar should render between the corners"
        );
    }
}

#[cfg(test)]
mod search_no_match_dim_tests {
    use super::*;
    use crate::search::search_events::open_search;
    use crate::test_utils::test_helpers::test_app;
    use std::sync::Arc;

    fn setup_app_with_results(content: &str) -> App {
        use ratatui::text::Text;

        let mut app = test_app(r#"{"name": "test"}"#);
        let arc = Arc::new(content.to_string());
        let q = app.query.as_mut().unwrap();
        q.last_successful_result = Some(Arc::clone(&arc));
        q.last_successful_result_unformatted = Some(Arc::clone(&arc));
        q.last_successful_result_rendered = Some(Text::raw(content.to_string()));
        q.result = Ok(content.to_string());
        q.is_empty_result = false;
        app
    }

    fn count_dim_cells(app: &mut App, width: u16, height: u16) -> (usize, usize) {
        use crate::search::search_render::SEARCH_BAR_HEIGHT;

        let mut terminal = create_test_terminal(width, height);
        terminal.draw(|f| app.render(f)).unwrap();
        let buffer = terminal.backend().buffer();

        // Scan only the interior viewport of the results pane: skip top/bottom
        // borders, the search bar, and the bottom-border row that may carry
        // overlaid hint text (which uses DIM styling unrelated to is_stale).
        let results_bottom = height.saturating_sub(SEARCH_BAR_HEIGHT);
        let scan_y_start: u16 = 1;
        let scan_y_end: u16 = results_bottom.saturating_sub(2);
        let scan_x_start: u16 = 1;
        let scan_x_end: u16 = width.saturating_sub(1);

        let mut dim = 0usize;
        let mut content = 0usize;
        for y in scan_y_start..scan_y_end {
            for x in scan_x_start..scan_x_end {
                let cell = &buffer[(x, y)];
                let sym = cell.symbol();
                if sym.trim().is_empty() {
                    continue;
                }
                if matches!(sym, "│" | "─" | "╭" | "╮" | "╰" | "╯" | "█") {
                    continue;
                }
                content += 1;
                if cell.modifier.contains(Modifier::DIM) {
                    dim += 1;
                }
            }
        }
        (dim, content)
    }

    #[test]
    fn no_dim_when_search_query_has_matches() {
        let content = "alpha line\nbeta line\nalpha again\n";
        let mut app = setup_app_with_results(content);

        open_search(&mut app);
        app.search.search_textarea_mut().insert_str("alpha");
        app.search.update_matches(content);

        let (dim, total) = count_dim_cells(&mut app, 60, 12);
        assert!(total > 0, "should have rendered content cells");
        assert_eq!(dim, 0, "no cells should be dimmed when matches exist");
    }

    #[test]
    fn dim_when_search_query_has_no_matches() {
        let content = "alpha line\nbeta line\nalpha again\n";
        let mut app = setup_app_with_results(content);

        open_search(&mut app);
        app.search.search_textarea_mut().insert_str("zzz");
        app.search.update_matches(content);

        let (dim, total) = count_dim_cells(&mut app, 60, 12);
        assert!(total > 0, "should have rendered content cells");
        assert!(dim > 0, "result cells should be dimmed when no matches");
    }

    #[test]
    fn no_dim_when_search_query_is_empty() {
        let content = "alpha line\nbeta line\nalpha again\n";
        let mut app = setup_app_with_results(content);

        open_search(&mut app);

        let (dim, total) = count_dim_cells(&mut app, 60, 12);
        assert!(total > 0, "should have rendered content cells");
        assert_eq!(dim, 0, "empty query must not dim");
    }

    fn render_to_string_local(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = create_test_terminal(width, height);
        terminal.draw(|f| app.render(f)).unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn shows_no_matches_badge_when_search_query_has_no_matches() {
        let content = "alpha line\nbeta line\n";
        let mut app = setup_app_with_results(content);

        open_search(&mut app);
        app.search.search_textarea_mut().insert_str("zzz");
        app.search.update_matches(content);

        let output = render_to_string_local(&mut app, 60, 12);
        assert!(
            output.contains("No Matches"),
            "results pane title should advertise the no-match state. Output:\n{}",
            output
        );
    }

    #[test]
    fn no_no_matches_badge_when_query_has_matches() {
        let content = "alpha line\nbeta line\n";
        let mut app = setup_app_with_results(content);

        open_search(&mut app);
        app.search.search_textarea_mut().insert_str("alpha");
        app.search.update_matches(content);

        let output = render_to_string_local(&mut app, 60, 12);
        assert!(
            !output.contains("No Matches"),
            "title must not show no-match badge while matches exist"
        );
    }

    #[test]
    fn no_no_matches_badge_when_query_is_empty() {
        let content = "alpha line\nbeta line\n";
        let mut app = setup_app_with_results(content);

        open_search(&mut app);

        let output = render_to_string_local(&mut app, 60, 12);
        assert!(
            !output.contains("No Matches"),
            "title must not show no-match badge while query is empty"
        );
    }

    #[test]
    fn no_no_matches_badge_when_confirmed() {
        let content = "alpha line\nbeta line\n";
        let mut app = setup_app_with_results(content);

        open_search(&mut app);
        app.search.search_textarea_mut().insert_str("zzz");
        app.search.update_matches(content);
        app.search.confirm();

        let output = render_to_string_local(&mut app, 60, 12);
        assert!(
            !output.contains("No Matches"),
            "confirmed mode must not show the no-match badge"
        );
    }

    #[test]
    fn no_dim_when_search_confirmed_with_no_matches() {
        let content = "alpha line\nbeta line\n";
        let mut app = setup_app_with_results(content);

        open_search(&mut app);
        app.search.search_textarea_mut().insert_str("zzz");
        app.search.update_matches(content);
        app.search.confirm();

        let (dim, total) = count_dim_cells(&mut app, 60, 12);
        assert!(total > 0, "should have rendered content cells");
        assert_eq!(dim, 0, "confirmed mode must not dim, even with no matches");
    }
}

#[cfg(test)]
mod back_button_tests {
    use super::*;
    use crate::test_utils::test_helpers::{key, test_app};
    use ratatui::crossterm::event::KeyCode;

    fn render(app: &mut App, width: u16, height: u16) -> String {
        let mut terminal = create_test_terminal(width, height);
        terminal.draw(|f| app.render(f)).unwrap();
        terminal.backend().to_string()
    }

    /// Drill once via `>` so the undo ring is non-empty. Mirrors the setup
    /// in mouse_click_tests.
    fn push_one_drill(app: &mut App) {
        app.focus = crate::app::Focus::ResultsPane;
        app.input.textarea.insert_str(".");
        if let Some(qs) = app.query.as_mut() {
            qs.execute(".");
        }
        let total = app.results_line_count_u32();
        app.results_cursor.update_total_lines(total);
        app.results_cursor.move_to_line(1);
        app.handle_key_event(key(KeyCode::Char('>')));
    }

    #[test]
    fn back_badge_hidden_when_undo_ring_empty() {
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        let output = render(&mut app, 80, 12);
        assert!(
            !output.contains("[ < Back ]"),
            "back badge must not render when there is nothing to undo:\n{}",
            output
        );
        assert!(
            app.layout_regions.back_button.is_none(),
            "back-button rect must be None when the badge is hidden",
        );
    }

    #[test]
    fn back_badge_renders_when_undo_ring_nonempty() {
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        push_one_drill(&mut app);

        let output = render(&mut app, 80, 12);
        assert!(
            output.contains("[ < Back ]"),
            "back badge must render when the undo ring is non-empty:\n{}",
            output
        );
        let rect = app
            .layout_regions
            .back_button
            .expect("back-button rect must be tracked when the badge is visible");
        assert_eq!(rect.height, 1);
        assert_eq!(rect.width, "[ < Back ]".len() as u16);
    }

    #[test]
    fn back_badge_coexists_with_chord_hint_in_strip() {
        // The bottom-border hint strip teaches the keyboard chord; the
        // top-border badge is a click target. Both surface `<` so the user
        // sees the visual affordance and learns the shortcut.
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        push_one_drill(&mut app);

        let output = render(&mut app, 120, 12);
        assert!(
            output.contains("[ < Back ]"),
            "top-border badge must render:\n{}",
            output,
        );
        assert!(
            output.contains("< back"),
            "bottom-border strip should still teach the `< back` chord:\n{}",
            output,
        );
    }

    /// X coordinate of the single cell separating the back badge from the
    /// status badge to its right, found by locating the badge's trailing `]`
    /// on the top border row and returning the cell immediately after it.
    fn cell_after_back_badge(buffer: &ratatui::buffer::Buffer, width: u16) -> (u16, u16) {
        let y = 0;
        for x in 0..width.saturating_sub(1) {
            if buffer[(x, y)].symbol() == "]" {
                return (x + 1, y);
            }
        }
        panic!("back badge `]` not found on top border row");
    }

    #[test]
    fn back_badge_separated_from_syntax_error_badge() {
        // The cyan back badge and the golden syntax-error badge both carry a
        // background color. Rendered flush they read as one block; a neutral
        // (no-bg) cell must sit between them.
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        app.query_undo
            .push("", crate::query_undo::ViewportState::default());
        if let Some(qs) = app.query.as_mut() {
            qs.execute(".[");
        }
        assert!(
            app.query.as_ref().is_some_and(|q| q.result.is_err()),
            "test setup must produce an error result",
        );

        let mut terminal = create_test_terminal(80, 12);
        terminal.draw(|f| app.render(f)).unwrap();
        let buffer = terminal.backend().buffer();
        let (x, y) = cell_after_back_badge(buffer, 80);
        let cell = &buffer[(x, y)];
        assert_eq!(
            cell.symbol().trim(),
            "",
            "the cell after the back badge must be blank",
        );
        assert_eq!(
            cell.bg,
            ratatui::style::Color::Reset,
            "the separator cell must have no background so the two badges don't touch",
        );
    }

    #[test]
    fn back_badge_separated_from_empty_result_badge() {
        // Same neutral-cell requirement for the steel-blue empty-result badge.
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        app.query_undo
            .push("", crate::query_undo::ViewportState::default());
        if let Some(qs) = app.query.as_mut() {
            qs.execute(".nonexistent");
        }
        assert!(
            app.query.as_ref().is_some_and(|q| q.is_empty_result),
            "test setup must produce an empty result",
        );

        let mut terminal = create_test_terminal(80, 12);
        terminal.draw(|f| app.render(f)).unwrap();
        let buffer = terminal.backend().buffer();
        let (x, y) = cell_after_back_badge(buffer, 80);
        let cell = &buffer[(x, y)];
        assert_eq!(
            cell.symbol().trim(),
            "",
            "the cell after the back badge must be blank",
        );
        assert_eq!(
            cell.bg,
            ratatui::style::Color::Reset,
            "the separator cell must have no background so the two badges don't touch",
        );
    }

    #[test]
    fn position_indicator_renders_on_top_right() {
        // Position info now anchors the TOP-RIGHT border so it stays visible
        // above the AI / help boxes that overlay the bottom of the screen.
        let mut app = test_app(r#"{"a": 1, "b": 2}"#);
        let output = render(&mut app, 80, 22);
        assert!(
            output.contains("L1-"),
            "position indicator must render somewhere:\n{}",
            output,
        );
        let lines: Vec<&str> = output.lines().collect();
        let top_border = lines.first().copied().unwrap_or_default();
        assert!(
            top_border.contains("L1-"),
            "position indicator must render on the top border (row 0):\n{}",
            output,
        );
        // The results-pane bottom border is the last `╰...╯` line before the
        // query pane. It must no longer carry the indicator.
        let results_bottom = lines
            .iter()
            .find(|l| l.contains('╰') && l.contains('╯'))
            .copied()
            .unwrap_or_default();
        assert!(
            !results_bottom.contains("L1-"),
            "position indicator must not be on the bottom border anymore:\n{}",
            output,
        );
        // And it is absent from the very last rendered row too.
        let last_row = lines.last().copied().unwrap_or_default();
        assert!(
            !last_row.contains("L1-"),
            "position indicator must not be on the last row:\n{}",
            output,
        );
    }
}
