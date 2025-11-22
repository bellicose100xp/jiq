# jiq — JSON Interactive Query tool with real-time filtering

## Features

- **Real-time query execution** - See results as you type
- **Context-aware autocomplete** - Intelligent suggestions for jq functions and JSON fields
- **Full VIM keybindings** - Modal editing (INSERT/NORMAL/OPERATOR modes)
- **Syntax highlighting** - Colorized JSON output
- **Flexible output** - Export results or query string
- **Undo/redo support** - Never lose your work
- **Zero configuration** - Works out of the box

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
5. Press `Enter` to output results, or `Shift+Enter` to output query

**VIM users:** Press `ESC` to enter NORMAL mode for advanced editing.

## Keybindings

<details>
<summary><b>Global Keys</b> (work anywhere)</summary>

| Key | Action |
|-----|--------|
| `Shift+Tab` | Switch focus between Input and Results |
| `Enter` | Exit and output filtered JSON |
| `Shift+Enter` | Exit and output query string only |
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
| `ESC` | Switch to NORMAL mode / Close autocomplete |

</details>

<details>
<summary><b>Input Field - NORMAL Mode</b> (yellow border)</summary>

**Navigation**
| Key | Action |
|-----|--------|
| `h` / `←` | Move left |
| `l` / `→` | Move right |
| `0` / `Home` | Line start |
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
| `d$` / `d0` | Delete to end/start |
| `dd` | Delete entire line |
| `cw` / `cb` / `ce` | Change word forward/back/end |
| `c$` / `cc` | Change to end/entire line |

**Undo/Redo**
| Key | Action |
|-----|--------|
| `u` | Undo |
| `Ctrl+r` | Redo |

</details>

<details>
<summary><b>Results Pane</b> (when focused)</summary>

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Scroll 1 line |
| `J` / `K` | Scroll 10 lines |
| `Ctrl+d` / `PageDown` | Scroll half page down |
| `Ctrl+u` / `PageUp` | Scroll half page up |
| `g` / `Home` | Jump to top |
| `G` | Jump to bottom |

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
# Press Shift+Enter to get just the query string
```

**Pipeline integration:**
```bash
# Build query interactively, then reuse
QUERY=$(echo '{}' | jiq)  # Press Shift+Enter after building query
echo $QUERY | xargs -I {} jq {} mydata.json
```

## Tips

- Empty query shows original JSON (identity filter `.`)
- Autocomplete suggests jq functions, JSON fields, operators, and patterns as you type
- Invalid queries display jq errors in red
- Color-coded modes: Cyan (INSERT), Yellow (NORMAL), Green (OPERATOR)
- Results auto-scroll to top when query changes

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE)

## Contributing

Issues and pull requests welcome at [github.com/bellicose100xp/jiq](https://github.com/bellicose100xp/jiq)
