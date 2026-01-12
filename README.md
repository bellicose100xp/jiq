# jiq — Interactive JSON query tool with real-time output

[![CI](https://github.com/bellicose100xp/jiq/workflows/CI/badge.svg)](https://github.com/bellicose100xp/jiq/actions)
[![Release](https://github.com/bellicose100xp/jiq/actions/workflows/release.yml/badge.svg)](https://github.com/bellicose100xp/jiq/actions/workflows/release.yml)
[![Coverage](https://codecov.io/github/bellicose100xp/jiq/graph/badge.svg?token=2NOB7SCD6R)](https://codecov.io/github/bellicose100xp/jiq)
[![Crates.io](https://img.shields.io/crates/v/jiq)](https://crates.io/crates/jiq)
[![License](https://img.shields.io/crates/l/jiq)](LICENSE-MIT)

## Features

- **Real-time query execution** - See results as you type
- **AI assistant** - Get intelligent query suggestions, error fixes, and natural language interpretation
- **Context-aware autocomplete** - Next function or field suggestion with JSON type information for fields
- **Function tooltip** - Quick reference help for jq functions with examples
- **Search in results** - Find and navigate text in JSON output with highlighting
- **Query history** - Searchable history of successful queries
- **Clipboard support** - Copy query or results to clipboard (also supports OSC 52 for remote terminals)
- **VIM keybindings** - VIM-style editing for power users
- **Syntax highlighting** - Colorized JSON output and jq query syntax
- **Stats bar** - Shows result type and count (e.g., "Array [5 objects]", "Stream [3 values]")
- **Flexible output** - Export results or query string

## Demo

### With AI Assistant
![AI Demo](https://raw.githubusercontent.com/bellicose100xp/assets/refs/heads/main/jiq/jiq-v3.9-ai-demo.gif)

### Offline Mode (with built-in help)
![Offline Demo](https://raw.githubusercontent.com/bellicose100xp/assets/refs/heads/main/jiq/jiq-v3.9-offline-demo.gif)

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
```

## Usage

**Workflow:**
1. Start typing your jq query (begins in INSERT mode)
2. Use autocomplete suggestions for functions and fields (`Tab` to accept)
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
| `Ctrl+Y` | Copy current query or results to clipboard |
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

**Operators** (delete/change + motion)
| Key | Action |
|-----|--------|
| `dw` / `db` / `de` | Delete word forward/back/end |
| `d$` / `d0` / `d^` | Delete to end/start |
| `dd` | Delete entire line |
| `D` | Delete to end of line (same as `d$`) |
| `cw` / `cb` / `ce` | Change word forward/back/end |
| `c$` / `c0` / `c^` / `cc` | Change to end/start/entire line |
| `C` | Change to end of line (same as `c$`) |

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

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Scroll 1 line |
| `J` / `K` | Scroll 10 lines |
| `h` / `l` / `←` / `→` | Scroll 1 column |
| `H` / `L` | Scroll 10 columns |
| `0` / `^` | Jump to left edge |
| `$` | Jump to right edge |
| `Ctrl+d` / `PageDown` | Scroll half page down (also works from input field) |
| `Ctrl+u` / `PageUp` | Scroll half page up (also works from input field) |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |

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
- Invalid queries display `Syntax Error` message above input while preserving last successful output.
- Results auto-scroll to top when query changes

## Configuration

jiq looks for a configuration file at `~/.config/jiq/config.toml` (or the platform default location).

```toml
[clipboard]
# Clipboard backend: "auto" (default), "system", or "osc52"
# - auto: tries system clipboard first, falls back to OSC 52
# - system: use only OS clipboard (may not work in SSH/tmux)
# - osc52: use terminal escape sequences (works in most modern terminals over SSH)
backend = "auto"

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

- **Autocomplete** - Suggestions are based on output visible in results area. Editing in the middle of a query may produce suboptimal or no suggestions.
- **Syntax highlighting** - Basic keyword-based only, does not analyze structure like tree-sitter.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on code architecture, testing, and pull requests.

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)
