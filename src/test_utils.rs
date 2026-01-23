#[cfg(test)]
pub mod test_helpers {
    use crate::app::App;
    use crate::config::Config;
    use crate::history::HistoryState;
    use crate::input::FileLoader;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    pub const TEST_JSON: &str = r#"{
        "name": "test",
        "age": 30,
        "city": "NYC",
        "services": [{"name": "svc1", "serviceArn": "arn1"}],
        "items": [{"tags": [{"name": "tag1"}]}]
    }"#;

    /// Create a FileLoader with pre-loaded JSON for testing
    ///
    /// This helper creates a FileLoader that immediately has the JSON available,
    /// avoiding the need for actual file I/O or background threads in tests.
    pub fn create_test_loader(json: String) -> FileLoader {
        use crate::input::loader::LoadingState;
        use std::sync::mpsc::channel;
        let (tx, rx) = channel();
        // Send the result immediately so poll() will return it
        let _ = tx.send(Ok(json));
        FileLoader {
            state: LoadingState::Loading,
            rx: Some(rx),
        }
    }

    pub fn test_app(json: &str) -> App {
        let loader = create_test_loader(json.to_string());
        let mut app = App::new_with_loader(loader, &Config::default());
        // Poll the loader to complete loading
        app.poll_file_loader();
        // Disable history persistence to avoid polluting real history file
        app.history = HistoryState::empty();
        app
    }

    pub fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    pub fn key_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    pub fn app_with_query(query: &str) -> App {
        let mut app = test_app(TEST_JSON);
        app.input.textarea.insert_str(query);
        if let Some(query_state) = &mut app.query {
            query_state.execute(query);
        }
        app.history = HistoryState::empty();
        app
    }

    /// Wait for async query to complete by polling
    ///
    /// Polls query_state.poll_response() until query completes or timeout.
    /// Returns true if query completed, false if timeout.
    pub fn wait_for_query_completion(app: &mut App, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if let Some(query_state) = &mut app.query {
                if !query_state.is_pending() {
                    return true;
                }
                // poll_response() now returns Option<String>, just call and discard
                let _ = query_state.poll_response();
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        false
    }

    /// Execute async query and wait for completion
    ///
    /// Helper for tests that need to wait for async query results.
    pub fn execute_query_and_wait(app: &mut App) {
        if let Some(query_state) = &mut app.query {
            let query = app.input.textarea.lines()[0].as_ref();
            query_state.execute_async(query);
        }

        assert!(
            wait_for_query_completion(app, 2000),
            "Query did not complete within timeout"
        );
    }
}
