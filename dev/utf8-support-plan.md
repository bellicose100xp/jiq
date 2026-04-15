# UTF-8 Support — Phased Implementation Plan

**Issue:** [#153 — Querying with non-ASCII characters causes crash](https://github.com/bellicose100xp/jiq/issues/153)
**Branch:** `utf8-support`
**Status:** Plan — not yet implemented

---

## Root Cause

`tui_textarea::TextArea::cursor().1` returns a **character index** (documented at `tui-textarea-0.7.0/src/textarea.rs:2026` as "0-base character-wise"), but the autocomplete module uses it directly as a **byte index** for Rust string slicing (`&query[cursor_pos..]`). For ASCII-only input, char index == byte index, so it works. For multi-byte UTF-8 characters (CJK, emoji, accented letters), they diverge — and Rust panics when you slice at a non-character-boundary byte offset.

**Example:** For `"Hello👋World"`, character position 6 (the `W`) corresponds to byte position 8 (the emoji is 4 bytes). `&query[6..]` panics because byte 6 is inside the emoji.

---

## Confirmed Crash Sites

| # | File | Line(s) | Pattern | Severity |
|---|------|---------|---------|----------|
| 1 | `insertion.rs` | 30-32 | `&query[..replacement_start]`, `&query[cursor_pos..]` | CRITICAL |
| 2 | `insertion.rs` | 199 | `&query[..cursor_pos.min(query.len())]` | CRITICAL |
| 3 | `insertion.rs` | 39, 102, 120, 138, 177, 179 | `cursor_pos.saturating_sub(partial.len())` — byte/char arithmetic | CRITICAL |
| 4 | `context.rs` | 627 | `&query[..cursor_pos.min(query.len())]` in `get_suggestions` | CRITICAL |
| 5 | `context.rs` | 378 | Same pattern in `detect_entry_context` | CRITICAL |
| 6 | `context.rs` | 853 | `query[cursor_pos..]` in `is_cursor_at_logical_end` | CRITICAL |
| 7 | `context.rs` | 829 | `before_cursor[..search_end]` with byte arithmetic | HIGH |
| 8 | `context.rs` | 175 | `before_cursor.chars().nth(dot_pos)` where `dot_pos` is byte-based | HIGH |
| 9 | `autocomplete_render.rs` | 96 | `&display_text[..available_for_text.saturating_sub(3)]` byte truncation | HIGH |

**Logic bug (no crash, wrong behavior):** `BraceTracker.BraceInfo.pos` stores byte positions (from `char_indices()`), but callers pass char-index `cursor_pos` to `context_at()`, `is_in_object()`, `is_in_element_context()`, `is_in_non_executing_context()`. Byte vs char comparison gives wrong autocomplete context for non-ASCII queries.

**NOT affected:** Input rendering pipeline (`input_state.rs`, `input_render.rs`), syntax highlighting, overlay modules all work consistently in character space.

---

## Extended Scope — Phases 8–9

**Why extended:** The crash fix (Phases 1–7) stops the panic, but surfaced a separate defect: autocomplete emits `.名前` for CJK keys, which is invalid jq syntax (jq only accepts ASCII `[A-Za-z_][A-Za-z_0-9]*` as identifier-index). Users typing a non-ASCII field get "syntax error" when accepting the suggestion — the feature's most compelling UX is broken. Ship together.

**Root cause:** `result_analyzer.rs:21-27` `is_simple_jq_identifier` uses Rust's `char::is_alphanumeric()` which is **Unicode-aware** — so CJK, accented Latin, etc. all incorrectly classify as identifiers. Fix: restrict to `is_ascii_alphanumeric()` + `is_ascii_digit()`.

**Emission fix:** Non-identifier keys currently emit quoted-dot `."name"`. Bracket notation `.["name"]` is preferred (unambiguous, composes cleanly, matches tooling convention). Applies uniformly to all emission sites.

### Phase 8 — Identifier classification + bracket-notation emission

**Production changes (single file: `src/autocomplete/result_analyzer.rs`, minimal, DRY):**

1. Line 21-27: tighten `is_simple_jq_identifier` to ASCII-only:
   ```rust
   !first_char.is_ascii_digit() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
   ```

2. Introduce DRY helper for bracket notation:
   ```rust
   fn format_bracket_access(key: &str) -> String {
       format!("[\"{}\"]", key)
   }
   ```

3. Update the two non-identifier emission branches to use it:
   - `format_field_name` line 34: `format!("{}{}", prefix, Self::format_bracket_access(name))`
   - `extract_union_fields_from_array` line 74: `format!("{}[]{}", prefix, Self::format_bracket_access(key))` — note: no dot between `[]` and `[`

**Why no ASCII regression:**
- `is_simple_jq_identifier("name")` — true before, true after
- `is_simple_jq_identifier("_foo")` — true before, true after
- `is_simple_jq_identifier("1foo")` — false before, false after (digit start)
- `is_simple_jq_identifier("foo-bar")` — false before, false after (hyphen)
- ASCII keys flow through the identifier branch unchanged; only the non-identifier emission format changes, and ASCII keys that were non-identifiers (e.g., `"foo-bar"`) previously emitted `."foo-bar"` — those users will now get `.["foo-bar"]`, which is equivalent jq (both valid). This is a cosmetic change for ASCII non-identifiers, not a regression.

**Validation gate:**
- All existing result_analyzer tests pass (ASCII parity baseline)
- New unit tests in `src/autocomplete/result_analyzer_tests.rs`:
  - `is_simple_jq_identifier` classification matrix: ASCII identifiers (true), ASCII non-identifiers (false), CJK (false), emoji (false), accented (false), empty (false), digit-start (false)
  - `format_field_name` output shape: ASCII → `.name`, CJK → `.["名前"]`, emoji → `.["👋"]`, accented → `.["café"]`
  - `format_bracket_access` helper: produces exact `["name"]` form
  - Extract-from-object tests covering multibyte keys

### Phase 9 — End-to-end bracket-notation integration

**No production changes** — test-only. Validates that Phase 8's emission change flows correctly through the whole autocomplete pipeline (context analysis → filtering → insertion → cursor landing).

**New tests extend `src/autocomplete/insertion_tests/utf8_tests.rs`** (existing file from Phase 6):

```rust
mod bracket_notation_emission {
    // Selecting a CJK field suggestion produces .["名前"] not .名前
    fn cjk_field_inserts_bracket_notation() { ... }
    // Same for emoji, accented
    fn emoji_field_inserts_bracket_notation() { ... }
    fn accented_field_inserts_bracket_notation() { ... }
    // Partial match replacement: ".名" + select → .["名前"]
    fn partial_cjk_match_replaces_correctly() { ... }
    fn partial_emoji_match_replaces_correctly() { ... }
    // Array iteration: .items[] → .items[]["名前"] (no extra dot)
    fn cjk_field_in_array_iteration_context() { ... }
    // Mixed: ASCII field in path, then CJK field
    fn ascii_then_cjk_produces_mixed_notation() { ... }
    // Cursor lands past the closing ]
    fn cursor_lands_after_closing_bracket() { ... }
}

mod ascii_notation_unchanged {
    // Explicit regression guards — ASCII still uses dot notation
    fn ascii_identifier_inserts_dot_notation() { ... }
    fn ascii_with_underscore_inserts_dot_notation() { ... }
    // ASCII non-identifier (hyphen, space) now uses bracket — cosmetic change
    fn ascii_nonidentifier_inserts_bracket_notation() { ... }
}
```

**Validation gate:**
- All existing insertion tests pass unchanged (2942+ baseline)
- All new tests pass
- Manual TUI validation with `tests/fixtures/utf8-cjk.json`, `utf8-emoji.json`, `utf8-accented.json`: typing `.` + selecting a field produces valid jq that returns the expected value (no syntax error)

### Phase 8–9 Risk Analysis

| Risk | Mitigation |
|------|-----------|
| R9 — ASCII identifier regression via tightened classifier | Unit test matrix covers every existing ASCII class; cargo test confirms zero regression |
| R10 — Cosmetic change for ASCII non-identifiers (hyphen, space keys) | Documented as intentional; bracket form is equivalent jq and composes better. Pre-existing tests that assert exact `."foo-bar"` output get updated to `.["foo-bar"]` with a comment pointing to this plan |
| R11 — Insertion math wrong for bracket-prefixed suggestions | Advisor traced: `.["名前"]` starts with `.` → hits existing `starts_with('.')` branch in insertion.rs:175; `replacement_start = cursor_pos - partial.len() - 1` correctly replaces `.名` with `.["名前"]`. Proven by unit tests in Phase 9 |
| R12 — Filter matching fails because suggestion text now has brackets/quotes | Advisor confirmed: substring match in `filter_suggestions_by_partial` still matches `会` inside `.["会社"]`. Covered by partial-match tests |
| R13 — Key escaping for keys containing `"` or `\\` | **Out of scope.** Pre-existing defect — current code doesn't escape either. Document as known limitation; separate issue for follow-up. Extreme edge case; crash fix doesn't depend on it |

## Phased Implementation Plan

**Incremental validation strategy:** Each phase ends with a validation gate — `cargo test`, `cargo clippy`, and the phase-specific test subset must all pass before the next phase starts. No phase touches more than one logical concern. A broken phase is rolled back, not layered over.

### Phase Dependency Graph

```
P1 (str_utils, standalone) ──┐
                             ├──► P2a (autocomplete_state ingress) ──► P2b (context.rs line 175)
                             │                                              │
                             └──► P3a (insert_suggestion ingress) ──► P3b (move_cursor back-conv) ──► P3c (line 162)
                                                                                                        │
                                  P4 (render truncation)  ◄──────────────────────────────────────────┘
                                                                            │
                                  P5 (BraceTracker UTF-8 tests — validation only)
                                                                            │
                                  P6 (E2E)  ──►  P7 (regression lock-in)
```

### Design Principle — Ingress Normalization

**Convert cursor_pos from char-index to byte-index ONCE at each entry point, not at every slicing site.** Downstream code then treats `cursor_pos` uniformly as a byte offset, and all `&query[..cursor_pos]` slicing is automatically safe. This is DRY — a single centralized conversion instead of ~10 site-specific fixes, and no way to "miss a site" in future code.

**Two entry points only:**
1. `autocomplete_state::update_suggestions_from_app` — where `textarea.cursor().1` is read
2. `insertion::insert_suggestion` (line 198) — same source

**Two back-conversions (byte→char) required:**
1. `move_cursor_to_column` — tui_textarea expects char positions
2. `insertion.rs:162` `query.chars().nth(cursor_pos - 1)` → rewrite as `query[..cursor_pos].chars().last()` (no conversion needed, works directly on bytes)

### Phase 1 — Utility Foundation

Create `src/str_utils.rs` with **two** conversion helpers (minimal surface area):

- **`char_pos_to_byte_pos(s: &str, char_pos: usize) -> usize`** — char index → byte offset via `char_indices()`. Clamps to `s.len()` when past end.
- **`byte_pos_to_char_pos(s: &str, byte_pos: usize) -> usize`** — inverse, for the two back-conversion sites.

**Tests — `src/str_utils_tests.rs`** (registered via `#[cfg(test)] #[path = ...] mod str_utils_tests;`):

Unit tests:
- Both functions with ASCII, 2-byte (é), 3-byte (中), 4-byte (😀), mixed, empty, boundary (0, len, past-end)

Property-based tests (proptest, following `brace_tracker_tests.rs`):
- `prop_roundtrip` — `byte_pos_to_char_pos(s, char_pos_to_byte_pos(s, n))` equals `n.min(char_count)` for arbitrary UTF-8
- `prop_byte_pos_always_on_boundary` — result is always a valid UTF-8 boundary (`s.is_char_boundary(result)`)

### Phase 2a — Ingress at `autocomplete_state::update_suggestions_from_app`

**Single production change:** in `update_suggestions_from_app`, convert `cursor_pos` from char to byte once:
```rust
let cursor_char = app.input.textarea.cursor().1;
let query = app.input.query();
let cursor_pos = str_utils::char_pos_to_byte_pos(query, cursor_char);
```

**Why this alone?** This one conversion fixes crash sites in `context.rs:378,627,853` automatically (they all slice `&query[..cursor_pos]`). No other production code changes in this phase.

**Validation gate:**
- All existing tests pass unchanged (ASCII parity baseline — R1)
- Add `ascii_parity_tests.rs` corpus tests (see Regression Test Matrix). They pass.
- Add 3 minimal UTF-8 smoke tests: "typing `.名` does not panic", "typing `.👋` does not panic", "typing `.é` does not panic". They pass.

### Phase 2b — Fix `context.rs:175` `needs_leading_dot`

**Single production change:** line 175 only.
```rust
// Before: before_cursor.chars().nth(dot_pos) == Some('.')
// After:  before_cursor.as_bytes().get(dot_pos) == Some(&b'.')
```

**Validation gate:**
- All existing tests pass unchanged
- Add unit tests for `needs_leading_dot` with multibyte content before dot

### Phase 3a — Ingress at `insert_suggestion`

**Single production change:** `insertion.rs` line 198, analogous to Phase 2a.
```rust
let cursor_char = textarea.cursor().1;
let cursor_pos = str_utils::char_pos_to_byte_pos(&query, cursor_char);
```

**Why safe without further fixes in this phase?** Downstream `replacement_start = cursor_pos.saturating_sub(partial.len())` is byte-vs-byte (partial comes from byte-sliced `before_cursor`). Slicing on lines 30, 32, 199 is now safe. The remaining issues are isolated to line 39 and 162, fixed in 3b/3c.

**Validation gate:**
- Existing insertion tests pass (ASCII parity)
- Add smoke tests: inserting a suggestion when query contains `é`, `中`, `👋` does not panic

### Phase 3b — Back-convert for `move_cursor_to_column` (line 39)

**Single production change:** wrap the one call site in `replace_partial_at_cursor`:
```rust
let target_char = str_utils::byte_pos_to_char_pos(&new_query, target_pos);
move_cursor_to_column(textarea, target_char);
```

**Validation gate:**
- Add `cursor_landing_tests.rs` (see Regression Test Matrix). All pass.
- Verify visually via manual TUI test with CJK query (user-validated per CLAUDE.md pre-commit checklist)

### Phase 3c — Fix `insertion.rs:162` char-before lookup

**Single production change:**
```rust
// Before: let char_before = query.chars().nth(cursor_pos - 1);
// After:  let char_before = query[..cursor_pos].chars().last();
```

**Validation gate:**
- Existing `should_replace_trailing_separator` tests pass
- Add unit tests: char-before lookup when the preceding char is 2/3/4-byte

**Tests — new file `src/autocomplete/insertion_tests/utf8_tests.rs`**, registered in `insertion_tests.rs` via:

```rust
#[path = "insertion_tests/utf8_tests.rs"]
mod utf8_tests;
```

Test modules (following `variable_insertion_tests.rs` nested-module pattern):

```rust
mod two_byte_chars {
    // Tests with é, ñ, ü, ö, ß
    fn inserts_suggestion_after_accented_field() { ... }
    fn inserts_suggestion_mid_query_with_accented_chars() { ... }
    fn inserts_before_accented_text_preserves_suffix() { ... }
    fn replaces_accented_partial_token() { ... }
}

mod three_byte_chars {
    // Tests with 中, 文, 日, 本, 語
    fn inserts_suggestion_after_cjk_field() { ... }
    fn inserts_suggestion_mid_query_with_cjk() { ... }
    fn cursor_at_end_of_cjk_query() { ... }
    fn replaces_cjk_partial_token() { ... }
}

mod four_byte_chars {
    // Tests with 😀, 👋, 🎉, 🚀
    fn inserts_suggestion_after_emoji_field() { ... }
    fn inserts_suggestion_mid_query_with_emoji() { ... }
    fn cursor_between_two_emojis() { ... }
    fn replaces_emoji_partial_token() { ... }
}

mod mixed_multibyte {
    fn query_with_ascii_cjk_and_emoji() { ... }
    fn inserts_suggestion_at_various_positions() { ... }
}

mod boundary_conditions {
    fn cursor_at_start_of_multibyte_query() { ... }
    fn cursor_at_end_of_multibyte_query() { ... }
    fn cursor_immediately_before_multibyte_char() { ... }
    fn cursor_immediately_after_multibyte_char() { ... }
    fn empty_query_with_multibyte_suggestion() { ... }
}
```

Property-based tests (extending existing `property_tests.rs` pattern):

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_insertion_never_panics_with_arbitrary_unicode(
        query in "[a-z\\u00E9\\u4E2D\\u{1F600}]{0,30}",
        cursor_offset_chars in 0usize..30,
        suggestion in "[a-z]{1,10}",
    ) {
        // Query contains ASCII, 2-byte, 3-byte, and 4-byte chars mixed
        // Cursor set at arbitrary char position
        // Insertion must not panic
    }

    #[test]
    fn prop_insertion_preserves_unicode_integrity(...) {
        // After insertion, resulting string is valid UTF-8
        // Character count = original + suggestion - replaced
    }
}
```

### Phase 3d — Comprehensive UTF-8 tests for `context.rs` (test-only)

No production changes — all fixes already landed in 2a/2b. This phase adds UTF-8 coverage to lock the fixes in.

**Tests — new file `src/autocomplete/context_tests/utf8_tests.rs`**, registered in `context_tests.rs`:

```rust
mod get_suggestions_utf8 {
    fn suggestions_after_cjk_field() { ... }
    fn suggestions_after_emoji_in_query() { ... }
    fn suggestions_mid_query_with_multibyte() { ... }
    fn suggestions_with_cursor_at_end_of_emoji() { ... }
}

mod detect_entry_context_utf8 {
    fn entry_context_after_to_entries_with_cjk() { ... }
    fn entry_context_with_emoji_in_key_path() { ... }
    fn with_entries_paren_with_multibyte_preceding() { ... }
}

mod is_cursor_at_logical_end_utf8 {
    fn cursor_at_end_with_cjk() { ... }
    fn cursor_followed_by_multibyte_whitespace() { ... }
    fn cursor_before_trailing_emoji() { ... }
}

mod needs_leading_dot_utf8 {
    fn dot_detection_after_cjk_identifier() { ... }
    fn dot_detection_with_emoji_prefix() { ... }
}

mod analyze_context_utf8 {
    fn field_context_with_multibyte_field_name() { ... }
    fn object_key_context_with_cjk_keys() { ... }
    fn variable_context_with_emoji_preceding() { ... }
}
```

Property-based tests:

```rust
proptest! {
    #[test]
    fn prop_get_suggestions_never_panics_with_unicode(
        query in "[\\u0020-\\u007E\\u00A0-\\u00FF\\u4E00-\\u9FFF\\u{1F600}-\\u{1F64F}]{0,40}",
        cursor_char_pos in 0usize..40,
    ) {
        // Any query with mixed Unicode + any cursor position must not panic
    }

    #[test]
    fn prop_detect_entry_context_never_panics_with_unicode(...) { ... }

    #[test]
    fn prop_analyze_context_never_panics_with_unicode(...) { ... }
}
```

### Phase 4 — Fix `autocomplete_render.rs`

**Production changes:**
1. Line 93-97: replace byte truncation with `display_text.chars().take(n).collect::<String>()`
2. Lines 63-64: change `.len()` to `.chars().count()` for char-count consistency

**Note:** CJK characters render 2 terminal columns wide. `.chars().count()` fixes the crash but may visually overflow for CJK-heavy suggestions. Tracked as follow-up (would require `unicode_width` crate); out of scope for crash fix.

**Tests — extend `src/autocomplete/autocomplete_render_tests.rs`** with a new submodule `utf8_tests`:

```rust
mod utf8_truncation {
    #[test]
    fn truncates_cjk_suggestion_at_char_boundary() { ... }
    #[test]
    fn truncates_emoji_suggestion_safely() { ... }
    #[test]
    fn truncates_mixed_multibyte_suggestion() { ... }
    #[test]
    fn does_not_truncate_short_unicode_suggestion() { ... }
}

mod utf8_width_calculation {
    #[test]
    fn popup_width_accounts_for_char_count_not_bytes() { ... }
    #[test]
    fn cjk_suggestion_width_matches_char_count() { ... }
}
```

Snapshot tests (following existing `assert_snapshot!` pattern):

```rust
#[test]
fn snapshot_autocomplete_with_cjk_suggestions() {
    let suggestions = vec![
        Suggestion::new(".名前", SuggestionType::Field),
        Suggestion::new(".年齢", SuggestionType::Field),
    ];
    let output = render_autocomplete_with_suggestions(suggestions);
    assert_snapshot!(output);
}

#[test]
fn snapshot_autocomplete_with_emoji_suggestions() { ... }

#[test]
fn snapshot_autocomplete_with_truncated_long_cjk() { ... }
```

### Phase 5 — BraceTracker Position Consistency

**No production changes needed.** After Phase 3's ingress normalization, `cursor_pos` arrives at BraceTracker as bytes, matching `BraceInfo.pos` (also bytes from `char_indices()`). The logic bug is fixed automatically. This phase becomes test-only: add UTF-8 test coverage to confirm correctness.

**Tests — extend `src/autocomplete/brace_tracker_tests.rs`** with a new submodule:

```rust
mod utf8_position_handling {
    #[test]
    fn context_at_correct_with_multibyte_before_brace() {
        // Query: ".名前(" — paren is at byte 7, char 4
        // Ensure context_at finds Paren when called with appropriate position
    }

    #[test]
    fn is_in_element_context_correct_with_cjk() { ... }

    #[test]
    fn is_in_object_correct_with_emoji_in_keys() { ... }

    #[test]
    fn is_in_non_executing_context_with_mixed_multibyte() { ... }

    #[test]
    fn detect_function_context_with_multibyte_before_function_name() {
        // Query: ".中文 | select(" — verify select is detected correctly
    }
}
```

Property-based tests (extending existing `prop_rebuild_never_panics`, `prop_context_at_never_panics`):

```rust
proptest! {
    #[test]
    fn prop_rebuild_never_panics_utf8(query in "[\\u{0020}-\\u{FFFF}]{0,50}") {
        // Broader Unicode range, not just ASCII
    }

    #[test]
    fn prop_context_queries_never_panic_utf8(
        query in "[\\u{0020}-\\u{FFFF}]{0,50}",
        pos in 0usize..200
    ) {
        // All BraceTracker query methods must be panic-free on arbitrary Unicode
    }

    #[test]
    fn prop_context_consistent_across_ascii_equivalent_queries(...) {
        // A query with CJK padding at known positions should give the same
        // structural context as the equivalent ASCII-spacing query
    }
}
```

### Phase 6 — End-to-End Integration Tests

**New file `src/autocomplete/insertion_tests/utf8_e2e_tests.rs`** — full App-level tests exercising the entire flow (following the `test_app` / `execute_query_and_wait` pattern):

```rust
mod end_to_end_crash_reproduction {
    #[test]
    fn reproduces_issue_153_cjk_query_does_not_crash() {
        // The original failing case: just typing CJK into query
        let mut app = test_app(r#"{"名前": "Alice"}"#);
        app.input.textarea.insert_str(".名前");
        // Triggers update_suggestions_from_app — must not panic
        crate::autocomplete::autocomplete_state::update_suggestions_from_app(&mut app);
    }

    #[test]
    fn reproduces_issue_153_emoji_query_does_not_crash() { ... }

    #[test]
    fn reproduces_issue_153_accented_query_does_not_crash() { ... }

    #[test]
    fn full_autocomplete_cycle_with_cjk() {
        // Type query with CJK → get suggestions → accept one → verify result
    }

    #[test]
    fn full_autocomplete_cycle_with_emoji_json_keys() { ... }

    #[test]
    fn mid_query_cursor_movement_and_autocomplete_with_multibyte() { ... }
}
```

**Shared fixtures** — add UTF-8 test JSON constants to `src/test_utils.rs`:

```rust
pub const TEST_JSON_CJK: &str = r#"{"名前": "Alice", "年齢": 30, "住所": {...}}"#;
pub const TEST_JSON_EMOJI: &str = r#"{"👋": "hello", "data": [{"🎉": "party"}]}"#;
pub const TEST_JSON_MIXED: &str = r#"{"café": ..., "中文": ..., "emoji😀": ...}"#;
```

### Phase 7 — Regression Protection

**Add to `src/autocomplete/insertion_tests/regression_tests.rs`** (new file) — tests that lock in the exact crash scenarios from issue #153:

```rust
// Each test is a specific, minimal reproducer of the crash
#[test]
fn regression_issue_153_cjk_after_dot() { ... }
#[test]
fn regression_issue_153_emoji_in_object_value() { ... }
#[test]
fn regression_issue_153_accented_latin_in_field_name() { ... }
#[test]
fn regression_issue_153_mixed_scripts_query() { ... }
```

These stay even after refactoring — they protect against reintroducing the bug.

---

## Testing Summary

| Phase | Unit Tests | Property Tests | Snapshot Tests | E2E Tests |
|-------|-----------|----------------|----------------|-----------|
| 1 — str_utils | ~15 | 3 | — | — |
| 2 — insertion | ~25 (5 modules) | 2 | — | — |
| 3 — context | ~18 (5 modules) | 3 | — | — |
| 4 — render | ~6 | — | 3 | — |
| 5 — brace_tracker | ~6 | 3 | — | — |
| 6 — E2E | — | — | — | ~8 |
| 7 — regression | ~6 | — | — | — |

**Coverage targets:** 100% line coverage for all new code in `str_utils.rs`, and every modified code path in Phases 2–5 must have at least one unit test AND be included in at least one property test's input domain. Per CLAUDE.md requirements.

**Pattern adherence:**
- Separate `_tests.rs` files, never co-located with implementation
- `#[path = "..."]` module declarations
- Sub-directory organization when test files grow large
- `proptest` for randomized input testing
- `insta::assert_snapshot!` for rendering output
- Shared helpers via parent test modules (`setup_insertion_test`, `tracker_for`, `test_app`)
- Nested `mod` blocks to group related tests within files

---

## Regression Risk Analysis

Because ingress normalization **changes the semantic meaning of `cursor_pos`** (char-index → byte-index) across the entire autocomplete module, every consumer must be audited. Here are the risks and mitigations:

### R1 — ASCII behavior silently breaks
**Risk:** The whole module currently "works" because char==byte for ASCII. Any subtle arithmetic bug introduced by normalization may only surface on non-ASCII input, letting ASCII regressions slip past manual testing.
**Mitigation:** All existing ASCII tests (entire `insertion_tests/`, `context_tests/`, `brace_tracker_tests.rs`) must continue to pass unchanged. No test modifications — they become the ASCII regression baseline. Plus: add a "parity test" module that runs a fixed set of ASCII queries and asserts the exact same suggestions/insertions before and after the change.

### R2 — Missed char-based consumer downstream
**Risk:** Some function we haven't audited takes `cursor_pos` as char-index today. After ingress conversion, it receives bytes and silently produces wrong results (no panic because ASCII tests still pass).
**Mitigation:** Explicit audit checklist in the PR: every function accepting `cursor_pos: usize` in `autocomplete/` must be reviewed and annotated (doc comment: `// cursor_pos: byte offset`). Property tests drive ASCII + non-ASCII through every public entry point — divergence surfaces.

### R3 — `move_cursor_to_column` back-conversion boundary
**Risk:** This is the only place where byte→char is required. If we miss converting, the cursor lands at the wrong visual column for multi-byte text, or worse, panics inside tui_textarea.
**Mitigation:** Centralize: wrap `move_cursor_to_column` itself to take bytes, convert internally, and update the single call site. One function to review, one place bugs can live. Explicit unit tests for cursor landing position with 2/3/4-byte chars before target.

### R4 — BraceTracker external callers outside autocomplete
**Risk:** BraceTracker is public (`pub use brace_tracker::BraceTracker`). If callers outside autocomplete pass char positions, Phase 5's "free fix" silently breaks them.
**Mitigation:** Grep for all `BraceTracker` consumers (done: only `input_state.rs` constructs one and only calls `rebuild`, which takes `&str` not positions — safe). Document on `BraceTracker`: "positions are byte offsets". Add compile-fence comment.

### R5 — Render width miscalculation (CJK double-width)
**Risk:** Phase 4's `.chars().count()` fixes the crash but under-counts display columns for CJK → popup overflows, truncation appears at wrong place.
**Mitigation:** Explicitly scoped out with a tracking note. Snapshot tests will capture current (crash-free but imperfect) behavior so future width work has a baseline. No silent regression — just known incomplete rendering.

### R6 — `partial.len()` / `suggestion.text.len()` byte assumptions
**Risk:** Downstream arithmetic like `cursor_pos.saturating_sub(partial.len() + 1)` assumes both operands are the same unit. After ingress conversion, both are bytes — safe. But if `partial` is ever constructed from char counts, arithmetic breaks.
**Mitigation:** `partial` comes from `analyze_context` which derives it via `&before_cursor[...]` byte slicing — inherently byte-sized. Add a doc comment. Property tests on `analyze_context` with multibyte input confirm.

### R7 — Test helpers in existing test files use `&query[..cursor_pos]`
**Risk:** Files like `mid_query_insertion_tests.rs:219,235` already slice by `cursor_pos`. If these tests set cursor via `textarea.cursor()`, they will now receive bytes and continue working. If they hard-code char positions expecting char-index semantics, they break.
**Mitigation:** Audit each such call site in tests; existing ones use ASCII so char==byte — safe. New UTF-8 tests must explicitly compute byte positions (helper in `str_utils` makes this easy and DRY).

### R8 — Scroll offset and rendering use char positions
**Risk:** `input_state.rs::calculate_scroll_offset` uses `textarea.cursor().1` (char) and `query().chars().count()` (chars). This is internally consistent in char-space and does **not** touch autocomplete. Must stay char-based.
**Mitigation:** Explicit scope boundary — ingress conversion applies **only** at the autocomplete entry points. Input/render pipeline is untouched. Confirmed by existing test coverage remaining green.

---

## Comprehensive Regression Test Matrix

The tests in Phases 1–7 combine to form a regression safety net. Here's how they map to risks:

| Risk | Covered By |
|------|-----------|
| R1 ASCII baseline | All existing tests must pass unchanged + new `ascii_parity_tests.rs` module (see below) |
| R2 Missed consumer | Phase 2/3 property tests with full Unicode range `[\\u0020-\\u{FFFF}]` |
| R3 Cursor landing | New `cursor_landing_tests.rs` — asserts visual column after insertion across char widths |
| R4 BraceTracker external | Phase 5 property tests + grep audit documented in PR |
| R5 Render width | Phase 4 snapshot tests lock current behavior |
| R6 partial arithmetic | Phase 2 property tests on insertion + explicit multibyte partial tests |
| R7 Test helpers | Audit note in PR; existing ASCII tests unchanged |
| R8 Scroll/render scope | No changes to input_state — existing tests green confirms |

**New test file: `src/autocomplete/ascii_parity_tests.rs`**

Purpose: lock in exact current behavior for pure-ASCII queries so we catch silent regressions where bytes-vs-chars arithmetic subtly differs.

```rust
mod ascii_parity {
    // A representative corpus of real-world ASCII queries, each asserting:
    //   - analyze_context returns identical (context, partial)
    //   - get_suggestions returns identical Vec<Suggestion>
    //   - insert_suggestion produces identical final query + cursor column
    #[test] fn parity_simple_field_access() { ... }       // .name
    #[test] fn parity_pipe_chain() { ... }                // .a | .b
    #[test] fn parity_array_iteration() { ... }           // .items[].name
    #[test] fn parity_object_construction() { ... }       // {a: .x}
    #[test] fn parity_function_call() { ... }             // select(.x > 0)
    #[test] fn parity_nested_braces() { ... }             // map(select(.a.b))
    #[test] fn parity_variable_binding() { ... }          // .x as $v | $v
    #[test] fn parity_cursor_mid_query() { ... }          // cursor between tokens
    #[test] fn parity_cursor_at_start() { ... }
    #[test] fn parity_cursor_at_end() { ... }
    #[test] fn parity_empty_partial() { ... }
    #[test] fn parity_trailing_dot() { ... }              // .foo.
}
```

**New test file: `src/autocomplete/insertion/cursor_landing_tests.rs`**

Purpose: verify `move_cursor_to_column` lands at the right **visual** column after byte→char back-conversion.

```rust
#[test] fn cursor_lands_after_ascii_insertion() { ... }
#[test] fn cursor_lands_after_insertion_with_cjk_before_target() { ... }
#[test] fn cursor_lands_after_insertion_with_emoji_before_target() { ... }
#[test] fn cursor_lands_after_insertion_between_multibyte_chars() { ... }
#[test] fn cursor_lands_correctly_with_mixed_scripts_before_target() { ... }
```

---

## Risk Assessment

- **Phase 1:** Zero risk — additive only with dedicated tests
- **Phase 2:** Medium risk — most complex change. Mitigated by ~25 unit + 2 property tests
- **Phase 3:** Medium risk — many call sites. Mitigated by ~18 unit + 3 property tests
- **Phase 4:** Low risk — isolated rendering. Snapshot tests catch regressions
- **Phase 5:** Low risk — call-site adjustments. Property tests cover broad input space
- **Phase 6:** Zero risk — integration tests only
- **Phase 7:** Zero risk — regression lock-in only

---

## Open Questions / Future Considerations

- **Typed newtypes:** Should we introduce `CharPos(usize)` and `BytePos(usize)` wrappers to make byte/char confusion a compile-time error? Would prevent future regressions but is a larger refactor.
- **Ingress vs egress normalization:** Current plan normalizes at egress (convert before slicing). An alternative is ingress normalization — convert cursor to byte immediately upon obtaining from textarea — but this complicates some helpers that legitimately need char positions.
- **Performance:** `char_pos_to_byte_pos` is O(n). For typical query lengths (< 100 chars) this is negligible, but if profiling shows hot-path concerns, consider caching within a single autocomplete invocation.
- **Extended Unicode edge cases to verify:**
  - Combining characters (é as `e + U+0301`)
  - Zero-width joiner sequences (👨‍👩‍👧)
  - Right-to-left scripts (Arabic, Hebrew)
  - These should work if we handle code-point boundaries correctly, but explicit tests would confirm.
