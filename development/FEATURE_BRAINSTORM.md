# jiq Feature Improvements & Additions - Brainstorm

This document outlines potential improvements and feature additions for jiq to enhance user experience.

## Current State Analysis

**Strengths:**
- Real-time query execution with instant feedback
- Context-aware autocomplete for JSON fields and jq functions
- Full VIM modal editing support
- Syntax highlighting for both jq queries and JSON output
- Clean, intuitive two-pane interface
- Flexible output modes (results or query string)

**Areas for Enhancement:**
Below are categorized suggestions for improving the tool.

---

## 1. Query Management & History

### 1.1 Query History
**Priority: HIGH** | **Impact: HIGH**

- Navigate through previously executed queries using Up/Down arrows (when not in autocomplete)
- Persist query history across sessions (~/.config/jiq/history)
- Configurable history size (default: 1000 entries)
- Search through history with Ctrl+R (reverse search like bash)
- Clear history command

**Use Case:** Users often iterate on similar queries and want to recall previous work.

**Implementation Notes:**
- Store in `~/.config/jiq/history.json`
- Deduplicate consecutive identical queries
- Add timestamp metadata for each query

### 1.2 Query Bookmarks/Favorites
**Priority: MEDIUM** | **Impact: MEDIUM**

- Save frequently used queries with custom names
- Quick access menu to load bookmarked queries
- Support for query templates with placeholders
- Export/import bookmarks for sharing

**Keybindings:**
- `Ctrl+S`: Save current query as bookmark
- `Ctrl+L`: Load from bookmarks menu
- `Ctrl+D`: Delete bookmark

**Storage:** `~/.config/jiq/bookmarks.json`

---

## 2. Search & Navigation

### 2.1 Search in Results Pane
**Priority: HIGH** | **Impact: HIGH**

- Forward search: `/` (VIM-style)
- Backward search: `?`
- Next match: `n`, Previous: `N`
- Highlight all matches in results
- Case-insensitive option: `/\c`

**Use Case:** Quickly locate specific values in large JSON outputs.

### 2.2 Incremental Search
**Priority: MEDIUM** | **Impact: MEDIUM**

- Show matches as you type the search pattern
- Display match count (e.g., "3/15")
- Jump to first match automatically

---

## 3. User Interface Enhancements

### 3.1 Resizable Panes
**Priority: MEDIUM** | **Impact: MEDIUM**

- Adjust the vertical split between input and results
- Keybindings: `Ctrl+W +/-` or mouse drag
- Remember pane sizes across sessions
- Horizontal layout option for wide terminals

### 3.2 Multiple Layout Modes
**Priority: LOW** | **Impact: MEDIUM**

- Horizontal split (input on left, results on right)
- Full-screen results mode (toggle input pane)
- Compact mode (single-line input at bottom)
- Configuration option for default layout

### 3.3 Status Line Improvements
**Priority: LOW** | **Impact: LOW**

- Show current file path/source
- Display query execution time
- Show JSON input size
- Display current line/column in results
- Memory usage indicator for large files

### 3.4 Mouse Support
**Priority: LOW** | **Impact: LOW**

- Click to switch focus between panes
- Scroll wheel navigation in results
- Drag to resize panes
- Click to position cursor in input field
- Optional (disabled by default for VIM purists)

---

## 4. Configuration & Customization

### 4.1 Configuration File
**Priority: MEDIUM** | **Impact: HIGH**

Support `~/.config/jiq/config.toml` with options:

```toml
[ui]
theme = "default"  # or "monokai", "solarized", etc.
show_line_numbers = true
tab_width = 2
default_layout = "vertical"  # or "horizontal"

[editor]
enable_vim_mode = true
enable_mouse = false
undo_levels = 100

[behavior]
auto_execute = true  # Execute queries on every keystroke
debounce_ms = 100    # Delay before executing query
history_size = 1000
save_history = true

[jq]
default_args = ["--color-output", "--indent", "2"]
timeout_seconds = 30

[keybindings]
# Custom keybinding overrides
exit = "Ctrl+Q"
save_bookmark = "Ctrl+S"
```

### 4.2 Color Themes
**Priority: LOW** | **Impact: MEDIUM**

- Predefined themes: default, monokai, solarized-dark, solarized-light, gruvbox
- Custom theme support via config file
- Theme preview command
- Separate themes for input and results

---

## 5. Performance & Scalability

### 5.1 Async Query Execution
**Priority: MEDIUM** | **Impact: MEDIUM**

- Show loading indicator for slow queries
- Timeout configuration (default: 30s)
- Cancel long-running queries with Ctrl+C (without exiting)
- Stream results for large outputs

### 5.2 Large File Handling
**Priority: MEDIUM** | **Impact: HIGH**

- Streaming JSON parser for files >100MB
- Pagination for large result sets
- Virtual scrolling in results pane
- Warning when loading very large files
- Option to sample large files

### 5.3 Query Caching
**Priority: LOW** | **Impact: LOW**

- Cache query results for repeated queries
- Invalidate cache when input changes
- Configurable cache size

---

## 6. Input/Output Features

### 6.1 Multiple Input Sources
**Priority: MEDIUM** | **Impact: MEDIUM**

- Switch between multiple JSON files/sources
- Tab-based interface for multiple inputs
- Compare mode (side-by-side comparison)
- Merge multiple JSON files

**Keybindings:**
- `Ctrl+T`: New tab
- `Ctrl+W`: Close tab
- `Ctrl+Tab`: Next tab
- `Ctrl+Shift+Tab`: Previous tab

### 6.2 Clipboard Integration
**Priority: HIGH** | **Impact: HIGH**

- Copy results to clipboard: `y` (VIM yank)
- Copy query to clipboard: `Y`
- Paste from clipboard: `p`
- System clipboard integration (when available)

**Dependencies:** Add `arboard` or `cli-clipboard` crate

### 6.3 Export Options
**Priority: MEDIUM** | **Impact: MEDIUM**

- Save results to file: `:w filename.json` (VIM-style)
- Append to file: `:w >> filename.json`
- Export in different formats: JSON, YAML, CSV
- Screenshot/save current view

### 6.4 Input Validation
**Priority: LOW** | **Impact: LOW**

- Validate JSON before starting jiq
- Suggest fixes for common JSON errors
- Option to auto-fix trailing commas, etc.

---

## 7. Query Building Assistance

### 7.1 Enhanced Error Messages
**Priority: HIGH** | **Impact: HIGH**

- Parse jq error messages and highlight problematic syntax
- Suggest corrections for common mistakes
- Show caret (^) pointing to error location in query
- Context-aware help for error types

### 7.2 Built-in jq Documentation
**Priority: MEDIUM** | **Impact: HIGH**

- Help command: `:help` or `?function_name`
- Inline documentation popup for functions
- Examples for each jq function
- Quick reference card: `Ctrl+?`

**Implementation:**
- Embed jq manual excerpts
- Show function signatures in autocomplete
- Link to online docs for detailed help

### 7.3 Query Builder/Wizard
**Priority: LOW** | **Impact: MEDIUM**

- Visual query builder for common operations
- Step-by-step wizard for complex queries
- Template library for common patterns
- Export wizard-built queries as text

### 7.4 Multi-line Query Editing
**Priority: MEDIUM** | **Impact: MEDIUM**

- Expand input pane for complex queries
- Better formatting for multi-line jq scripts
- Syntax folding/collapsing
- Auto-indentation for nested expressions

---

## 8. Advanced Features

### 8.1 Diff Mode
**Priority: LOW** | **Impact: MEDIUM**

- Compare two JSON files side-by-side
- Highlight differences
- Apply query to both and compare results
- Export diff output

**Command:** `jiq --diff file1.json file2.json`

### 8.2 Watch Mode
**Priority: MEDIUM** | **Impact: MEDIUM**

- Watch file for changes and auto-reload
- Useful for monitoring log files or API responses
- Configurable refresh interval
- Highlight what changed since last refresh

**Command:** `jiq --watch data.json`

### 8.3 Macro Recording
**Priority: LOW** | **Impact: LOW**

- Record sequence of keystrokes (VIM-style)
- Replay macros: `@a` (replay macro 'a')
- Save macros to config
- Share macro recordings

### 8.4 Pipeline Visualization
**Priority: LOW** | **Impact: LOW**

- Visual representation of jq pipeline stages
- Show intermediate results for each pipe
- Step-through execution mode
- Debug mode showing data flow

---

## 9. Integration & Interoperability

### 9.1 Shell Integration
**Priority: LOW** | **Impact: LOW**

- Shell completion for fish, zsh, bash
- Function to quickly open jiq from shell
- Integration with fzf for file selection

### 9.2 Format Support
**Priority: MEDIUM** | **Impact: MEDIUM**

- Auto-detect and convert YAML to JSON
- Support for TOML input (convert to JSON)
- CSV to JSON conversion
- XML to JSON conversion (optional)

**Command:** `jiq --from yaml data.yaml`

### 9.3 Remote Data Sources
**Priority: LOW** | **Impact: MEDIUM**

- Fetch JSON from URLs
- Basic HTTP authentication support
- Follow redirects
- Cache remote responses

**Command:** `jiq --url https://api.example.com/data`

---

## 10. Developer Experience

### 10.1 Debug Mode
**Priority: LOW** | **Impact: LOW**

- Verbose logging option
- Show jq command being executed
- Performance profiling
- Memory usage tracking

**Command:** `jiq --debug data.json`

### 10.2 Testing Support
**Priority: LOW** | **Impact: LOW**

- Save query + input as test case
- Regression testing for queries
- Snapshot testing for expected outputs

### 10.3 Plugin System
**Priority: LOW** | **Impact: HIGH (long-term)**

- Extension API for custom features
- Custom autocomplete providers
- Custom output formatters
- Community plugin repository

---

## 11. Quality of Life Improvements

### 11.1 Better Startup Experience
**Priority: MEDIUM** | **Impact: MEDIUM**

- Show helpful tips on first run
- Interactive tutorial mode
- Sample JSON files for practice
- Quick-start guide

### 11.2 Smart Defaults
**Priority: MEDIUM** | **Impact: MEDIUM**

- Auto-detect terminal capabilities (color support, etc.)
- Adjust UI based on terminal size
- Sensible defaults based on input size
- Context-aware keybindings

### 11.3 Error Recovery
**Priority: MEDIUM** | **Impact: MEDIUM**

- Graceful handling of jq crashes
- Auto-save query on unexpected exit
- Recovery mode for corrupted config
- Better error messages for missing jq binary

### 11.4 Accessibility
**Priority: LOW** | **Impact: MEDIUM**

- Screen reader support
- High contrast mode
- Customizable font sizes
- Keyboard-only navigation (already good)

---

## 12. Command-line Enhancements

### 12.1 Additional CLI Options
**Priority: MEDIUM** | **Impact: MEDIUM**

```bash
jiq data.json --query '.users[]'          # Start with query
jiq data.json --raw                       # Raw output (no colors)
jiq data.json --compact                   # Compact JSON output
jiq --generate-config                     # Generate default config
jiq --list-themes                         # List available themes
jiq --validate data.json                  # Validate JSON only
jiq --benchmark query.jq data.json        # Benchmark query
```

### 12.2 Batch Mode
**Priority: LOW** | **Impact: LOW**

- Process multiple files with same query
- Output to separate files or combined
- Parallel processing option

**Command:** `jiq --batch '*.json' --query '.name'`

---

## Implementation Priority Matrix

### Phase 1 - Quick Wins (High Impact, Low Effort)
1. Query history (Up/Down navigation)
2. Search in results pane (`/`, `?`)
3. Clipboard integration (`y`, `Y`)
4. Enhanced error messages
5. CLI option: `--query` to start with query

### Phase 2 - Core Features (High Impact, Medium Effort)
1. Configuration file support
2. Built-in jq documentation
3. Format support (YAML, TOML)
4. Watch mode for file changes
5. Export to file (`:w` command)

### Phase 3 - Advanced Features (Medium Impact, High Effort)
1. Multiple input sources (tabs)
2. Resizable panes
3. Async query execution
4. Large file handling improvements
5. Query bookmarks

### Phase 4 - Nice-to-Have (Lower Priority)
1. Diff mode
2. Plugin system
3. Mouse support
4. Color themes
5. Remote data sources

---

## Community Feedback Ideas

Consider gathering user input on:
- Most-wanted features from this list
- Pain points with current implementation
- New use cases not covered here
- Integration requests with other tools

---

## Technical Considerations

### Dependencies to Consider
- `arboard` - Clipboard support
- `notify` - File watching
- `reqwest` - HTTP requests
- `serde_yaml` - YAML support
- `tokio` - Async runtime (if needed)

### Backward Compatibility
- Maintain current keybindings as defaults
- Config file should be optional
- New features should be opt-in when breaking

### Performance Budget
- Startup time: <100ms (currently ~50ms)
- Query execution overhead: <10ms
- Memory usage: <50MB for files <10MB

---

## Conclusion

This brainstorm covers a wide range of improvements from quick wins to long-term ambitious features. The key is to prioritize based on:

1. **User demand** - What users actually request
2. **Impact/effort ratio** - Quick wins first
3. **Coherence** - Features that fit the tool's philosophy
4. **Maintenance burden** - Avoid over-complication

The goal is to enhance jiq while maintaining its core strengths: simplicity, speed, and excellent VIM integration.
