# JQ Suggestions After Transforming Functions

## Core Insight

The autocomplete system's path extraction only looks at the **innermost context** (e.g., inside `select(`), missing crucial context from outer expressions like `.value | map(...)`. When this outer context involves opaque transformations (like `.value` from `to_entries`), the system incorrectly navigates to the wrong structure.

**The fix:** Before navigation, scan the FULL expression from `to_entries` to cursor to detect if we've crossed into an opaque context (pipe or nested function after `.value`).

---

## Problem Statement

After transforming functions like `to_entries`, suggestions incorrectly show parent context fields instead of recognizing unknown context.

### Example Issue

```jq
to_entries | map({service: .key, config: .value | map(select(.name
                                                            ^cursor
```

**Current behavior:** Suggests `.value` (from the `{key, value}` context of `to_entries`)

**Expected behavior:** Suggest all fields from original JSON (since `.value` contents are unknown), or provide generic suggestions

### Why This Happens

1. `to_entries` transforms `{a: 1, b: 2}` into `[{key: "a", value: 1}, {key: "b", value: 2}]`
2. Inside `map(...)`, the element context is `{key, value}`
3. When typing `select(.`, the path extraction only sees the **innermost context** (`select(`)
4. The extracted path is just `.` - missing the `.value | map(...)` that precedes it
5. System prepends `ArrayIterator` and navigates from `to_entries` cache → lands on `{key, value}`
6. Suggests `key` and `value` instead of recognizing we're inside an opaque `.value` context

---

## Root Cause Analysis

### Current Type Flow

```
Original JSON: {services: {web: {name: "api", port: 80}, db: {name: "postgres", port: 5432}}}
                                    ↓
Query: to_entries
                                    ↓
Result: [{key: "services", value: {web: {...}, db: {...}}}]
                                    ↓
Cache (last_successful_result_parsed): [{key: "services", value: {...}}]
                                    ↓
Query continues: to_entries | map({service: .key, config: .value | map(select(.
                                    ↓
Path extraction at cursor: finds innermost context is `select(`
                                    ↓
Extracted path: just "." (the expression after `select(`)
                                    ↓
Navigation: prepends ArrayIterator → [ArrayIterator] → navigates to {key, value}
                                    ↓
Suggests: key, value (WRONG - should recognize opaque context)
```

### The Problem Points

1. **Path extraction is too narrow**: The `find_expression_boundary()` function only looks at the innermost context (`select(`), completely ignoring the `.value | map(...)` expression that precedes it. This context is crucial - it tells us we're operating on opaque data.

2. **No expression ancestry tracking**: The system doesn't track the "expression path" from the cached result to the current cursor position. It only sees:
   - Cached result: `[{key, value}]` from `to_entries`
   - Current path: `.` (from inside `select(`)
   - Missing: the `.value | map(...)` that connects them

3. **`.value` contents are opaque**: After `to_entries`:
   - `.key` is always a string (the original object's key)
   - `.value` is the original object's value (unknown structure)
   - Any nested context AFTER `.value` should inherit this opacity

4. **Nested transformations compound the problem**:
   - `.value | map(...)` creates a second layer of iteration
   - We're iterating over `.value`'s contents (which could be anything)
   - The `select(.)` is filtering elements whose structure is unknown

---

## Proposed Solution

### Approach 1: Opaque Field Tracking (Recommended)

Track which fields produce "opaque" values where the structure is unknown.

**Define opaque fields by function:**

| Function | Result Structure | Opaque Fields |
|----------|------------------|---------------|
| `to_entries` | `[{key, value}]` | `.value` (original object's value) |
| `from_entries` | `{...}` | All fields (reconstructed from entries) |
| `with_entries(f)` | `{...}` | All fields (transformed entries) |
| `paths` | `[[...]]` | Element contents (path arrays) |
| `getpath(p)` | `any` | Result is opaque |
| `group_by(f)` | `[[...]]` | Inner array elements |

**Implementation:**

```rust
/// Check if we're accessing an opaque field after a transforming function
fn is_in_opaque_context(before_cursor: &str, path_after_transform: &str) -> bool {
    // Check for to_entries followed by .value access
    if query_contains_to_entries(before_cursor) {
        let path = extract_current_path(path_after_transform);
        if path_accesses_value_field(path) {
            return true;
        }
    }

    // Check for other transforming functions...
    false
}

/// When in opaque context, fall back to all-fields suggestions
fn get_suggestions_for_opaque_context(original_json: &Value) -> Vec<Suggestion> {
    extract_all_fields_recursive(original_json)
}
```

### Approach 2: Transformation Chain Tracking

Track the chain of transformations to understand when structure is lost.

**State machine for structure tracking:**

```
Known → to_entries → EntryStructure{key: String, value: Opaque}
                         ↓
         .value access → Opaque
                         ↓
         Any further path → Opaque (stay in opaque state)
```

```rust
enum StructureKnowledge {
    /// We know the exact structure from cached result or original JSON
    Known(Value),

    /// Structure follows to_entries pattern
    EntryStructure,

    /// Structure is unknown/opaque
    Opaque,
}

fn track_structure_through_path(
    initial: StructureKnowledge,
    path_segments: &[PathSegment],
) -> StructureKnowledge {
    let mut current = initial;

    for segment in path_segments {
        current = match (current, segment) {
            // Accessing .value in entry structure → opaque
            (StructureKnowledge::EntryStructure, PathSegment::Field(f)) if f == "value" => {
                StructureKnowledge::Opaque
            }
            // .key in entry structure → known string
            (StructureKnowledge::EntryStructure, PathSegment::Field(f)) if f == "key" => {
                StructureKnowledge::Known(Value::String(String::new()))
            }
            // Once opaque, always opaque
            (StructureKnowledge::Opaque, _) => StructureKnowledge::Opaque,
            // Known structure navigates normally
            (StructureKnowledge::Known(v), segment) => {
                match navigate_single(&v, segment) {
                    Some(next) => StructureKnowledge::Known(next.clone()),
                    None => StructureKnowledge::Opaque,
                }
            }
            _ => current,
        };
    }

    current
}
```

### Approach 3: Expression Analysis (Most Accurate)

Parse the expression to detect transformation patterns.

**Pattern detection:**

```rust
/// Detect if the current context is after a transformation that loses structure
fn detect_transformation_context(before_cursor: &str) -> Option<TransformContext> {
    // Pattern: to_entries | ... | .value ...
    // Indicates we're inside the value of a to_entries result

    let patterns = [
        (r"to_entries\s*\|\s*.*\.value", TransformContext::ToEntriesValue),
        (r"from_entries", TransformContext::Opaque),
        (r"group_by\([^)]*\)\s*\|\s*\.\[\]", TransformContext::Opaque),
        // ... more patterns
    ];

    for (pattern, context) in patterns {
        if Regex::new(pattern).unwrap().is_match(before_cursor) {
            return Some(context);
        }
    }

    None
}
```

---

## Recommended Implementation: Hybrid Approach

Combine Approach 1 and 3 for accuracy without complexity:

### Phase 1: Detect `to_entries` + `.value` + Nested Context Pattern

**Goal:** When inside a NESTED context (pipe, map, select) after `.value` from `to_entries`, recognize context is opaque.

**Key distinction:**
- `to_entries | .[].value.` → NOT opaque (direct navigation, can trace through original JSON)
- `to_entries | .[].value | .` → OPAQUE (pipe resets context, we don't know what `.value` contains)
- `to_entries | map(.value | map(.` → OPAQUE (nested map after `.value`)

```rust
/// Check if cursor is in an opaque context after a transforming function.
///
/// Returns true ONLY when ALL conditions are met:
/// 1. Query contains `to_entries` before cursor
/// 2. After `to_entries`, there's a `.value` access
/// 3. After `.value`, there's a NESTED CONTEXT (pipe, map, select, etc.)
///
/// Does NOT return true for direct navigation like `to_entries | .[].value.field.`
fn is_in_opaque_value_context(query: &str, cursor_pos: usize) -> bool {
    let before_cursor = &query[..cursor_pos];

    // Find the most recent to_entries
    let to_entries_pos = match before_cursor.rfind("to_entries") {
        Some(pos) => pos,
        None => return false,
    };

    let after_to_entries = &before_cursor[to_entries_pos..];

    // Find .value access (with possible array iteration before it)
    // Patterns: .[].value, .[0].value, .value (less common)
    let value_pos = match after_to_entries.find(".value") {
        Some(pos) => pos,
        None => return false,
    };

    let after_value = &after_to_entries[value_pos + 6..]; // ".value".len() == 6

    // ONLY opaque if there's a NESTED CONTEXT after .value
    // A pipe (|) or element-context function (map, select) indicates nested context
    // Direct field access (.field.) is NOT opaque - we can navigate

    // Check for pipe - this resets context, making it opaque
    if after_value.contains('|') {
        return true;
    }

    // Check for nested element-context functions after .value
    let nested_functions = ["map(", "select(", "sort_by(", "group_by(", "unique_by("];
    for func in nested_functions {
        if after_value.contains(func) {
            return true;
        }
    }

    // Direct field access like .value.services.web. is NOT opaque
    // We can trace through original JSON
    false
}
```

**Important:** The distinction between direct navigation and nested context is crucial for avoiding regressions. Direct `.value.field.` access should continue to work by navigating through the original JSON.

### Phase 2: Integration with Existing Code Flow

The opaque check should be integrated into `context.rs:get_suggestions()` BEFORE the navigation logic, but AFTER we've determined we're in FieldContext:

```rust
// In get_suggestions(), within FieldContext handling:
SuggestionContext::FieldContext => {
    let needs_dot = needs_leading_dot(before_cursor, &partial);
    let is_at_end = is_cursor_at_logical_end(query, cursor_pos);
    let is_non_executing = brace_tracker.is_in_non_executing_context(cursor_pos);

    // NEW: Check for opaque context FIRST, before any navigation attempts
    if is_in_opaque_value_context(query, cursor_pos) {
        let suggestions = get_all_field_suggestions(&all_field_names, needs_dot);
        return filter_suggestions_by_partial_if_nonempty(suggestions, &partial);
    }

    // Existing logic continues unchanged...
    let mut suggestions = if is_non_executing && is_at_end {
        // ... existing navigation logic
    }
    // ...
}
```

**Why this placement:**
1. Check opaque context BEFORE navigation - avoids unnecessary navigation attempts
2. Still respects `with_entries` injection (happens later in the function)
3. Uses existing `all_field_names` cache - no performance impact
4. Returns early with all-fields fallback

### Phase 3: Special `.key`/`.value` Suggestions After `to_entries`

When directly after `to_entries | .[].` or `to_entries | map(.`, suggest `.key` and `.value`:

```rust
fn should_suggest_entry_fields(query: &str, cursor_pos: usize) -> bool {
    let before = &query[..cursor_pos];

    // Check for to_entries followed by iteration and dot
    // to_entries | .[].
    // to_entries | map(.
    let patterns = [
        r"to_entries\s*\|\s*\.\[\]\.$",
        r"to_entries\s*\|\s*map\(\.$",
    ];

    patterns.iter().any(|p| Regex::new(p).unwrap().is_match(before))
}

fn get_entry_field_suggestions() -> Vec<Suggestion> {
    vec![
        Suggestion::field(".key", "string"),
        Suggestion::field(".value", "any"),
    ]
}
```

---

## Detailed Example Walkthrough

### Input JSON
```json
{
  "services": {
    "web": {"name": "api", "port": 80},
    "db": {"name": "postgres", "port": 5432}
  }
}
```

### Query Progression

| Query State | Context Analysis | Expected Suggestions |
|-------------|------------------|---------------------|
| `to_entries \| .` | After `to_entries`, result is `[{key, value}]` | `.[]`, `.[0]` |
| `to_entries \| .[].` | Element is `{key, value}` | `.key`, `.value` |
| `to_entries \| .[].value.` | `.value` = original `{services: {...}}` | `.services` |
| `to_entries \| .[].value.services \| to_entries \| .[].` | Nested `to_entries` | `.key`, `.value` |
| `to_entries \| map({... .value \| .` | Inside `.value`, unknown | All fields from original |
| `to_entries \| map({... .value \| map(select(.` | Nested unknown | All fields from original |

### The Bug Case in Detail

```jq
to_entries | map({service: .key, config: .value | map(select(.name
```

**Current (incorrect) flow:**
1. `to_entries` detected → cache is `[{key, value}]`
2. Inside `map(...)` → element context, looking at `{key, value}`
3. Building object `{service: .key, config: ...}` → non-executing context
4. At `.name` → navigates from `{key, value}` → suggests `.key`, `.value`

**Correct flow:**
1. `to_entries` detected → cache is `[{key, value}]`
2. Inside `map(...)` → element is `{key, value}`
3. Accessing `.value` → **mark as opaque context**
4. `.value | map(...)` → **nested element context, still opaque**
5. Inside `select(.name` → **opaque, fall back to all fields**
6. Suggest all fields from original JSON: `services`, `web`, `db`, `name`, `port`

---

## Edge Cases to Handle

### 1. Chained `to_entries`
```jq
to_entries | map(.value | to_entries | map(.value | .
```
Each `to_entries` resets the structure. The final `.value` is opaque.

**Expected:** All fields from original JSON.

### 2. `with_entries` (Transforms in Place)
```jq
with_entries(.value |= . * 2) | .
```
Result has same structure as input (values transformed).

**Expected:** Same suggestions as original JSON.

### 3. `from_entries` (Reconstructs Object)
```jq
to_entries | map(.key |= "prefix_" + .) | from_entries | .
```
Result is object with modified keys. Structure is unknown.

**Expected:** All fields (can't predict new key names).

### 4. Nested `.key` Access (Known Structure)
```jq
to_entries | map(.key |
```
`.key` is always a string after `to_entries`.

**Expected:** String functions like `ascii_downcase`, `split`, etc.

### 5. Direct `.value` Field Navigation (NOT Opaque)
```jq
to_entries | map({k: .key, v: .value.services.web.})
```
This is **direct** field access after `.value` - no pipe or nested function between `.value` and the cursor.

**Expected behavior:**
1. Detect we're in `map(...)` element context
2. Notice `.value.services.web.` is direct navigation (no pipe/nested context after `.value`)
3. Navigate original JSON: find what `.value` maps to → `.services` → `.web`
4. Suggest: `name`, `port`

**How this works with opaque detection:**
- `is_in_opaque_value_context()` returns `false` because there's no pipe or nested function after `.value`
- Normal navigation proceeds: the path parser extracts `.value.services.web.`
- Navigation maps `.value` to the original JSON value, then navigates `.services.web`

**Contrast with opaque case:**
```jq
to_entries | map({k: .key, v: .value | map(.services.web.})
                                     ^^^^ pipe makes it opaque
```
Here the pipe after `.value` means `.value | map(.services.web.` is opaque.

---

## Implementation Phases

### Phase 1: Opaque Context Detection
- Add `is_in_opaque_value_context()` function
- Detect `to_entries` + `.value` + nested access pattern
- Unit tests for pattern detection

### Phase 2: Fallback Integration
- Modify `get_suggestions()` to check opaque context first
- Fall back to `all_field_names` when opaque
- Integration tests

### Phase 3: Entry Field Suggestions
- Add special `.key`/`.value` suggestions directly after `to_entries`
- Similar to existing `with_entries` handling in `context.rs:481-483`

### Phase 4: Extended Transforming Functions
- Add detection for `from_entries`, `group_by`, `unique_by`, etc.
- Extend opaque context detection

### Phase 5: Smart Navigation Through `.value`
- When accessing `.value.path.to.field`, attempt to navigate original JSON
- Fall back to opaque only when navigation fails
- This gives best suggestions when the structure is actually known

---

## Test Cases

### Unit Tests

```rust
#[test]
fn test_opaque_context_to_entries_value_nested() {
    // to_entries | map({... .value | map(select(.
    // Inside nested context after .value - should be opaque
    assert!(is_in_opaque_value_context(
        "to_entries | map({service: .key, config: .value | map(select(.",
        64  // cursor at end
    ));
}

#[test]
fn test_not_opaque_direct_value_access_with_trailing_dot() {
    // to_entries | .[].value.
    // Direct .value access with trailing dot - can navigate through original JSON
    // This should NOT be opaque because we can trace .value back to original structure
    assert!(!is_in_opaque_value_context(
        "to_entries | .[].value.",
        23
    ));
}

#[test]
fn test_opaque_value_with_pipe() {
    // to_entries | .[].value | .
    // After pipe following .value - context is opaque
    assert!(is_in_opaque_value_context(
        "to_entries | .[].value | .",
        26
    ));
}

#[test]
fn test_opaque_value_with_nested_map() {
    // to_entries | map(.value | map(.
    // Nested map after .value - inner map's elements are opaque
    assert!(is_in_opaque_value_context(
        "to_entries | map(.value | map(.",
        31
    ));
}

#[test]
fn test_entry_field_suggestions() {
    // to_entries | .[].
    assert!(should_suggest_entry_fields("to_entries | .[].", 17));
}

#[test]
fn test_not_entry_field_after_value() {
    // to_entries | .[].value.
    // After .value, we should NOT suggest .key/.value - we should navigate original
    assert!(!should_suggest_entry_fields("to_entries | .[].value.", 23));
}
```

### Integration Tests

```rust
#[test]
fn test_suggestions_after_to_entries_value_nested() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    // Type the problematic query - cursor is at the dot after select(
    simulate_typing(&mut app, "to_entries | map({service: .key, config: .value | map(select(.");

    let suggestions = app.autocomplete.suggestions();

    // Should suggest all fields from original JSON, not .key/.value from to_entries
    assert!(suggestions.iter().any(|s| s.text.contains("services") || s.text.contains("name") || s.text.contains("web")));
    // Must NOT suggest the to_entries structure fields
    assert!(!suggestions.iter().any(|s| s.text == ".key" || s.text == "key"));
    assert!(!suggestions.iter().any(|s| s.text == ".value" || s.text == "value"));
}

#[test]
fn test_direct_value_navigation_still_works() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    // Direct .value access WITHOUT pipe/nested function - should navigate
    simulate_typing(&mut app, "to_entries | .[].value.");

    let suggestions = app.autocomplete.suggestions();

    // Should navigate through original JSON and suggest "services"
    assert!(suggestions.iter().any(|s| s.text.contains("services")));
    // Should NOT fall back to all fields (would include "name", "web" at wrong level)
}

#[test]
fn test_value_with_pipe_is_opaque() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    // .value followed by pipe - context is opaque
    simulate_typing(&mut app, "to_entries | .[].value | .");

    let suggestions = app.autocomplete.suggestions();

    // Should fall back to all fields from original JSON
    assert!(suggestions.iter().any(|s| s.text.contains("services") || s.text.contains("name")));
}
```

---

## Manual TUI Validation

Test with `tests/fixtures/ecs.json`:

| Query | Expected Suggestions |
|-------|---------------------|
| `to_entries \| .` | `.[]` |
| `to_entries \| .[].` | `.key`, `.value` |
| `to_entries \| .[].value.` | `.services` (navigates through original) |
| `to_entries \| map(.` | `.key`, `.value` |
| `to_entries \| map(.value \| .` | All fields from original (opaque context) |
| `to_entries \| map({k: .key, v: .value \| map(select(.` | All fields from original |

---

## Success Criteria

1. After `to_entries | map({... .value | map(select(.`, suggests all original JSON fields
2. After `to_entries | .[].`, suggests `.key` and `.value`
3. After `to_entries | .[].value.`, navigates original JSON correctly
4. Existing suggestions unchanged
5. No performance regression

---

## Regression Risk Analysis

### High Risk Areas

| Risk | Impact | Mitigation |
|------|--------|------------|
| **False positives for `.value` detection** | May incorrectly mark legitimate `.value` fields as opaque in non-`to_entries` contexts | Only check for opaque context AFTER confirming `to_entries` is in the expression ancestry |
| **Breaking direct `.value.field.` navigation** | After `to_entries \| .[].value.services.`, should still navigate through original JSON | Opaque detection should only trigger when there's a NESTED context (pipe, map, select) after `.value`, not for direct field access |
| **Conflict with existing `with_entries` handling** | `context.rs:481-483` already injects `.key`/`.value` for `with_entries()` | Ensure opaque detection doesn't prevent this injection; `with_entries` is different from `to_entries` |
| **Performance regression from regex matching** | Regex on every keystroke could slow down suggestions | Use efficient string matching (`contains`, `rfind`) instead of compiled regex |
| **Breaking element context ArrayIterator logic** | The existing Phase 7 streaming detection could conflict with opaque detection | Opaque check should happen BEFORE navigation, not interfere with ArrayIterator prepending |

### Regression Test Cases (Must Pass)

```rust
// Existing behavior that MUST continue to work:

#[test]
fn regression_top_level_field_suggestions() {
    // Basic field suggestions at root level
    let json = r#"{"name": "test", "value": 42}"#;
    simulate_query(json, ".");
    assert_suggests(&["name", "value"]);
}

#[test]
fn regression_with_entries_still_injects_key_value() {
    // with_entries() should still suggest .key and .value
    let json = r#"{"a": 1}"#;
    simulate_query(json, "with_entries(.");
    assert_suggests(&["key", "value"]);  // Special injection
}

#[test]
fn regression_normal_value_field_not_affected() {
    // A field literally named "value" in normal JSON should work
    let json = r#"{"data": {"value": 123, "name": "test"}}"#;
    simulate_query(json, ".data.");
    assert_suggests(&["value", "name"]);  // Normal navigation
}

#[test]
fn regression_select_in_normal_context() {
    // select() without to_entries should work normally
    let json = r#"[{"name": "a"}, {"name": "b"}]"#;
    simulate_query(json, "map(select(.");
    assert_suggests(&["name"]);  // Element fields
}

#[test]
fn regression_to_entries_direct_value_navigation() {
    // to_entries | .[].value. should navigate through original
    let json = r#"{"services": {"web": {"port": 80}}}"#;
    simulate_query(json, "to_entries | .[].value.");
    assert_suggests(&["services"]);  // Navigate original JSON
}

#[test]
fn regression_nested_autosuggestion_in_map() {
    // Nested suggestions inside map() without to_entries
    let json = r#"[{"user": {"name": "test"}}]"#;
    simulate_query(json, "map(.user.");
    assert_suggests(&["name"]);  // Nested navigation works
}
```

### Safe Implementation Order

1. **First**: Add opaque detection as an **additional check** that only triggers in very specific conditions (must have `to_entries` AND `.value` AND nested context)
2. **Second**: Ensure the check is SKIPPED for direct navigation (`to_entries | .[].value.field.`)
3. **Third**: Add comprehensive regression tests BEFORE implementing the feature
4. **Fourth**: Run full test suite after each change

---

## Related Work

- **PLAN_NESTED_AUTOSUGGESTION.md**: Phase 4 Notes mention transforming function detection was removed for executing context. This plan addresses non-executing context after transforming functions.
- **context.rs:481-483**: Existing `with_entries` special handling provides a pattern to follow.
- **all_field_names cache**: Already implemented in `JqExecutor` for fallback suggestions.
