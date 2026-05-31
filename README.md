# jiq — Interactive JSON query tool with real-time output

[![CI](https://github.com/bellicose100xp/jiq/workflows/CI/badge.svg)](https://github.com/bellicose100xp/jiq/actions)
[![Release](https://github.com/bellicose100xp/jiq/actions/workflows/release.yml/badge.svg)](https://github.com/bellicose100xp/jiq/actions/workflows/release.yml)
[![Coverage](https://codecov.io/github/bellicose100xp/jiq/graph/badge.svg?token=2NOB7SCD6R)](https://codecov.io/github/bellicose100xp/jiq)
[![Crates.io](https://img.shields.io/crates/v/jiq)](https://crates.io/crates/jiq)
[![License](https://img.shields.io/crates/l/jiq)](LICENSE-MIT)

<h2 align="left">
  <a href="https://bellicose100xp.github.io/jiq/"><img src="https://img.shields.io/badge/%E2%86%92%20Full%20jiq%20documentation%20site-1f6feb?style=for-the-badge&logoColor=white" alt="Full jiq documentation site" height="60" /></a>
</h2>

## Features

- **Real-time query execution** — results update as you type
- **AI assistant** — query suggestions, error fixes, natural-language input
- **Context-aware autocomplete** — schema-aware fields with type hints, plus value suggestions in comparisons
- **Smart input picker** — peek your clipboard at launch, pick clipboard or paste
- **Save result to file** — Ctrl+W writes the rendered output with a live path preview and inline overwrite warning
- **Snippet library** — save and reuse jq queries
- **Search in results** — find and navigate matches in the output
- **Query history** — searchable history of successful queries
- **VIM keybindings** — full motions, operators, text objects
- **Mouse support** — click, scroll, drag-select
- **Syntax highlighting** — colorized JSON output and jq query
- **Themes** — light or dark color scheme

## Demo

### Autocompletion
![Demo](https://raw.githubusercontent.com/bellicose100xp/assets/refs/heads/main/jiq/jiq-demo-v3.20.gif)

### With AI Assistant
![AI Demo](https://raw.githubusercontent.com/bellicose100xp/assets/refs/heads/main/jiq/jiq-demo-ai-v3.20.gif)

### Snippets
![Snippets Demo](https://raw.githubusercontent.com/bellicose100xp/assets/refs/heads/main/jiq/jiq-demo-snippets-v3.20.gif)

### Search
![Search Demo](https://raw.githubusercontent.com/bellicose100xp/assets/refs/heads/main/jiq/jiq-demo-search-v3.20.gif)

## Installation

### Requirements
- **jq** - JSON processor ([installation guide](https://jqlang.org/download/))

### Install via Script (macOS/Linux)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh
```

### Install via Homebrew (macOS)
```bash
brew install bellicose100xp/tap/jiq
```

### Install via Cargo
```bash
cargo install jiq
```

### Download Binary
Download pre-built binaries from [GitHub Releases](https://github.com/bellicose100xp/jiq/releases/latest)

<details>
<summary>From Source</summary>

```bash
git clone https://github.com/bellicose100xp/jiq
cd jiq
cargo build --release
sudo cp target/release/jiq /usr/local/bin/
```

</details>

## Quick Start

```bash
# From file
jiq data.json

# From stdin
cat data.json | jiq
echo '{"name": "Alice", "age": 30}' | jiq
curl https://api.example.com/data | jiq

# Smart picker: Clipboard or Interactive Paste
jiq

# Force a specific source, skip the picker
jiq --clipboard
jiq --paste
```

## Usage

**Workflow:**
1. Start typing your jq query
2. Use autocomplete suggestions for functions and fields
3. See results update in real-time
4. Press `Shift+Tab` to navigate results
5. Press `Enter` to output results, or `Ctrl+Q` to output query

**VIM users:** Press `ESC` to enter NORMAL mode for advanced editing.

## Keybindings

<details>
<summary><b>Global Keys</b> (work anywhere)</summary>

| Key | Action |
|-----|--------|
| `F1` or `?` | Toggle keyboard shortcuts help popup |
| `Shift+Tab` | Switch focus between Input and Results |
| `Ctrl+Y` | Copy current query or results to clipboard (focus-aware) |
| `Ctrl+O` | Copy results to clipboard regardless of focus |
| `Ctrl+W` | Save result to file (live path preview, overwrite warning) |
| `yy` | Copy current query or results to clipboard (NORMAL mode) |
| `Ctrl+T` | Toggle function tooltip (when cursor is on a function) |
| `Ctrl+E` | Toggle error overlay (when syntax error exists) |
| `Ctrl+A` | Toggle AI assistant popup |
| `Enter` | Exit and output filtered JSON |
| `Ctrl+Q` | Exit and output query string only (`Shift+Enter` may also work in some modern terminal emulators) |
| `q` / `Ctrl+C` | Quit without output |

</details>

<details>
<summary><b>Input Field - INSERT Mode</b> (cyan border)</summary>

| Key | Action |
|-----|--------|
| Type characters | Edit jq query (real-time execution) |
| `Tab` | Accept autocomplete suggestion |
| `↑` / `↓` | Navigate autocomplete suggestions |
| `←` / `→` | Move cursor |
| `Home` / `End` | Jump to line start/end |
| `Backspace` / `Delete` | Delete characters |
| `Ctrl+d` / `Ctrl+u` | Scroll results half page down/up |
| `ESC` | Switch to NORMAL mode / Close autocomplete |
| `Mouse click` | Position cursor at click location (when focused) |
| `Mouse wheel` | Horizontal scroll through query |

</details>

<details>
<summary><b>Input Field - NORMAL Mode</b> (yellow border)</summary>

**Navigation**
| Key | Action |
|-----|--------|
| `h` / `←` | Move left |
| `l` / `→` | Move right |
| `0` / `^` / `Home` | Line start |
| `$` / `End` | Line end |
| `w` | Next word start |
| `b` | Previous word start |
| `e` | Word end |

**Editing**
| Key | Action |
|-----|--------|
| `i` | Enter INSERT at cursor |
| `a` | Enter INSERT after cursor |
| `I` | Enter INSERT at line start |
| `A` | Enter INSERT at line end |
| `x` | Delete char at cursor |
| `X` | Delete char before cursor |

**Character Search**
| Key | Action |
|-----|--------|
| `f{char}` | Find forward to character |
| `F{char}` | Find backward to character |
| `t{char}` | Till forward (stop before character) |
| `T{char}` | Till backward (stop after character) |
| `;` | Repeat last search in same direction |
| `,` | Repeat last search in opposite direction |

**Operators** (delete/change + motion)
| Key | Action |
|-----|--------|
| `dw` / `db` / `de` | Delete word forward/back/end |
| `d$` / `d0` / `d^` | Delete to end/start |
| `dd` | Delete entire line |
| `D` | Delete to end of line (same as `d$`) |
| `df{char}` / `dF{char}` / `dt{char}` / `dT{char}` | Delete to/till character forward/backward |
| `cw` / `cb` / `ce` | Change word forward/back/end |
| `c$` / `c0` / `c^` / `cc` | Change to end/start/entire line |
| `C` | Change to end of line (same as `c$`) |
| `cf{char}` / `cF{char}` / `ct{char}` / `cT{char}` | Change to/till character forward/backward |

**Text Objects** (delete/change with scope)
| Key | Action |
|-----|--------|
| `ciw` / `diw` | Change/delete inner word |
| `ci"` / `di"` / `ci'` / `di'` / `ci`` ` `` / `di`` ` `` | Change/delete inside quotes |
| `ci(` / `di(` / `ci[` / `di[` / `ci{` / `di{` | Change/delete inside brackets |
| `ci\|` / `di\|` | Change/delete inside pipe segment |
| `ca"` / `da"` / `ca'` / `da'` / `ca`` ` `` / `da`` ` `` | Change/delete around quotes (including quotes) |
| `ca(` / `da(` / `ca[` / `da[` / `ca{` / `da{` | Change/delete around brackets (including brackets) |
| `ca\|` / `da\|` | Change/delete around pipe segment (including one pipe) |

**Undo/Redo**
| Key | Action |
|-----|--------|
| `u` | Undo |
| `Ctrl+r` | Redo |

**Results Navigation**
| Key | Action |
|-----|--------|
| `Ctrl+d` / `Ctrl+u` | Scroll results half page down/up |

</details>

<details>
<summary><b>Results Pane</b> (when focused)</summary>

**Cursor Navigation**
| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Move cursor up/down 1 line |
| `J` / `K` | Move cursor up/down 10 lines |
| `Ctrl+d` / `PageDown` | Move cursor half page down (also works from input field) |
| `Ctrl+u` / `PageUp` | Move cursor half page up (also works from input field) |
| `g` / `Home` | Jump cursor to top |
| `G` / `End` | Jump cursor to bottom |

**Query Navigation (navigate into and between values)**
| Key | Action |
|-----|--------|
| `>` (or double-click row) | Zoom into value at cursor (appends its path to your query) |
| `<` | Step back to the prior query (undo the last `>`) |
| `*` | Iterate over the nearest array (replace `[N]` with `[]` to show all elements) |
| `^` | Step up one level (remove the last path segment from your query) |
| `}` | Wrap the cursor's value as an object: `.path` becomes `.parent \| {key}` |
| `]` / `[` | Jump cursor to next / previous sibling (wraps around) |

**Horizontal Scrolling**
| Key | Action |
|-----|--------|
| `h` / `l` / `←` / `→` | Scroll 1 column |
| `H` / `L` | Scroll 10 columns |
| `0` | Jump to left edge |
| `$` | Jump to right edge |

**Visual Line Selection**
| Key | Action |
|-----|--------|
| `v` / `V` | Enter visual line selection mode |
| `j` / `k` / `↑` / `↓` | Extend selection up/down |
| `y` | Copy selected lines to clipboard |
| `ESC` / `v` / `V` | Exit visual mode |
| `Click + Drag` | Select multiple lines with mouse |

**Mouse**
| Key | Action |
|-----|--------|
| `Mouse wheel` | Scroll up/down |
| `Click + Drag` | Multi-line visual selection |

</details>

<details>
<summary><b>Search in Results</b></summary>

| Key | Action |
|-----|--------|
| `Ctrl+F` | Open search (from any pane) |
| `/` | Open search (from results pane) |
| `Enter` | Confirm search and jump to next match |
| `n` / `Enter` | Next match |
| `N` / `Shift+Enter` | Previous match |
| `Ctrl+F` / `/` | Re-enter edit mode |
| `ESC` | Close search |

Note: Search is case-insensitive.

</details>

<details>
<summary><b>Query History</b> (last 1000 entries)</summary>

Successful queries are saved to your platform's application data directory:
- **Linux:** `~/.local/share/jiq/history`
- **macOS:** `~/Library/Application Support/jiq/history`
- **Windows:** `%APPDATA%\jiq\history`

**Quick Cycling** (without opening popup):
| Key | Action |
|-----|--------|
| `Ctrl+P` | Previous (older) query |
| `Ctrl+N` | Next (newer) query |

**History Search Popup**:
| Key | Action |
|-----|--------|
| `Ctrl+R` or `↑` | Open history search |
| `↑` / `↓` | Navigate entries |
| Type characters | Fuzzy search filter |
| `Enter` / `Tab` | Select entry and close |
| `Ctrl+D` | Delete selected entry |
| Click `✕` | Delete entry under mouse (revealed on hover) |
| `ESC` | Close without selecting |

</details>

<details>
<summary><b>AI Assistant</b> (context-aware query suggestions)</summary>

The AI assistant analyzes your query and data to provide intelligent suggestions for fixing errors, improving queries, or interpreting natural language.

**Requires configuration** (see Configuration section below)

| Key | Action |
|-----|--------|
| `Ctrl+A` | Toggle AI assistant popup |
| `Alt+1-5` | Apply suggestion 1-5 directly |
| `Alt+↑` / `Alt+↓` | Navigate suggestions |
| `Alt+j` / `Alt+k` | Navigate suggestions (vim style) |
| `Enter` | Apply selected suggestion |
| `Ctrl+A` | Close popup |

</details>

<details>
<summary><b>Snippet Library</b> (save and reuse queries)</summary>

Save frequently used jq queries for quick access. Snippets are stored in `~/.config/jiq/snippets.toml`.

**Browse Mode**
| Key | Action |
|-----|--------|
| `Ctrl+S` | Open snippet library |
| `↑` / `↓` | Navigate snippets |
| Type characters | Fuzzy search filter |
| `Enter` | Apply selected snippet |
| `Ctrl+N` | Create new snippet from current query |
| `Ctrl+E` | Edit selected snippet |
| `Ctrl+R` | Update snippet query with current input |
| `Ctrl+D` | Delete selected snippet |
| `ESC` | Close popup |

**Create/Edit Mode**
| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Navigate between fields |
| `Enter` | Save snippet |
| `ESC` | Cancel |

</details>

## Examples

**Filter active users:**
```bash
cat users.json | jiq
# Type: .users[] | select(.active == true)
# Press Enter to output results
```

**Extract query for scripts:**
```bash
cat data.json | jiq
# Experiment with: .items[] | select(.price > 100) | .name
# Press Ctrl+Q to get just the query string
```

**Pipeline integration:**
```bash
# Build query interactively, then reuse
QUERY=$(echo '{}' | jiq)  # Press Ctrl+Q after building query
echo $QUERY | xargs -I {} jq {} mydata.json
```

## Tips

- Empty query shows original JSON (identity filter `.`)
- Invalid queries display `Syntax Error` above input while preserving last successful output; press `Ctrl+E` for a plain-language explanation and fix hint (jq's raw error, rewritten; works with jq 1.6+).
- Results auto-scroll to top when query changes
- **Non-ASCII keys** (CJK, emoji, accented Latin, hyphens, digit-start) must use bracket notation or quoted-dot notation — jq's `.field` shorthand only accepts ASCII identifiers. jiq's autocomplete emits bracket notation by default:

  ```
  .["名前"]      ✓ works (bracket notation, used by autocomplete)
  ."名前"        ✓ works (quoted-dot, valid alternative)
  .名前           ✗ jq syntax error
  ```

  Same rule applies to `.["café"]`, `.["👋"]`, `.["中文"]`, `.["日本語"]`, etc.

## Configuration

jiq looks for a configuration file at `~/.config/jiq/config.toml` (or the platform default location).

```toml
[clipboard]
# Clipboard backend: "auto" (default), "system", or "osc52"
# - auto: tries system clipboard first, falls back to OSC 52
# - system: use only OS clipboard (may not work in SSH/tmux)
# - osc52: use terminal escape sequences (works in most modern terminals over SSH)
backend = "auto"

[theme]
# Color theme: "auto" (default), "light", or "dark"
# - auto: detect the terminal background at startup, fall back to dark
# - light: force the light palette
# - dark: force the dark palette (the classic Galaxy theme)
mode = "auto"

[autocomplete]
# Number of array elements sampled to discover field suggestions for arrays where fields
# differ across elements. Increasing this may improve suggestions but adds a performance cost.
# Range: 1 - 1000 (default: 10)
array_sample_size = 10

[ai]
# Enable AI assistant
# For faster responses, prefer lightweight models:
# - Anthropic: claude-haiku-4-5-20251001
# - OpenAI: gpt-4o-mini
# - Gemini: gemini-3-flash
enabled = true
# Provider: "anthropic", "openai", "gemini", or "bedrock"
provider = "anthropic"
# Character limit at which JSON schema and output samples are truncated (default: 100000)
# Larger values send more context to AI but increase token usage/costs
# Smaller values send less context and decrease token usage/costs
max_context_length = 100000

# ─────────────────────────────────────────────────────────
# Anthropic
# ─────────────────────────────────────────────────────────
[ai.anthropic]
# Get your API key from: https://console.anthropic.com/settings/keys
api_key = "your-api-key-here"
model = "claude-haiku-4-5-20251001"

# ─────────────────────────────────────────────────────────
# OpenAI
# ─────────────────────────────────────────────────────────
[ai.openai]
# Get your OpenAI API key from: https://platform.openai.com/api-keys
api_key = "sk-proj-..."
model = "gpt-4o-mini"

# ═════════════════════════════════════════════════════════
# OpenAI-Compatible APIs
# ═════════════════════════════════════════════════════════
# Any API that follows the OpenAI format can be used by setting provider = "openai"
# and configuring the base_url and model fields.
#
# Basic pattern:
# [ai.openai]
# base_url = "https://your-api-endpoint/v1"  # API endpoint URL
# api_key = "your-api-key"                   # Optional: only if required by provider
# model = "model-name"                       # Model identifier

# Example configurations:

# Ollama (local)
[ai.openai]
base_url = "http://localhost:11434/v1"
model = "llama3"

# LM Studio (local)
[ai.openai]
base_url = "http://localhost:1234/v1"
model = "local-model"

# x.ai Grok
[ai.openai]
api_key = "your-xai-api-key"
base_url = "https://api.x.ai/v1"
model = "grok-4-fast-non-reasoning"

# ─────────────────────────────────────────────────────────
# Gemini
# ─────────────────────────────────────────────────────────
[ai.gemini]
# Get your API key from: https://aistudio.google.com/apikey
api_key = "AIza..."
# Gemini model to use (e.g., "gemini-3-flash-preview", "gemini-1.5-flash")
model = "gemini-3-flash-preview"

# ─────────────────────────────────────────────────────────
# AWS Bedrock
# ─────────────────────────────────────────────────────────
[ai.bedrock]
region = "us-east-1"
model = "global.anthropic.claude-haiku-4-5-20251001-v1:0"
profile = "default"  # Optional: AWS profile name (uses default credential chain if omitted)
```

## Known Limitations

- **Autocomplete** - Editing in the middle of a query falls back to root-level suggestions; for arrays, a configurable number of elements are sampled to build field suggestions (default: 10, configurable via `array_sample_size` in `[autocomplete]` config section).
- **Syntax highlighting** - Basic keyword-based only, does not analyze structure like tree-sitter.

## Troubleshooting

When reporting a bug, re-run with debug logging and attach `/tmp/jiq-debug.log`:

```bash
jiq --debug data.json        # or: JIQ_DEBUG=1 jiq data.json
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on code architecture, testing, and pull requests.

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)
