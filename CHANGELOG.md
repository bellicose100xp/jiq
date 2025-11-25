# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
