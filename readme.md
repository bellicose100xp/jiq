# jiq - JSON Interactive Query Tool

Interactive command-line tool for querying JSON data in real-time using jq syntax.

## Requirements

- **jq** - JSON processor (required)
  - macOS: `brew install jq`
- **jless** - JSON viewer (optional, future versions)
  - macOS: `brew install jless`

## Installation

```sh
# From source
git clone https://github.com/chahcha/jiq
cd jiq
cargo build --release
sudo cp target/release/jiq /usr/local/bin/
```

## Usage

```sh
# Read from file
jiq data.json

# Read from stdin
cat data.json | jiq
echo '{"key": "value"}' | jiq

# Output only the jq query (for scripting)
jiq -q data.json
```

## Interactive Mode

1. Type jq filter expressions in the input field
2. Results update in real-time as you type
3. Press `Tab` to switch between input and results
4. Use arrow keys to scroll results
5. Press `ESC`, `q` or `Ctrl+C` to exit

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch focus between input and results |
| `ESC`, `Ctrl+C` | Exit |
| `↑/↓`, `j/k` | Scroll results |
| `PgUp/PgDn` | Page up/down |
| `Home` | Jump to top |

## Features

- Real-time JSON filtering with jq syntax
- Two-pane interface (results + input)
- Preserves jq's color output and formatting
- Stateless (no config files or history)

## License

Licensed under MIT OR Apache-2.0
