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

## Recommended Implementation: Unified Entry Context Handling

Rather than having separate handling for `to_entries` and `with_entries`, unify them under a single "entry context" system. Both functions produce `{key, value}` objects where `.value` is opaque.

**Current state (problematic):**
- `with_entries` has special one-off injection at `context.rs:481-483`
- `to_entries` has no special handling (the bug we're fixing)
- These should behave consistently

**Unified approach:**

```rust
/// Entry context state - applies to both to_entries and with_entries
enum EntryContext {
    /// Not in any entry-related context
    None,
    /// Directly in entry context - suggest .key/.value
    /// Examples: to_entries | .[]., to_entries | map(., with_entries(.
    Direct,
    /// Inside .value with nested context - opaque, fall back to all fields
    /// Examples: to_entries | map(.value | ., with_entries(.value | map(.
    OpaqueValue,
}
```

**Benefits:**
1. Single source of truth for entry-related suggestions
2. Consistent behavior: `with_entries(.value | map(.` and `to_entries | map(.value | map(.` both recognized as opaque
3. Easier to maintain and extend
4. Removes the special one-off `with_entries` injection

---

### Phase 1: Unified Entry Context Detection

**Goal:** Detect entry context for both `to_entries` and `with_entries`, handling:
1. Direct entry context → suggest `.key`/`.value`
2. Opaque `.value` context → fall back to all fields

**Key distinctions (apply to BOTH functions):**
- `to_entries | .[].` or `with_entries(.` → DIRECT (suggest `.key`, `.value`)
- `to_entries | .[].value.` or `with_entries(.value.` → NOT opaque (direct navigation works)
- `to_entries | .[].value | .` or `with_entries(.value | .` → OPAQUE (fall back to all fields)
- `to_entries | map(.value | map(.` or `with_entries(.value | map(.` → OPAQUE

```rust
/// Entry context state for both to_entries and with_entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryContext {
    /// Not in any entry-related context
    None,
    /// Directly in entry context - suggest .key/.value
    Direct,
    /// Inside .value with nested context - opaque, fall back to all fields
    OpaqueValue,
}

/// Detect entry context for unified handling of to_entries and with_entries.
///
/// Returns:
/// - `EntryContext::Direct` when we should suggest .key/.value
/// - `EntryContext::OpaqueValue` when .value has nested context (opaque)
/// - `EntryContext::None` when not in entry context
pub fn detect_entry_context(query: &str, cursor_pos: usize) -> EntryContext {
    let before_cursor = &query[..cursor_pos];

    // Check for with_entries first (simpler - single function boundary)
    if let Some(we_pos) = find_unclosed_with_entries(before_cursor) {
        let inside_we = &before_cursor[we_pos + 13..]; // "with_entries(".len()
        return classify_entry_path(inside_we);
    }

    // Check for to_entries
    if let Some(te_pos) = before_cursor.rfind("to_entries") {
        let after_te = &before_cursor[te_pos + 10..]; // "to_entries".len()

        // Must be in element context after to_entries (.[]. or map()
        if !is_in_entry_element_context(after_te) {
            return EntryContext::None;
        }

        // Find the element context start to analyze the path
        if let Some(elem_start) = find_entry_element_start(after_te) {
            let path_inside = &after_te[elem_start..];
            return classify_entry_path(path_inside);
        }
    }

    EntryContext::None
}

/// Classify the path inside an entry context.
/// Returns Direct if no .value access or direct .value navigation.
/// Returns OpaqueValue if .value followed by nested context.
fn classify_entry_path(path: &str) -> EntryContext {
    // Find .value access
    let value_pos = match path.find(".value") {
        Some(pos) => pos,
        None => return EntryContext::Direct, // No .value access, suggest .key/.value
    };

    let after_value = &path[value_pos + 6..]; // ".value".len() == 6

    // Check for nested context after .value
    // Pipe resets context → opaque
    if after_value.contains('|') {
        return EntryContext::OpaqueValue;
    }

    // Nested element-context functions → opaque
    let nested_functions = ["map(", "select(", "sort_by(", "group_by(", "unique_by("];
    for func in nested_functions {
        if after_value.contains(func) {
            return EntryContext::OpaqueValue;
        }
    }

    // Direct field access like .value.field. - can navigate, not opaque
    // But we're past the entry context, so don't suggest .key/.value either
    EntryContext::None
}

/// Find unclosed with_entries( - returns position if cursor is inside
fn find_unclosed_with_entries(before_cursor: &str) -> Option<usize> {
    // Find last with_entries( and check if it's unclosed
    let pos = before_cursor.rfind("with_entries(")?;
    let inside = &before_cursor[pos + 13..];

    // Count parens to check if still inside
    let mut depth = 1;
    for ch in inside.chars() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return None; // with_entries is closed
                }
            }
            _ => {}
        }
    }

    Some(pos) // Still inside with_entries
}

/// Check if we're in element context after to_entries (.[]. or map()
fn is_in_entry_element_context(after_to_entries: &str) -> bool {
    // Look for patterns like: | .[]., | .[0]., | map(.
    after_to_entries.contains("| .[")
        || after_to_entries.contains("|.[")
        || after_to_entries.contains("| map(")
        || after_to_entries.contains("|map(")
}

/// Find where the entry element context starts (after .[]( or map()
fn find_entry_element_start(after_to_entries: &str) -> Option<usize> {
    // Find .[]. or map( and return position after it
    if let Some(pos) = after_to_entries.find("].") {
        return Some(pos + 2);
    }
    if let Some(pos) = after_to_entries.find("map(") {
        return Some(pos + 4);
    }
    None
}
```

**This replaces the existing `with_entries` special handling** at `context.rs:481-483` with the unified system.

### Phase 2: Integration with Existing Code Flow

The unified entry context check replaces the existing `with_entries` injection and adds `to_entries` support:

```rust
// In get_suggestions(), within FieldContext handling:
SuggestionContext::FieldContext => {
    let needs_dot = needs_leading_dot(before_cursor, &partial);
    let is_at_end = is_cursor_at_logical_end(query, cursor_pos);
    let is_non_executing = brace_tracker.is_in_non_executing_context(cursor_pos);

    // NEW: Unified entry context detection (replaces old with_entries special case)
    let entry_context = detect_entry_context(query, cursor_pos);

    match entry_context {
        EntryContext::OpaqueValue => {
            // Inside .value with nested context - fall back to all fields
            let suggestions = get_all_field_suggestions(&all_field_names, needs_dot);
            return filter_suggestions_by_partial_if_nonempty(suggestions, &partial);
        }
        EntryContext::Direct => {
            // Direct entry context - inject .key/.value and continue
            // (handled below with suggestions injection)
        }
        EntryContext::None => {
            // Not in entry context - normal flow
        }
    }

    // Existing navigation logic...
    let mut suggestions = if is_non_executing && is_at_end {
        // ... existing navigation logic
    } else {
        // ... existing executing context logic
    };

    // Inject .key/.value for Direct entry context (replaces old with_entries injection)
    if entry_context == EntryContext::Direct {
        inject_entry_field_suggestions(&mut suggestions, needs_dot);
    }

    filter_suggestions_by_partial_if_nonempty(suggestions, &partial)
}

/// Inject .key and .value suggestions for entry context.
/// Replaces the old inject_with_entries_suggestions function.
fn inject_entry_field_suggestions(suggestions: &mut Vec<Suggestion>, needs_leading_dot: bool) {
    let prefix = if needs_leading_dot { "." } else { "" };

    suggestions.insert(
        0,
        Suggestion::new_with_type(format!("{}value", prefix), SuggestionType::Field, None)
            .with_description("Entry value (original object's value)"),
    );
    suggestions.insert(
        0,
        Suggestion::new_with_type(
            format!("{}key", prefix),
            SuggestionType::Field,
            Some(JsonFieldType::String),
        )
        .with_description("Entry key (original object's key)"),
    );
}
```

**Changes from current implementation:**
1. **Remove** `is_in_with_entries_context()` check from `brace_tracker`
2. **Remove** `inject_with_entries_suggestions()` call at line 481-483
3. **Add** unified `detect_entry_context()` at the start of FieldContext handling
4. **Add** new `inject_entry_field_suggestions()` that handles both functions

**Why this approach:**
1. Single detection point for all entry-related contexts
2. Opaque check happens FIRST, before any navigation attempts
3. Direct entry context gets `.key`/`.value` injection consistently
4. Existing navigation logic unchanged for non-entry contexts

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

### 2. `with_entries` (Now Unified with `to_entries`)
```jq
with_entries(.value |= . * 2) | .
```
After `with_entries()` is closed, the result structure matches input (values transformed).

**Expected:** Same suggestions as original JSON (after the closing paren).

**Inside `with_entries`:**
```jq
with_entries(.value | map(.          # Opaque - fall back to all fields
with_entries(.key |= "prefix_" + .   # Direct - .key is a string, suggest string functions
with_entries(.                        # Direct - suggest .key, .value
```

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
// === to_entries tests ===

#[test]
fn test_to_entries_direct_context() {
    // to_entries | .[]. - direct entry context
    assert_eq!(
        detect_entry_context("to_entries | .[].", 17),
        EntryContext::Direct
    );
}

#[test]
fn test_to_entries_map_direct_context() {
    // to_entries | map(. - direct entry context
    assert_eq!(
        detect_entry_context("to_entries | map(.", 18),
        EntryContext::Direct
    );
}

#[test]
fn test_to_entries_value_direct_navigation() {
    // to_entries | .[].value. - direct .value access, NOT opaque
    // Should return None (not Direct) because we're past the entry context
    assert_eq!(
        detect_entry_context("to_entries | .[].value.", 23),
        EntryContext::None
    );
}

#[test]
fn test_to_entries_value_with_pipe_opaque() {
    // to_entries | .[].value | . - pipe after .value = opaque
    assert_eq!(
        detect_entry_context("to_entries | .[].value | .", 26),
        EntryContext::OpaqueValue
    );
}

#[test]
fn test_to_entries_value_with_nested_map_opaque() {
    // to_entries | map(.value | map(. - nested map after .value = opaque
    assert_eq!(
        detect_entry_context("to_entries | map(.value | map(.", 31),
        EntryContext::OpaqueValue
    );
}

#[test]
fn test_to_entries_complex_opaque() {
    // to_entries | map({service: .key, config: .value | map(select(.
    assert_eq!(
        detect_entry_context(
            "to_entries | map({service: .key, config: .value | map(select(.",
            64
        ),
        EntryContext::OpaqueValue
    );
}

// === with_entries tests (same behavior as to_entries) ===

#[test]
fn test_with_entries_direct_context() {
    // with_entries(. - direct entry context
    assert_eq!(
        detect_entry_context("with_entries(.", 14),
        EntryContext::Direct
    );
}

#[test]
fn test_with_entries_value_direct_navigation() {
    // with_entries(.value. - direct .value access, NOT opaque
    assert_eq!(
        detect_entry_context("with_entries(.value.", 20),
        EntryContext::None
    );
}

#[test]
fn test_with_entries_value_with_pipe_opaque() {
    // with_entries(.value | . - pipe after .value = opaque
    assert_eq!(
        detect_entry_context("with_entries(.value | .", 23),
        EntryContext::OpaqueValue
    );
}

#[test]
fn test_with_entries_value_with_nested_map_opaque() {
    // with_entries(.value | map(. - nested map after .value = opaque
    assert_eq!(
        detect_entry_context("with_entries(.value | map(.", 27),
        EntryContext::OpaqueValue
    );
}

#[test]
fn test_with_entries_closed_not_in_context() {
    // with_entries(.key) | . - with_entries is closed, not in entry context
    assert_eq!(
        detect_entry_context("with_entries(.key) | .", 22),
        EntryContext::None
    );
}

// === Edge cases ===

#[test]
fn test_no_entry_context() {
    // Regular query without to_entries or with_entries
    assert_eq!(
        detect_entry_context(".users | map(.", 14),
        EntryContext::None
    );
}

#[test]
fn test_to_entries_without_element_context() {
    // to_entries | . - not in element context yet
    assert_eq!(
        detect_entry_context("to_entries | .", 14),
        EntryContext::None
    );
}
```

### Integration Tests

```rust
// === to_entries integration tests ===

#[test]
fn test_to_entries_direct_suggests_key_value() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    simulate_typing(&mut app, "to_entries | .[].");

    let suggestions = app.autocomplete.suggestions();

    // Should suggest .key and .value
    assert!(suggestions.iter().any(|s| s.text.contains("key")));
    assert!(suggestions.iter().any(|s| s.text.contains("value")));
}

#[test]
fn test_to_entries_value_nested_is_opaque() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    simulate_typing(&mut app, "to_entries | map({service: .key, config: .value | map(select(.");

    let suggestions = app.autocomplete.suggestions();

    // Should suggest all fields from original JSON
    assert!(suggestions.iter().any(|s|
        s.text.contains("services") || s.text.contains("name") || s.text.contains("web")
    ));
    // Must NOT suggest entry structure fields
    assert!(!suggestions.iter().any(|s| s.text == ".key" || s.text == "key"));
    assert!(!suggestions.iter().any(|s| s.text == ".value" || s.text == "value"));
}

#[test]
fn test_to_entries_direct_value_navigation_works() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    simulate_typing(&mut app, "to_entries | .[].value.");

    let suggestions = app.autocomplete.suggestions();

    // Should navigate through original JSON and suggest "services"
    assert!(suggestions.iter().any(|s| s.text.contains("services")));
}

#[test]
fn test_to_entries_value_with_pipe_is_opaque() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    simulate_typing(&mut app, "to_entries | .[].value | .");

    let suggestions = app.autocomplete.suggestions();

    // Should fall back to all fields
    assert!(suggestions.iter().any(|s| s.text.contains("services") || s.text.contains("name")));
}

// === with_entries integration tests (same behavior) ===

#[test]
fn test_with_entries_direct_suggests_key_value() {
    let json = r#"{"a": 1, "b": 2}"#;
    let app = app_with_json(json);

    simulate_typing(&mut app, "with_entries(.");

    let suggestions = app.autocomplete.suggestions();

    // Should suggest .key and .value (same as current behavior)
    assert!(suggestions.iter().any(|s| s.text.contains("key")));
    assert!(suggestions.iter().any(|s| s.text.contains("value")));
}

#[test]
fn test_with_entries_value_nested_is_opaque() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    simulate_typing(&mut app, "with_entries(.value | map(.");

    let suggestions = app.autocomplete.suggestions();

    // Should fall back to all fields from original JSON
    assert!(suggestions.iter().any(|s|
        s.text.contains("services") || s.text.contains("name") || s.text.contains("web")
    ));
    // Must NOT suggest entry structure fields
    assert!(!suggestions.iter().any(|s| s.text == ".key" || s.text == "key"));
}

#[test]
fn test_with_entries_closed_normal_context() {
    let json = r#"{"a": 1, "b": 2}"#;
    let app = app_with_json(json);

    simulate_typing(&mut app, "with_entries(.key |= \"prefix_\" + .) | .");

    let suggestions = app.autocomplete.suggestions();

    // After with_entries is closed, normal suggestions
    // (from_entries output structure is unknown, so all fields)
    assert!(suggestions.iter().any(|s| s.text.contains("a") || s.text.contains("b")));
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

1. **Unified entry context**: Both `to_entries` and `with_entries` handled by single `detect_entry_context()` function
2. **Direct entry context**: After `to_entries | .[].` and `with_entries(.`, suggests `.key` and `.value`
3. **Opaque detection**: After `to_entries | map(.value | .` and `with_entries(.value | .`, falls back to all fields
4. **Direct navigation works**: After `to_entries | .[].value.`, navigates through original JSON correctly
5. **Existing behavior preserved**: All current suggestions unchanged except for the unified handling
6. **Code cleanup**: Remove `inject_with_entries_suggestions()`, `is_in_with_entries_context()`, and `FunctionContext::WithEntries` in favor of unified system
7. No performance regression

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
fn regression_with_entries_still_suggests_key_value() {
    // with_entries() should still suggest .key and .value
    // (now via unified entry context, not special injection)
    let json = r#"{"a": 1}"#;
    simulate_query(json, "with_entries(.");
    assert_suggests(&["key", "value"]);
}

#[test]
fn regression_with_entries_value_nested_now_opaque() {
    // NEW BEHAVIOR: with_entries(.value | map(.) should be opaque
    // (Previously this case wasn't handled)
    let json = r#"{"a": {"name": "test"}}"#;
    simulate_query(json, "with_entries(.value | map(.");
    // Should fall back to all fields, not suggest .key/.value
    assert_suggests(&["a", "name"]);
    assert_not_suggests(&["key", "value"]);
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
- **context.rs:481-483**: Existing `with_entries` special handling **will be replaced** by the unified entry context system. The current `inject_with_entries_suggestions()` and `is_in_with_entries_context()` calls will be removed in favor of the new `detect_entry_context()` approach.
- **brace_tracker.rs**: The `is_in_with_entries_context()` method can be removed after this change, as entry context detection moves to the new unified function.
- **all_field_names cache**: Already implemented in `JqExecutor` for fallback suggestions - will be used for opaque context.
