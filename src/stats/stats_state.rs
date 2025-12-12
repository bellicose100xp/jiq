//! State management for result statistics
//!
//! This module provides the `StatsState` struct which caches computed statistics
//! about query results and provides display formatting.

use crate::app::App;
use crate::stats::parser::StatsParser;
use crate::stats::types::ResultStats;

/// Update stats based on the last successful result from the App
///
/// This is the delegation function called by `App::update_stats()`.
/// It extracts the last successful unformatted result and computes stats if available.
pub fn update_stats_from_app(app: &mut App) {
    if let Some(result) = &app.query.last_successful_result_unformatted {
        app.stats.compute(result);
    }
}

/// State for managing cached result statistics
///
/// `StatsState` caches the computed statistics for the current query result.
/// When a syntax error occurs, the cached stats are preserved to show the
/// stats from the last successful result (per Requirement 4.4).
#[derive(Debug, Clone, Default)]
pub struct StatsState {
    /// Cached stats for the current/last successful result
    stats: Option<ResultStats>,
}

impl StatsState {
    /// Compute stats from a result string and cache them
    ///
    /// This parses the result string using the fast character-based parser
    /// and caches the computed statistics.
    pub fn compute(&mut self, result: &str) {
        let trimmed = result.trim();
        if trimmed.is_empty() {
            // Don't update stats for empty results (preserves last stats)
            return;
        }
        self.stats = Some(StatsParser::parse(result));
    }

    /// Get the display string for the stats bar
    ///
    /// Returns `None` if no stats have been computed yet.
    pub fn display(&self) -> Option<String> {
        self.stats.as_ref().map(|s| s.to_string())
    }

    /// Get a reference to the cached stats (used in tests)
    #[cfg(test)]
    pub fn stats(&self) -> Option<&ResultStats> {
        self.stats.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::types::ElementType;
    use proptest::prelude::*;

    #[test]
    fn test_default_state_has_no_stats() {
        let state = StatsState::default();
        assert!(state.stats().is_none());
        assert!(state.display().is_none());
    }

    #[test]
    fn test_compute_array_stats() {
        let mut state = StatsState::default();
        state.compute("[1, 2, 3]");
        
        assert!(state.stats().is_some());
        assert_eq!(
            state.stats(),
            Some(&ResultStats::Array { count: 3, element_type: ElementType::Numbers })
        );
        assert_eq!(state.display(), Some("Array [3 numbers]".to_string()));
    }

    #[test]
    fn test_compute_object_stats() {
        let mut state = StatsState::default();
        state.compute(r#"{"key": "value"}"#);
        
        assert_eq!(state.stats(), Some(&ResultStats::Object));
        assert_eq!(state.display(), Some("Object".to_string()));
    }

    #[test]
    fn test_compute_string_stats() {
        let mut state = StatsState::default();
        state.compute(r#""hello world""#);
        
        assert_eq!(state.stats(), Some(&ResultStats::String));
        assert_eq!(state.display(), Some("String".to_string()));
    }

    #[test]
    fn test_compute_number_stats() {
        let mut state = StatsState::default();
        state.compute("42");
        
        assert_eq!(state.stats(), Some(&ResultStats::Number));
        assert_eq!(state.display(), Some("Number".to_string()));
    }

    #[test]
    fn test_compute_boolean_stats() {
        let mut state = StatsState::default();
        state.compute("true");
        
        assert_eq!(state.stats(), Some(&ResultStats::Boolean));
        assert_eq!(state.display(), Some("Boolean".to_string()));
    }

    #[test]
    fn test_compute_null_stats() {
        let mut state = StatsState::default();
        state.compute("null");
        
        assert_eq!(state.stats(), Some(&ResultStats::Null));
        assert_eq!(state.display(), Some("null".to_string()));
    }

    #[test]
    fn test_compute_stream_stats() {
        let mut state = StatsState::default();
        state.compute("{}\n{}\n{}");
        
        assert_eq!(state.stats(), Some(&ResultStats::Stream { count: 3 }));
        assert_eq!(state.display(), Some("Stream [3]".to_string()));
    }

    #[test]
    fn test_empty_result_preserves_stats() {
        let mut state = StatsState::default();
        state.compute("[1, 2, 3]");
        let original_stats = state.stats().cloned();
        
        // Empty result should not update stats (preserves last successful)
        state.compute("");
        assert_eq!(state.stats().cloned(), original_stats);
        
        // Whitespace-only result should also preserve
        state.compute("   ");
        assert_eq!(state.stats().cloned(), original_stats);
    }

    #[test]
    fn test_stats_update_on_new_result() {
        let mut state = StatsState::default();
        
        state.compute("[1, 2, 3]");
        assert_eq!(state.display(), Some("Array [3 numbers]".to_string()));
        
        state.compute(r#"{"a": 1}"#);
        assert_eq!(state.display(), Some("Object".to_string()));
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    /// Strategy to generate a simple JSON value (non-container)
    fn arb_simple_json_value() -> impl Strategy<Value = String> {
        prop_oneof![
            // Numbers
            (-1000i64..1000).prop_map(|n| n.to_string()),
            // Strings (simple, no special chars)
            "[a-zA-Z0-9]{0,10}".prop_map(|s| format!(r#""{}""#, s)),
            // Booleans
            Just("true".to_string()),
            Just("false".to_string()),
            // Null
            Just("null".to_string()),
        ]
    }

    /// Strategy to generate valid JSON values
    fn arb_valid_json() -> impl Strategy<Value = String> {
        prop_oneof![
            // Simple values
            arb_simple_json_value(),
            // Arrays
            prop::collection::vec(arb_simple_json_value(), 0..10)
                .prop_map(|elements| format!("[{}]", elements.join(", "))),
            // Objects
            prop::collection::vec(
                ("[a-z]{1,5}", arb_simple_json_value()),
                0..5
            ).prop_map(|pairs| {
                let fields: Vec<String> = pairs.iter()
                    .map(|(k, v)| format!(r#""{}": {}"#, k, v))
                    .collect();
                format!("{{{}}}", fields.join(", "))
            }),
        ]
    }

    /// Strategy to generate "error" results (empty or whitespace-only strings)
    /// These simulate what happens when a query fails - the result is empty
    fn arb_error_result() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("".to_string()),
            Just("   ".to_string()),
            Just("\n".to_string()),
            Just("\t".to_string()),
            Just("  \n  ".to_string()),
        ]
    }

    // Feature: stats-bar, Property 5: Stats persistence on error
    // *For any* sequence of queries where a valid query is followed by an invalid
    // query, the stats SHALL continue to display the stats from the last successful result.
    // **Validates: Requirements 4.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_stats_persist_on_error(
            valid_json in arb_valid_json(),
            error_result in arb_error_result()
        ) {
            let mut state = StatsState::default();
            
            // First, compute stats from a valid result
            state.compute(&valid_json);
            let stats_after_valid = state.stats().cloned();
            let display_after_valid = state.display();
            
            // Stats should be computed (not None) for valid JSON
            prop_assert!(
                stats_after_valid.is_some(),
                "Stats should be computed for valid JSON: '{}'",
                valid_json
            );
            
            // Now simulate an error (empty result)
            state.compute(&error_result);
            let stats_after_error = state.stats().cloned();
            let display_after_error = state.display();
            
            // Stats should be preserved (same as before the error)
            prop_assert_eq!(
                &stats_after_error, &stats_after_valid,
                "Stats should persist after error. Before: {:?}, After: {:?}",
                &stats_after_valid, &stats_after_error
            );
            
            // Display should also be preserved
            prop_assert_eq!(
                &display_after_error, &display_after_valid,
                "Display should persist after error. Before: {:?}, After: {:?}",
                &display_after_valid, &display_after_error
            );
        }

        #[test]
        fn prop_multiple_errors_preserve_last_valid_stats(
            valid_json in arb_valid_json(),
            error_count in 1usize..5
        ) {
            let mut state = StatsState::default();
            
            // Compute stats from valid result
            state.compute(&valid_json);
            let original_stats = state.stats().cloned();
            let original_display = state.display();
            
            prop_assert!(
                original_stats.is_some(),
                "Stats should be computed for valid JSON"
            );
            
            // Apply multiple error results
            for _ in 0..error_count {
                state.compute("");
            }
            
            // Stats should still be preserved
            prop_assert_eq!(
                state.stats().cloned(), original_stats,
                "Stats should persist after {} errors",
                error_count
            );
            prop_assert_eq!(
                state.display(), original_display,
                "Display should persist after {} errors",
                error_count
            );
        }

        #[test]
        fn prop_new_valid_result_updates_stats(
            first_json in arb_valid_json(),
            second_json in arb_valid_json()
        ) {
            let mut state = StatsState::default();
            
            // Compute stats from first valid result
            state.compute(&first_json);
            let first_stats = state.stats().cloned();
            
            // Compute stats from second valid result
            state.compute(&second_json);
            let second_stats = state.stats().cloned();
            
            // Both should have stats
            prop_assert!(first_stats.is_some(), "First result should have stats");
            prop_assert!(second_stats.is_some(), "Second result should have stats");
            
            // The second stats should reflect the second JSON
            // (We can't easily compare the exact stats without parsing,
            // but we can verify that stats are computed)
            let expected_stats = StatsParser::parse(&second_json);
            prop_assert_eq!(
                second_stats, Some(expected_stats),
                "Stats should be updated to reflect second JSON"
            );
        }
    }

    // =========================================================================
    // App Integration Tests for Stats
    // =========================================================================
    // These tests verify the update_stats_from_app() delegation function

    use crate::test_utils::test_helpers::test_app;

    #[test]
    fn test_update_stats_from_app_with_object() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let mut app = test_app(json);
        
        // Initial query executes identity filter, which sets last_successful_result_unformatted
        update_stats_from_app(&mut app);
        
        assert_eq!(app.stats.display(), Some("Object".to_string()));
    }

    #[test]
    fn test_update_stats_from_app_with_array() {
        let json = r#"[1, 2, 3, 4, 5]"#;
        let mut app = test_app(json);
        
        update_stats_from_app(&mut app);
        
        assert_eq!(app.stats.display(), Some("Array [5 numbers]".to_string()));
    }

    #[test]
    fn test_update_stats_from_app_no_result() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);
        
        // Clear the last successful result to simulate no result available
        app.query.last_successful_result_unformatted = None;
        
        // Stats should remain unchanged (None)
        let stats_before = app.stats.display();
        update_stats_from_app(&mut app);
        let stats_after = app.stats.display();
        
        assert_eq!(stats_before, stats_after);
    }

    #[test]
    fn test_update_stats_from_app_preserves_on_error() {
        let json = r#"[1, 2, 3]"#;
        let mut app = test_app(json);
        
        // First update with valid result
        update_stats_from_app(&mut app);
        assert_eq!(app.stats.display(), Some("Array [3 numbers]".to_string()));
        
        // Simulate an error by setting result to error but keeping last_successful_result_unformatted
        app.query.result = Err("syntax error".to_string());
        // Note: last_successful_result_unformatted is still set from the initial query
        
        // Update stats again - should still show the last successful stats
        update_stats_from_app(&mut app);
        assert_eq!(app.stats.display(), Some("Array [3 numbers]".to_string()));
    }

    #[test]
    fn test_update_stats_from_app_updates_on_new_query() {
        let json = r#"{"items": [1, 2, 3]}"#;
        let mut app = test_app(json);
        
        // Initial stats for the object
        update_stats_from_app(&mut app);
        assert_eq!(app.stats.display(), Some("Object".to_string()));
        
        // Execute a new query that returns an array
        app.query.execute(".items");
        update_stats_from_app(&mut app);
        
        assert_eq!(app.stats.display(), Some("Array [3 numbers]".to_string()));
    }
}
