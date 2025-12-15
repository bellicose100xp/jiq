# Phase 3 Issues and Fixes

## Issues Reported

### 1. Ctrl+1-5 Not Working
**Status**: Needs user testing with debug build

**Problem**: Direct selection keybindings (Ctrl+1-5) are not responding when pressed.

**Investigation**:
- All unit tests pass, including tests that specifically verify Ctrl+1-5 functionality
- Integration tests confirm the keybinding handlers work correctly
- The event flow is correct: global handler → AI handler → keybinding handler

**Likely Causes**:
1. **Terminal Emulator Issue**: Some terminals don't send Ctrl+digit combinations correctly
   - iTerm2, Terminal.app, and some other terminals may intercept these keys
   - The terminal might be sending different key codes than expected

2. **Key Conflict**: Another application or system shortcut might be intercepting the keys

**Fix Applied**:
- Added debug logging to help diagnose the issue
- Debug output will show:
  - When `handle_suggestion_selection` is called
  - What key code and modifiers are received
  - Whether the selection is valid
  - Whether the suggestion is applied

**Testing Instructions**:
1. Build with debug assertions: `cargo build`
2. Run jiq: `./target/debug/jiq < some.json`
3. Trigger AI suggestions (make an error or successful query)
4. Press Ctrl+1 (or other Ctrl+digit)
5. Check the debug log file: `tail -f /tmp/jiq-debug.log`
6. Look for debug messages like:
   ```
   handle_suggestion_selection: visible=true, suggestions=3, key=Char('1')
   handle_direct_selection: key=Char('1'), modifiers=CONTROL, suggestion_count=3
   Parsed digit: 1, index: 0
   Valid selection: index=0
   Direct selection matched: index=0
   Applying suggestion: query=.some_query
   ```

**Alternative Workaround**:
- Use Alt+Up/Down to navigate suggestions, then press Enter to apply
- This navigation method works reliably across all terminals

---

### 2. Backticks Around Applied Queries
**Status**: ✅ FIXED

**Problem**: When applying a suggestion via Alt+Up/Down → Enter, the query input shows backticks around the query: `` `<query>` ``

**Root Cause**: 
The AI response sometimes wraps queries in backticks (markdown code formatting), and the parser was not stripping them.

**Fix Applied**:
Modified `src/ai/suggestion/parser.rs` to strip backticks from parsed queries:

```rust
// Strip backticks if present (AI sometimes wraps queries in backticks)
if query.starts_with('`') && query.ends_with('`') && query.len() > 2 {
    query = &query[1..query.len() - 1];
}
```

**Tests Added**:
- `test_parse_suggestions_with_backticks`: Single suggestion with backticks
- `test_parse_suggestions_with_backticks_multiple`: Multiple suggestions with backticks
- `test_parse_suggestions_without_backticks_unchanged`: Ensures non-backtick queries still work
- `test_parse_suggestions_single_backtick_not_stripped`: Edge case for unpaired backticks

**Verification**:
```bash
cargo test test_parse_suggestions_with_backticks
```

All tests pass ✅

---

## Summary

| Issue | Status | Action Required |
|-------|--------|-----------------|
| Backticks in queries | ✅ Fixed | None - already resolved |
| Ctrl+1-5 not working | 🔍 Investigating | User testing with debug build |

## Next Steps

1. **For Backtick Issue**: 
   - ✅ Fixed and tested
   - Will work in next build

2. **For Ctrl+1-5 Issue**:
   - Build with debug: `cargo build`
   - Test and share debug output
   - Based on output, we can determine if it's:
     - Terminal emulator issue (need to document workaround)
     - Key code mismatch (need to adjust detection)
     - System shortcut conflict (need to suggest alternative keys)

## Workarounds Available Now

Even if Ctrl+1-5 doesn't work in your terminal:
- ✅ Alt+Up/Down navigation works reliably
- ✅ Enter applies the navigated selection
- ✅ All functionality is accessible via navigation method
