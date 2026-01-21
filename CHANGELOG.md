# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.13.7] - 2026-01-20

### Changed
- **Full-screen search and snippets overlays** - Query input is now hidden when search or snippets manager is open, giving overlays more screen space
- **Distinct overlay border colors** - Search uses light purple (LightMagenta), snippets manager uses light green (LightGreen)
- **Cohesive search mode styling** - Results pane border and title text match light purple when search is active

## [3.13.6] - 2026-01-20

### Added
- **Position indicator in results title bar** - Shows current line range and scroll percentage
  - Format: `L{start}-{end}/{total} ({percent}%)`
  - Displays visible line range (1-indexed), total line count, and scroll position
  - Appears in top-right corner of results pane

- **Quick snippet query replacement** - Replace snippet query with current input using `Ctrl+R`
  - Shows confirmation dialog with old vs new query comparison
  - Press `Enter` to confirm, `Esc` to cancel
  - Warns if queries are identical (no changes needed)

## [3.13.5] - 2026-01-19

### Fixed
- **Search popup focus** - AI assistant and tooltip popups now auto-hide when search is opened
  - Prevents visual clutter and overlap when entering search mode
  - Popups restore automatically when search is closed

## [3.13.4] - 2026-01-19

### Fixed
- **History navigation boundaries** - Arrow keys now stop at first/last entry instead of wrapping around
  - Up arrow stops at oldest history entry
  - Down arrow stops at newest history entry
  - Consistent with AI popup navigation behavior (v3.12.2)

## [3.13.3] - 2026-01-19

### Fixed
- **Snippet manager search interference** - Changed keybindings to use Ctrl modifiers to prevent interference with search query typing
  - Previously, keybindings without modifiers would trigger actions while typing in the search box
  - Search input now works correctly without unintended keybinding activation

## [3.13.2] - 2026-01-19

### Added
- **Mouse wheel scrolling** for results pane
  - Scroll up/down using mouse wheel when results pane is focused
  - Provides intuitive navigation for users who prefer mouse input

## [3.13.1] - 2026-01-19

### Fixed
- Added missing Ctrl+S keybinding documentation to help popup and global keybinds section

## [3.13.0] - 2026-01-19

### Added
- **Snippet library** - Save and reuse frequently used jq queries
  - `Ctrl+S` to open snippet library popup
  - Create new snippets from current query with name and optional description
  - Edit existing snippets (name, query, description)
  - Delete snippets with confirmation dialog
  - Fuzzy search to quickly find saved snippets
  - Apply snippet with `Enter` to replace current query
  - Snippets persist to `~/.config/jiq/snippets.toml`
  - Tab/Shift+Tab to cycle between fields in create/edit mode

## [3.12.2] - 2026-01-17

### Fixed
- **AI popup navigation boundaries** - Disable wrap-around when navigating suggestions
  - Arrow keys now stop at first/last suggestion instead of wrapping
  - Prevents accidental selection jumps when navigating through suggestions

## [3.12.1] - 2026-01-17

### Added
- **Scrollable autocomplete popup** - Autocomplete suggestions now scroll when there are many matches
  - Fixed-width type labels for consistent alignment
  - Smooth scrolling through long suggestion lists
  - Improved readability for large JSON structures with many fields

## [3.12.0] - 2026-01-17

### Added
- **Nested path navigation for autocomplete** - Multi-level field suggestions in non-executing contexts
  - Suggestions now work inside `map()`, `select()`, array builders `[.a, .b]`, and object builders `{key: .val}`
  - Path parser extracts expressions like `.user.profile.` for JSON tree navigation
  - Zero-copy navigation using borrowed references for performance
  - Graceful fallback to root-level suggestions when navigation fails

## [3.11.3] - 2026-01-15

### Added
- **Visual bracket pair highlighting** - Shows matching brackets when cursor is positioned on them
  - Highlights matching pairs of (), [], {} when cursor is on opening or closing bracket
  - Helps identify bracket scope and nesting in complex queries
  - Improves code navigation and readability

### Fixed
- Missing import in bracket_matcher module documentation test

### Tests
- Updated help popup snapshot for $ keybinding

## [3.11.2] - 2026-01-14

### Added
- **Vim operator character search motions** - Delete and change operators now work with character search
  - `df{char}` / `dF{char}` - delete forward/backward to character
  - `dt{char}` / `dT{char}` - delete till forward/backward (stop before character)
  - `cf{char}` / `cF{char}` - change forward/backward to character
  - `ct{char}` / `cT{char}` - change till forward/backward (stop before character)
  - Combines existing character search motions (f/F/t/T) with delete/change operators
  - Example usage:
    - `df"` - delete from cursor to next quote
    - `ct|` - change from cursor till next pipe
    - `dT(` - delete backward till opening parenthesis
  - Enables precise text manipulation using character targets

## [3.11.1] - 2026-01-14

### Added
- **Visual dimming for stale results** - Last non-empty/last successful results are dimmed to make stale output more obvious

### Tests
- Updated help popup snapshots for v3.11.0 features

## [3.11.0] - 2026-01-14

### Added
- **Vim-style text object operators** - Advanced vim text manipulation with ci/di/ca/da operators
  - `ci`/`di` for change/delete inner (content only)
  - `ca`/`da` for change/delete around (including delimiters)
  - Supported targets:
    - `w` - word (alphanumeric + underscore)
    - `"`, `'`, `` ` `` - quote pairs
    - `(`, `)`, `[`, `]`, `{`, `}` - bracket pairs with nesting support
    - `|` - pipe segments in jq queries
  - Example usage:
    - `ciw` - change inner word
    - `di"` - delete inside quotes
    - `ca(` - change around parentheses (including the parens)
    - `ci|` - change inside pipe segment
  - Inner scope operates on content only, around scope includes delimiters
  - Pipe text objects handle jq query segments intelligently

- **Vim character search navigation** - Fast character-based cursor movement
  - `f{char}` - find forward to character
  - `F{char}` - find backward to character
  - `t{char}` - till forward (stop before character)
  - `T{char}` - till backward (stop after character)
  - `;` - repeat last search in same direction
  - `,` - repeat last search in opposite direction
  - Enables quick navigation within queries without using word motions
  - Integrates with operators (e.g., `df"` to delete from cursor to next quote)

## [3.10.7] - 2026-01-13

### Fixed
- **Terminal cleanup on error paths** - Ensures terminal is properly restored on all error conditions
  - Terminal state now correctly restored even when errors occur during initialization
  - Prevents terminal corruption when application exits via error paths
  - Improves reliability and robustness of terminal handling

## [3.10.6] - 2026-01-13

### Added
- **Variable autosuggestion for jq variables** - Autocomplete now suggests jq variables (`$var`)
  - Detects variable definitions in query (`as $var`, `| . as $var`)
  - Suggests defined variables when typing `$`
  - Handles multiple variable definitions and scoping
  - Improves workflow when working with jq variable assignments
- **Variable syntax highlighting** - Variables now highlighted in distinct color for better readability
  - All `$variable` references shown in dedicated color
  - Helps distinguish variables from other jq syntax elements

### Internal
- Comprehensive test coverage for variable autosuggestion edge cases
- Refactored helper functions in context.rs for improved maintainability
- Enhanced code documentation for autocomplete helper functions

## [3.10.5] - 2026-01-12

### Added
- **Ctrl+D/U navigation from input field** - Page scrolling now works from input field without focus switch
  - Ctrl+D scrolls results half-page down from input field
  - Ctrl+U scrolls results half-page up from input field
  - Maintains focus on input field while navigating results
  - Improves workflow by eliminating need to switch focus for quick result browsing

### Internal
- Enhanced test coverage for autocomplete edge cases
- Refactored insertion logic with focused helper functions for improved maintainability

## [3.10.4] - 2026-01-09

### Fixed
- **Autocomplete suggestions after question mark operator** - Suggestions no longer appear after bare `?` operator
  - Prevents incorrect suggestions after try-catch operators
  - Autocomplete properly detects when `?` is used as error suppression syntax
- **Text duplication in mid-query suggestion insertion** - Fixed text duplication when accepting suggestions in the middle of a query
  - Autocomplete now correctly replaces text at cursor position
  - Prevents duplicate text from appearing when inserting suggestions

## [3.10.3] - 2026-01-07

### Added
- **Autocomplete suggestions for `fromjson` and `tojson` functions** - JSON string conversion functions now appear in autocomplete
  - `fromjson` suggestion for parsing JSON strings into objects
  - `tojson` suggestion for serializing objects into JSON strings
  - Improves discoverability of string/JSON conversion operations

## [3.10.2] - 2026-01-07

### Fixed
- **Stdin terminal detection** - Immediately errors when stdin is not a terminal instead of blocking
  - Prevents application from hanging when stdin is a pipe but not providing data
  - Provides clear error message directing users to use file input or pipe JSON data
  - Improved CI stability with more reliable subprocess tests

## [3.10.1] - 2026-01-05

### Fixed
- **History popup scroll support** - History popup now supports mouse wheel and keyboard scrolling
  - Added scroll support to navigate through long history lists
  - Improves usability when viewing many saved queries

## [3.10.0] - 2026-01-03

### Added
- **OpenAI-compatible API endpoint support** - AI provider can now connect to any OpenAI-compatible API
  - Custom base URL configuration via `base_url` field in provider config
  - Enables use of local LLM servers (Ollama, LM Studio, etc.)
  - Supports alternative API providers with OpenAI-compatible interfaces
  - Configuration example:
    ```toml
    [ai]
    enabled = true
    provider = "openai"

    [ai.openai]
    api_key = "not-needed-for-local"
    model = "llama3.2"
    base_url = "http://localhost:11434/v1"  # Ollama example
    ```
  - Works with Anthropic, OpenAI, Gemini, and Bedrock providers
  - Allows mixing cloud and local AI models in same configuration

## [3.9.2] - 2026-01-02

### Changed
- Code cleanup and quality improvements
  - Removed duplicate execute() method in query module
  - Fixed clippy linting warnings for improved code quality

## [3.9.1] - 2026-01-02

### Performance
- **SIMD-accelerated ANSI stripping** - Uses memchr for faster ANSI color code removal
  - Leverages SIMD instructions for improved performance on large result sets
  - Optimizes query result preprocessing and rendering

### Changed
- Removed temporary debug statements for cleaner codebase
- Updated README documentation

## [3.9.0] - 2025-12-31

### Added
- **JSON response format for AI suggestions** - Structured JSON output from AI providers
  - Enables better parsing and validation of AI responses
  - Improves reliability of AI-generated query suggestions
  - Graceful fallback to text parsing when JSON format fails

### Fixed
- **AI empty/null output handling** - Enhanced context for empty or null query results
  - AI now receives additional context when queries return empty/null values
  - Improved suggestion quality for edge cases

### Changed
- **Increased AI context limits** - Enhanced context capacity for better suggestions
  - Raised default context limit from 50KB to 100KB
  - Raised minification threshold from 250KB to 5MB
  - Provides AI with more comprehensive JSON structure and content
  - Updated default configuration documentation to reflect new 100KB limit
- **Configurable context truncation** - Added max_context_length config option
  - Users can now customize how much context is sent to AI
  - Balances suggestion quality vs token usage/costs
  - Default remains at 100KB for optimal results

### Internal
- Marked test helper functions as cfg(test) for cleaner production builds
- Improved metadata field documentation
- Enhanced code clarity with better inline documentation

## [3.8.3] - 2025-12-30

### Fixed
- **AI token overflow** - Fixed token overflow issues in AI prompts
  - Removed redundant input sample from error prompts to reduce token usage
  - Input schema now properly truncated to 25KB limit before being sent to AI
  - Better token management prevents exceeding model context limits
- **AI empty result context** - Enhanced AI context for empty/null query results
  - AI now receives last non-empty query and its output when current query returns empty/null
  - Success prompts include cursor position for better suggestion accuracy
  - Improved suggestions for queries that filter down to empty results

## [3.8.2] - 2025-12-29

### Fixed
- **AI suggestion spacing** - Fixed spacing disappearing when selecting last AI suggestion
  - Cursor now remains in the correct position after accepting the final suggestion
  - Prevents text collapsing when selecting the last item in the suggestion list

### Changed
- **AI popup border color** - Updated AI assistant popup border color for better visual distinction
  - Improved visual clarity and consistency with other UI elements

## [3.8.1] - 2025-12-29

### Added
- **Visual state indicators for result states** - Color-coded borders and text to show result status
  - SUCCESS: Green text and border for current successful query results
  - EMPTY: Gray text with "∅ No Results | Type | Showing last non-empty result" message
  - ERROR: Yellow text with "⚠ Syntax Error | Type | Showing last successful result" message
  - All states use cyan border when results pane is focused, state-specific colors when unfocused
  - Improves visual feedback about query execution status at a glance

### Changed
- **AI assistant border color** - Changed from green to magenta to avoid conflict with success state indicator
  - Maintains clear visual distinction between different UI elements
  - Prevents confusion between AI suggestions and successful query results

### Fixed
- **File loading error display** - Prevents nested overlapping error boxes during startup
  - AI popup no longer renders when query is None during file loading errors
  - Shows brief notification instead of full error overlay for cleaner error handling
  - Eliminates visual glitches from multiple error indicators competing for screen space

## [3.8.0] - 2025-12-29

### Added
- **Suggestion counter in AI popup** - Shows current selection position (e.g., "Suggestion 2 of 5")
  - Helps users track their position when navigating through multiple AI suggestions
  - Displays in the popup header for easy reference

### Improved
- **Scrolling support in AI suggestion box** - Long suggestions now scrollable for better readability
  - Automatically scrolls when suggestions exceed popup height
  - Prevents content truncation for lengthy AI responses
  - Maintains consistent popup sizing
- **Natural language query handling** - Enhanced AI prompts for better query interpretation
  - Improved context and instructions for processing user's natural language intent
  - Better suggestions when users describe what they want in plain English
  - More accurate query generation from conversational input

### Changed
- **Removed word limit constraints from AI responses** - AI can now provide more detailed explanations
  - No artificial restrictions on response length
  - Better explanations and context in suggestions
  - Improved query reasoning and justification

## [3.7.3] - 2025-12-28

### Changed
- **AI context limit increased from 2,000 to 25,000 characters (12.5x)** - Provides AI with significantly more JSON structure and content for better suggestions
  - Input JSON sample, output sample, and last successful result all use enhanced limit
  - Improves suggestion quality for complex or deeply nested JSON structures

### Added
- **Smart JSON minification for 25KB-250KB context** - Automatically compresses pretty-printed JSON to preserve more semantic content
  - Minification threshold: 10x the context limit (250KB)
  - Removes whitespace and formatting while preserving all data
  - Falls back to direct truncation for invalid JSON or sizes >250KB
  - Negligible performance impact (~8-10ms, <0.5% of typical AI API latency)

### Tests
- 26 new tests for context preparation and minification logic
  - 7 unit tests for `try_minify_json()`
  - 13 unit tests for `prepare_json_for_context()`
  - 3 integration tests with QueryContext
  - 3 edge case tests (unicode, deep nesting, special values)
  - 4 property-based tests for robustness
- Total test count: 1569 → 1595 tests, all passing

## [3.7.2] - 2025-12-27

### Added
- **Autocomplete suggestions inside `with_entries()`** - Suggests `.key` and `.value` fields for object entry manipulation
  - `.key` and `.value` appear as the first two suggestions when cursor is inside `with_entries()`
  - Data-driven suggestions from the JSON structure also appear alongside
  - Context-aware: suggestions disappear after closing parenthesis
  - Works with nested functions like `with_entries(select(.key | ...))`

### Tests
- 30 new tests for `with_entries` context detection and suggestion behavior
  - 13 unit tests for `BraceTracker` WithEntries context detection
  - 17 integration tests for full suggestion flow
- Total test count: 1536 → 1566 tests, all passing

## [3.7.1] - 2025-12-26

### Fixed
- **Autocomplete suggestions inside element-context functions** - `.array | map(.<tab>)` now correctly suggests `.field` instead of `.[].field`
  - Functions like `map()`, `select()`, `sort_by()`, `group_by()`, `unique_by()`, `min_by()`, `max_by()`, `recurse()`, and `walk()` already provide element iteration
  - Autocomplete now detects this context and omits the redundant `[].` prefix
  - `.[]` suggestion also suppressed inside these functions since iteration is already provided
  - ObjectKeyContext (`{na<tab>`) correctly suppresses `.[]` suggestion

### Added
- `ELEMENT_CONTEXT_FUNCTIONS` HashSet for O(1) lookup of element-iterating functions
- `FunctionContext` enum and `BraceInfo` struct in `BraceTracker` for tracking function context
- `is_in_element_context()` method to detect cursor position inside element-iterating functions
- `suppress_array_brackets` parameter in `ResultAnalyzer` for context-aware suggestions

### Tests
- 51 new tests for element context detection and suggestion behavior
  - Unit tests for `BraceTracker` function context detection
  - Property-based tests for element context functions
  - Integration tests for full suggestion flow
  - Regression tests for existing behavior
- Total test count: 1416 → 1467 tests, all passing

## [3.7.0] - 2025-12-23

### Performance
- **Async query execution with worker thread** - Eliminates 58-second UI freeze on large files
  - Query preprocessing (ANSI parsing, JSON parsing, line metrics) moved to background thread
  - Non-blocking execution via `execute_async()` and `poll_response()` methods
  - Cancellable requests using `CancellationToken` for instant abort
  - UI remains responsive during query execution regardless of file size
  - Proper panic handling prevents worker thread crashes from corrupting TUI
- **Dirty flag rendering system** - Eliminates idle CPU usage with on-demand rendering
  - Added `needs_render` flag to track state changes
  - Main loop only renders when `should_render()` returns true
  - `needs_animation()` automatically handles spinner animations
  - Event handlers mark dirty after state modifications
  - Reduces CPU usage from continuous redraw to event-driven updates
- **Pre-rendered ANSI caching** - Eliminates per-frame color conversion overhead
  - ANSI codes parsed once and cached in `last_successful_result_rendered`
  - Viewport slicing: only visible lines cloned (50 vs 100K+ for large files)
  - O(1) instead of O(n) per-render overhead for large result sets
- **Cached line metrics** - Eliminates redundant line counting on every render
  - Line count and max width computed once during preprocessing
  - Cached values reused across frames until query changes
- **Search optimization** - O(1) match lookup during rendering
  - `matches_by_line: HashMap<u32, Vec<usize>>` for instant line-based queries
  - `matches_on_line(line: u32)` method replaces linear search
  - Significantly faster search highlighting for large result sets

### Fixed
- Rendered cache now preserved for null query results
  - Prevents cache loss when query returns null or empty values
  - Maintains UI stability and performance across query variations

### Tests
- **Comprehensive test coverage** - 27 new tests for performance features
  - Dirty flag system: 13 tests (unit + property tests)
  - Search `matches_by_line`: 11 tests (unit + property tests)
  - File loader integration: 3 tests
  - Total test count: 1389 → 1416 tests, all passing
- **Test organization improvements**
  - Split `app_render_tests.rs` into focused modules (`basic_ui_tests.rs`, `popup_tests.rs`, etc.)
  - Removed debug println statements from test code

### Technical
- New modules: `query/worker/preprocess.rs`, `query/worker/thread.rs`, `query/worker/types.rs`
- Request/response architecture with `QueryRequest` and `QueryResponse` types
- Worker thread uses `std::sync::mpsc` channels for communication
- Zero clippy warnings, zero build warnings, all formatting checks pass

## [3.6.0] - 2025-12-22

### Changed
- **Dynamic AI context depth** - Schema extraction depth now scales with file size
  - Small files (<1MB): depth 30 for comprehensive context
  - Medium files (1-10MB): depth 20 for balanced extraction
  - Large files (10-100MB): depth 10 for focused structure
  - Very large files (>100MB): depth 5 for minimal extraction
  - Provides better AI assistance for small files while maintaining performance
  - Array sampling optimization keeps extraction fast regardless of depth
  - Negligible load-time impact (<5ms for most files)

## [3.5.1] - 2025-12-22

### Added
- **Rainbow spinner animation** during query processing
  - Non-intrusive animated spinner in title bar using Braille dot characters
  - Cycles through 8 rainbow colors (coral, orange, yellow, green, blue, indigo, violet, pink)
  - Results remain visible during processing - no more screen flashing
  - Animation speed: ~133ms per frame for smooth, subtle indication

### Changed
- Removed intrusive "Processing query..." overlay that replaced results
- Query processing now indicated by small animated spinner in title bar

### Technical
- Added frame counter to App struct for animation timing
- Implemented independent character and color cycling (10 chars, 8 colors)
- Comprehensive test coverage: 9 unit tests + 5 snapshot tests

## [3.5.0] - 2025-12-20

### Performance
- **Async query execution infrastructure**
  - Query worker with background thread for non-blocking execution
  - Automatic cancellation of stale queries with request ID tracking
  - Eliminates UI freezing during query execution
  - Proper pipe handling prevents deadlocks on large outputs (>64KB)
- **Arc-based optimizations for large file handling**
  - Arc<String> for JSON input (O(1) cloning vs O(n))
  - Arc<String> for cached results (eliminates copy on every keystroke)
  - Cached parsed JSON (Arc<Value>) for autocomplete
  - Eliminates O(n) string copies and JSON parsing on every keystroke
- **Deferred/lazy loading for instant UI**
  - Background file loading with progress indication
  - Instant UI startup even with very large input files
  - Responsive interaction during file loading
- **Improved debouncing**
  - Increased debounce delay from 50ms to 150ms for better performance
  - Reduces query execution spam during fast typing

### Fixed
- **AI context matching**
  - AI suggestions now correctly match executed queries
  - Fixed race condition where AI received mismatched query/result pairs
  - poll_response() now returns the completed query for accurate context
  - Query field added to QueryResponse::Error for consistency
- **Autocomplete improvements**
  - Unified async execution eliminates sync/async race conditions
  - No more stale autocomplete results
  - Paste operation now uses async execution correctly
  - Base query context properly updated via async path

### Added
- Comprehensive test coverage with 24+ new tests
  - Worker thread tests for concurrent query handling
  - Async execution lifecycle tests
  - Parsed result caching tests
  - Request ID filtering and cancellation tests
  - Coverage for query-in-response and AI context correctness

### Changed
- Removed dead code and unused imports
- Fixed 11 clippy collapsible_if warnings
- Cleaned up implementation comments for better code clarity

## [3.4.0] - 2025-12-18

### Changed
- **BREAKING**: Removed default AI provider - users must now explicitly configure their provider
  - `AiProviderType` no longer has a default value
  - `provider` field in `AiConfig` is now `Option<AiProviderType>` (was `AiProviderType`)
  - `AiConfig::default()` now has `provider = None` instead of defaulting to Anthropic
  - Users must add `provider = "anthropic"` (or "openai", "gemini", "bedrock") to their config
  - When no provider is configured, AI popup shows setup instructions with README link

### Added
- Input border hint "Press Ctrl+A for AI Assistant" when AI popup is hidden
  - Improves discoverability of AI features
  - Only shown when AI popup is not visible

### Fixed
- `AsyncAiProvider::from_config` now returns `AiError::NotConfigured` when provider is None
  - Error message includes README URL for setup guidance
- Setup instructions in AI popup when no provider is configured
  - Clear message: "AI provider not configured"
  - Direct link to configuration documentation
  - Example configuration showing provider selection
- Input border hint "Press Ctrl+A for AI Assistant" when AI popup is hidden
  - Improves discoverability of AI features
  - Hint disappears when AI popup is visible

### Fixed
- `AsyncAiProvider::from_config` now returns proper error when provider is None
  - Returns `AiError::NotConfigured` with helpful message and README URL
  - No silent fallback to default provider

## [3.3.1] - 2025-12-18

### Added
- **Full-width selection highlighting** for AI suggestions
  - Selected suggestions now display edge-to-edge background highlighting
  - Replaced fragmented text-only highlighting with proper widget-based rendering
  - Each suggestion rendered as independent widget with full-width DarkGray background
  - Improves visual clarity and makes selection state immediately obvious
- **Height persistence during loading** to prevent popup flickering
  - Popup maintains consistent size when user types
  - Stored height reused during loading transitions
  - Eliminates jarring size changes between suggestion updates
- **Consistent spacing** between AI suggestions
  - Uniform 1-line spacing between all suggestions
  - Fixed inconsistent spacing that made suggestions appear cluttered
  - Clean visual separation using Layout spacing chunks
- **Model name display** in AI popup header
  - Shows currently configured AI model (e.g., "claude-3-5-sonnet-20241022")
  - Displayed on right side of popup title bar
  - Truncates with ellipsis if name is too long
- Comprehensive test suite for widget rendering
  - 5 height persistence tests
  - 5 spacing validation snapshot tests
  - 7 widget selection snapshot tests
  - 2 widget background unit tests

### Fixed
- AI suggestions now accessible with Ctrl+A even when `enabled = false` in config
  - Previously required `enabled = true` for keyboard shortcuts to work
  - Now correctly checks `configured` status instead of `enabled`
  - Improves UX for users who keep AI disabled by default

### Changed
- AI suggestion popup positioning uses dynamic height based on content
  - No more wasted vertical space below suggestions
  - Popup shrinks to fit actual content height
  - Maintains bottom-right position above input bar
- Keybindings moved from popup title to bottom border
  - Better visual hierarchy and cleaner title area
  - Easier to reference while interacting with suggestions

### Refactored
- Split `provider_tests.rs` into focused test modules
  - `anthropic_tests.rs`, `bedrock_tests.rs`, `openai_tests.rs`, `gemini_tests.rs`
  - `error_tests.rs` for cross-provider error handling
  - Improved maintainability and test organization

## [3.3.0] - 2025-12-17

### Added
- **Google Gemini AI Provider** - Google Gemini as an alternative AI provider for query suggestions
  - Support for Gemini models (e.g., gemini-2.0-flash-exp) with API key authentication
  - Streaming support via Server-Sent Events (SSE) for real-time token-by-token responses
  - Comprehensive error handling with detailed messages for:
    - Missing/invalid API keys
    - Network errors
    - Model access issues
    - Rate limiting
    - Configuration validation
  - Configuration in `~/.config/jiq/config.toml`:
    ```toml
    [ai]
    enabled = true
    provider = "gemini"

    [ai.gemini]
    api_key = "your-api-key-here"
    model = "gemini-2.0-flash-exp"
    ```
  - Provider information displayed in AI popup header
  - Full test coverage with property-based tests, snapshot tests, and SSE stream parsing tests
  - Reusable SSE parsing utilities for all streaming providers

### Changed
- Enhanced SSE parser to support multiple provider formats (OpenAI, Anthropic, Gemini)
- Provider system now supports three streaming AI providers with consistent interfaces

## [3.2.0] - 2025-12-17

### Added
- **OpenAI AI Provider** - OpenAI as an alternative AI provider for query suggestions
  - Support for OpenAI models (e.g., GPT-4, GPT-3.5) with API key authentication
  - Streaming support via Server-Sent Events (SSE) for real-time token-by-token responses
  - SSE parser supporting both OpenAI and Anthropic streaming formats
  - Comprehensive error handling with detailed messages for:
    - Missing/invalid API keys
    - Network errors
    - Model access issues
    - Rate limiting
    - Configuration validation
  - Configuration in `~/.config/jiq/config.toml`:
    ```toml
    [ai]
    enabled = true
    provider = "openai"

    [ai.openai]
    api_key = "sk-..."
    model = "gpt-4o"
    ```
  - Provider information displayed in AI popup header
  - Full test coverage with snapshot tests and error handling tests

### Changed
- AI provider system now supports multiple streaming formats (OpenAI and Anthropic)
- Provider-specific UI rendering shows which AI provider is in use

## [3.1.0] - 2025-12-17

### Added
- **AWS Bedrock AI Provider** - Alternative to Anthropic for AI-powered query suggestions
  - Support for AWS Bedrock models (e.g., Claude via Bedrock) with AWS credentials
  - Streaming support via AWS SDK for real-time token-by-token responses
  - Comprehensive error handling with detailed messages for:
    - Missing/invalid AWS credentials
    - Network errors
    - Model access issues
    - Configuration validation
  - AWS credential chain support (environment vars, `~/.aws/credentials`, named profiles)
  - Panic handling to prevent AWS SDK panics from corrupting TUI
  - Configuration in `~/.config/jiq/config.toml`:
    ```toml
    [ai]
    enabled = true
    provider = "bedrock"

    [ai.bedrock]
    region = "us-east-1"
    model = "anthropic.claude-3-haiku-20240307-v1:0"
    profile = "my-aws-profile"  # optional
    ```

### Changed
- **Multi-Provider Error Context** - Refactored `AiError` enum for better provider support
  - Convert tuple variants to struct variants with named fields
  - Add `provider` field to error variants (NotConfigured, Network, Api, Parse)
  - Error messages now include `[Provider]` prefix for clearer diagnostics
  - Add `provider_name()` method to `AsyncAiProvider` trait
  - Mark `AiError` as `#[non_exhaustive]` for future extensibility

### Dependencies
- Added `aws-config` (with rustls for musl compatibility)
- Added `aws-sdk-bedrockruntime` (with rustls)

## [3.0.4] - 2025-12-17

### Added
- **AI Context Enhancement**: Full nested JSON schema and last working query context
  - Created `json` module with `extract_json_schema()` for recursive type extraction
  - Schema extracted once at startup and reused for all AI requests
  - Example: `{"users": [{"name": "John"}]}` → `{"users": [{"name": "string"}]}`
  - On error: AI now receives last successful query + its output for better suggestions
  - Increased JSON sample size from 1000 to 2000 characters
  - Added `ContextParams` struct to group AI context parameters

### Changed
- AI prompts now include full nested JSON structure instead of shallow top-level keys
- Error prompts include "Last Working Query" and "Its Output" sections when available
- Success prompts only include schema enhancement (base query not relevant)

## [3.0.3] - 2025-12-16

### Fixed
- Reverted manual workflow changes to use cargo-dist generated workflow
  - Manual edits to release.yml are not supported by cargo-dist
  - Original workflow configuration restored for proper CI/CD operation
- Coverage generation now uses single-threaded test execution
  - Prevents jq process race conditions during parallel test execution
  - Matches test job configuration for consistency

## [3.0.2] - 2025-12-16

### Fixed
- Release workflow: Download cached dist binary in build-local-artifacts job
  - Fixes "dist: command not found" error in musl container builds
  - Ensures dist is available before running build commands (reverted in 3.0.3)

## [3.0.1] - 2025-12-16

### Fixed
- Build failures on x86_64-unknown-linux-musl target by switching from native-tls to rustls-tls
  - Resolves static linking issues with OpenSSL on musl targets
  - Enables successful cross-platform binary distribution
  - No functional changes - same HTTP/TLS behavior with better compatibility
- Release badge added to README

## [3.0.0] - 2025-12-16

### Added
- **[EXPERIMENTAL] AI Assistant** - Context-aware query suggestions powered by Anthropic Claude API
  - `Ctrl+A` to toggle AI assistant popup
  - Intelligent suggestions for query fixes, optimizations, and natural language interpretation
  - Streaming responses with real-time token-by-token display
  - Direct suggestion selection with `Alt+1-5` or navigation with `Alt+↑↓`/`Alt+j/k`
  - Token-based request cancellation for instant abort
  - Configuration support in `~/.config/jiq/config.toml`:
    ```toml
    [ai]
    enabled = true

    [ai.anthropic]
    api_key = "your-api-key-here"
    model = "claude-haiku-4-5-20251001"
    ```
  - Comprehensive test coverage with 100+ property-based tests and snapshot tests
  - Query-change-only triggering to prevent excessive API calls
  - Graceful fallback to raw responses when structured parsing fails

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

### Documentation
- Updated README with AI assistant section, keybindings, and configuration examples
- Added AI keybindings to help popup (`Ctrl+A`, `Alt+1-5`, `Alt+↑↓`)

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
