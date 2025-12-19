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
