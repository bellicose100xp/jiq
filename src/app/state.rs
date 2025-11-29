use tui_textarea::CursorMove;

use crate::help::HelpPopupState;
use super::input_state::InputState;
use super::query_state::QueryState;
use crate::autocomplete::{AutocompleteState, get_suggestions};
use crate::config::ClipboardBackend;
use crate::history::HistoryState;
use crate::notification::NotificationState;
use crate::scroll::ScrollState;

#[cfg(debug_assertions)]
use log::debug;

// Autocomplete performance constants
const MIN_CHARS_FOR_AUTOCOMPLETE: usize = 1;

/// Which pane has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    InputField,
    ResultsPane,
}

/// What to output when exiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Results, // Output filtered JSON results (Enter)
    Query,   // Output query string only (Shift+Enter)
}

/// Application state
pub struct App {
    pub input: InputState,
    pub query: QueryState,
    pub focus: Focus,
    pub results_scroll: ScrollState,
    pub output_mode: Option<OutputMode>,
    pub should_quit: bool,
    pub autocomplete: AutocompleteState,
    pub error_overlay_visible: bool,
    pub history: HistoryState,
    pub help: HelpPopupState,
    pub notification: NotificationState,
    pub clipboard_backend: ClipboardBackend,
}

impl App {
    /// Create a new App instance with JSON input and clipboard backend preference
    pub fn new(json_input: String, clipboard_backend: ClipboardBackend) -> Self {
        Self {
            input: InputState::new(),
            query: QueryState::new(json_input),
            focus: Focus::InputField,
            results_scroll: ScrollState::new(),
            output_mode: None,
            should_quit: false,
            autocomplete: AutocompleteState::new(),
            error_overlay_visible: false,
            history: HistoryState::new(),
            help: HelpPopupState::new(),
            notification: NotificationState::new(),
            clipboard_backend,
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get the output mode (if set)
    pub fn output_mode(&self) -> Option<OutputMode> {
        self.output_mode
    }

    /// Get the current query text
    pub fn query(&self) -> &str {
        self.input.query()
    }

    /// Get the total number of lines in the current results
    /// Note: Returns u32 to handle large files (>65K lines) correctly
    /// When there's an error, uses last_successful_result since that's what gets rendered
    pub fn results_line_count_u32(&self) -> u32 {
        self.query.line_count()
    }


    /// Update autocomplete suggestions based on current query and cursor position
    pub fn update_autocomplete(&mut self) {
        let query = self.query();
        let cursor_pos = self.input.textarea.cursor().1; // Column position

        // Performance optimization: only show autocomplete for non-empty queries
        if query.trim().len() < MIN_CHARS_FOR_AUTOCOMPLETE {
            self.autocomplete.hide();
            return;
        }

        // Get suggestions based on unformatted query result (no ANSI codes)
        let result = self.query.last_successful_result_unformatted.as_deref();
        let result_type = self.query.base_type_for_suggestions.clone();
        let suggestions = get_suggestions(query, cursor_pos, result, result_type);

        // Update autocomplete state
        self.autocomplete.update_suggestions(suggestions);
    }

    /// Insert an autocomplete suggestion at the current cursor position
    /// Uses explicit state-based formulas for each context type
    pub fn insert_autocomplete_suggestion(&mut self, suggestion: &str) {
        use crate::app::query_state::QueryState;
        use crate::autocomplete::{analyze_context, find_char_before_field_access, SuggestionContext};

        // Get base query that produced these suggestions
        let base_query = match &self.query.base_query_for_suggestions {
            Some(b) => b.clone(),
            None => {
                // Fallback to current query if no base (shouldn't happen)
                self.query().to_string()
            }
        };

        let query = self.query().to_string();
        let cursor_pos = self.input.textarea.cursor().1;
        let before_cursor = &query[..cursor_pos.min(query.len())];

        #[cfg(debug_assertions)]
        debug!(
            "insert_autocomplete_suggestion: current_query='{}' base_query='{}' suggestion='{}' cursor_pos={}",
            query, base_query, suggestion, cursor_pos
        );

        // Determine the trigger context
        let (context, partial) = analyze_context(before_cursor);
        
        #[cfg(debug_assertions)]
        debug!(
            "context_analysis: context={:?} partial='{}'",
            context, partial
        );
        
        // For function/operator context (jq keywords like then, else, end, etc.),
        // we should do simple replacement without adding dots or complex formulas
        if context == SuggestionContext::FunctionContext {
            // Simple replacement: remove the partial and insert the suggestion
            let replacement_start = cursor_pos.saturating_sub(partial.len());
            let new_query = format!("{}{}{}", 
                &query[..replacement_start],
                suggestion,
                &query[cursor_pos..]
            );
            
            #[cfg(debug_assertions)]
            debug!("function_context_replacement: partial='{}' new_query='{}'", partial, new_query);
            
            // Replace the entire line with new query
            self.input.textarea.delete_line_by_head();
            self.input.textarea.insert_str(&new_query);
            
            // Move cursor to end of inserted suggestion
            let target_pos = replacement_start + suggestion.len();
            self.move_cursor_to_column(target_pos);
            
            // Hide autocomplete and execute query
            self.autocomplete.hide();
            self.execute_query_and_update();
            return;
        }
        
        // For field context, continue with the existing complex logic
        let char_before = find_char_before_field_access(before_cursor, &partial);
        let trigger_type = QueryState::classify_char(char_before);

        // Extract middle_query: everything between base and current field being typed
        // This preserves complex expressions like if/then/else, functions, etc.
        let mut middle_query = Self::extract_middle_query(&query, &base_query, before_cursor, &partial);
        let mut adjusted_base = base_query.clone();
        let mut adjusted_suggestion = suggestion.to_string();

        #[cfg(debug_assertions)]
        debug!(
            "field_context: partial='{}' char_before={:?} trigger_type={:?} middle_query='{}'",
            partial, char_before, trigger_type, middle_query
        );

        // Special handling for CloseBracket trigger with [] in middle_query
        // This handles nested arrays like: .services[].capacityProviderStrategy[].field
        // When user types [], it becomes part of middle_query, but should be part of base
        if trigger_type == crate::app::query_state::CharType::CloseBracket && middle_query == "[]" {
            #[cfg(debug_assertions)]
            debug!("nested_array_adjustment: detected [] in middle_query, moving to base");
            
            // Move [] from middle to base
            adjusted_base = format!("{}{}", base_query, middle_query);
            middle_query = String::new();
            
            // Strip [] prefix from suggestion if present (it's already in the query)
            // Also strip the leading dot since CloseBracket formula will add it
            if let Some(stripped) = adjusted_suggestion.strip_prefix("[]") {
                // Strip leading dot if present (e.g., "[].base" -> "base")
                adjusted_suggestion = stripped.strip_prefix('.').unwrap_or(stripped).to_string();
                
                #[cfg(debug_assertions)]
                debug!("nested_array_adjustment: stripped [] and leading dot from suggestion");
            }
            
            #[cfg(debug_assertions)]
            debug!(
                "nested_array_adjustment: adjusted_base='{}' adjusted_suggestion='{}' middle_query='{}'",
                adjusted_base, adjusted_suggestion, middle_query
            );
        }

        // Special case: if base is root "." and suggestion starts with ".",
        // replace the base entirely instead of appending
        // This prevents: "." + ".services" = "..services"
        let new_query = if adjusted_base == "." && adjusted_suggestion.starts_with('.') && middle_query.is_empty() {
            #[cfg(debug_assertions)]
            debug!("formula: root_replacement (special case for root '.')");
            
            adjusted_suggestion.to_string()
        } else {
            // Apply insertion formula: base + middle + (operator) + suggestion
            // The middle preserves complex expressions between base and current field
            let formula_result = match trigger_type {
                crate::app::query_state::CharType::NoOp => {
                    // NoOp means continuing a path, but we need to check if suggestion needs a dot
                    // - If suggestion starts with special char like [, {, etc., don't add dot
                    // - If base is root ".", don't add another dot
                    // - Otherwise, add dot for path continuation (like .user.name)
                    let needs_dot = !adjusted_suggestion.starts_with('[') 
                        && !adjusted_suggestion.starts_with('{')
                        && !adjusted_suggestion.starts_with('.')
                        && adjusted_base != ".";
                    
                    if needs_dot {
                        #[cfg(debug_assertions)]
                        debug!("formula: NoOp -> base + middle + '.' + suggestion");
                        
                        format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
                    } else {
                        #[cfg(debug_assertions)]
                        debug!("formula: NoOp -> base + middle + suggestion (no dot added)");
                        
                        format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
                    }
                }
            crate::app::query_state::CharType::CloseBracket => {
                #[cfg(debug_assertions)]
                debug!("formula: CloseBracket -> base + middle + '.' + suggestion");
                
                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            crate::app::query_state::CharType::PipeOperator => {
                #[cfg(debug_assertions)]
                debug!("formula: PipeOperator -> base + middle + ' ' + suggestion");
                
                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!("{}{} {}", adjusted_base, trimmed_middle, adjusted_suggestion)
            }
            crate::app::query_state::CharType::Semicolon => {
                #[cfg(debug_assertions)]
                debug!("formula: Semicolon -> base + middle + ' ' + suggestion");
                
                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!("{}{} {}", adjusted_base, trimmed_middle, adjusted_suggestion)
            }
            crate::app::query_state::CharType::Comma => {
                #[cfg(debug_assertions)]
                debug!("formula: Comma -> base + middle + ' ' + suggestion");
                
                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!("{}{} {}", adjusted_base, trimmed_middle, adjusted_suggestion)
            }
            crate::app::query_state::CharType::Colon => {
                #[cfg(debug_assertions)]
                debug!("formula: Colon -> base + middle + ' ' + suggestion");
                
                // Formula: base + middle + " " + suggestion
                // Trim trailing space from middle to avoid double spaces
                let trimmed_middle = middle_query.trim_end();
                format!("{}{} {}", adjusted_base, trimmed_middle, adjusted_suggestion)
            }
            crate::app::query_state::CharType::OpenParen => {
                #[cfg(debug_assertions)]
                debug!("formula: OpenParen -> base + middle + suggestion (paren already in middle)");
                
                // Formula: base + middle + suggestion
                // The ( is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            crate::app::query_state::CharType::OpenBracket => {
                #[cfg(debug_assertions)]
                debug!("formula: OpenBracket -> base + middle + suggestion (bracket already in middle)");
                
                // Formula: base + middle + suggestion
                // The [ is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            crate::app::query_state::CharType::OpenBrace => {
                #[cfg(debug_assertions)]
                debug!("formula: OpenBrace -> base + middle + suggestion (brace already in middle)");
                
                // Formula: base + middle + suggestion
                // The { is already in middle_query, don't add it again
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            crate::app::query_state::CharType::QuestionMark => {
                #[cfg(debug_assertions)]
                debug!("formula: QuestionMark -> base + middle + '.' + suggestion");
                
                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            crate::app::query_state::CharType::Dot => {
                #[cfg(debug_assertions)]
                debug!("formula: Dot -> base + middle + suggestion");
                
                // Formula: base + middle + suggestion
                format!("{}{}{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            crate::app::query_state::CharType::CloseParen |
            crate::app::query_state::CharType::CloseBrace => {
                #[cfg(debug_assertions)]
                debug!("formula: CloseParen/CloseBrace -> base + middle + '.' + suggestion");
                
                // Formula: base + middle + "." + suggestion
                format!("{}{}.{}", adjusted_base, middle_query, adjusted_suggestion)
            }
            };
            
            #[cfg(debug_assertions)]
            debug!("formula_components: base='{}' middle='{}' suggestion='{}'", 
                   adjusted_base, middle_query, adjusted_suggestion);
            
            formula_result
        };

        #[cfg(debug_assertions)]
        debug!("new_query_constructed: '{}'", new_query);

        // Replace the entire line with new query
        self.input.textarea.delete_line_by_head();
        self.input.textarea.insert_str(&new_query);

        #[cfg(debug_assertions)]
        debug!("query_after_insertion: '{}'", self.query());

        // Move cursor to end of query
        let target_pos = new_query.len();
        self.move_cursor_to_column(target_pos);

        // Hide autocomplete and execute query
        self.autocomplete.hide();
        self.execute_query_and_update();
    }

    /// Extract middle query: everything between base and current field being typed
    ///
    /// Examples:
    /// - Query: ".services | if has(...) then .ca", base: ".services"
    ///   → middle: " | if has(...) then "
    /// - Query: ".services | .ca", base: ".services"
    ///   → middle: " | "
    /// - Query: ".services.ca", base: ".services"
    ///   → middle: ""
    fn extract_middle_query(
        current_query: &str,
        base_query: &str,
        before_cursor: &str,
        partial: &str,
    ) -> String {
        // Find where base ends in current query
        if !current_query.starts_with(base_query) {
            // Base is not a prefix of current query (shouldn't happen, but handle gracefully)
            return String::new();
        }


        // Find where the trigger char is in before_cursor
        // Middle should be: everything after base, up to but not including trigger char
        // Examples:
        //   Query: ".services | .ca", partial: "ca", base: ".services"
        //   → trigger is the dot at position 11
        //   → middle = query[9..11] = " | " (with trailing space, no dot)
        let trigger_pos_in_before_cursor = if partial.is_empty() {
            // Just typed trigger char - it's the last char
            before_cursor.len().saturating_sub(1)
        } else {
            // Partial being typed - trigger is one char before partial
            before_cursor.len().saturating_sub(partial.len() + 1)
        };
        
        #[cfg(debug_assertions)]
        debug!(
            "extract_middle_query: current_query='{}' before_cursor='{}' partial='{}' trigger_pos={} base_len={}",
            current_query, before_cursor, partial, trigger_pos_in_before_cursor, base_query.len()
        );

        // Middle is everything from end of base to (but not including) trigger
        let base_len = base_query.len();
        if trigger_pos_in_before_cursor <= base_len {
            // Trigger at or before base ends - no middle
            return String::new();
        }

        // Extract middle - preserve all whitespace as it may be significant
        // (e.g., "then " needs the space before the field access)
        let middle = current_query[base_len..trigger_pos_in_before_cursor].to_string();
        
        #[cfg(debug_assertions)]
        debug!("extract_middle_query: extracted_middle='{}'", middle);
        
        middle
    }

    /// Move cursor to a specific column position (helper method)
    fn move_cursor_to_column(&mut self, target_col: usize) {
        let current_col = self.input.textarea.cursor().1;

        match target_col.cmp(&current_col) {
            std::cmp::Ordering::Less => {
                // Move backward
                for _ in 0..(current_col - target_col) {
                    self.input.textarea.move_cursor(CursorMove::Back);
                }
            }
            std::cmp::Ordering::Greater => {
                // Move forward
                for _ in 0..(target_col - current_col) {
                    self.input.textarea.move_cursor(CursorMove::Forward);
                }
            }
            std::cmp::Ordering::Equal => {
                // Already at target position
            }
        }
    }

    /// Execute query and update results (helper method)
    fn execute_query_and_update(&mut self) {
        let query_text = self.query().to_string();
        self.query.execute(&query_text);
        self.results_scroll.reset();
        self.error_overlay_visible = false; // Auto-hide error overlay on query change
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::query_state::ResultType;
    use crate::config::ClipboardBackend;

    /// Helper to create App with default clipboard backend for tests
    fn test_app(json: &str) -> App {
        App::new(json.to_string(), ClipboardBackend::Auto)
    }

    #[test]
    fn test_app_initialization() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let app = test_app(json);

        // Check default state
        assert_eq!(app.focus, Focus::InputField);
        assert_eq!(app.results_scroll.offset, 0);
        assert_eq!(app.output_mode, None);
        assert!(!app.should_quit);
        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_initial_query_result() {
        let json = r#"{"name": "Bob"}"#;
        let app = test_app(json);

        // Initial query should execute identity filter "."
        assert!(app.query.result.is_ok());
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("Bob"));
    }

    #[test]
    fn test_focus_enum() {
        assert_eq!(Focus::InputField, Focus::InputField);
        assert_eq!(Focus::ResultsPane, Focus::ResultsPane);
        assert_ne!(Focus::InputField, Focus::ResultsPane);
    }

    #[test]
    fn test_output_mode_enum() {
        assert_eq!(OutputMode::Results, OutputMode::Results);
        assert_eq!(OutputMode::Query, OutputMode::Query);
        assert_ne!(OutputMode::Results, OutputMode::Query);
    }

    #[test]
    fn test_should_quit_getter() {
        let json = r#"{}"#;
        let mut app = test_app(json);

        assert!(!app.should_quit());

        app.should_quit = true;
        assert!(app.should_quit());
    }

    #[test]
    fn test_output_mode_getter() {
        let json = r#"{}"#;
        let mut app = test_app(json);

        assert_eq!(app.output_mode(), None);

        app.output_mode = Some(OutputMode::Results);
        assert_eq!(app.output_mode(), Some(OutputMode::Results));

        app.output_mode = Some(OutputMode::Query);
        assert_eq!(app.output_mode(), Some(OutputMode::Query));
    }

    #[test]
    fn test_query_getter_empty() {
        let json = r#"{"test": true}"#;
        let app = test_app(json);

        assert_eq!(app.query(), "");
    }

    #[test]
    fn test_app_with_empty_json_object() {
        let json = "{}";
        let app = test_app(json);

        assert!(app.query.result.is_ok());
    }

    #[test]
    fn test_app_with_json_array() {
        let json = r#"[1, 2, 3]"#;
        let app = test_app(json);

        assert!(app.query.result.is_ok());
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    // Tests for large file handling (>65K lines)
    #[test]
    fn test_max_scroll_large_content() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        // Simulate large content result
        let large_result: String = (0..70000).map(|i| format!("line {}\n", i)).collect();
        app.query.result = Ok(large_result);

        // Should handle >65K lines without overflow
        let line_count = app.results_line_count_u32();
        assert!(line_count > 65535);

        // Update scroll bounds
        app.results_scroll.update_bounds(line_count, 20);

        // max_offset should be clamped to u16::MAX
        assert_eq!(app.results_scroll.max_offset, u16::MAX);
    }

    #[test]
    fn test_results_line_count_large_file() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        // Simulate result with exactly u16::MAX lines
        let result: String = (0..65535).map(|_| "x\n").collect();
        app.query.result = Ok(result);

        // Verify line count is correct (using internal method)
        assert_eq!(app.results_line_count_u32(), 65535);

        // Update scroll bounds
        app.results_scroll.update_bounds(65535, 10);

        // Verify max_offset handles it correctly
        assert_eq!(app.results_scroll.max_offset, 65525); // 65535 - 10
    }

    #[test]
    fn test_line_count_uses_last_result_on_error() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        // Execute a valid query first to cache result
        let valid_result: String = (0..50).map(|i| format!("line{}\n", i)).collect();
        app.query.result = Ok(valid_result.clone());
        app.query.last_successful_result = Some(valid_result);

        // Verify line count with valid result
        assert_eq!(app.results_line_count_u32(), 50);

        // Now simulate an error (short error message)
        app.query.result = Err("syntax error\nline 2\nline 3".to_string());

        // Line count should use last_successful_result (50 lines), not error (3 lines)
        assert_eq!(app.results_line_count_u32(), 50);

        // Update scroll bounds and verify max_offset is calculated correctly
        app.results_scroll.update_bounds(50, 10);
        assert_eq!(app.results_scroll.max_offset, 40); // 50 - 10 = 40
    }

    #[test]
    fn test_line_count_with_error_no_cached_result() {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        // Set error without any cached result
        app.query.last_successful_result = None;
        app.query.result = Err("error message".to_string());

        // Should return 0 when no cached result available
        assert_eq!(app.results_line_count_u32(), 0);

        // Update scroll bounds
        app.results_scroll.update_bounds(0, 10);
        assert_eq!(app.results_scroll.max_offset, 0);
    }

    #[test]
    fn test_array_suggestion_appends_to_path() {
        // When accepting [].field suggestion for .services, should produce .services[].field
        let json = r#"{"services": [{"name": "alice"}, {"name": "bob"}, {"name": "charlie"}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute ".services" to cache base
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state after ".services"
        assert_eq!(app.query.base_query_for_suggestions, Some(".services".to_string()),
                   "base_query should be '.services'");
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::ArrayOfObjects),
                   "base_type should be ArrayOfObjects");

        // Step 2: Accept autocomplete suggestion "[].name" (no leading dot since after NoOp)
        app.insert_autocomplete_suggestion("[].name");

        // Should produce .services[].name (append, not replace)
        assert_eq!(app.query(), ".services[].name");

        // CRITICAL: Verify the query EXECUTES correctly and returns ALL array elements
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("alice"), "Should contain first element");
        assert!(result.contains("bob"), "Should contain second element");
        assert!(result.contains("charlie"), "Should contain third element");

        // Verify it does NOT just return nulls or single value
        let line_count = result.lines().count();
        assert!(line_count >= 3, "Should return at least 3 lines for 3 array elements");
    }

    #[test]
    fn test_simple_path_continuation_with_dot() {
        // Test simple path continuation: .object.field
        // This is the bug: .services[0].deploymentConfiguration.alarms becomes deploymentConfigurationalarms
        let json = r#"{"user": {"name": "Alice", "age": 30, "address": {"city": "NYC"}}}"#;
        let mut app = test_app(json);

        // Step 1: Execute base query
        app.input.textarea.insert_str(".user");
        app.query.execute(".user");

        // Validate cached state
        assert_eq!(app.query.base_query_for_suggestions, Some(".user".to_string()));
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::Object));

        // Step 2: Type ".na" (partial field access)
        app.input.textarea.insert_str(".na");

        // Step 3: Accept suggestion "name" (no leading dot since continuing path)
        app.insert_autocomplete_suggestion("name");

        // Should produce: .user.name
        // NOT: .username (missing dot)
        assert_eq!(app.query(), ".user.name");
        
        // Verify execution
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_array_suggestion_replaces_partial_field() {
        // When user types partial field after array name, accepting [] suggestion should replace partial
        let json = r#"{"services": [{"serviceArn": "arn1"}, {"serviceArn": "arn2"}, {"serviceArn": "arn3"}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute ".services" to cache base
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state
        assert_eq!(app.query.base_query_for_suggestions, Some(".services".to_string()));
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::ArrayOfObjects));

        // Step 2: Type ".s" (partial)
        app.input.textarea.insert_char('.');
        app.input.textarea.insert_char('s');

        // Step 3: Accept autocomplete suggestion "[].serviceArn"
        app.insert_autocomplete_suggestion("[].serviceArn");

        // Should produce .services[].serviceArn (replace ".s" with "[].serviceArn")
        assert_eq!(app.query(), ".services[].serviceArn");

        // CRITICAL: Verify execution returns ALL serviceArns
        let result = app.query.result.as_ref().unwrap();
        eprintln!("Query result:\n{}", result);

        assert!(result.contains("arn1"), "Should contain first serviceArn");
        assert!(result.contains("arn2"), "Should contain second serviceArn");
        assert!(result.contains("arn3"), "Should contain third serviceArn");

        // Should NOT have nulls (would indicate query failed to iterate array)
        let null_count = result.matches("null").count();
        assert_eq!(null_count, 0, "Should not have any null values - query should iterate all array elements");
    }

    #[test]
    fn test_array_suggestion_replaces_trailing_dot() {
        // When user types ".services." (trailing dot, no partial), array suggestion should replace the dot
        let json = r#"{"services": [{"deploymentConfiguration": {"x": 1}}, {"deploymentConfiguration": {"x": 2}}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute ".services" to cache base query and type
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state
        assert_eq!(app.query.base_query_for_suggestions, Some(".services".to_string()),
                   "base_query should be '.services'");
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::ArrayOfObjects),
                   "base_type should be ArrayOfObjects");

        // Step 2: Type a dot (syntax error, doesn't update base)
        app.input.textarea.insert_char('.');

        // Step 3: Accept autocomplete suggestion "[].deploymentConfiguration"
        app.insert_autocomplete_suggestion("[].deploymentConfiguration");

        // Should produce .services[].deploymentConfiguration (NOT .services.[].deploymentConfiguration)
        assert_eq!(app.query(), ".services[].deploymentConfiguration");

        // Verify the query executes correctly
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("x"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
    }

    #[test]
    fn test_nested_array_suggestion_replaces_trailing_dot() {
        // Test deeply nested arrays: .services[].capacityProviderStrategy[].
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0, "weight": 1}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute base query to cache state
        app.input.textarea.insert_str(".services[].capacityProviderStrategy[]");
        app.query.execute(".services[].capacityProviderStrategy[]");

        // Validate cached state
        assert_eq!(app.query.base_query_for_suggestions, Some(".services[].capacityProviderStrategy[]".to_string()));
        // With only 1 service, this returns a single object, not destructured
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::Object));

        // Step 2: Type trailing dot
        app.input.textarea.insert_char('.');

        // Step 3: Accept autocomplete suggestion "base"
        // Note: suggestion is "base" (no prefix) since Object after CloseBracket
        app.insert_autocomplete_suggestion("base");

        // Should produce .services[].capacityProviderStrategy[].base
        assert_eq!(app.query(), ".services[].capacityProviderStrategy[].base");

        // Verify the query executes and returns the base values
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("0"));
    }

    #[test]
    fn test_array_suggestion_after_pipe() {
        // After pipe, array suggestions should include leading dot
        let json = r#"{"services": [{"name": "svc1"}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute base query
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state
        assert_eq!(app.query.base_query_for_suggestions, Some(".services".to_string()));
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::ArrayOfObjects));

        // Step 2: Type " | ."
        app.input.textarea.insert_str(" | .");

        // Step 3: Accept autocomplete suggestion ".[].name" (WITH leading dot after pipe)
        app.insert_autocomplete_suggestion(".[].name");

        // Should produce .services | .[].name (NOT .services | . | .[].name)
        assert_eq!(app.query(), ".services | .[].name");

        // Verify execution
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("svc1"));
    }

    #[test]
    fn test_array_suggestion_after_pipe_exact_user_flow() {
        // Replicate exact user flow: type partial, select, then pipe
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Type ".ser" (partial)
        app.input.textarea.insert_str(".ser");
        // Note: .ser returns null, base stays at "."

        // Step 2: Select ".services" from autocomplete
        // In real app, user would Tab here with suggestion ".services"
        app.insert_autocomplete_suggestion(".services");

        // Validate: should now be ".services"
        assert_eq!(app.query(), ".services");

        // Step 3: Verify base is now cached after successful execution
        assert_eq!(app.query.base_query_for_suggestions, Some(".services".to_string()),
                   "base should be '.services' after insertion executed it");
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::ArrayOfObjects));

        // Step 4: Type " | ."
        app.input.textarea.insert_str(" | .");

        // Step 5: Select ".[].capacityProviderStrategy"
        app.insert_autocomplete_suggestion(".[].capacityProviderStrategy");

        // Should produce: .services | .[].capacityProviderStrategy
        // NOT: .services | . | .[].capacityProviderStrategy
        assert_eq!(app.query(), ".services | .[].capacityProviderStrategy");
    }

    #[test]
    fn test_pipe_after_typing_space() {
        // Test typing space then pipe character by character
        let json = r#"{"services": [{"name": "svc1"}]}"#;
        let mut app = test_app(json);

        // Step 1: Type and execute ".services"
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        assert_eq!(app.query.base_query_for_suggestions, Some(".services".to_string()));

        // Step 2: Type space (executes ".services ")
        app.input.textarea.insert_char(' ');
        app.query.execute(".services ");

        // Step 3: Type | (executes ".services |" - syntax error, base stays at ".services")
        app.input.textarea.insert_char('|');
        app.query.execute(".services |");

        // Step 4: Type space then dot
        app.input.textarea.insert_str(" .");

        // Step 5: Accept suggestion
        app.insert_autocomplete_suggestion(".[].name");

        // Should be: base + " | " + suggestion
        // Base is trimmed, so: ".services" + " | " + ".[].name" = ".services | .[].name" ✅
        assert_eq!(app.query(), ".services | .[].name");
    }

    #[test]
    fn test_suggestions_persist_when_typing_partial_after_array() {
        // Critical: When typing partial field after [], suggestions should persist
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0, "weight": 1, "capacityProvider": "x"}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Type the full path up to the last array
        app.input.textarea.insert_str(".services[].capacityProviderStrategy[]");
        app.query.execute(".services[].capacityProviderStrategy[]");
        app.update_autocomplete();

        // Cache should have the array element objects with fields: base, weight, capacityProvider
        assert!(app.query.last_successful_result_unformatted.is_some());
        let cached = app.query.last_successful_result_unformatted.clone();

        // Step 2: Type a dot - should still have cached result
        app.input.textarea.insert_char('.');
        // Query is now ".services[].capacityProviderStrategy[]." which is syntax error
        app.query.execute(".services[].capacityProviderStrategy[].");

        // Cache should NOT be cleared (syntax error doesn't update cache)
        assert_eq!(app.query.last_successful_result_unformatted, cached);

        // Step 3: Type a partial "b" - query returns multiple nulls
        app.input.textarea.insert_char('b');
        // Query is now ".services[].capacityProviderStrategy[].b" which returns multiple nulls
        app.query.execute(".services[].capacityProviderStrategy[].b");

        // CRITICAL: Cache should STILL not be cleared (multiple nulls shouldn't overwrite)
        assert_eq!(app.query.last_successful_result_unformatted, cached);

        // Step 4: Update autocomplete - should still show suggestions based on cached result
        app.update_autocomplete();

        // Should have suggestions for the cached object fields
        let suggestions = app.autocomplete.suggestions();
        assert!(!suggestions.is_empty(), "Suggestions should persist when typing partial that returns null");

        // Should have "base" suggestion (filtered by partial "b")
        assert!(suggestions.iter().any(|s| s.text.contains("base")),
                "Should suggest 'base' field when filtering by 'b'");
    }

    #[test]
    fn test_suggestions_persist_with_optional_chaining_and_partial() {
        // Critical: When typing partial after []?, suggestions should persist
        // Realistic scenario: some services have capacityProviderStrategy, some don't
        let json = r#"{
            "services": [
                {
                    "serviceName": "service1",
                    "capacityProviderStrategy": [
                        {"base": 0, "weight": 1, "capacityProvider": "FARGATE"},
                        {"base": 0, "weight": 2, "capacityProvider": "FARGATE_SPOT"}
                    ]
                },
                {
                    "serviceName": "service2"
                },
                {
                    "serviceName": "service3",
                    "capacityProviderStrategy": [
                        {"base": 1, "weight": 3, "capacityProvider": "EC2"}
                    ]
                }
            ]
        }"#;
        let mut app = test_app(json);

        // Step 1: Execute query with optional chaining up to the array
        app.input.textarea.insert_str(".services[].capacityProviderStrategy[]?");
        app.query.execute(".services[].capacityProviderStrategy[]?");

        // This should return the object with base, weight, capacityProvider fields
        let cached_before_partial = app.query.last_successful_result_unformatted.clone();
        assert!(cached_before_partial.is_some());
        assert!(cached_before_partial.as_ref().unwrap().contains("base"));

        // Step 2: Type a dot
        app.input.textarea.insert_char('.');
        app.query.execute(".services[].capacityProviderStrategy[]?.");
        // Syntax error - cache should remain
        assert_eq!(app.query.last_successful_result_unformatted, cached_before_partial);

        // Step 3: Type partial "b"
        app.input.textarea.insert_char('b');
        app.query.execute(".services[].capacityProviderStrategy[]?.b");

        // This returns single "null" (not multiple) due to optional chaining
        // Cache should NOT be updated
        assert_eq!(app.query.last_successful_result_unformatted, cached_before_partial,
                   "Cache should not be overwritten by null result from partial field");

        // Step 4: Update autocomplete
        app.update_autocomplete();

        // Should have suggestions based on the cached object
        let suggestions = app.autocomplete.suggestions();
        assert!(!suggestions.is_empty(), "Suggestions should persist when typing partial after []?");

        // Should suggest "base" (filtered by partial "b")
        assert!(suggestions.iter().any(|s| s.text.contains("base")),
                "Should suggest 'base' field when filtering by 'b' after []?");
    }

    #[test]
    fn test_jq_keyword_autocomplete_no_dot_prefix() {
        // Test that jq keywords like "then", "else", "end" don't get a dot prefix
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Type the beginning of an if statement
        app.input.textarea.insert_str(".services | if has(\"capacityProviderStrategy\")");
        app.query.execute(".services | if has(\"capacityProviderStrategy\")");

        // Step 2: Type partial "the" to trigger autocomplete for "then"
        app.input.textarea.insert_str(" the");

        // Step 3: Accept "then" from autocomplete
        // This should NOT add a dot before "then"
        app.insert_autocomplete_suggestion("then");

        // Should produce: .services | if has("capacityProviderStrategy") then
        // NOT: .services | if has("capacityProviderStrategy") .then
        assert_eq!(app.query(), ".services | if has(\"capacityProviderStrategy\") then");
        
        // Verify no extra dot was added
        assert!(!app.query().contains(" .then"), "Should not have dot before 'then' keyword");
    }

    #[test]
    fn test_jq_keyword_else_autocomplete() {
        // Test "else" keyword autocomplete
        let json = r#"{"value": 42}"#;
        let mut app = test_app(json);

        // Type an if-then statement
        app.input.textarea.insert_str("if .value > 10 then \"high\" el");
        
        // Accept "else" from autocomplete
        app.insert_autocomplete_suggestion("else");

        // Should produce: if .value > 10 then "high" else
        // NOT: if .value > 10 then "high" .else
        assert_eq!(app.query(), "if .value > 10 then \"high\" else");
        assert!(!app.query().contains(".else"), "Should not have dot before 'else' keyword");
    }

    #[test]
    fn test_jq_keyword_end_autocomplete() {
        // Test "end" keyword autocomplete
        let json = r#"{"value": 42}"#;
        let mut app = test_app(json);

        // Type a complete if-then-else statement
        app.input.textarea.insert_str("if .value > 10 then \"high\" else \"low\" en");
        
        // Accept "end" from autocomplete
        app.insert_autocomplete_suggestion("end");

        // Should produce: if .value > 10 then "high" else "low" end
        // NOT: if .value > 10 then "high" else "low" .end
        assert_eq!(app.query(), "if .value > 10 then \"high\" else \"low\" end");
        assert!(!app.query().contains(".end"), "Should not have dot before 'end' keyword");
    }

    #[test]
    fn test_field_access_after_jq_keyword_preserves_space() {
        // Test that field access after "then" preserves the space
        // Bug: ".services[] | if has(\"x\") then .field" becomes "then.field" (no space)
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
        let mut app = test_app(json);

        // Step 1: Execute base query
        app.input.textarea.insert_str(".services[]");
        app.query.execute(".services[]");

        // Step 2: Type if-then with field access
        app.input.textarea.insert_str(" | if has(\"capacityProviderStrategy\") then .ca");

        // Step 3: Accept field suggestion (with leading dot as it would come from get_suggestions)
        app.insert_autocomplete_suggestion(".capacityProviderStrategy");

        // Should produce: .services[] | if has("capacityProviderStrategy") then .capacityProviderStrategy
        // NOT: .services[] | if has("capacityProviderStrategy") thencapacityProviderStrategy
        assert_eq!(
            app.query(),
            ".services[] | if has(\"capacityProviderStrategy\") then .capacityProviderStrategy"
        );
        
        // Verify there's a space before the field name
        assert!(app.query().contains("then .capacityProviderStrategy"), 
                "Should have space between 'then' and field name");
        assert!(!app.query().contains("thencapacityProviderStrategy"), 
                "Should NOT concatenate 'then' with field name");
    }

    #[test]
    fn test_field_access_after_else_preserves_space() {
        // Test that field access after "else" preserves the space
        let json = r#"{"services": [{"name": "test"}]}"#;
        let mut app = test_app(json);

        // Execute base query
        app.input.textarea.insert_str(".services[]");
        app.query.execute(".services[]");

        // Type if-then-else with field access
        app.input.textarea.insert_str(" | if has(\"name\") then .name else .na");

        // Accept field suggestion (with leading dot as it would come from get_suggestions)
        app.insert_autocomplete_suggestion(".name");

        // Should have space between "else" and field
        assert!(app.query().contains("else .name"), 
                "Should have space between 'else' and field name");
        assert!(!app.query().contains("elsename"), 
                "Should NOT concatenate 'else' with field name");
    }

    // ============================================================================
    // Middle Query Extraction Tests
    // ============================================================================

    #[test]
    fn test_extract_middle_query_simple_path() {
        // Simple path: no middle
        let result = App::extract_middle_query(".services.ca", ".services", ".services.ca", "ca");
        assert_eq!(result, "", "Simple path should have empty middle");
    }

    #[test]
    fn test_extract_middle_query_after_pipe() {
        // After pipe with identity - preserves trailing space
        let result = App::extract_middle_query(".services | .ca", ".services", ".services | .ca", "ca");
        assert_eq!(result, " | ", "Middle: pipe with trailing space (before dot)");
    }

    #[test]
    fn test_extract_middle_query_with_if_then() {
        // Complex: if/then between base and current field - preserves trailing space
        let query = ".services | if has(\"x\") then .ca";
        let before_cursor = query;
        let result = App::extract_middle_query(query, ".services", before_cursor, "ca");
        assert_eq!(result, " | if has(\"x\") then ", "Middle with trailing space (important for 'then ')");
    }

    #[test]
    fn test_extract_middle_query_with_select() {
        // With select function - preserves trailing space
        let query = ".items | select(.active) | .na";
        let result = App::extract_middle_query(query, ".items", query, "na");
        assert_eq!(result, " | select(.active) | ", "Middle: includes pipe with trailing space");
    }

    #[test]
    fn test_extract_middle_query_no_partial() {
        // Just typed dot, no partial yet - preserves trailing space
        let result = App::extract_middle_query(".services | .", ".services", ".services | .", "");
        assert_eq!(result, " | ", "Middle with trailing space before trigger dot");
    }

    #[test]
    fn test_extract_middle_query_base_not_prefix() {
        // Edge case: base is not prefix of current query (shouldn't happen)
        let result = App::extract_middle_query(".items.ca", ".services", ".items.ca", "ca");
        assert_eq!(result, "", "Should return empty if base not a prefix");
    }

    #[test]
    fn test_extract_middle_query_nested_pipes() {
        // Multiple pipes and functions - preserves trailing space
        let query = ".a | .b | map(.c) | .d";
        let result = App::extract_middle_query(query, ".a", query, "d");
        assert_eq!(result, " | .b | map(.c) | ", "Middle with trailing space");
    }

    #[test]
    fn test_autocomplete_inside_if_statement() {
        // Autocomplete inside complex query should only replace the local part
        let json = r#"{"services": [{"capacityProviderStrategy": [{"base": 0}]}]}"#;
        let mut app = test_app(json);

        // User types complex query with if/then
        app.input.textarea.insert_str(".services | if has(\"capacityProviderStrategy\") then .ca");

        // Execute to cache state (this will likely error due to incomplete query)
        app.query.execute(".services | if has(\"capacityProviderStrategy\") then .ca");

        // The issue: when Tab is pressed, entire query gets replaced with base + suggestion
        // Expected: only ".ca" should be replaced
        // Actual: entire query replaced with ".services[].capacityProviderStrategy"

        // TODO: This test documents the bug - we need smarter insertion
        // For now, this is a known limitation when using autocomplete inside complex expressions
    }

    #[test]
    fn test_root_field_suggestion() {
        // At root, typing "." and selecting field should replace "." with ".field"
        let json = r#"{"services": [{"name": "test"}], "status": "active"}"#;
        let mut app = test_app(json);

        // Validate initial state
        assert_eq!(app.query.base_query_for_suggestions, Some(".".to_string()),
                   "base_query should be '.' initially");
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::Object),
                   "base_type should be Object");

        // User types "."
        app.input.textarea.insert_str(".");

        // Accept suggestion ".services" (with leading dot since at root after NoOp)
        app.insert_autocomplete_suggestion(".services");

        // Should produce ".services" NOT "..services"
        assert_eq!(app.query(), ".services");

        // Verify query executes correctly
        let result = app.query.result.as_ref().unwrap();
        assert!(result.contains("name"));
    }

    #[test]
    fn test_field_suggestion_replaces_from_dot() {
        // When accepting .field suggestion at root, should replace from last dot
        let json = r#"{"name": "test", "age": 30}"#;
        let mut app = test_app(json);

        // Initial state: "." was executed during App::new()
        // Validate initial state
        assert_eq!(app.query.base_query_for_suggestions, Some(".".to_string()),
                   "base_query should be '.' initially");
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::Object),
                   "base_type should be Object for root");

        // Simulate: user typed ".na" and cursor is at end
        app.input.textarea.insert_str(".na");

        // Accept autocomplete suggestion "name" (no leading dot since after Dot)
        app.insert_autocomplete_suggestion("name");

        // Should produce .name (replace from the dot)
        assert_eq!(app.query(), ".name");
    }

    #[test]
    fn test_autocomplete_with_real_ecs_like_data() {
        // Test with data structure similar to AWS ECS services
        let json = r#"{
            "services": [
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc1", "serviceName": "service1"},
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc2", "serviceName": "service2"},
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc3", "serviceName": "service3"},
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc4", "serviceName": "service4"},
                {"serviceArn": "arn:aws:ecs:region:account:service/cluster/svc5", "serviceName": "service5"}
            ]
        }"#;
        let mut app = test_app(json);

        // Step 1: Execute ".services" to cache base
        app.input.textarea.insert_str(".services");
        app.query.execute(".services");

        // Validate cached state
        assert_eq!(app.query.base_query_for_suggestions, Some(".services".to_string()),
                   "base_query should be '.services'");
        assert_eq!(app.query.base_type_for_suggestions, Some(ResultType::ArrayOfObjects),
                   "base_type should be ArrayOfObjects");

        // Step 2: Type ".s" (partial)
        app.input.textarea.insert_str(".s");

        // Step 3: Accept "[].serviceArn" (no leading dot since after NoOp)
        app.insert_autocomplete_suggestion("[].serviceArn");

        let query_text = app.query();
        assert_eq!(query_text, ".services[].serviceArn");

        // Verify execution returns ALL 5 serviceArns
        let result = app.query.result.as_ref().unwrap();

        // Check for all service ARNs
        assert!(result.contains("svc1"));
        assert!(result.contains("svc2"));
        assert!(result.contains("svc3"));
        assert!(result.contains("svc4"));
        assert!(result.contains("svc5"));

        // Count non-null values
        let lines: Vec<&str> = result.lines().collect();
        let non_null_lines: Vec<&str> = lines.iter()
            .filter(|line| !line.trim().contains("null"))
            .copied()
            .collect();

        assert!(non_null_lines.len() >= 5, "Should have at least 5 non-null results, got {}", non_null_lines.len());
    }
}
