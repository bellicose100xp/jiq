//! Shared test utilities for jiq
//!
//! This module provides common test fixtures and helper functions
//! used across multiple test modules.

#[cfg(test)]
pub mod test_helpers {
    use crate::app::App;
    use crate::config::Config;
    use crate::history::HistoryState;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// Test fixture JSON data
    pub const TEST_JSON: &str = r#"{
        "name": "test",
        "age": 30,
        "city": "NYC",
        "services": [{"name": "svc1", "serviceArn": "arn1"}],
        "items": [{"tags": [{"name": "tag1"}]}]
    }"#;

    /// Helper to create App with default config for tests
    pub fn test_app(json: &str) -> App {
        App::new(json.to_string(), &Config::default())
    }

    /// Helper to create a KeyEvent without modifiers
    pub fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    /// Helper to create a KeyEvent with specific modifiers
    pub fn key_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    /// Helper to set up an app with text in the query field
    pub fn app_with_query(query: &str) -> App {
        let mut app = test_app(TEST_JSON);
        app.input.textarea.insert_str(query);
        // Execute the query to set up base state for autosuggestions
        app.query.execute(query);
        // Use empty in-memory history for all tests to prevent disk writes
        app.history = HistoryState::empty();
        app
    }
}
