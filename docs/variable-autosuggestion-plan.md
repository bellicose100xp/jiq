# Variable Autosuggestion Implementation Plan

## Overview

Add autosuggestion for jq variables that start with `$`. Show suggestions when the user intends to **use** a variable, but not when **defining** a variable (e.g., `as $var`).

## Design Decisions

| Decision | Choice |
|----------|--------|
| Variable scope | Show all declared variables from entire query |
| Built-in variables | Always suggest `$ENV`, `$__loc__` |
| Case sensitivity | Case-sensitive matching (jq behavior) |
| Duplicate handling | Show each variable once (deduplicated) |
| Description text | None |

## jq Variable Definition Patterns

Variables can be created in jq via:

1. **`as $var`** - Basic binding: `expr as $var | ...`
2. **`reduce expr as $var (init; update)`** - Reduce accumulator
3. **`foreach expr as $var (init; update; extract)`** - Foreach iteration
4. **Destructuring arrays** - `as [$first, $second]`
5. **Destructuring objects** - `as {name: $name}`
6. **`label $name`** - Control flow labels

## Implementation Phases

### Phase 1: Variable Extractor Module

**New file: `src/autocomplete/variable_extractor.rs`**

```rust
/// Extracts all unique variable names defined in the query.
/// Returns a deduplicated list including built-in variables.
pub fn extract_variables(query: &str) -> Vec<String>
```

**Logic:**
1. Scan for `as $name` patterns (handles `as`, `reduce...as`, `foreach...as`)
2. Handle destructuring: `as [$a, $b]` and `as {key: $c}`
3. Handle `label $name`
4. Skip variables inside strings
5. Add built-ins: `$ENV`, `$__loc__`
6. Deduplicate and return

### Phase 2: Context Detection

**Update: `src/autocomplete/context.rs`**

Add `VariableContext` to `SuggestionContext` enum.

**Detection logic in `analyze_context()`:**
- If partial starts with `$` AND not in definition context → `VariableContext`

**New helper:**
```rust
fn is_in_variable_definition_context(before_cursor: &str) -> bool
```

Definition context = cursor immediately after:
- `as $` (with optional whitespace)
- `as [` ... `$` (in array destructure)
- `as {` ... `$` (in object destructure)
- `label $`

### Phase 3: Suggestion Generation

**Update `get_suggestions()` in `context.rs`:**

```rust
SuggestionContext::VariableContext => {
    let all_vars = extract_variables(query);
    let suggestions: Vec<Suggestion> = all_vars
        .into_iter()
        .map(|name| Suggestion::new_with_type(name, SuggestionType::Variable, None))
        .collect();
    filter_suggestions_by_partial_case_sensitive(suggestions, &partial)
}
```

### Phase 4: State & Rendering Updates

- Add `Variable` to `SuggestionType` enum in `autocomplete_state.rs`
- Add Red color for variables in `autocomplete_render.rs`

### Phase 5: Insertion Logic

- Insert full variable name (with `$`)
- Replace partial being typed

## Test Scenarios

### A. Basic Usage (show suggestions)

| Query (cursor at `\|`) | Expected |
|------------------------|----------|
| `. as $x \| $\|` | `$x`, `$ENV`, `$__loc__` |
| `reduce .[] as $item (0; $\|)` | `$item`, `$ENV`, `$__loc__` |
| `$\|` | `$ENV`, `$__loc__` (no user vars) |

### B. Definition Context (NO suggestions)

| Query (cursor at `\|`) | Expected |
|------------------------|----------|
| `. as $\|` | No suggestions |
| `reduce .[] as $\|` | No suggestions |
| `foreach .[] as $\|` | No suggestions |
| `label $\|` | No suggestions |
| `. as [$\|` | No suggestions |
| `. as [$a, $\|` | No suggestions |
| `. as {key: $\|` | No suggestions |

### C. Filtering (case-sensitive)

| Query | Expected |
|-------|----------|
| `. as $Item \| $it\|` | No match (case-sensitive) |
| `. as $item \| $it\|` | `$item` |
| `. as $Item \| $It\|` | `$Item` |
| `$E\|` | `$ENV` |
| `$__\|` | `$__loc__` |

### D. Multiple Variables (deduplicated)

| Query | Expected |
|-------|----------|
| `. as $x \| . as $x \| $\|` | `$x` (once), `$ENV`, `$__loc__` |
| `. as $a \| . as $b \| $\|` | `$a`, `$b`, `$ENV`, `$__loc__` |

### E. Destructuring

| Query | Expected vars extracted |
|-------|-------------------------|
| `. as [$first, $second] \| $\|` | `$first`, `$second` |
| `. as {name: $n, age: $a} \| $\|` | `$n`, `$a` |

### F. Strings (variables in strings ignored)

| Query | Expected |
|-------|----------|
| `"as $fake" \| $\|` | `$ENV`, `$__loc__` only |
| `. as $real \| "as $fake" \| $\|` | `$real`, `$ENV`, `$__loc__` |

### G. Mid-query Editing

| Query | Expected |
|-------|----------|
| `. as $x \| $\| \| . as $y` | `$x`, `$y`, `$ENV`, `$__loc__` |
| `$\| \| . as $z` | `$z`, `$ENV`, `$__loc__` |

### H. Edge Cases

| Query | Expected |
|-------|----------|
| `$\|` (empty otherwise) | `$ENV`, `$__loc__` |
| `. as $my_var \| $my\|` | `$my_var` |
| `. as $x123 \| $x\|` | `$x123` |

## File Changes Summary

| File | Action |
|------|--------|
| `variable_extractor.rs` | NEW |
| `variable_extractor_tests.rs` | NEW |
| `context.rs` | MODIFY |
| `context_tests/variable_context_tests.rs` | NEW |
| `autocomplete_state.rs` | MODIFY (add Variable type) |
| `insertion.rs` | MODIFY |
| `insertion_tests/variable_insertion_tests.rs` | NEW |
| `autocomplete_render.rs` | MODIFY (add color) |
