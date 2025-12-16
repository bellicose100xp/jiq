#[cfg(test)]
pub mod test_helpers {
    use crate::app::App;
    use crate::config::Config;
    use crate::history::HistoryState;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    pub const TEST_JSON: &str = r#"{
        "name": "test",
        "age": 30,
        "city": "NYC",
        "services": [{"name": "svc1", "serviceArn": "arn1"}],
        "items": [{"tags": [{"name": "tag1"}]}]
    }"#;

    pub fn test_app(json: &str) -> App {
        App::new(json.to_string(), &Config::default())
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
        app.query.execute(query);
        app.history = HistoryState::empty();
        app
    }
}
