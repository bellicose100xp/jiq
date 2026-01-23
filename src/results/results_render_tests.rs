use crate::app::App;
use crate::config::Config;
use crate::input::FileLoader;
use proptest::prelude::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
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
    use super::super::{SPINNER_CHARS, SPINNER_COLORS, get_spinner};

    #[test]
    fn test_spinner_first_frame() {
        let (char, color) = get_spinner(0);
        assert_eq!(char, SPINNER_CHARS[0]);
        assert_eq!(color, SPINNER_COLORS[0]);
    }

    #[test]
    fn test_spinner_second_frame() {
        let (char, color) = get_spinner(8);
        assert_eq!(char, SPINNER_CHARS[1]);
        assert_eq!(color, SPINNER_COLORS[1]);
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
                SPINNER_COLORS[i as usize],
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
        assert_eq!(color, SPINNER_COLORS[0]);
    }

    #[test]
    fn test_spinner_large_frame_count() {
        // Test with large frame count to ensure no overflow/panic
        let (char, color) = get_spinner(u64::MAX);
        // Should still produce valid char and color
        assert!(SPINNER_CHARS.contains(&char));
        assert!(SPINNER_COLORS.contains(&color));
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
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;

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
}
