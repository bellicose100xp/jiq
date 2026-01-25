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

**The Fix:**

Combine into a single function that parses once and returns both:

```rust
fn parse_and_detect_type(text: &str) -> (Option<Value>, ResultType) {
    // Parse JSON once
    let value = parse_first_value(text);

    // Determine type from the already-parsed value (no re-parsing!)
    let result_type = match &value {
        Some(Value::Object(_)) => ResultType::Object,
        Some(Value::Array(arr)) if arr.first().map(|v| v.is_object()).unwrap_or(false) => {
            ResultType::ArrayOfObjects
        }
        Some(Value::Array(_)) => ResultType::Array,
        Some(Value::String(_)) => ResultType::String,
        Some(Value::Number(_)) => ResultType::Number,
        Some(Value::Bool(_)) => ResultType::Boolean,
        Some(Value::Null) | None => ResultType::Null,
    };

    (value, result_type)
}
```

**Impact:** ~50% reduction in JSON parsing time during preprocessing.

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
