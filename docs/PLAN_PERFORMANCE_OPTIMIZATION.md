# Performance Optimization Plan

## Overview

This document outlines performance optimization opportunities identified through deep analysis of the JIQ codebase. The focus is on improving rendering speed and execution responsiveness.

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
