use crate::query::executor::JqExecutor;

/// Query execution state
pub struct QueryState {
    pub executor: JqExecutor,
    pub result: Result<String, String>,
    pub last_successful_result: Option<String>,
}

impl QueryState {
    /// Create a new QueryState with the given JSON input
    pub fn new(json_input: String) -> Self {
        let executor = JqExecutor::new(json_input);
        let result = executor.execute(".");
        let last_successful_result = result.as_ref().ok().cloned();

        Self {
            executor,
            result,
            last_successful_result,
        }
    }

    /// Execute a query and update results
    pub fn execute(&mut self, query: &str) {
        self.result = self.executor.execute(query);
        if let Ok(result) = &self.result {
            self.last_successful_result = Some(result.clone());
        }
    }

    /// Get the total number of lines in the current results
    /// Note: Returns u32 to handle large files (>65K lines) correctly
    /// When there's an error, uses last_successful_result since that's what gets rendered
    pub fn line_count(&self) -> u32 {
        match &self.result {
            Ok(result) => result.lines().count() as u32,
            Err(_) => {
                self.last_successful_result
                    .as_ref()
                    .map(|r| r.lines().count() as u32)
                    .unwrap_or(0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_query_state() {
        let json = r#"{"name": "test"}"#;
        let state = QueryState::new(json.to_string());

        assert!(state.result.is_ok());
        assert!(state.last_successful_result.is_some());
    }

    #[test]
    fn test_execute_updates_result() {
        let json = r#"{"name": "test", "age": 30}"#;
        let mut state = QueryState::new(json.to_string());

        state.execute(".name");
        assert!(state.result.is_ok());
        assert!(state.last_successful_result.is_some());
    }

    #[test]
    fn test_execute_caches_successful_result() {
        let json = r#"{"value": 42}"#;
        let mut state = QueryState::new(json.to_string());

        state.execute(".value");
        let cached = state.last_successful_result.clone();
        assert!(cached.is_some());

        // Execute invalid query (syntax error)
        state.execute(".[invalid syntax");
        assert!(state.result.is_err());

        // Last successful result should still be cached
        assert_eq!(state.last_successful_result, cached);
    }

    #[test]
    fn test_line_count_with_ok_result() {
        let json = r#"{"test": true}"#;
        let mut state = QueryState::new(json.to_string());

        let content: String = (0..50).map(|i| format!("line{}\n", i)).collect();
        state.result = Ok(content);

        assert_eq!(state.line_count(), 50);
    }

    #[test]
    fn test_line_count_uses_cached_on_error() {
        let json = r#"{"test": true}"#;
        let mut state = QueryState::new(json.to_string());

        let valid_result: String = (0..30).map(|i| format!("line{}\n", i)).collect();
        state.result = Ok(valid_result.clone());
        state.last_successful_result = Some(valid_result);

        // Now set an error
        state.result = Err("syntax error".to_string());

        // Should use cached result line count
        assert_eq!(state.line_count(), 30);
    }

    #[test]
    fn test_line_count_zero_on_error_without_cache() {
        let json = r#"{"test": true}"#;
        let mut state = QueryState::new(json.to_string());

        state.result = Err("error".to_string());
        state.last_successful_result = None;

        assert_eq!(state.line_count(), 0);
    }
}
