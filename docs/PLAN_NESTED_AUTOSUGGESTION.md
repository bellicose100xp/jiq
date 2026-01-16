# Multi-Level Nested Autosuggestion Planning Document

---

## State Summary

Quick reference for all tracked states that affect suggestion behavior:

| State | Values | Determines |
|-------|--------|------------|
| **Execution Context** | Executing / Non-Executing | Whether cache updates automatically |
| **Certainty** | Deterministic / Non-Deterministic | Whether we can navigate path accurately |
| **Element Context** | Iterator-Scoped / Value-Scoped | Whether to prepend implicit `ArrayIterator` |
| **Builder Context** | Array `[...]` / Object `{...}` / None | Expression boundary detection |
| **Cursor Position** | End / Middle | Path extraction scope |

### State Definitions

**Execution Context**
- *Executing*: Query runs on each keystroke, cache updates (standard `.field.` access)
- *Non-Executing*: Inside `map()`, `select()`, builders - cache doesn't update

**Certainty** (determined by navigation result)
- *Deterministic*: Navigation succeeds → suggest target's fields
- *Non-Deterministic*: Navigation fails → fall back to `original_json_parsed`, show all available suggestions for syntax context

**Element Context**
- *Iterator-Scoped*: Within `map()`, `select()`, `sort_by()`, etc. — input is array element → prepend `ArrayIterator`
- *Value-Scoped*: Standard context — input is full value → use path as-is

**Builder Context**
- *Array*: Inside `[...]` → boundary at `[` or `,`
- *Object*: Inside `{...}` → boundary at `:` or `,`
- *None*: Top-level → boundary at `|`, `;`, or start

**Cursor Position** (affects data source selection)
- *End*: `query▎` → cache is current, try `last_successful_result_parsed` first
- *Middle*: `que▎ry` → cache is "ahead" of cursor, use `original_json_parsed` only

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

**Relationship with existing `analyze_context()`**:

The existing `analyze_context(before_cursor, brace_tracker)` function remains the **entry point** for all context detection. It determines the `SuggestionContext` type:

| Context Type | Current Behavior | Change |
|--------------|------------------|--------|
| `FieldContext` | Suggests fields from cache | **Enhanced**: Uses path extraction + navigation |
| `FunctionContext` | Suggests jq functions | Unchanged |
| `VariableContext` | Suggests defined variables | Unchanged |
| `ObjectKeyContext` | Suggests object keys | Unchanged |
| `IndexContext` | Suggests array operations | Unchanged |

**Key point**: `extract_path_context()` is called **only within FieldContext**, after `analyze_context()` has already determined we're in field context. Other contexts bypass path extraction entirely and use existing behavior.

```
analyze_context()              ← Entry point (unchanged)
    │
    ├─► FieldContext           ← Path extraction applies here
    │       └─► extract_path_context() → parse_path() → navigate()
    │
    ├─► FunctionContext        ← Existing behavior (unchanged)
    ├─► VariableContext        ← Existing behavior (unchanged)
    ├─► ObjectKeyContext       ← Existing behavior (unchanged)
    └─► IndexContext           ← Existing behavior (unchanged)
```

Modified `get_suggestions()` flow:

```rust
SuggestionContext::FieldContext => {
    let is_cursor_at_end = cursor_pos == query.len();
    let is_executing_context = !brace_tracker.is_in_non_executing_context();

    if is_executing_context && is_cursor_at_end {
        // EXECUTING CONTEXT: Cache is current, suggest its fields directly
        get_field_suggestions(last_successful_result, ...)
    } else if is_cursor_at_end {
        // NON-EXECUTING CONTEXT: Cache is stale, extract path and navigate
        let path_context = extract_path_context(before_cursor, brace_tracker);
        let parsed_path = parse_path(&path_context);

        if let Some(nested) = navigate(last_successful_result, &parsed_path.segments) {
            get_field_suggestions(nested, ...)
        } else {
            // Navigation failed: fall back to original_json
            get_all_available_suggestions(original_json, partial_filter)
        }
    } else {
        // MIDDLE OF QUERY: Cache is "ahead", navigate from original_json
        let path_context = extract_path_context(before_cursor, brace_tracker);
        let parsed_path = parse_path(&path_context);

        if let Some(nested) = navigate(original_json, &parsed_path.segments) {
            get_field_suggestions(nested, ...)
        } else {
            get_all_available_suggestions(original_json, partial_filter)
        }
    }
}
```

### Expression Boundaries (Non-Executing Contexts Only)

In **non-executing contexts** (map, select, builders), the cache is stale. We extract path from expression boundary and navigate.

```rust
/// Find where current expression starts (for path extraction)
/// ONLY used in non-executing contexts
fn find_expression_start(before_cursor: &str, brace_tracker: &BraceTracker) -> usize {
    match brace_tracker.innermost_context() {
        Some(BraceType::Paren) => // Inside function: start after '('
        Some(BraceType::Square) => // Array builder: start after '[' or last ','
        Some(BraceType::Curly) => // Object builder: start after ':' or last ','
        None => // Should not reach here in non-executing context
    }
}
```

| Context | Boundary | Example | Extracted Path |
|---------|----------|---------|----------------|
| Function | `(` | `map(.user.profile.)` | `.user.profile.` |
| Array builder | `[`, `,` | `[.a, .b.c.]` | `.b.c.` |
| Object builder | `:`, `,` | `{x: .a.b.}` | `.a.b.` |

**Note**: In **executing context**, cache is already current—just suggest cache's fields directly, no path extraction needed.

---

## Context Types

### Executing vs Non-Executing

| Context | Example | Cache Behavior | Suggestion Strategy |
|---------|---------|----------------|---------------------|
| **Executing** | `.user.profile.` | Cache = result of query | Suggest cache's fields directly |
| **Non-Executing** | `map(.)`, `[.]`, `{x: .}` | Cache is stale | Extract path, navigate from cache |

### Element Context (Element-Iterating Functions)

In element-iterating functions (`map()`, `select()`, `sort_by()`, etc.), the input is implicitly an array element. Prepend `ArrayIterator` to navigate correctly:

```rust
if brace_tracker.is_in_element_iterating_context(cursor_pos) {
    segments.insert(0, PathSegment::ArrayIterator);
}
```

---

## Suggestion Certainty: Deterministic vs Non-Deterministic

**Core Logic**:

**Executing Context + Cursor at END:**
- Cache is current (reflects query result)
- **Deterministic**: Suggest cache's fields directly (no navigation)

**Non-Executing Context + Cursor at END:**
1. Extract path from expression boundary
2. Navigate from `last_successful_result_parsed`
3. If succeeds → **Deterministic** (show navigated fields)
4. If fails → **Non-Deterministic** (fall back to `original_json_parsed`)

**Cursor in MIDDLE of query (any context):**
1. Extract path up to cursor
2. Navigate from `original_json_parsed` (cache is "ahead")
3. If succeeds → **Deterministic** (show navigated fields)
4. If fails → **Non-Deterministic** (fall back to `original_json_parsed`)

### Deterministic (Navigation Succeeds)

Path exists in the navigation source. Show **targeted suggestions**:

| Context | Example | Why Deterministic |
|---------|---------|-------------------|
| Simple field path | `.user.profile.` | Direct navigation through known structure |
| Array iteration | `.items[].name.` | First element provides field schema |
| Element-context functions | `map(.profile.)` | Input is array, navigate first element |
| Nested in builders | `{x: .config.db.}` | Path from root is known |

**Behavior**: Suggest fields of the navigated target object.

### Non-Deterministic (Navigation Fails)

Path doesn't exist in the navigation source. Fall back to **all available suggestions** from `original_json_parsed`:

| Context | Example | Why Non-Deterministic |
|---------|---------|----------------------|
| After transforming functions | `keys \| .` | `keys` returns `[string]`, unknown field names |
| After `to_entries` | `to_entries \| .[].` | Structure is `{key, value}` not original |
| After `group_by` | `group_by(.x) \| .[].` | Nested arrays, unknown structure |
| After pipe with complex expr | `.a + .b \| .` | Result type depends on runtime values |
| Path navigation fails | `.nonexistent.` | Target doesn't exist in JSON |
| After conditionals | `if .x then .a else .b end \| .` | Branch depends on runtime |

**Behavior**: Show all available suggestions from `original_json_parsed`, scoped by syntax context:

| Syntax | Suggestions | Source |
|--------|-------------|--------|
| After `.` | All fields | `original_json_parsed` |
| After `\|` (no dot) | Functions and operators | Static list |
| After `$` | All defined variables | Query parser |
| After `[` | Fields, functions | `original_json_parsed` + static |
| After `{` key `:` | Fields, expressions | `original_json_parsed` + static |

### Detection Logic

```rust
enum SuggestionCertainty {
    Deterministic,      // Navigate and suggest target fields
    NonDeterministic,   // Show all available suggestions
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

| Context | Cursor | Strategy | On Failure |
|---------|--------|----------|------------|
| **Executing** | End | Suggest cache's fields directly | N/A (cache is valid) |
| **Non-Executing** | End | Extract path, navigate from cache | Fall back to `original_json` |
| **Any** | Middle | Extract path, navigate from `original_json` | Fall back to `original_json` |

---

## Comprehensive Examples

### Test JSON

```json
{
  "user": {
    "profile": {"name": "John", "age": 30},
    "settings": {"theme": "dark", "lang": "en"}
  },
  "orders": [
    {"id": 1, "items": [{"sku": "A1", "qty": 2}], "status": "shipped"},
    {"id": 2, "items": [{"sku": "B2", "qty": 1}], "status": "pending"}
  ],
  "meta": {"version": "1.0"}
}
```

**Root fields**: `user`, `orders`, `meta`

### Scenario Table

| # | Query (▎=cursor) | Context | Strategy | Cache/Nav | Certainty | Suggestions |
|---|------------------|---------|----------|-----------|-----------|-------------|
| **Executing Context (cursor at end) — use cache directly** |
| 1 | `.user.▎` | Exec | Cache direct | Cache = `.user` result | Det | `profile`, `settings` |
| 2 | `.user.profile.▎` | Exec | Cache direct | Cache = `.user.profile` result | Det | `name`, `age` |
| 3 | `.orders[].▎` | Exec | Cache direct | Cache = `.orders[]` result | Det | `id`, `items`, `status` |
| 4 | `.orders[].items[].▎` | Exec | Cache direct | Cache = `.orders[].items[]` result | Det | `sku`, `qty` |
| 5 | `.fake.▎` | Exec | Cache direct | Cache = error/empty | Non-Det | `user`, `orders`, `meta` |
| **Non-Executing Context (map/select) — extract path, navigate** |
| 6 | `.orders \| map(.▎)` | Non-Exec | Nav from cache | Path: `.` + elem ctx | Det | `id`, `items`, `status` |
| 7 | `.orders \| map(.items[].▎)` | Non-Exec | Nav from cache | Path: `.items[]` | Det | `sku`, `qty` |
| 8 | `.orders \| map(.fake.▎)` | Non-Exec | Nav from cache | Path: `.fake` ✗ | Non-Det | `user`, `orders`, `meta` |
| 9 | `.orders \| select(.status == "shipped").▎` | Non-Exec | Nav from cache | Path: `.` + elem ctx | Det | `id`, `items`, `status` |
| **Non-Executing Context (builders) — extract path, navigate** |
| 10 | `[.user.profile.▎]` | Non-Exec | Nav from cache | Path: `.user.profile` | Det | `name`, `age` |
| 11 | `{x: .user.settings.▎}` | Non-Exec | Nav from cache | Path: `.user.settings` | Det | `theme`, `lang` |
| 12 | `[.orders[].items[].▎]` | Non-Exec | Nav from cache | Path: `.orders[].items[]` | Det | `sku`, `qty` |
| 13 | `{a: .user.▎, b: .meta}` | Non-Exec | Nav from cache | Path: `.user` | Det | `profile`, `settings` |
| **Middle-of-query editing — navigate from original_json** |
| 14 | `.user.▎profile.name` | Mid | Nav from original | Path: `.user` | Det | `profile`, `settings` |
| 15 | `.orders[].▎items[].sku` | Mid | Nav from original | Path: `.orders[]` | Det | `id`, `items`, `status` |
| 16 | `.fake.▎something` | Mid | Nav from original | Path: `.fake` ✗ | Non-Det | `user`, `orders`, `meta` |
| 17 | `map(.▎id)` | Mid | Nav from original | Path: `.` + elem ctx | Det | element fields |
| **Transforming functions — cache structure unknown** |
| 18 | `keys \| .▎` | Exec | Cache direct | Cache = `["meta","orders","user"]` | Non-Det | `user`, `orders`, `meta` |
| 19 | `.user \| to_entries \| .[].▎` | Exec | Cache direct | Cache = different shape | Non-Det | `user`, `orders`, `meta` |
| 20 | `.orders \| group_by(.status) \| .[].▎` | Exec | Cache direct | Cache = grouped arrays | Non-Det | `user`, `orders`, `meta` |
| **Edge cases** |
| 21 | `.user?.profile?.▎` | Exec | Cache direct | Cache = `.user.profile` | Det | `name`, `age` |
| 22 | `.["user"].profile.▎` | Exec | Cache direct | Cache = `.user.profile` | Det | `name`, `age` |
| 23 | `.orders[0].items[0].▎` | Exec | Cache direct | Cache = specific element | Det | `sku`, `qty` |
| 24 | `. \| .user.▎` | Exec | Cache direct | Cache = `.user` | Det | `profile`, `settings` |
| 25 | `.a + .b \| .▎` | Exec | Cache direct | Cache = runtime result | Non-Det | `user`, `orders`, `meta` |

### Key Observations

1. **Executing context uses cache directly**: No path extraction or navigation needed
2. **Non-executing context extracts path**: From boundary, then navigates from cache
3. **Middle-of-query navigates from original**: Cache is "ahead" of cursor
4. **Element context prepends ArrayIterator**: `map(.x.)` navigates as `[0].x`
5. **Transforming functions = non-det**: `keys`, `to_entries`, `group_by` produce unknown structure
6. **Syntax variations are equivalent**: `?`, bracket notation treated same for cache lookup

---

## Edge Cases

| Case | Handling |
|------|----------|
| `.items[0].` vs `.items[].` | Both → first element (same suggestions) |
| `.data[][].name.` | Chain ArrayIterators: `data[0][0].name` |
| `.nonexistent.` | Show all available suggestions (graceful degradation) |
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

1. **Source `original_json_parsed` from `JqExecutor::json_input`**:

   The original input JSON is stored in `JqExecutor::json_input` and accessed via `json_input()`.
   This is the true original file input that never changes during the session.

   **Important**: Do NOT use `last_successful_result_parsed` - it represents query results,
   not original input, and changes on every successful query execution.

   ```rust
   // In autocomplete context, obtain original JSON from executor:
   let original_json_parsed: Option<Arc<Value>> = jq_executor.json_input_parsed();
   ```

   If `json_input_parsed()` doesn't exist, add a method to parse and cache the original input:
   ```rust
   impl JqExecutor {
       pub fn json_input_parsed(&self) -> Option<Arc<Value>> {
           // Parse json_input once and cache, or return cached value
       }
   }
   ```

2. **Modify ResultAnalyzer API** (`autocomplete/result_analyzer.rs`):
   - Change `analyze_parsed_result(&Arc<Value>, ...)` to `analyze_parsed_result(&Value, ...)`
   - Update all call sites (minimal changes - just remove Arc dereferencing)

   **Critical**: Current `analyze_parsed_result` takes both `Value` and `ResultType`, but `ResultType`
   is derived from the full query output (`QueryState::base_type_for_suggestions`), not from the
   navigated nested value. When we navigate to `.user.profile`, the `ResultType` still describes
   the root structure, causing incorrect suggestions.

   **Solution**: Modify `ResultAnalyzer` to infer type directly from the `Value` itself:
   ```rust
   // Before: requires external ResultType (wrong for nested navigation)
   fn analyze_parsed_result(value: &Value, result_type: &ResultType, ...) -> Vec<Suggestion>

   // After: infers type from Value (works for any navigated value)
   fn analyze_parsed_result(value: &Value, ...) -> Vec<Suggestion> {
       let inferred_type = infer_type_from_value(value);
       // ... use inferred_type for suggestions
   }

   fn infer_type_from_value(value: &Value) -> InferredType {
       match value {
           Value::Object(map) => InferredType::Object(map.keys().cloned().collect()),
           Value::Array(arr) => {
               // Peek first element to determine element type
               match arr.first() {
                   Some(Value::Object(obj)) => InferredType::ArrayOfObjects(obj.keys().cloned().collect()),
                   Some(Value::Array(_)) => InferredType::ArrayOfArrays,
                   _ => InferredType::ArrayOfScalars,
               }
           }
           _ => InferredType::Scalar,
       }
   }
   ```

   This ensures navigated values get correct type inference regardless of original query structure.

3. **Pass `original_json_parsed` to autocomplete** (`autocomplete_state.rs`):
   ```rust
   pub fn update_suggestions_from_app(app: &mut App) {
       // ...
       let original_json = app.jq_executor.json_input_parsed();
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
- [ ] Non-existent path: `.fake.` shows all available suggestions
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

## Implementation Gaps & Resolutions

This section documents gaps identified during code review and their resolutions.

### Gap 1: Missing BraceTracker Methods

**Problem**: The plan references methods that don't exist in `brace_tracker.rs`:

| Plan References | Current Status |
|-----------------|----------------|
| `is_in_non_executing_context()` | Does not exist |
| `innermost_context()` | Does not exist |
| `is_in_element_iterating_context()` | Exists as `is_in_element_context()` |

**Resolution**: Add the following methods to `BraceTracker`:

```rust
impl BraceTracker {
    /// Check if cursor is in a non-executing context where cache doesn't reflect
    /// the expression being typed. This includes:
    /// - Inside element-iterating functions (map, select, etc.)
    /// - Inside array builders [expr, expr]
    /// - Inside object builder values {key: expr}
    pub fn is_in_non_executing_context(&self, pos: usize) -> bool {
        // Element-iterating functions (existing check)
        if self.is_in_element_context(pos) {
            return true;
        }

        // Check innermost brace context
        for info in self.open_braces.iter().rev() {
            if info.pos >= pos {
                continue;
            }

            match info.brace_type {
                // Array builder: always non-executing
                BraceType::Square => {
                    // Distinguish array builder [expr] from array iteration .[]
                    // Array iteration has nothing or . before [
                    // Array builder has expression content
                    if self.is_array_builder(info.pos) {
                        return true;
                    }
                }
                // Object builder value position: non-executing
                BraceType::Curly => {
                    if self.is_after_colon_in_object(pos) {
                        return true;
                    }
                }
                // Parentheses: only non-executing if it's an element-context function
                // (already handled above by is_in_element_context)
                BraceType::Paren => {}
            }
        }

        false
    }

    /// Get the innermost open brace info at a position
    pub fn innermost_brace_info(&self, pos: usize) -> Option<&BraceInfo> {
        self.open_braces.iter().rev().find(|info| info.pos < pos)
    }

    /// Check if the square bracket at `bracket_pos` is an array builder vs iteration
    /// Array iteration: `.[]`, `.[0]`, `.foo[]`
    /// Array builder: `[.a, .b]`, `[1, 2, 3]`
    fn is_array_builder(&self, bracket_pos: usize) -> bool {
        if bracket_pos == 0 {
            return true; // `[...]` at start is always builder
        }

        let before = &self.query_snapshot[..bracket_pos];
        let trimmed = before.trim_end();

        if trimmed.is_empty() {
            return true;
        }

        let last_char = trimmed.chars().last().unwrap();

        // Array iteration follows: `.`, `]`, `?`, identifier chars
        // Array builder follows: `|`, `;`, `(`, `[`, `{`, `,`, `:`
        matches!(last_char, '|' | ';' | '(' | '[' | '{' | ',' | ':')
    }

    /// Check if position is after a colon in an object (value position)
    fn is_after_colon_in_object(&self, pos: usize) -> bool {
        // Find the innermost curly brace
        let curly_pos = self.open_braces.iter().rev()
            .find(|info| info.pos < pos && info.brace_type == BraceType::Curly)
            .map(|info| info.pos);

        if let Some(curly_pos) = curly_pos {
            let inside = &self.query_snapshot[curly_pos + 1..pos];
            // Check if we're after a colon (value position) vs before (key position)
            // Simple heuristic: find last colon or comma
            if let Some(last_colon) = inside.rfind(':') {
                if let Some(last_comma) = inside.rfind(',') {
                    return last_colon > last_comma;
                }
                return true;
            }
        }
        false
    }
}
```

---

### Gap 2: Array Builder vs Array Iteration Detection

**Problem**: The plan assumes we can distinguish:
- `.users[]` - Array iteration (executing)
- `[.name, .age]` - Array builder (non-executing)

But current BraceTracker tracks all `[` as `BraceType::Square` without distinction.

**Resolution**: The `is_array_builder()` method above handles this by examining the character before `[`:

| Character Before `[` | Interpretation |
|---------------------|----------------|
| `.` | Array iteration: `.[]`, `.users[]` |
| `]` | Chained iteration: `.[0][]` |
| `?` | Optional iteration: `.[]?` |
| Identifier char | Field iteration: `users[]` |
| `\|`, `;`, `(`, `[`, `{`, `,`, `:` | Array builder |
| Start of query | Array builder |

**Examples**:
```
.users[]           → iteration (. before [)
[.name, .age]      → builder (start of query)
.data | [.x, .y]   → builder (| before [)
{arr: [.a]}        → builder (: before [)
```

---

### Gap 3: Pipe Handling in Nested Contexts

**Problem**: For query `map(.a | .b.)`:
- Is the expression boundary at `|` or at `(`?
- Plan says `|` is boundary for "top-level" but doesn't address nested pipes.

**Resolution**: Pipes inside non-executing contexts **do** act as expression boundaries.

```rust
fn find_expression_boundary(before_cursor: &str, brace_tracker: &BraceTracker) -> usize {
    let innermost = brace_tracker.innermost_brace_info(before_cursor.len());

    match innermost {
        Some(info) => {
            let after_brace = &before_cursor[info.pos + 1..];

            // Within the brace context, find the last boundary character
            // Boundaries: |, ;, , (for arrays), : (for object values)
            let boundary_chars = match info.brace_type {
                BraceType::Paren => &['|', ';'][..],
                BraceType::Square => &['|', ';', ','][..],
                BraceType::Curly => &['|', ';', ',', ':'][..],
            };

            // Find last boundary within this context
            let last_boundary = after_brace.rfind(|c| boundary_chars.contains(&c));

            match last_boundary {
                Some(offset) => info.pos + 1 + offset + 1, // +1 to skip the boundary char
                None => info.pos + 1, // Start after the opening brace
            }
        }
        None => {
            // Top-level: boundary at |, ;, or start
            before_cursor.rfind(|c| c == '|' || c == ';')
                .map(|pos| pos + 1)
                .unwrap_or(0)
        }
    }
}
```

**Examples**:
| Query | Innermost Context | Last Boundary | Extracted Path |
|-------|-------------------|---------------|----------------|
| `map(.a \| .b.)` | `(` at pos 3 | `\|` at pos 7 | `.b.` |
| `map(.a.b.)` | `(` at pos 3 | None | `.a.b.` |
| `[.x, .y.]` | `[` at pos 0 | `,` at pos 3 | `.y.` |
| `{k: .a \| .b.}` | `{` at pos 0 | `\|` at pos 7 | `.b.` |

---

### Gap 4: `get_all_available_suggestions()` Definition

**Problem**: The plan references this function for non-deterministic fallback but never defines it.

**Resolution**: Define the function and its behavior:

```rust
/// Get all available suggestions when navigation fails (non-deterministic).
/// Suggestions are scoped by the syntax context at cursor.
///
/// # Arguments
/// - `original_json`: The original input JSON (never changes)
/// - `before_cursor`: Query text before cursor (for syntax context detection)
/// - `partial`: The incomplete token being typed (for filtering)
fn get_all_available_suggestions(
    original_json: &Value,
    before_cursor: &str,
    partial: &str,
) -> Vec<Suggestion> {
    let syntax_context = detect_syntax_context(before_cursor);

    let suggestions = match syntax_context {
        SyntaxContext::AfterDot => {
            // After `.` → show all fields from original JSON
            extract_all_fields_recursive(original_json, /* needs_dot */ false)
        }
        SyntaxContext::AfterPipe => {
            // After `|` (no dot) → show functions and operators
            get_function_suggestions()
        }
        SyntaxContext::AfterDollar => {
            // After `$` → handled by VariableContext, shouldn't reach here
            Vec::new()
        }
        SyntaxContext::InArrayBuilder => {
            // Inside `[` → fields + functions
            let mut suggestions = extract_all_fields_recursive(original_json, true);
            suggestions.extend(get_common_functions());
            suggestions
        }
        SyntaxContext::InObjectValue => {
            // After `{key:` → fields + expressions
            let mut suggestions = extract_all_fields_recursive(original_json, true);
            suggestions.extend(get_common_functions());
            suggestions
        }
    };

    filter_suggestions_by_partial(suggestions, partial)
}

/// Extract all unique field names from JSON, recursively traversing nested structures
fn extract_all_fields_recursive(value: &Value, needs_dot: bool) -> Vec<Suggestion> {
    let mut fields = HashSet::new();
    collect_fields_recursive(value, &mut fields);

    let prefix = if needs_dot { "." } else { "" };
    fields.into_iter()
        .map(|name| Suggestion::new_with_type(
            format!("{}{}", prefix, name),
            SuggestionType::Field,
            None, // Type unknown in non-deterministic context
        ))
        .collect()
}

fn collect_fields_recursive(value: &Value, fields: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                fields.insert(key.clone());
                collect_fields_recursive(val, fields);
            }
        }
        Value::Array(arr) => {
            if let Some(first) = arr.first() {
                collect_fields_recursive(first, fields);
            }
        }
        _ => {}
    }
}
```

**Syntax Context Detection**:
```rust
enum SyntaxContext {
    AfterDot,       // `.` or `.field.`
    AfterPipe,      // `| ` (pipe with space, no dot yet)
    AfterDollar,    // `$`
    InArrayBuilder, // `[` or `[expr,`
    InObjectValue,  // `{key:` or `{k1: v1, k2:`
}

fn detect_syntax_context(before_cursor: &str) -> SyntaxContext {
    let trimmed = before_cursor.trim_end();

    if trimmed.ends_with('.') {
        return SyntaxContext::AfterDot;
    }

    // Check last non-whitespace character
    if let Some(last_char) = trimmed.chars().last() {
        match last_char {
            '$' => return SyntaxContext::AfterDollar,
            '|' => return SyntaxContext::AfterPipe,
            '[' | ',' if is_in_array_builder(before_cursor) => {
                return SyntaxContext::InArrayBuilder;
            }
            ':' if is_in_object_builder(before_cursor) => {
                return SyntaxContext::InObjectValue;
            }
            _ => {}
        }
    }

    // Default to function context
    SyntaxContext::AfterPipe
}
```

---

### Gap 5: Original JSON Access Path

**Problem**: `update_suggestions_from_app` needs access to original JSON, but the access path isn't clear.

**Resolution**: The access path is:

```rust
// In autocomplete_state.rs
pub fn update_suggestions_from_app(app: &mut App) {
    let query_state = match &app.query {
        Some(q) => q,
        None => { app.autocomplete.hide(); return; }
    };

    // Access original JSON through executor
    // NOTE: json_input_parsed() must be added to JqExecutor
    let original_json = query_state.executor.json_input_parsed();

    // ... rest of function
    update_suggestions(
        &mut app.autocomplete,
        &query,
        cursor_pos,
        result_parsed,
        result_type,
        original_json,  // NEW parameter
        &app.input.brace_tracker,
    );
}
```

**JqExecutor Addition** (already in Phase 0, but clarifying the full implementation):

```rust
// In query/executor.rs
pub struct JqExecutor {
    json_input: Arc<String>,
    json_input_parsed: OnceCell<Option<Arc<Value>>>,  // Lazy-parsed cache
}

impl JqExecutor {
    pub fn new(json_input: String) -> Self {
        Self {
            json_input: Arc::new(json_input),
            json_input_parsed: OnceCell::new(),
        }
    }

    /// Get parsed JSON input, lazily parsing on first access
    pub fn json_input_parsed(&self) -> Option<Arc<Value>> {
        self.json_input_parsed.get_or_init(|| {
            serde_json::from_str(&self.json_input)
                .ok()
                .map(Arc::new)
        }).clone()
    }
}
```

---

### Gap 6: Middle-of-Query Detection Refinement

**Problem**: `cursor_pos == query.len()` is too simple:
- `.user. ` (trailing space) - cursor not at len but logically "at end"
- `.user.na|me` - cursor in middle of token

**Resolution**: Refine the detection:

```rust
/// Determine if cursor is at the "logical end" of the query
/// (at end, or only whitespace after cursor)
fn is_cursor_at_logical_end(query: &str, cursor_pos: usize) -> bool {
    if cursor_pos >= query.len() {
        return true;
    }

    // Check if everything after cursor is whitespace
    query[cursor_pos..].chars().all(|c| c.is_whitespace())
}

/// Determine if cursor is in the middle of an identifier/token
fn is_cursor_mid_token(query: &str, cursor_pos: usize) -> bool {
    if cursor_pos >= query.len() {
        return false;
    }

    let after = query[cursor_pos..].chars().next();
    matches!(after, Some(c) if c.is_alphanumeric() || c == '_')
}
```

**Updated Integration Logic**:
```rust
SuggestionContext::FieldContext => {
    let is_at_end = is_cursor_at_logical_end(query, cursor_pos);
    let is_mid_token = is_cursor_mid_token(query, cursor_pos);
    let is_executing = !brace_tracker.is_in_non_executing_context(cursor_pos);

    if is_executing && is_at_end {
        // EXECUTING + END: Cache is current
        get_field_suggestions(last_successful_result, ...)
    } else if is_at_end {
        // NON-EXECUTING + END: Extract path, navigate from cache
        // ...
    } else if is_mid_token {
        // MID-TOKEN: Don't show suggestions (user is editing existing token)
        Vec::new()
    } else {
        // MIDDLE OF QUERY: Navigate from original_json
        // ...
    }
}
```

---

### Gap 7: Function Signature Updates

**Problem**: Several functions need signature changes not fully documented.

**Resolution**: Document all signature changes:

```rust
// context.rs - get_suggestions() signature change
pub fn get_suggestions(
    query: &str,
    cursor_pos: usize,
    result_parsed: Option<Arc<Value>>,
    result_type: Option<ResultType>,
    original_json: Option<Arc<Value>>,  // NEW
    brace_tracker: &BraceTracker,
) -> Vec<Suggestion>

// autocomplete.rs - update_suggestions() signature change
pub fn update_suggestions(
    autocomplete: &mut AutocompleteState,
    query: &str,
    cursor_pos: usize,
    result_parsed: Option<Arc<Value>>,
    result_type: Option<ResultType>,
    original_json: Option<Arc<Value>>,  // NEW
    brace_tracker: &BraceTracker,
)
```

---

## Open Questions

1. **Pipe behavior**: ~~Should we try to evaluate partial queries to get intermediate results? Or accept that pipes reset context to "last successful result"?~~
   **RESOLVED**: Pipes act as expression boundaries within non-executing contexts. See Gap 3.

2. **Error tolerance**: If path parsing fails partway, should we:
   - Show all available suggestions? (Current recommendation: Yes)
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
- Show all available suggestions on navigation failure (graceful degradation)
- Use zero-copy references to avoid Monaco's performance concerns
