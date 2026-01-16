# Multi-Level Nested Autosuggestion Planning Document

> **Document Version**: 1.3

---

## Critical Design Corrections (Post-Review)

After deep analysis of the codebase, several critical issues were identified that must be addressed:

### Issue 1: Cache Contains Query Result, NOT Root JSON

**Problem**: The plan originally assumed we could navigate from `last_successful_result_parsed`. However, this cache is updated with EVERY successful query result:

```rust
// query_state.rs:220-221
self.last_successful_result_parsed = Self::parse_first_value(&unformatted).map(Arc::new);
```

If user's last successful query was `.users`, then `last_successful_result_parsed` contains the users array, NOT the root object. Navigating `.config.database.` from a users array would fail.

**Solution**: Add a new field to `QueryState`:

```rust
pub struct QueryState {
    // ... existing fields ...

    /// Original root JSON, parsed once at initialization.
    /// Used for path navigation in autocomplete - NEVER overwritten.
    pub original_json_parsed: Option<Arc<Value>>,
}
```

Initialize in `QueryState::new()`:
```rust
// Parse the initial "." query result as the original root
let original_json_parsed = last_successful_result_parsed.clone();
```

### Issue 2: ResultAnalyzer API Takes Arc, Requires Clone

**Problem**: `ResultAnalyzer::analyze_parsed_result()` takes `&Arc<Value>`:

```rust
// result_analyzer.rs:38-43
pub fn analyze_parsed_result(
    value: &Arc<Value>,  // ← Takes Arc reference
    result_type: ResultType,
    // ...
) -> Vec<Suggestion>
```

To pass a navigated `&Value`, we'd need to clone and wrap in Arc, which is expensive for large nested structures.

**Solution**: Modify the API to accept `&Value` directly:

```rust
pub fn analyze_parsed_result(
    value: &Value,  // ← Changed to plain reference
    result_type: ResultType,
    // ...
) -> Vec<Suggestion>
```

This is safe because the internal `extract_suggestions_for_type()` already takes `&Value`. The change is API-only, no logic changes needed.

### Issue 3: Element Context Needs Implicit Array Navigation

**Problem**: Inside `map(.field.)`, the BraceTracker correctly detects element context, but the path `.field.` doesn't include the implicit array iteration.

**Example**:
```
Input JSON: {"items": [{"name": {"first": "John"}}]}
Query: .items | map(.name.)
```

- BraceTracker detects: inside `map()` (element context)
- User typed path: `.name.`
- What we need to navigate: The `name` field of array ELEMENTS

**Solution**: When in element context AND navigating from root, prepend implicit `ArrayIterator`:

```rust
fn get_navigation_path(
    parsed_path: &ParsedPath,
    brace_tracker: &BraceTracker,
    cursor_pos: usize,
) -> Vec<PathSegment> {
    let mut segments = parsed_path.segments.clone();

    // If in element context (map, select, etc.), the input is implicitly
    // iterating an array. Prepend ArrayIterator for correct navigation.
    if brace_tracker.is_in_element_context(cursor_pos) {
        segments.insert(0, PathSegment::ArrayIterator);
    }

    segments
}
```

### Issue 4: Pipe Handling Strategy Clarification

**Problem**: Pipes reset the evaluation context. After `.users | .profile.`, the `.profile.` operates on the result of `.users`, not the root.

**MVP Strategy (Hybrid Approach)**:

| Scenario | Navigation Source | Rationale |
|----------|-------------------|-----------|
| No pipe in query | `original_json_parsed` | Direct path navigation from root |
| Pipe present | `last_successful_result_parsed` | Cache likely has pipe's input result |

```rust
fn get_navigation_source(
    query: &str,
    original_json: &Arc<Value>,
    cached_result: &Option<Arc<Value>>,
) -> &Value {
    if query.contains('|') {
        // Pipe present - use cached result (from last successful query)
        cached_result.as_deref().unwrap_or(original_json)
    } else {
        // No pipe - navigate from original root
        original_json
    }
}
```

**Future Enhancement**: Track intermediate results at pipe boundaries for more accurate suggestions.

---

## State Summary

Quick reference for all tracked states that affect suggestion behavior:

| State | Values | Determines |
|-------|--------|------------|
| **Execution Context** | Executing / Non-Executing | Whether cache updates automatically |
| **Certainty** | Deterministic / Non-Deterministic | Whether we can navigate path accurately |
| **Element Context** | Inside / Outside | Whether to prepend implicit `ArrayIterator` |
| **Builder Context** | Array `[...]` / Object `{...}` / None | Expression boundary detection |
| **Pipe Context** | Has Pipe / No Pipe | Navigation source (cache vs root) |
| **Cursor Position** | End / Middle | Path extraction scope |

### State Definitions

**Execution Context**
- *Executing*: Query runs on each keystroke, cache updates (standard `.field.` access)
- *Non-Executing*: Inside `map()`, `select()`, builders - cache doesn't update

**Certainty**
- *Deterministic*: Path exists in JSON → suggest target's fields
- *Non-Deterministic*: Path fails OR after transforming function → suggest root fields

**Element Context**
- *Inside*: Within `map()`, `select()`, `sort_by()`, etc. → prepend `ArrayIterator`
- *Outside*: Normal context → use path as-is

**Builder Context**
- *Array*: Inside `[...]` → boundary at `[` or `,`
- *Object*: Inside `{...}` → boundary at `:` or `,`
- *None*: Top-level → boundary at `|`, `;`, or start

**Pipe Context**
- *Has Pipe*: Use `last_successful_result_parsed` (cache has transformed data)
- *No Pipe*: Use `original_json_parsed` (navigate from root)

**Cursor Position**
- *End*: `query▎` → extract full path before cursor
- *Middle*: `que▎ry` → extract path up to cursor, ignore text after

Middle-of-query editing is **not** inherently non-deterministic. We extract the path up to cursor position, then apply normal determinism rules.

---

## Problem Statement

In standard field access (`.user.profile.`), suggestions work correctly because each intermediate query executes and updates the cache. However, suggestions fail in **non-executing contexts**:

### Failing Contexts

Given this JSON:
```json
{
  "users": [{"profile": {"name": "John", "age": 30}}],
  "config": {"db": {"host": "localhost"}}
}
```

| Context | Query | Current | Expected |
|---------|-------|---------|----------|
| `map()` | `map(.profile.)` | top-level fields | `name`, `age` |
| `select()` | `select(.profile.)` | top-level fields | `name`, `age` |
| Array builder | `[.config.db.]` | top-level fields | `host` |
| Object builder | `{x: .config.db.}` | top-level fields | `host` |

### Root Cause

In these contexts, the intermediate path (`.config.db`) never executes as a standalone query, so `last_successful_result_parsed` still contains the previous result (often root JSON). The system has no mechanism to navigate into nested structures based on the typed path.

---

## Current Architecture

```
editor_events.rs:26 → update_autocomplete()
    ↓
autocomplete_state.rs:6-29 → extracts last_successful_result_parsed
    ↓
context.rs:366-421 → get_suggestions()
    ↓
result_analyzer.rs:38-123 → analyzes cached value (NOT navigated path)
```

**Key files**:
- `context.rs` - Context detection, suggestion generation
- `result_analyzer.rs` - Field extraction from JSON value
- `brace_tracker.rs` - Tracks `()`, `[]`, `{}` nesting and function context
- `query_state.rs` - Caches `last_successful_result_parsed`

---

## Solution

### New Components

**1. Path Parser** (`autocomplete/path_parser.rs`)

```rust
pub enum PathSegment {
    Field(String),          // .name
    OptionalField(String),  // .name?
    ArrayIterator,          // .[]
    ArrayIndex(i64),        // .[0]
}

pub struct ParsedPath {
    pub segments: Vec<PathSegment>,
    pub partial: String,  // incomplete field being typed
}

/// Parse ".user.profile." → [Field("user"), Field("profile")], partial=""
/// Parse ".user.prof" → [Field("user")], partial="prof"
pub fn parse_path(input: &str) -> ParsedPath
```

**2. JSON Navigator** (`autocomplete/json_navigator.rs`)

```rust
/// Navigate JSON tree following path segments.
/// ArrayIterator uses first element (industry standard).
/// Returns None if path doesn't exist.
pub fn navigate<'a>(root: &'a Value, segments: &[PathSegment]) -> Option<&'a Value> {
    let mut current = root;
    for segment in segments {
        current = match (segment, current) {
            (PathSegment::Field(name), Value::Object(map)) => map.get(name)?,
            (PathSegment::ArrayIterator, Value::Array(arr)) => arr.first()?,
            (PathSegment::ArrayIndex(i), Value::Array(arr)) => arr.get(*i as usize)?,
            _ => return None,
        };
    }
    Some(current)
}
```

### Integration (`context.rs`)

Modified `get_suggestions()` flow:

```rust
SuggestionContext::FieldContext => {
    let path_context = extract_path_context(before_cursor, brace_tracker);
    let parsed_path = parse_path(&path_context);

    let (target, target_type) = if let Some(nested) = navigate(original_json, &parsed_path.segments) {
        (nested, detect_value_type(nested))
    } else {
        // Fallback: show root fields if path doesn't exist
        (original_json, detect_value_type(original_json))
    };

    get_field_suggestions(target, target_type, ...)
}
```

### Expression Boundaries

Path context resets at expression boundaries. Use `find_expression_start()`:

```rust
/// Find where current expression starts (for path extraction)
fn find_expression_start(before_cursor: &str, brace_tracker: &BraceTracker) -> usize {
    // Check innermost context from BraceTracker
    match brace_tracker.innermost_context() {
        Some(BraceType::Paren) => // Inside function: start after '('
        Some(BraceType::Square) => // Array builder: start after '[' or last ','
        Some(BraceType::Curly) => // Object builder: start after ':' or last ','
        None => // Top-level: start after '|' or ';' or beginning
    }
}
```

| Context | Boundary | Example |
|---------|----------|---------|
| Top-level | `\|`, `;`, start | `.a \| .b.c.` → `.b.c.` |
| Function | `(` | `map(.user.)` → `.user.` |
| Array builder | `[`, `,` | `[.a, .b.c.]` → `.b.c.` |
| Object builder | `:`, `,` | `{x: .a.b.}` → `.a.b.` |

---

## Context Types

### Executing Context (Standard)

Query executes, cache updates automatically. Nested suggestions work via cache.

```
.user.profile.    ← Each intermediate query executes
```

### Non-Executing Context (Requires Navigation)

Query doesn't execute independently. Must navigate from `original_json_parsed`.

| Context | Detection | Navigation Source |
|---------|-----------|-------------------|
| `map()`, `select()` | `is_in_element_context()` | `original_json[0]` (array element) |
| Array builder `[...]` | `is_in_array_builder()` | `original_json` |
| Object builder `{...}` | `is_in_object()` + after `:` | `original_json` |

### Element Context (Special Case)

Inside `map()`, `select()`, etc., prepend implicit `ArrayIterator`:

```rust
if brace_tracker.is_in_element_context(cursor_pos) {
    segments.insert(0, PathSegment::ArrayIterator);
}
```

---

## Suggestion Certainty: Deterministic vs Non-Deterministic

Not all contexts allow accurate path navigation. We classify contexts by whether we can **deterministically** know the data type:

### Deterministic Contexts

We can navigate the path and provide **targeted suggestions**:

| Context | Example | Why Deterministic |
|---------|---------|-------------------|
| Simple field path | `.user.profile.` | Direct navigation through known structure |
| Array iteration | `.items[].name.` | First element provides field schema |
| Element-context functions | `map(.profile.)` | Input is array, navigate first element |
| Nested in builders | `{x: .config.db.}` | Path from root is known |

**Behavior**: Navigate path → suggest fields of target object.

### Non-Deterministic Contexts

We **cannot** know the result type. Fall back to **all available suggestions**:

| Context | Example | Why Non-Deterministic |
|---------|---------|----------------------|
| After transforming functions | `keys \| .` | `keys` returns `[string]`, unknown field names |
| After `to_entries` | `to_entries \| .[].` | Structure is `{key, value}` not original |
| After `group_by` | `group_by(.x) \| .[].` | Nested arrays, unknown structure |
| After pipe with complex expr | `.a + .b \| .` | Result type depends on runtime values |
| Path navigation fails | `.nonexistent.` | Target doesn't exist in JSON |
| After conditionals | `if .x then .a else .b end \| .` | Branch depends on runtime |

**Behavior**: Fall back to root-level field suggestions (graceful degradation).

### Detection Logic

```rust
enum SuggestionCertainty {
    Deterministic,      // Navigate and suggest target fields
    NonDeterministic,   // Fall back to root fields
}

fn determine_certainty(
    path_context: &str,
    brace_tracker: &BraceTracker,
    navigation_result: Option<&Value>,
) -> SuggestionCertainty {
    // Non-deterministic if navigation failed
    if navigation_result.is_none() {
        return SuggestionCertainty::NonDeterministic;
    }

    // Non-deterministic if preceded by transforming function
    let transforming_functions = ["keys", "keys_unsorted", "to_entries",
                                   "from_entries", "group_by", "unique_by",
                                   "flatten", "transpose", "combinations"];

    if preceded_by_any(path_context, &transforming_functions) {
        return SuggestionCertainty::NonDeterministic;
    }

    SuggestionCertainty::Deterministic
}
```

### Summary Table

| Certainty | Navigation | Suggestions |
|-----------|------------|-------------|
| Deterministic | Path exists in JSON | Target object's fields |
| Non-Deterministic | Path fails OR transforming function | Root-level fields (fallback) |

---

## Edge Cases

| Case | Handling |
|------|----------|
| `.items[0].` vs `.items[].` | Both → first element (same suggestions) |
| `.data[][].name.` | Chain ArrayIterators: `data[0][0].name` |
| `.nonexistent.` | Fallback to root fields (graceful degradation) |
| `.user?.profile?.` | Ignore `?` for navigation |
| `.["field-name"].` | Parse bracket notation as field |

---

## Performance Guarantees

### Zero Query Execution

**Critical constraint**: This feature must NEVER execute jq queries for suggestions.

All operations work on **pre-parsed, cached JSON** (`original_json_parsed: Arc<Value>`):

| Operation | What it does | Complexity |
|-----------|--------------|------------|
| Path parsing | String scan for `.`, `[]`, field names | O(query_length) |
| JSON navigation | Follow pointers in parsed tree | O(path_depth) ≈ O(5) |
| Type detection | Check `Value` variant, peek first array element | O(1) |
| Field extraction | Iterate object keys | O(num_fields) |
| Suggestion filtering | String prefix match | O(suggestions × partial_length) |

**Total per-keystroke cost**: O(query_length + num_fields)

This is **identical** to current behavior - we just navigate to a different starting point in the same JSON tree.

### Memory: No Cloning

With the API change (`&Value` instead of `Arc<Value>`), we pass **references** throughout:

```rust
// Navigation returns borrowed reference - no allocation
fn navigate<'a>(root: &'a Value, segments: &[PathSegment]) -> Option<&'a Value>

// Analyzer takes borrowed reference - no clone needed
fn analyze_parsed_result(value: &Value, ...) -> Vec<Suggestion>
```

The only allocations are:
1. `Vec<PathSegment>` - typically 1-5 elements
2. `Vec<Suggestion>` - same as current behavior

### Benchmarking Targets

Before merging, verify:

| Metric | Target | How to measure |
|--------|--------|----------------|
| Keystroke latency | < 5ms p99 | Profile `update_suggestions()` |
| Memory per keystroke | < 1KB additional | Heap profiling |
| Large file (10MB JSON) | No regression | Compare before/after |
| Deep nesting (10 levels) | < 10ms | Synthetic benchmark |

```rust
#[bench]
fn bench_nested_path_navigation() {
    // Navigate 10 levels deep, measure time
}

#[bench]
fn bench_path_parsing() {
    // Parse ".a.b.c.d.e.f.g.h.i.j.", measure time
}
```

---

## Implementation Phases

### Phase 0: Infrastructure Prerequisites (Critical)

**Must be completed first** - these changes enable the core feature.

**Deliverables**:

1. **Add `original_json_parsed` to QueryState** (`query/query_state.rs`):
   ```rust
   pub struct QueryState {
       // ... existing fields ...
       pub original_json_parsed: Option<Arc<Value>>,
   }
   ```

2. **Initialize in `QueryState::new()`**:
   ```rust
   let original_json_parsed = last_successful_result_parsed.clone();
   ```

3. **Modify ResultAnalyzer API** (`autocomplete/result_analyzer.rs`):
   - Change `analyze_parsed_result(&Arc<Value>, ...)` to `analyze_parsed_result(&Value, ...)`
   - Update all call sites (minimal changes - just remove Arc dereferencing)

4. **Pass `original_json_parsed` to autocomplete** (`autocomplete_state.rs`):
   ```rust
   pub fn update_suggestions_from_app(app: &mut App) {
       // ...
       let original_json = query_state.original_json_parsed.clone();
       // Pass to update_suggestions
   }
   ```

**Test Cases**:
```rust
#[test] fn test_original_json_preserved_after_queries() { ... }
#[test] fn test_result_analyzer_accepts_value_reference() { ... }
```

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

### Regression Tests (Critical)

Ensure existing functionality remains unchanged:

```rust
// autocomplete/regression_tests.rs

#[test]
fn test_top_level_suggestions_unchanged() {
    // Verify that "." still suggests top-level fields correctly
    let json = r#"{"name": "test", "value": 42}"#;
    let app = app_with_json(json);
    simulate_typing(&mut app, ".");

    let suggestions = app.autocomplete.suggestions();
    assert!(suggestions.iter().any(|s| s.text == "name" || s.text == ".name"));
    assert!(suggestions.iter().any(|s| s.text == "value" || s.text == ".value"));
}

#[test]
fn test_function_suggestions_unchanged() {
    // Verify that function context still works
    let app = app_with_json("{}");
    simulate_typing(&mut app, "sel");

    let suggestions = app.autocomplete.suggestions();
    assert!(suggestions.iter().any(|s| s.text == "select"));
}

#[test]
fn test_variable_suggestions_unchanged() {
    // Verify that variable suggestions still work
    let app = app_with_json("{}");
    simulate_typing(&mut app, ". as $x | $");

    let suggestions = app.autocomplete.suggestions();
    assert!(suggestions.iter().any(|s| s.text == "$x"));
}

#[test]
fn test_array_of_objects_iteration_unchanged() {
    // Verify .[].field suggestions for arrays of objects
    let json = r#"[{"id": 1}, {"id": 2}]"#;
    let app = app_with_json(json);
    simulate_typing(&mut app, ".");

    let suggestions = app.autocomplete.suggestions();
    assert!(suggestions.iter().any(|s| s.text.contains("[]")));
    assert!(suggestions.iter().any(|s| s.text.contains("id")));
}

#[test]
fn test_with_entries_context_unchanged() {
    // Verify .key/.value suggestions in with_entries
    let app = app_with_json(r#"{"a": 1}"#);
    simulate_typing(&mut app, "with_entries(.");

    let suggestions = app.autocomplete.suggestions();
    assert!(suggestions.iter().any(|s| s.text.contains("key")));
    assert!(suggestions.iter().any(|s| s.text.contains("value")));
}

#[test]
fn test_map_element_context_unchanged() {
    // Verify suggestions inside map() use element context
    let json = r#"[{"name": "test"}]"#;
    let app = app_with_json(json);
    simulate_typing(&mut app, "map(.");

    let suggestions = app.autocomplete.suggestions();
    // Should suggest .name (element field), not .[].name
    assert!(suggestions.iter().any(|s| s.text == "name" || s.text == ".name"));
}
```

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

1. ✅ `map(.field.)` suggests nested fields inside `field`
2. ✅ Array builder `[.a.b.]` suggests fields inside `b`
3. ✅ Object builder `{x: .a.b.}` suggests fields inside `b`
4. ✅ Deep nesting in non-executing contexts works correctly
5. ✅ Existing suggestions unchanged (top-level, functions, variables)
6. ✅ No perceptible latency on keystroke
7. ✅ All existing tests pass

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

---

## Appendix: Industry Research

Brief survey of existing jq/JSON autocomplete implementations:

| Project | Approach | Key Insight |
|---------|----------|-------------|
| [jq-lsp](https://github.com/wader/jq-lsp) | gojq parser for AST | Lists "broken syntax" and "input completion" as unsolved TODOs |
| [Monaco Editor](https://gist.github.com/mwrouse/05d8c11cd3872c19c684bd1904a2202e) | Split path, traverse sequentially, use `[0]` for arrays | Notes performance concerns with full traversal |
| [vscode-yaml #621](https://github.com/redhat-developer/vscode-yaml/issues/621) | Bug report | Exact issue we're solving: nested array suggestions show root fields |

**Key takeaways applied to our design**:
- Navigate JSON directly (don't parse incomplete jq syntax)
- Use first array element for field suggestions (industry standard)
- Fall back to root fields on navigation failure (graceful degradation)
- Use zero-copy references to avoid Monaco's performance concerns
