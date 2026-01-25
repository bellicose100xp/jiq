# Performance Optimization Plan

## Overview

This document outlines performance optimization opportunities identified through deep analysis of the JIQ codebase. The focus is on improving rendering speed and execution responsiveness.

## Implementation Guidelines

1. **Commit after each phase** - Each phase should be committed separately with a descriptive commit message
2. **100% test coverage** - All new code must have complete test coverage before committing
3. **Manual TUI testing** - Verify functionality manually before marking phase complete
4. **Update docs for deviations** - Any changes made during implementation that differ from the original plan must be documented. Update architecture decisions and modify affected later phases to account for these changes

---

## Improvement #1: Cache Line Widths (HIGHEST IMPACT)

**Location:** `src/results/results_render.rs:146-151`

**The Problem:**

Every single time the screen renders, this code runs:

```rust
let widths: Vec<u16> = unformatted
    .lines()
    .map(|l| l.len().min(u16::MAX as usize) as u16)
    .collect();
app.results_cursor.update_line_widths(std::sync::Arc::new(widths));
```

This iterates through **every line** in the JSON result and creates a new vector with the width of each line.

**Why it's bad:**
- If you have a 100,000 line JSON result, this allocates a 200KB vector (~2 bytes per line)
- This happens on EVERY render frame (whenever you scroll, type, or anything changes)
- The result doesn't change between renders - the line widths are the same!

**The Fix:**

Compute line widths ONCE when the query result comes back (in the worker thread), store it in `QueryState`, and just reference that cached value during rendering.

```rust
// In ProcessedResult (worker output):
pub line_widths: Arc<Vec<u16>>,

// In QueryState:
pub cached_line_widths: Option<Arc<Vec<u16>>>,

// In results_render.rs - just reference the cached value:
if let Some(widths) = &q.cached_line_widths {
    app.results_cursor.update_line_widths(Arc::clone(widths));
}
```

**Impact:** Eliminates O(n) work per frame. Massive win for large files.

**Priority:** HIGH

---

## Improvement #2: Single-Pass Line Metrics

**Location:** `src/query/worker/preprocess.rs:41-47`

**The Problem:**

When processing a query result, this code runs:

```rust
let line_count = output.lines().count() as u32;
let max_width = output
    .lines()
    .map(|l| l.len())
    .max()
    .unwrap_or(0)
    .min(u16::MAX as usize) as u16;
```

**Why it's bad:**
- This iterates through ALL lines twice - once to count them, once to find the longest
- For a 100,000 line file, that's 200,000 iterations instead of 100,000

**The Fix:**

Combine into a single loop:

```rust
fn compute_line_metrics(output: &str) -> (u32, u16) {
    let mut line_count: u32 = 0;
    let mut max_width: usize = 0;

    for line in output.lines() {
        line_count += 1;
        if line.len() > max_width {
            max_width = line.len();
        }
    }

    (line_count, max_width.min(u16::MAX as usize) as u16)
}
```

**Impact:** ~2x faster line metrics computation. Runs in worker thread so doesn't block UI, but faster is still better.

**Priority:** Medium

---

## Improvement #3: Eliminate Duplicate JSON Parsing

**Location:** `src/query/worker/preprocess.rs:59-61`

**The Problem:**

Two separate functions parse the same JSON:

```rust
let parsed = parse_first_value(&unformatted).map(Arc::new);
let result_type = detect_result_type(&unformatted);
```

- `parse_first_value()` (line 179-193) parses JSON to get a `Value`
- `detect_result_type()` (line 201-232) ALSO parses JSON to determine if it's an Object, Array, String, etc.

**Why it's bad:**
- JSON parsing is expensive - it validates syntax, allocates memory for the structure
- We're doing this work twice on the exact same string
- For large JSON results, this is wasteful

**Important Trade-off:**

The current implementation has nuances that must be preserved:

1. **Fast-path optimization**: `parse_first_value` uses `serde_json::from_str` first (faster for single JSON values - the common case), only falling back to streaming for destructured output.

2. **Multiple value detection**: `detect_result_type` must peek at the SECOND value to distinguish:
   - `{"a":1}` → `ResultType::Object` (single object)
   - `{"a":1}\n{"b":2}` → `ResultType::DestructuredObjects` (multiple objects)

**The Fix:**

Combine into a single function that preserves both behaviors:

```rust
fn parse_and_detect_type(text: &str) -> (Option<Value>, ResultType) {
    let text = text.trim();
    if text.is_empty() {
        return (None, ResultType::Null);
    }

    // FAST PATH: Try full parse first (common case: single value)
    // This is more efficient than streaming for well-formed single JSON values
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        // Single value - determine type directly from parsed value
        // No need to check for multiple values since full parse succeeded
        // (full parse fails on destructured output like `{"a":1}\n{"b":2}`)
        let result_type = match &value {
            Value::Object(_) => ResultType::Object,
            Value::Array(arr) => {
                if arr.is_empty() {
                    ResultType::Array
                } else if matches!(arr.first(), Some(Value::Object(_))) {
                    ResultType::ArrayOfObjects
                } else {
                    ResultType::Array
                }
            }
            Value::String(_) => ResultType::String,
            Value::Number(_) => ResultType::Number,
            Value::Bool(_) => ResultType::Boolean,
            Value::Null => ResultType::Null,
        };
        return (Some(value), result_type);
    }

    // FALLBACK: Streaming parse for destructured output (multiple JSON values)
    let mut deserializer = serde_json::Deserializer::from_str(text).into_iter();

    let first_value = match deserializer.next() {
        Some(Ok(v)) => v,
        _ => return (None, ResultType::Null),
    };

    // Check for multiple values (destructured output)
    let has_multiple = deserializer.next().is_some();

    let result_type = match &first_value {
        Value::Object(_) if has_multiple => ResultType::DestructuredObjects,
        Value::Object(_) => ResultType::Object,
        Value::Array(arr) => {
            if arr.is_empty() {
                ResultType::Array
            } else if matches!(arr.first(), Some(Value::Object(_))) {
                ResultType::ArrayOfObjects
            } else {
                ResultType::Array
            }
        }
        Value::String(_) => ResultType::String,
        Value::Number(_) => ResultType::Number,
        Value::Bool(_) => ResultType::Boolean,
        Value::Null => ResultType::Null,
    };

    (Some(first_value), result_type)
}
```

**Key points:**
- Fast-path (`from_str`) preserved for single values (most common case)
- Streaming fallback only used when fast-path fails (destructured output)
- Multiple value detection preserved in streaming path

**Impact:** ~50% reduction in JSON parsing time during preprocessing (one parse instead of two).

**Priority:** Medium

---

## Summary

| # | Improvement | Impact | Priority |
|---|-------------|--------|----------|
| 1 | Cache Line Widths | Eliminates O(n) per frame | **HIGH** |
| 2 | Single-Pass Line Metrics | ~2x faster metrics | Medium |
| 3 | Eliminate Duplicate JSON Parsing | ~50% less parse time | Medium |

### Implementation Order

1. **Improvement #1** - Highest impact, should be done first
2. **Improvement #2** - Simple refactor, quick win
3. **Improvement #3** - Moderate refactor, good improvement
