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
