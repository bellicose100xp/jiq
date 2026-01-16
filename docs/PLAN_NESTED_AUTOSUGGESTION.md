# Multi-Level Nested Autosuggestion Planning Document

## Problem Statement

Currently, JIQ's autosuggestion system only provides top-level field suggestions. When a user types:

```
.field1.
```

The system suggests top-level fields from the JSON result (e.g., `field1`, `field2`, `field3`) instead of the nested fields **inside** `field1`.

### Expected Behavior

Given this JSON:
```json
{
  "user": {
    "profile": {
      "name": "John",
      "email": "john@example.com"
    },
    "settings": {
      "theme": "dark"
    }
  },
  "items": [
    {"id": 1, "name": "Item 1"},
    {"id": 2, "name": "Item 2"}
  ]
}
```

| Query Being Typed | Current Suggestions | Expected Suggestions |
|-------------------|--------------------|--------------------|
| `.` | `user`, `items` | `user`, `items` ✓ |
| `.user.` | `user`, `items` ❌ | `profile`, `settings` |
| `.user.profile.` | `user`, `items` ❌ | `name`, `email` |
| `.items[].` | `user`, `items` ❌ | `id`, `name` |
| `.items[0].` | `user`, `items` ❌ | `id`, `name` |

---

## Current Architecture Analysis

### Data Flow for Suggestions

```
┌──────────────────────────────────────────────────────────────────────┐
│ User types in query input                                            │
└────────────────────────────┬─────────────────────────────────────────┘
                             │
┌────────────────────────────▼─────────────────────────────────────────┐
│ editor_events.rs:26 → app_state.rs:202                               │
│ update_autocomplete() called after each keystroke                    │
└────────────────────────────┬─────────────────────────────────────────┘
                             │
┌────────────────────────────▼─────────────────────────────────────────┐
│ autocomplete_state.rs:6-29                                           │
│ update_suggestions_from_app() extracts:                              │
│   - query text & cursor position                                     │
│   - last_successful_result_parsed (Arc<Value>)  ◀── THE JSON DATA   │
│   - result_type (Object, Array, ArrayOfObjects, etc.)                │
└────────────────────────────┬─────────────────────────────────────────┘
                             │
┌────────────────────────────▼─────────────────────────────────────────┐
│ context.rs:366-421 get_suggestions()                                 │
│   1. analyze_context() → determines FieldContext                     │
│   2. get_field_suggestions() → calls ResultAnalyzer                  │
│      ├─ ResultAnalyzer.analyze_parsed_result(&root_value, ...)       │
│      │     └─ ❌ PROBLEM: Always analyzes ROOT, not nested path      │
│      └─ Returns suggestions for TOP-LEVEL fields only                │
└──────────────────────────────────────────────────────────────────────┘
```

### Key Files and Their Roles

| File | Role | Lines of Interest |
|------|------|-------------------|
| `autocomplete/context.rs` | Context analysis, main `get_suggestions()` | 366-421 |
| `autocomplete/result_analyzer.rs` | Extracts fields from JSON value | 38-123 |
| `autocomplete/brace_tracker.rs` | Tracks nesting context (parens, braces, brackets) | 29-195 |
| `query/query_state.rs` | Caches `last_successful_result_parsed` | 46-48, 109-112 |

### Current Limitations

1. **No Path Awareness**: `ResultAnalyzer::analyze_parsed_result()` receives the root JSON value and doesn't know what path the user has already typed.

2. **No JSON Navigation**: There's no mechanism to traverse into nested JSON structure based on the typed path.

3. **Context Loss After Dot**: When user types `.field1.`, the system detects `FieldContext` with empty partial (`""`), but doesn't extract `field1` as the path prefix.

---

## Proposed Solution Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    NEW: Path-Aware Suggestion Flow                   │
└─────────────────────────────────────────────────────────────────────┘

User types: ".user.profile."
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 1. PATH PARSER (NEW)                                                │
│    Input: ".user.profile."                                          │
│    Output: PathSegments = [Field("user"), Field("profile")]         │
│                                                                     │
│    Handles:                                                         │
│    - .field → Field("field")                                        │
│    - [] → ArrayIterator                                             │
│    - [0] → ArrayIndex(0)                                            │
│    - .field? → OptionalField("field")                               │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 2. JSON NAVIGATOR (NEW)                                             │
│    Input: root_json, PathSegments                                   │
│    Output: Option<&Value> (the nested value at path)                │
│                                                                     │
│    Navigation rules:                                                │
│    - Field("x") on Object → object["x"]                             │
│    - ArrayIterator on Array → array[0] (use first element)          │
│    - ArrayIndex(n) on Array → array[n]                              │
│    - Any segment on wrong type → None (path doesn't exist)          │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 3. RESULT ANALYZER (MODIFIED)                                       │
│    Input: nested_value (not root!), result_type                     │
│    Output: Vec<Suggestion> for fields in nested_value               │
│                                                                     │
│    Same logic as before, but operating on navigated value           │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Detailed Component Design

### Component 1: Path Parser

**Purpose**: Parse the jq path expression before the cursor into structured segments.

**Location**: New file `autocomplete/path_parser.rs`

#### Data Structures

```rust
/// A single segment in a jq path
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    /// Field access: .name, .["complex-key"]
    Field(String),

    /// Optional field access: .name?
    OptionalField(String),

    /// Array iteration: .[]
    ArrayIterator,

    /// Array index access: .[0], .[-1]
    ArrayIndex(i64),

    /// Object iteration: .{}  (rare but valid)
    ObjectIterator,
}

/// Result of parsing a path expression
#[derive(Debug, Clone)]
pub struct ParsedPath {
    /// The path segments extracted
    pub segments: Vec<PathSegment>,

    /// Whether the path ends with a dot (expecting more input)
    pub ends_with_dot: bool,

    /// The partial field name being typed (if any)
    /// e.g., ".user.na" → partial = "na"
    pub partial: String,
}
```

#### Parsing Logic

```rust
/// Parse a jq path expression into segments
///
/// Examples:
/// - ".user" → [Field("user")], ends_with_dot=false, partial=""
/// - ".user." → [Field("user")], ends_with_dot=true, partial=""
/// - ".user.na" → [Field("user")], ends_with_dot=false, partial="na"
/// - ".items[]." → [Field("items"), ArrayIterator], ends_with_dot=true
/// - ".data[0].name" → [Field("data"), ArrayIndex(0), Field("name")]
pub fn parse_path(input: &str) -> ParsedPath {
    // Implementation details below
}
```

#### Parsing State Machine

```
States:
- Start: expecting '.' or end
- AfterDot: expecting field name, '[', or end (trailing dot)
- InField: consuming field name characters
- InBracket: inside [...], expecting index, ']', or ':'
- AfterOptional: after '?', expecting '.' or end

Transitions:
- Start + '.' → AfterDot
- AfterDot + identifier_char → InField (start accumulating)
- AfterDot + '[' → InBracket
- AfterDot + end → ParsedPath with ends_with_dot=true
- InField + '.' → emit Field segment, → AfterDot
- InField + '?' → emit OptionalField, → AfterOptional
- InField + '[' → emit Field segment, → InBracket
- InField + end → partial = accumulated chars
- InBracket + ']' → emit ArrayIterator or ArrayIndex, → Start
```

#### Edge Cases to Handle

| Input | Expected Output |
|-------|-----------------|
| `.` | segments=[], ends_with_dot=true, partial="" |
| `.user` | segments=[], partial="user" |
| `.user.` | segments=[Field("user")], ends_with_dot=true |
| `.user.profile.na` | segments=[Field("user"), Field("profile")], partial="na" |
| `.items[].` | segments=[Field("items"), ArrayIterator], ends_with_dot=true |
| `.items[0].` | segments=[Field("items"), ArrayIndex(0)], ends_with_dot=true |
| `.["weird-key"].` | segments=[Field("weird-key")], ends_with_dot=true |
| `.user?.profile.` | segments=[OptionalField("user"), Field("profile")], ends_with_dot=true |
| `.[].name.` | segments=[ArrayIterator, Field("name")], ends_with_dot=true |

#### Complex Cases

1. **Bracket notation for field names**: `.["field-with-dashes"]`
   - Parse string inside brackets as field name

2. **Nested arrays**: `.data[][].name`
   - Multiple ArrayIterator segments in sequence

3. **Mixed access**: `.users[0].posts[].title`
   - Interleaved Field, ArrayIndex, ArrayIterator

4. **Pipe boundaries**: `.user | .profile.`
   - Path resets after pipe! Only parse from last `|`
   - This is CRITICAL - the path context changes after pipes

5. **Function context**: `map(.user.)`
   - Path should start from `.user.` not include `map(`
   - Already handled by `extract_partial_token()` in current system

---

### Component 2: JSON Navigator

**Purpose**: Traverse a JSON value following path segments.

**Location**: New file `autocomplete/json_navigator.rs`

#### Core Function

```rust
/// Navigate into a JSON value following the given path segments
///
/// Returns the value at the path, or None if navigation fails.
/// For arrays, uses first element (index 0) when encountering ArrayIterator.
///
/// # Arguments
/// * `root` - The root JSON value to navigate from
/// * `segments` - Path segments to follow
///
/// # Returns
/// * `Some(&Value)` - The value at the end of the path
/// * `None` - If path doesn't exist or type mismatch
pub fn navigate<'a>(root: &'a Value, segments: &[PathSegment]) -> Option<&'a Value> {
    let mut current = root;

    for segment in segments {
        current = match (segment, current) {
            // Field access on object
            (PathSegment::Field(name), Value::Object(map)) => {
                map.get(name)?
            }
            (PathSegment::OptionalField(name), Value::Object(map)) => {
                map.get(name)?
            }

            // Array iteration - use first element for suggestions
            (PathSegment::ArrayIterator, Value::Array(arr)) => {
                arr.first()?
            }

            // Array index access
            (PathSegment::ArrayIndex(idx), Value::Array(arr)) => {
                let index = if *idx < 0 {
                    // Negative index: count from end
                    let len = arr.len() as i64;
                    (len + idx) as usize
                } else {
                    *idx as usize
                };
                arr.get(index)?
            }

            // Object iteration - get first value
            (PathSegment::ObjectIterator, Value::Object(map)) => {
                map.values().next()?
            }

            // Type mismatch - path doesn't exist
            _ => return None,
        };
    }

    Some(current)
}
```

#### Result Type Detection for Navigated Value

After navigation, we need to detect the result type of the nested value:

```rust
/// Determine the ResultType for a given JSON value
pub fn detect_value_type(value: &Value) -> ResultType {
    match value {
        Value::Object(_) => ResultType::Object,
        Value::Array(arr) => {
            if arr.is_empty() {
                ResultType::Array
            } else if matches!(arr[0], Value::Object(_)) {
                ResultType::ArrayOfObjects
            } else {
                ResultType::Array
            }
        }
        Value::String(_) => ResultType::String,
        Value::Number(_) => ResultType::Number,
        Value::Bool(_) => ResultType::Boolean,
        Value::Null => ResultType::Null,
    }
}
```

---

### Component 3: Integration with Existing System

**Location**: Modifications to `autocomplete/context.rs`

#### Modified `get_suggestions()` Flow

```rust
pub fn get_suggestions(
    query: &str,
    cursor_pos: usize,
    result_parsed: Option<Arc<Value>>,
    result_type: Option<ResultType>,
    brace_tracker: &BraceTracker,
) -> Vec<Suggestion> {
    let before_cursor = &query[..cursor_pos.min(query.len())];
    let (context, partial) = analyze_context(before_cursor, brace_tracker);

    let suppress_array_brackets = brace_tracker.is_in_element_context(cursor_pos);
    let in_with_entries = brace_tracker.is_in_with_entries_context(cursor_pos);

    match context {
        SuggestionContext::FieldContext => {
            // ═══════════════════════════════════════════════════════════
            // NEW: Parse path and navigate to nested value
            // ═══════════════════════════════════════════════════════════

            let path_context = extract_path_context(before_cursor, brace_tracker);
            let parsed_path = parse_path(&path_context);

            // Navigate to nested value
            let (target_value, target_type) = if let Some(root) = result_parsed.as_ref() {
                if let Some(nested) = navigate(root, &parsed_path.segments) {
                    let nested_type = detect_value_type(nested);
                    // Clone into Arc for ResultAnalyzer (it expects Arc<Value>)
                    (Some(Arc::new(nested.clone())), Some(nested_type))
                } else {
                    // Path doesn't exist in JSON - no suggestions
                    (None, None)
                }
            } else {
                (result_parsed.clone(), result_type)
            };

            // ═══════════════════════════════════════════════════════════

            let needs_dot = needs_leading_dot(before_cursor, &partial);
            let mut suggestions = get_field_suggestions(
                target_value,      // ← NOW: nested value instead of root
                target_type,       // ← NOW: type of nested value
                needs_dot,
                suppress_array_brackets,
            );

            if in_with_entries {
                inject_with_entries_suggestions(&mut suggestions, needs_dot);
            }

            filter_suggestions_by_partial_if_nonempty(suggestions, &partial)
        }
        // ... other contexts unchanged ...
    }
}
```

#### New Function: `extract_path_context()`

This function extracts the relevant path portion from the query, handling pipes and function boundaries:

```rust
/// Extract the path context for suggestion generation
///
/// The path context is the portion of the query that represents
/// the current "navigation path" into the JSON structure.
///
/// Boundaries that reset the path context:
/// - Pipe operator `|` - after pipe, context comes from pipe input
/// - Semicolon `;` - jq expression separator
/// - Opening paren `(` - function argument start
///
/// # Examples
/// - ".user.profile." → ".user.profile."
/// - ".data | .user." → ".user."
/// - "map(.items.)" → ".items."
/// - ".users | map(.profile.)" → ".profile."
fn extract_path_context(before_cursor: &str, brace_tracker: &BraceTracker) -> String {
    // Find the last context boundary
    let boundary_chars = ['|', ';'];

    let last_boundary = before_cursor
        .char_indices()
        .rev()
        .find(|(_, ch)| boundary_chars.contains(ch))
        .map(|(pos, _)| pos + 1)
        .unwrap_or(0);

    // Also consider function context from brace_tracker
    // If inside map(), select(), etc., the path starts from inside the function

    let path_start = last_boundary;
    before_cursor[path_start..].trim_start().to_string()
}
```

---

## Special Cases and Edge Cases

### Case 1: Pipe Operator Resets Context

```
Query: .users | map(.profile.)
JSON: {"users": [{"profile": {"name": "John"}}]}

Path context for suggestions: ".profile."
Navigate from: The RESULT of ".users | map(...)"

PROBLEM: We don't have the result of the pipe - we have root JSON!
```

**Solution Options**:

**Option A: Hybrid Approach (Recommended)**
- For simple paths (no pipes), navigate directly in cached root JSON
- For paths after pipe, fall back to current behavior (top-level of last successful result)
- The `last_successful_result_parsed` already contains the result of the last successful query

**Option B: Re-execute Partial Query**
- Execute the query up to the last pipe to get intermediate result
- Use that result for navigation
- EXPENSIVE - adds jq execution on every keystroke

**Option C: Maintain Execution Context Stack**
- Track intermediate results as user builds query
- Complex state management

**Recommendation**: Start with Option A. It handles the majority of cases (direct nested access) without complexity. Pipe scenarios already work "okay" with current system.

### Case 2: Element Context Functions

Inside `map()`, `select()`, etc., the context already represents array elements:

```
Query: .items | map(.name.)
JSON: {"items": [{"name": {"first": "John", "last": "Doe"}}]}

Without element context: Would try to find .name in root
With element context: Correctly knows we're iterating .items elements
```

**Integration**: The `BraceTracker` already detects element context. We need to:
1. When in element context AND the path starts with `.`, navigate from array's first element
2. Combine with path parser for nested access within element context

### Case 3: Array Index vs Iterator

```
Query: .items[0].
Query: .items[].

Both should suggest fields of items' objects, but:
- [0] navigates to specific index
- [] navigates to first element (for suggestions)
```

Both cases produce same suggestions - this is correct behavior.

### Case 4: Mixed Array Depths

```
Query: .data[][].name.
JSON: {"data": [[{"name": {"first": "A"}}]]}

Path: [Field("data"), ArrayIterator, ArrayIterator, Field("name")]
Navigation: data → data[0] → data[0][0] → data[0][0].name
```

Each `ArrayIterator` dives into first element of current array.

### Case 5: Non-Existent Path

```
Query: .nonexistent.field.
JSON: {"user": {"name": "John"}}

Navigation returns None → No suggestions (field doesn't exist)
```

This is correct - don't suggest anything for paths that don't exist.

### Case 6: Optional Access Chain

```
Query: .user?.profile?.
JSON: {"user": {"profile": {"name": "John"}}}

Optional marker `?` doesn't change navigation for suggestions.
We navigate as if the field exists (for suggestion purposes).
```

### Case 7: Bracket Notation for Field Names

```
Query: .["user-data"].profile.
JSON: {"user-data": {"profile": {"name": "John"}}}

Path parser must handle bracket string notation.
```

---

## Implementation Phases

### Phase 1: Path Parser (Foundation)

**Deliverables**:
- `autocomplete/path_parser.rs` with `parse_path()` function
- `PathSegment` and `ParsedPath` types
- Comprehensive unit tests for all path patterns

**Test Cases**:
```rust
#[test] fn test_simple_field() { ... }
#[test] fn test_nested_fields() { ... }
#[test] fn test_array_iterator() { ... }
#[test] fn test_array_index() { ... }
#[test] fn test_optional_field() { ... }
#[test] fn test_bracket_notation() { ... }
#[test] fn test_trailing_dot() { ... }
#[test] fn test_partial_field() { ... }
#[test] fn test_complex_mixed_path() { ... }
```

### Phase 2: JSON Navigator

**Deliverables**:
- `autocomplete/json_navigator.rs` with `navigate()` function
- `detect_value_type()` helper
- Unit tests for navigation scenarios

**Test Cases**:
```rust
#[test] fn test_navigate_simple_field() { ... }
#[test] fn test_navigate_nested_fields() { ... }
#[test] fn test_navigate_array_first_element() { ... }
#[test] fn test_navigate_array_index() { ... }
#[test] fn test_navigate_nonexistent_path() { ... }
#[test] fn test_navigate_type_mismatch() { ... }
#[test] fn test_navigate_empty_array() { ... }
```

### Phase 3: Integration

**Deliverables**:
- Modified `context.rs` with path-aware suggestion flow
- `extract_path_context()` function
- Integration tests with full App context

**Test Cases**:
```rust
#[test] fn test_nested_field_suggestions() { ... }
#[test] fn test_array_element_field_suggestions() { ... }
#[test] fn test_deep_nesting_suggestions() { ... }
#[test] fn test_pipe_context_boundary() { ... }
#[test] fn test_function_context_integration() { ... }
```

### Phase 4: Edge Cases and Polish

**Deliverables**:
- Handle pipe operator edge cases
- Optimize for large JSON files (avoid cloning when possible)
- Performance testing and optimization

---

## Performance Considerations

### Current Performance Profile

The autocomplete system is called on **every keystroke**. Current optimizations:
- `Arc<Value>` for parsed JSON (no re-parsing)
- Pre-rendered results cached
- Minimal allocations in hot path

### New Performance Concerns

1. **Path Parsing**: Must be fast - called every keystroke
   - Use zero-allocation parsing where possible
   - Return string slices instead of owned strings when feasible

2. **JSON Navigation**: Must not clone the entire nested subtree
   - Return `&Value` reference, not owned `Value`
   - Only clone when passing to `ResultAnalyzer` (unavoidable with current API)

3. **Memory**: Large nested objects could be cloned repeatedly
   - Consider modifying `ResultAnalyzer` to take `&Value` instead of `Arc<Value>`
   - Or cache navigated results (but cache invalidation is complex)

### Optimization Strategy

**Immediate (Phase 1-3)**:
- Keep it simple, profile first
- Use `&Value` references throughout navigation
- Accept one clone when passing to `ResultAnalyzer`

**Future (if needed)**:
- Modify `ResultAnalyzer` API to accept `&Value`
- Cache navigation results keyed by path string
- Lazy evaluation of suggestions

---

## Testing Strategy

### Unit Tests

Each new module should have comprehensive unit tests:

```
autocomplete/path_parser_tests.rs
  ├── test_empty_input
  ├── test_root_dot_only
  ├── test_single_field
  ├── test_nested_fields
  ├── test_array_iterator
  ├── test_array_index_positive
  ├── test_array_index_negative
  ├── test_optional_field
  ├── test_bracket_string_field
  ├── test_mixed_complex_path
  ├── test_trailing_dot_detection
  └── test_partial_field_extraction

autocomplete/json_navigator_tests.rs
  ├── test_navigate_root
  ├── test_navigate_single_field
  ├── test_navigate_nested_fields
  ├── test_navigate_array_iterator
  ├── test_navigate_array_index
  ├── test_navigate_type_mismatch
  ├── test_navigate_nonexistent_field
  └── test_navigate_empty_structures
```

### Integration Tests

Test the full suggestion flow:

```rust
#[test]
fn test_nested_suggestions_user_profile() {
    let json = r#"{"user": {"profile": {"name": "John", "age": 30}}}"#;
    let app = app_with_json(json);

    // Type ".user.profile."
    simulate_typing(&mut app, ".user.profile.");

    let suggestions = app.autocomplete.suggestions();
    assert!(suggestions.iter().any(|s| s.text == "name"));
    assert!(suggestions.iter().any(|s| s.text == "age"));
    assert!(!suggestions.iter().any(|s| s.text == "user")); // NOT top-level
}
```

### Manual Testing Checklist

Before release, manually verify:

- [ ] Simple nested field: `.user.` suggests user's fields
- [ ] Deep nesting: `.a.b.c.` suggests c's fields
- [ ] Array iteration: `.items[].` suggests item fields
- [ ] Array index: `.items[0].` suggests item fields
- [ ] Mixed: `.data[].user.profile.` suggests profile fields
- [ ] Non-existent path: `.fake.` shows no suggestions
- [ ] After pipe: `.data | .` behaves correctly
- [ ] In map(): `map(.field.)` suggests field's nested fields
- [ ] Large JSON file: Performance is acceptable

---

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Performance regression | High | Medium | Profile early, optimize hot paths |
| Breaking existing suggestions | High | Low | Comprehensive test coverage |
| Complex edge cases in jq syntax | Medium | High | Incremental implementation, skip exotic syntax |
| Memory usage increase | Medium | Low | Use references, avoid cloning |
| Pipe context confusion | Medium | Medium | Clear documentation, sensible defaults |

---

## Open Questions

1. **Pipe behavior**: Should we try to evaluate partial queries to get intermediate results? Or accept that pipes reset context to "last successful result"?

2. **Error tolerance**: If path parsing fails partway, should we:
   - Return no suggestions?
   - Fall back to top-level suggestions?
   - Suggest from last valid path segment?

3. **Optional access (`?`)**: Should we suggest fields even when the path might be null at runtime? (Current recommendation: Yes, for better UX)

4. **Recursive structures**: How to handle circular references or very deep nesting? (Current recommendation: Navigate up to reasonable depth, e.g., 20 levels)

5. **Multiple JSON values**: For destructured output (`{"a":1}\n{"b":2}`), which value to use for navigation? (Current: First value, same as existing behavior)

---

## Success Criteria

The feature is complete when:

1. ✅ Typing `.field.` suggests fields inside `field`, not top-level fields
2. ✅ Array access (`.items[].` and `.items[0].`) suggests element fields
3. ✅ Deep nesting (`.a.b.c.d.`) works correctly
4. ✅ Existing functionality (top-level suggestions, function suggestions, variable suggestions) unchanged
5. ✅ Performance is acceptable (no perceptible lag on keystroke)
6. ✅ All existing tests pass
7. ✅ New tests cover nested suggestion scenarios

---

## Appendix: jq Path Syntax Reference

For reference, valid jq field access patterns:

| Pattern | Meaning |
|---------|---------|
| `.foo` | Access field "foo" |
| `.foo.bar` | Nested field access |
| `.foo?` | Optional field access (null if missing) |
| `.["foo"]` | String key access (same as .foo) |
| `.["foo-bar"]` | String key with special chars |
| `.[0]` | Array index access |
| `.[-1]` | Negative index (from end) |
| `.[]` | Iterate all array elements |
| `.[]?` | Optional iteration |
| `.foo[]` | Access foo, then iterate |
| `.foo[].bar` | Iterate foo, access bar on each |

Patterns we DON'T need to handle for suggestions:
- `.foo[1:3]` - Slice syntax (result is array, suggest [])
- `.foo | .bar` - Pipe resets context
- `.foo as $x | .bar` - Variable binding
- `..` - Recursive descent
