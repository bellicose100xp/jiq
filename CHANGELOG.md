# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Code Organization**: Refactored large source files (>1000 lines) into smaller, focused modules
  - Split `ai_events_tests.rs` (1064 lines) into 8 test modules
  - Split `ai_state.rs` (1036 lines) into lifecycle, suggestions, and response modules
  - Split `ai_render.rs` (1265 lines) into layout, content, and text modules
  - Split `search_events.rs` (1432 lines) into navigation and scroll modules
  - Split `autocomplete/insertion.rs` (1619 lines) into query manipulation, cursor, and execution modules
  - All source files now under 1000 lines for improved maintainability
  - Maintained existing module structure pattern (no mod.rs files)
  - All tests pass, no functionality changes

## [2.21.2] - 2025-12-12

### Changed
- Removed unnecessary requirement reference comments from dependencies
- Added CI status and coverage badges to README for better visibility

## [2.21.1] - 2025-12-13

### Added
- CI workflow with automated testing, code coverage, and linting
  - GitHub Actions workflow running tests, cargo-tarpaulin coverage, and clippy/rustfmt checks
  - Codecov integration for coverage tracking and reporting
  - Coverage badge in README showing current test coverage
  - Pre-commit hook for automatic code formatting

### Changed
- Applied rustfmt formatting to entire codebase for consistency
- CI installs jq 1.8.1 for consistent snapshot test results
- Tests run serially in CI to avoid jq process race conditions

## [2.21.0] - 2025-12-12

### Added
- Object key autocomplete for jq object literals
  - Intelligent suggestions when building jq objects with `{key: value}` syntax
  - Brace tracking system to understand nested object/array contexts
  - Enhanced context detection for object construction scenarios
  - Improved insertion logic with proper cursor positioning for object keys
  - Supports complex nested structures and multi-pass context scanning

## [2.20.6] - 2025-12-11

### Fixed
- Resolved merge conflict between local and remote branches
  - Synchronized local repository with merged PR changes
  - Preserved version history and changelog integrity

## [2.20.5] - 2025-12-11

### Added
- Enter key now accepts autocomplete suggestions (same behavior as Tab)
  - Pressing Enter when autocomplete popup is visible accepts the selected suggestion
  - Falls through to existing exit behavior when autocomplete is not visible
  - Maintains existing modifier key behaviors (Shift+Enter, Alt+Enter for output modes)
  - Provides Enter/Tab equivalence for improved user experience and workflow flexibility

## [2.20.4] - 2025-12-10

### Added
- Results pane navigation while search is confirmed
  - Can now navigate through results using arrow keys/page keys while search matches remain highlighted
  - Preserves search highlighting during navigation
  - Improves workflow by allowing result exploration without canceling search

## [2.20.3] - 2025-12-01

### Internal
- Refactored state management into feature-specific modules
  - Extracted autocomplete state into `autocomplete/autocomplete_state.rs` and `autocomplete/insertion.rs`
  - Extracted stats state into `stats/stats_state.rs`
  - Extracted tooltip state into `tooltip/tooltip_state.rs`
  - Reduced `app_state.rs` from ~1300 lines to ~150 lines
- Improved code organization and maintainability with better separation of concerns

## [2.20.2] - 2025-01-30

### Added
- Operator tooltips for `//`, `|=`, `//=`, and `..` operators
  - Shows quick reference help when cursor is on these operators
  - Includes practical examples and usage tips
  - Complements existing function tooltip system

## [2.20.1] - 2025-12-01

### Internal
- Refactored render methods into component-specific modules (`*_render.rs`)
- Renamed module files to use component-prefixed naming convention
  - `state.rs` → `*_state.rs`, `events.rs` → `*_events.rs`, `content.rs` → `*_content.rs`
- Fixed flaky proptest for tooltip field access detection

## [2.20.0] - 2025-11-30

### Added
- Search-in-results functionality with real-time highlighting
  - `/` in Results pane to open search bar
  - Case-insensitive fuzzy matching across result lines
  - Live match highlighting with distinct colors (magenta for current, yellow for others)
  - Match count display in search bar (e.g., "Match 2/5")
  - `n` to jump to next match, `N` to jump to previous match
  - `Enter` to accept search and keep matches highlighted
  - `ESC` to cancel search and clear highlighting
  - Search bar auto-clears when switching focus or exiting
  - Maintains horizontal scroll position when jumping between matches
  - Comprehensive property-based and snapshot tests for search behavior

## [2.19.3] - 2025-11-30

### Added
- Configuration option to control tooltip auto-show behavior
  - New `[tooltip]` section in config with `auto_show` field (defaults to `true`)
  - Set to `false` to hide tooltip by default, requiring Ctrl+I to show manually

### Changed
- Refactored `App::new()` to accept `&Config` for better extensibility

## [2.19.2] - 2025-11-30

### Fixed
- ESC key now closes autocomplete and switches to Normal mode in a single keypress
  - Previously required two ESC presses when autocomplete was visible
  - Aligns with standard Vim behavior where ESC always returns to Normal mode

## [2.19.1] - 2025-11-30

### Added
- Bracketed paste mode for efficient paste handling
  - Terminal now enables bracketed paste mode on startup
  - Pasted content is handled as a single operation
  - Immediate query execution after paste completes
- Query debouncing for improved performance during fast typing
  - 50ms delay before query execution while typing
  - Prevents excessive jq invocations during rapid input
  - Debounce bypassed for Enter/Tab for immediate feedback

### Internal
- Added Debouncer module with configurable delay
- Property-based tests for debounce timer reset and state consistency
- Unit tests for paste event handling and debouncer functionality

## [2.18.0] - 2025-11-30

### Added
- Stats bar showing result type and count
  - Displays JSON type (Object, Array, String, Number, Boolean, Null)
  - Shows element/key count for arrays and objects
  - Appears in the results pane title for quick reference

### Internal
- Migrated from `mod.rs` to modern Rust module naming convention
  - Follows Rust 2018+ idiom of using `module.rs` instead of `module/mod.rs`
  - Improves file discoverability and IDE navigation

## [2.17.0] - 2025-11-29

### Added
- Function tooltip with quick reference help
  - Contextual help popup when cursor is on a jq function
  - Shows practical examples and usage tips
  - Supports nested function detection (shows innermost function)
  - Purple border styling to distinguish from autocomplete
  - Hint text on input border when tooltip is available but disabled

## [2.16.0] - 2025-11-28

### Added
- Smart parenthesis insertion in autocomplete
  - Functions like `map`, `select`, `has` now auto-insert `()` with cursor positioned inside
  - Non-function suggestions (fields, operators) work as before without parentheses

### Internal
- Refactored event handlers and state into dedicated feature modules
- Moved help module from `app/` to dedicated `src/help/` directory

## [2.15.0] - 2025-11-28

### Added
- Clipboard support with multiple backends
  - `Ctrl+Y` in Insert mode to copy query
  - `yy` in Normal mode to copy query
  - `yy` in Results pane to copy filtered JSON output
  - Auto-strips ANSI color codes from results before copying
  - Three clipboard backends configurable via `~/.config/jiq/config.toml`:
    - `auto` (default): tries system clipboard, falls back to OSC 52
    - `system`: OS clipboard only (via arboard)
    - `osc52`: terminal escape sequences (works over SSH/tmux)
- Notification system for transient UI messages
  - Info messages (gray, 1.5s) for confirmations like "Copied!"
  - Warning messages (yellow, 10s) for config errors
  - Error messages (red, permanent) for critical errors
  - Notifications display in top-right corner with styled borders
- Configuration system with TOML support
  - Config file at `~/.config/jiq/config.toml` (XDG paths on all platforms)
  - Graceful handling of missing or malformed config files
  - Comprehensive property-based tests for config parsing

### Documentation
- Added CONTRIBUTING.md with project standards and guidelines
- Updated README with clipboard keybindings and configuration

## [2.14.2] - 2025-11-27

### Added
- Horizontal scrolling in results pane for viewing wide JSON content
  - `h`/`l`/`←`/`→` to scroll 1 column left/right
  - `H`/`L` to scroll 10 columns left/right
  - `0`/`^` to jump to left edge
  - `$` to jump to right edge
  - `End` key to jump to bottom (alongside `G`)

### Documentation
- Updated README and help popup with horizontal scroll keybindings

## [2.14.1] - 2025-11-27

### Improved
- Context-aware help text in INPUT mode that adapts to input state
  - Empty input: Shows history navigation shortcuts (Ctrl+P/N to cycle, ↑/Ctrl+R to open popup)
  - Has content: Shows exit shortcuts and history popup access
  - Always visible: Shift+Tab for pane switching, ↑/Ctrl+R for history
- Up arrow (↑) now always opens history popup, not just when input is empty
  - More consistent with Ctrl+R behavior
  - Better discoverability of history features

### Changed
- Improved terminology in help text for clarity
  - "Exit with Results" → "Output Result"
  - "Exit with Query" → "Output Query"
  - "Switch Focus" → "Switch Pane"
- Use arrow symbol (↑) for more compact display in help text

### Documentation
- Updated README to reflect new Up arrow behavior

## [2.14.0] - 2025-11-27

### Improved
- Refactored autosuggestions to use query execution results
  - Suggestions now based on actual jq output structure instead of input JSON
  - More accurate field suggestions after transformations (map, select, etc.)
  - Better handling of complex query chains and filters

### Documentation
- Updated development documentation to reflect current codebase structure
- Improved README with clearer examples and usage instructions

## [2.13.0] - 2025-11-25

### Added
- Ctrl+Q keybind to exit with query output (primary method, works in all terminals)
  - Alternative to Shift+Enter and Alt+Enter which may not work in all terminals
  - Saves successful queries to history before exiting
  - Failed queries still exit but are not saved to history

### Fixed
- Input text now scrolls left to fill space when deleting characters
  - Previously left empty space on the right when deleting from long queries
  - Text automatically adjusts to show as much content as possible
  - Cursor remains visible throughout deletion

### Changed
- Refactored input rendering architecture to eliminate cursor desynchronization
  - Replaced overlay rendering with direct styled Paragraph rendering
  - Use tui-textarea for editing logic only, not for rendering
  - Simplifies codebase and prevents cursor position bugs
  - Matches the proven pattern used in results pane rendering

### Internal
- Added 12 scroll offset and synchronization tests
- Added 9 unit tests for span extraction and cursor insertion
- Added 6 comprehensive tests for Ctrl+Q, Shift+Enter, Alt+Enter
- Replaced 5 redundant tests with proper unit tests
- All 374 tests passing

## [2.12.1] - 2025-11-25

### Added
- VIM keybinding `^` in Normal mode - moves cursor to start of line (same as `0`)
- Support for `^` in Operator mode - enables `d^` and `c^` operations

## [2.12.0] - 2025-11-25

### Added
- Syntax highlighting now works for queries of any length
  - Previously disabled for queries longer than terminal width
  - Highlighted text correctly follows horizontal scrolling
  - Colors remain visible even in very long queries

### Fixed
- Cursor jump bug when deleting characters from long queries
  - Cursor would jump several positions left when query crossed viewport width threshold
  - Occurred at the transition between scrolled and non-scrolled states
  - Now maintains correct cursor position at all query lengths

### Internal
- Implemented scroll-aware syntax highlighting overlay
- Added scroll offset tracking to mirror tui-textarea's internal state
- Extract visible portions of highlighted text based on scroll position
- All 352 tests passing

## [2.11.0] - 2025-11-25

### Fixed
- Autocomplete Tab now correctly appends array suggestions instead of replacing entire query
  - Example: Typing `.services` + accepting `[].name` now correctly produces `.services[].name`
  - Previously replaced entire query with just `[].name`
- History now correctly saves successful queries on Enter/Shift+Enter/Alt+Enter
  - Queries are persisted to `~/.jiq_history` for use in future sessions
  - Failed queries and empty queries are not saved

### Changed
- **Major internal refactoring for improved maintainability** (no user-facing changes)
  - Extracted event handlers into focused modules (vim, global, history, results)
  - Reduced `events.rs` from 2,469 to 148 lines (94% reduction)
  - Extracted popup rendering utilities for code reuse
  - Extracted syntax highlighting and help content into dedicated modules
  - All tests now colocated with code following Rust best practices

### Added
- 17 new tests for critical dispatch order and focus interactions
- Comprehensive regression tests for autocomplete behavior
- Tests for history persistence logic
- Total test count increased from 335 to 360 tests

### Internal
- Created 6 new well-organized modules with clear responsibilities
- Extracted reusable popup positioning utilities (`widgets/popup.rs`)
- Help content now maintained in constants file for easier updates
- Improved code organization with clear module boundaries

## [2.10.0] - 2025-11-25

### Added
- Comprehensive keyboard shortcuts help popup
  - `F1` or `?` to toggle help (works universally across terminals)
  - Full vim-style scrolling: `j/k`, `J/K`, `g/G`, `Ctrl+D/U`, arrow keys, PageUp/PageDown, Home/End
  - Organized by context: Global, Insert mode, Normal mode, Autocomplete, Results pane, History, Error overlay
  - Mode-aware bottom help line shows relevant keys for current mode

### Fixed
- Help popup now works reliably across all terminal emulators
  - Replaced `Ctrl+/` (sends ASCII 0x1F, poorly supported) with `F1` and `?`
  - `F1` works in all modes and focus contexts
  - `?` works in Normal mode and Results pane only
- Terminal resize no longer crashes when help popup is open
  - Popup dimensions now clamp to terminal size
  - Gracefully skips rendering on very small terminals (<20x10)
- Scroll position correctly resets after jumping to bottom with `G`
  - Previously required multiple key presses to scroll up after `G`
  - Now immediate response to `k` after `G`

### Improved
- Removed code duplication in help popup key handling
- Added 16 comprehensive tests for help popup functionality
- Emojis replaced with ASCII text headers for better terminal compatibility

## [2.9.1] - 2025-11-24

### Fixed
- History search box now shows a visible cursor with full editing support
  - Left/right arrow keys move cursor within search text
  - Insert and delete at any position (not just end)
- History popup no longer collapses when search has no matches
- Multi-word search now works correctly (space-separated terms are ANDed, like fzf)
  - e.g., `headquarters building` matches entries containing both terms

## [2.9.0] - 2025-11-24

### Added
- Autocomplete suggestions for root-level JSON arrays
  - Pressing `.` on `[{"id": 1}, ...]` now suggests `.[]`, `.[].id`, `.[].field`, etc.
  - Previously showed no suggestions for top-level arrays
  - Supports prefix filtering (typing `n` filters to `.[].name`, `.[].notes`, etc.)
  - Handles edge cases: empty arrays, arrays of primitives, nested arrays

## [2.8.0] - 2025-11-24

### Added
- Smart array field autocomplete with proper jq syntax
  - Typing `.array.` now suggests `[]`, `[].field1`, `[].field2` (guides user to use iterator)
  - Typing `.array | .` now suggests `.[]`, `.[].field1`, `.[].field2` (preserves dot after pipe)
  - Typing `.array[].` shows normal `.field` suggestions (iterator already present)
  - Works with all array access patterns: `[]`, `[0]`, `[0:5]`
- Standalone `[]` iterator suggestion when at an array (with description "iterate all elements")

### Improved
- Suggestion type labels are now more descriptive:
  - `fn` → `function`
  - `op` → `operator`
  - `pat` → `iterator`
- Added 12 new tests for comprehensive array autocomplete coverage

## [2.7.2] - 2025-11-24

### Fixed
- Scroll position now properly clamped to content bounds - prevents scrolling past results into empty space
- 'q' key now works correctly in Results pane regardless of editor mode (fixes confusing behavior after focus switch)
- Line count calculation now matches displayed content when showing cached results after query errors
- Large files with >65,535 lines now handled correctly without overflow

### Improved
- Added comprehensive test coverage for scroll behavior, quit key interactions, and large file handling

## [2.7.1] - 2025-11-24

### Fixed
- Pressing 'q' in Insert mode no longer quits the application - allows typing queries like `unique`, `eq`, etc. (thanks @dithmer!)

## [2.7.0] - 2025-11-24

### Added
- Persistent query history with fuzzy search
- `Ctrl+P` / `Ctrl+N` - Quick cycle through previous/next queries
- `Ctrl+R` - Open full-width fuzzy search popup for history
- Queries persist across sessions in platform-specific data directories
- Automatic deduplication of duplicate queries
- History limited to last 1000 entries

## [2.6.2] - 2025-11-24

### Added
- `Ctrl+Q` keybinding to exit and output query string only (universal fallback for terminals that don't support `Shift+Enter`)

### Fixed
- `Shift+Enter` and `Alt+Enter` not working in some terminal emulators (e.g., macOS Terminal.app)

## [2.6.1] - 2025-11-23

### Added
- JSON type information in autocomplete suggestions (String, Number, Boolean, Array, Object, Null)
- Array element type detection - shows specific types like Array[String], Array[Object], Array[Number]
- Nested array type support - displays Array[Array[Number]] for multi-dimensional arrays
- Floating error overlay with Ctrl+E toggle - errors no longer disrupt results pane layout
- Error indicator (⚠) in input field title when syntax error exists

### Improved
- Autocomplete popup now has solid background with better contrast
- Selected suggestion uses cyan highlight with black text for improved visibility
- Type labels right-aligned in autocomplete for cleaner appearance
- Results pane maintains constant height - no more jarring movement when errors occur
- Error overlay auto-hides when query is modified

### Fixed
- Fixed popup width calculation to accommodate longer type labels
- Fixed transparency issue causing JSON text to show through suggestion popup
- Fixed error overlay not hiding correctly in Insert mode when query changes

## [2.5.0] - 2025-11-22

### Added
- Syntax highlighting for jq query input (experimental)
- Color-coded jq keywords, operators, and functions in the query field

### Fixed
- Fixed autosuggestion issues in jq query field
- Fixed all clippy warnings for improved code quality

## [2.4.0] - 2025-11-22

### Added
- Autocomplete support for array and object outputs
- Context-aware suggestions for bracket notation (.[n], .["key"])

### Improved
- Enhanced documentation with additional examples

## [2.3.1] - 2025-11-22

### Fixed
- Fixed autosuggestion not appearing after pipe character
- Fixed unused import warning for `Suggestion` type in non-test builds

### Improved
- Enhanced test coverage quality and assertions

## [2.3.0] - 2025-11-21

### Added
- VIM keybinding `C` - Change to end of line (same as `c$`)
- VIM keybinding `D` - Delete to end of line (same as `d$`)

## [2.2.0] - 2025-11-21

### Added
- Context-aware autocomplete system for jq queries
- JSON field suggestions based on input data structure
- jq built-in function autocomplete (map, select, keys, etc.)
- Operator and pattern suggestions (|, //, .[], etc.)
- Nested field path support in autocomplete
- Tab key to accept autocomplete suggestions
- Up/Down arrow navigation through suggestions
- Color-coded suggestion types in popup

### Changed
- Tab key now accepts autocomplete suggestions in INSERT mode
- Shift+Tab switched focus between Input and Results panes
- ESC key closes autocomplete popup or switches to NORMAL mode

### Performance
- Static data initialization with LazyLock for instant responses
- Optimized suggestion filtering for large JSON files

## [2.0.0] - 2025-11-18

### Added
- Complete VIM keybinding system for input field
- Modal editing: INSERT, NORMAL, and OPERATOR modes
- VIM navigation: h/l/0/$/w/b/e
- VIM insert commands: i/a/I/A
- VIM delete operator: x/X, dw/db/de/d$/dd
- VIM change operator: cw/cb/ce/c$/cc
- Undo/redo support: u, Ctrl+r
- Mode-based visual indicators (color-coded borders and cursor)
- Edit hint in NORMAL mode title
- Comprehensive VIM documentation in README

### Changed
- Input field now supports full VIM modal editing
- Cursor color changes per mode (cyan/yellow/green)
- Border colors match mode colors

## [1.5.0] - 2025-11-18

### Added
- VIM operator system (d and c operators)
- Operator+motion combinations (dw, db, de, d$, cw, etc.)
- Special operators (dd, cc for full line)
- Simple delete operations (x, X)

## [1.4.0] - 2025-11-18

### Added
- VIM navigation commands (h/l/0/$/w/b/e) in NORMAL mode
- VIM insert mode commands (i/a/I/A)
- Mode-based styling with color indicators
- Cursor color changes per mode

## [1.3.0] - 2025-11-18

### Added
- VIM modal editing foundation (INSERT/NORMAL modes)
- ESC to switch to NORMAL mode
- Mode indicator in input field title
- Separate handling for INSERT and NORMAL mode keys

## [1.2.0] - 2025-11-18

### Added
- Enhanced VIM navigation for results pane
- g / Home - Jump to top
- G - Jump to bottom
- Ctrl+d / PageDown - Scroll half page down
- Ctrl+u / PageUp - Scroll half page up

### Changed
- Simplified help text at bottom
- Updated README with results navigation keys

## [1.1.0] - 2025-11-17

### Added
- Comprehensive unit test suite (25 tests)
- Integration test suite (8 tests)
- Test fixtures in tests/fixtures/
- Modern Rust 2024 testing patterns with cargo_bin_cmd!()

### Fixed
- Removed terminal-corrupting interactive tests
- Fixed clippy warnings (bool_assert_comparison)

## [1.0.0] - 2025-11-16

### Added
- Initial stable release
- Interactive TUI with two-pane layout
- Real-time jq query execution
- JSON input from file or stdin
- Tab focus switching between panes
- Results scrolling (j/k, arrows, Page Up/Down)
- JSON syntax highlighting with ANSI colors
- Enter key outputs filtered JSON
- Shift+Enter outputs query string only
- Built with Ratatui 0.29 and Crossterm 0.28.1

## Version Numbering

- **Major version (2.x.x)** - Complete VIM implementation, breaking UX changes
- **Minor version (x.Y.x)** - New features (VIM modes, navigation, operators)
- **Patch version (x.x.Z)** - Bug fixes, documentation updates
