# Performance Optimization Plan

## Overview

This document outlines performance optimization opportunities identified through deep analysis of the JIQ codebase. The focus is on improving rendering speed and execution responsiveness.

## Implementation Guidelines

1. **Commit after each phase** - Each phase should be committed separately with a descriptive commit message
2. **100% test coverage** - All new code must have complete test coverage before committing
3. **Manual TUI testing** - Verify functionality manually before marking phase complete
4. **Update docs for deviations** - Any changes made during implementation that differ from the original plan must be documented. Update architecture decisions and modify affected later phases to account for these changes

## Phase Checklist

- [x] Improvement #1: Cache Line Widths
- [x] Improvement #2: Single-Pass Line Metrics (implemented with #1)
- [x] Improvement #3: Eliminate Duplicate JSON Parsing
- [ ] Improvement #4: Use `into_owned()` in `apply_dim_to_text`
- [ ] Improvement #5: Use `into_owned()` in `apply_cursor_highlights`

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

**Current Flow vs New Flow:**

| Step | Current (wasteful) | After Fix (efficient) |
|------|-------------------|----------------------|
| 1 | `parse_first_value()` parses 10MB JSON → ~50ms | `parse_and_detect_type()` parses 10MB JSON → ~50ms |
| 2 | `detect_result_type()` parses 10MB JSON AGAIN → ~50ms | Check type of already-parsed value → ~0ms |
| **Total** | **~100ms** | **~50ms** |

The fix simply avoids parsing the same JSON twice. Once we have the parsed `Value` from step 1, checking its type is instant (just a `match` statement).

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

## Improvement #4: Use `into_owned()` in `apply_dim_to_text`

**Location:** `src/results/results_render.rs:501-520`

**The Problem:**

When showing stale results (error state or empty result), this function dims all text:

```rust
fn apply_dim_to_text(text: Text<'_>) -> Text<'static> {
    Text::from(
        text.lines
            .into_iter()
            .map(|line| {
                Line::from(
                    line.spans
                        .into_iter()
                        .map(|span| {
                            Span::styled(
                                span.content.to_string(),  // <-- CLONES the string!
                                span.style.add_modifier(Modifier::DIM),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
```

**Why it's bad:**

The pre-rendered text (`last_successful_result_rendered`) already contains **owned strings** (`Cow::Owned`). These were created in the worker thread via `into_text()`. When we call `to_string()` on an already-owned `Cow::Owned(String)`, we **clone the entire string unnecessarily**.

For a 50-line viewport with average 80 characters per line:
- `to_string()`: Allocates 50 new strings, copies ~4,000 bytes
- `into_owned()`: Zero allocations, just transfers ownership

**The Fix:**

Use `into_owned()` instead of `to_string()`. For `Cow::Owned`, this extracts the String without cloning:

```rust
fn apply_dim_to_text(text: Text<'_>) -> Text<'static> {
    Text::from(
        text.lines
            .into_iter()
            .map(|line| {
                Line::from(
                    line.spans
                        .into_iter()
                        .map(|span| {
                            Span::styled(
                                span.content.into_owned(),  // No clone for Cow::Owned!
                                span.style.add_modifier(Modifier::DIM),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
```

**How `into_owned()` works:**

```rust
impl Cow<'_, str> {
    fn into_owned(self) -> String {
        match self {
            Cow::Borrowed(s) => s.to_owned(),  // Clone only if borrowed
            Cow::Owned(s) => s,                // No clone - just return the String!
        }
    }
}
```

**Time Savings:**

| Scenario | `to_string()` | `into_owned()` | Savings |
|----------|--------------|----------------|---------|
| 50 lines × 80 chars | ~50 allocations + 4KB copy | 0 allocations | ~100% |
| Per render frame | ~50-100μs | ~0μs | ~50-100μs/frame |
| At 10fps during error state | ~500-1000μs/sec | ~0μs/sec | ~0.5-1ms/sec |

**When this matters:** Every render frame while showing stale results (syntax error or empty result state).

**Priority:** Medium

---

## Improvement #5: Use `into_owned()` in `apply_cursor_highlights`

**Location:** `src/results/results_render.rs:636-682`

**The Problem:**

When the results pane is focused, this function highlights the cursor line:

```rust
fn apply_cursor_highlights(
    text: Text<'_>,
    cursor_state: &CursorState,
    scroll_offset: u16,
) -> Text<'static> {
    // ...
    Text::from(
        text.lines
            .into_iter()
            .enumerate()
            .map(|(line_idx, line)| {
                let bg_color = /* determine if this line needs highlighting */;

                if let Some(bg) = bg_color {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| Span::styled(
                                span.content.to_string(),  // <-- CLONES!
                                span.style.bg(bg)
                            ))
                            .collect::<Vec<_>>(),
                    )
                } else {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| Span::styled(
                                span.content.to_string(),  // <-- CLONES even unchanged lines!
                                span.style
                            ))
                            .collect::<Vec<_>>(),
                    )
                }
            })
            .collect::<Vec<_>>(),
    )
}
```

**Why it's bad:**

1. **ALL lines are cloned**, even lines that don't need cursor highlighting (the `else` branch)
2. The pre-rendered text already contains owned strings, so `to_string()` clones unnecessarily
3. Only 1-3 lines typically need modification (cursor line, maybe hover line, maybe visual selection)

**The Fix:**

Use `into_owned()` instead of `to_string()`:

```rust
fn apply_cursor_highlights(
    text: Text<'_>,
    cursor_state: &CursorState,
    scroll_offset: u16,
) -> Text<'static> {
    // ...
    Text::from(
        text.lines
            .into_iter()
            .enumerate()
            .map(|(line_idx, line)| {
                let bg_color = /* ... */;

                if let Some(bg) = bg_color {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| Span::styled(
                                span.content.into_owned(),  // No clone!
                                span.style.bg(bg)
                            ))
                            .collect::<Vec<_>>(),
                    )
                } else {
                    Line::from(
                        line.spans
                            .into_iter()
                            .map(|span| Span::styled(
                                span.content.into_owned(),  // No clone!
                                span.style
                            ))
                            .collect::<Vec<_>>(),
                    )
                }
            })
            .collect::<Vec<_>>(),
    )
}
```

**Time Savings:**

| Scenario | `to_string()` | `into_owned()` | Savings |
|----------|--------------|----------------|---------|
| 50 lines × 80 chars | ~50 allocations + 4KB copy | 0 allocations | ~100% |
| Per render frame | ~50-100μs | ~0μs | ~50-100μs/frame |
| At 10fps with cursor in results | ~500-1000μs/sec | ~0μs/sec | ~0.5-1ms/sec |

**When this matters:** Every render frame while results pane is focused (user navigating results with j/k keys, visual selection, etc.).

**Priority:** Medium

---

## Summary

| # | Improvement | Impact | Priority | Status |
|---|-------------|--------|----------|--------|
| 1 | Cache Line Widths | Eliminates O(n) per frame | **HIGH** | ✅ Done |
| 2 | Single-Pass Line Metrics | ~2x faster metrics | Medium | ✅ Done |
| 3 | Eliminate Duplicate JSON Parsing | ~50% less parse time | Medium | ✅ Done |
| 4 | `into_owned()` in `apply_dim_to_text` | ~50-100μs/frame savings | Medium | Pending |
| 5 | `into_owned()` in `apply_cursor_highlights` | ~50-100μs/frame savings | Medium | Pending |

### Implementation Order

**Completed:**
1. **Improvement #1** - Highest impact, done first
2. **Improvement #2** - Simple refactor, quick win
3. **Improvement #3** - Moderate refactor, good improvement

**Pending:**
4. **Improvement #4** - Single line change, eliminates cloning during error/empty states
5. **Improvement #5** - Single line change, eliminates cloning when results pane focused
