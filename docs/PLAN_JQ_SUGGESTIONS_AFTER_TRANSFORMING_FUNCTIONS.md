# JQ Suggestions After Transforming Functions

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
3. When navigating to `.value`, the system sees `.value` as a valid field and navigates to its contents
4. But `.value` contains the **original object's values**, which could be anything
5. The system doesn't track that `.value` from `to_entries` represents "unknown structure"
6. When typing `.name` inside `map(select(.name`, it suggests `.value` from the parent `{key, value}` context

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
Query continues: to_entries | map({service: .key, config: .value | map(select(.name
                                    ↓
Navigation from cache: .value → finds {web: {...}, db: {...}}
                                    ↓
But .value | map(select(...)) creates NESTED unknown context
```

### The Problem Points

1. **Nested element context not tracked**: Inside `.value | map(select(.name`, the system:
   - Correctly identifies we're in an element context (`select`)
   - But navigates from the cached `to_entries` result
   - Gets the `{key, value}` structure, suggesting `.value` again

2. **`.value` contents are opaque**: After `to_entries`:
   - `.key` is always a string (the original object's key)
   - `.value` is the original object's value (unknown structure)
   - The system should recognize `.value` as "opaque/unknown"

3. **Nested transformations compound the problem**:
   - `.value | map(...)` creates a second layer of transformation
   - We're now iterating over `.value`'s contents (if it's an array)
   - The field structure at `.name` is completely unknown

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

### Phase 1: Detect `to_entries` + `.value` Pattern

**Goal:** When inside `.value` after `to_entries`, recognize context is opaque.

```rust
/// Check if cursor is in an opaque context after a transforming function
fn is_in_opaque_value_context(query: &str, cursor_pos: usize) -> bool {
    let before_cursor = &query[..cursor_pos];

    // Find the most recent to_entries
    if let Some(to_entries_pos) = before_cursor.rfind("to_entries") {
        let after_to_entries = &before_cursor[to_entries_pos..];

        // Check if we're accessing .value (with possible array iteration)
        // Patterns: .value, .[].value, .[0].value, etc.
        let value_access_pattern = Regex::new(r"\.\[?\]?\.?value").unwrap();

        if value_access_pattern.is_match(after_to_entries) {
            // Check if there's a nested context after .value
            if let Some(value_pos) = after_to_entries.rfind(".value") {
                let after_value = &after_to_entries[value_pos + 6..]; // ".value".len() == 6

                // If there's a pipe or element context after .value, we're in opaque context
                if after_value.contains('|') ||
                   after_value.contains("map(") ||
                   after_value.contains("select(") ||
                   after_value.contains('.') // Any field access
                {
                    return true;
                }
            }
        }
    }

    false
}
```

### Phase 2: Fall Back to All Fields

When in opaque context, show all fields from original JSON:

```rust
// In get_suggestions(), add check before navigation:
if is_in_opaque_value_context(query, cursor_pos) {
    return get_all_field_suggestions(&all_field_names, partial);
}
```

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

### 5. Mixed Access Patterns
```jq
to_entries | map({k: .key, v: .value.services.web.})
```
`.value.services.web` can be navigated if we track through the original JSON.

**Expected:** Navigate through original JSON to find `.value` → `.services` → `.web` → suggest `name`, `port`.

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
    // to_entries | map({... .value | map(select(.name
    assert!(is_in_opaque_value_context(
        "to_entries | map({service: .key, config: .value | map(select(.name",
        67  // cursor at end
    ));
}

#[test]
fn test_not_opaque_direct_value_access() {
    // to_entries | .[].value.
    // Direct .value access - can navigate through original JSON
    assert!(!is_in_opaque_value_context(
        "to_entries | .[].value.",
        23
    ));
}

#[test]
fn test_opaque_value_with_pipe() {
    // to_entries | .[].value | .
    assert!(is_in_opaque_value_context(
        "to_entries | .[].value | .",
        26
    ));
}

#[test]
fn test_entry_field_suggestions() {
    // to_entries | .[].
    assert!(should_suggest_entry_fields("to_entries | .[].", 17));
}
```

### Integration Tests

```rust
#[test]
fn test_suggestions_after_to_entries_value_nested() {
    let json = r#"{"services": {"web": {"name": "api"}}}"#;
    let app = app_with_json(json);

    // Type the problematic query
    simulate_typing(&mut app, "to_entries | map({service: .key, config: .value | map(select(.");

    let suggestions = app.autocomplete.suggestions();

    // Should suggest all fields, not .value
    assert!(suggestions.iter().any(|s| s.text.contains("services") || s.text.contains("name")));
    assert!(!suggestions.iter().any(|s| s.text == ".value"));
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

## Related Work

- **PLAN_NESTED_AUTOSUGGESTION.md**: Phase 4 Notes mention transforming function detection was removed for executing context. This plan addresses non-executing context after transforming functions.
- **context.rs:481-483**: Existing `with_entries` special handling provides a pattern to follow.
- **all_field_names cache**: Already implemented in `JqExecutor` for fallback suggestions.
