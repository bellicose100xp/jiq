# JSON Interactive Query (jiq) Tool
`jiq` is a command-line tool that allows you to interactively query JSON data in real-time using a simple and intuitive interface.

## Requirements
- `jq` - Command-line JSON processor
- `jless` - JSON viewer with interactive capabilities

## Usage
### CLI Usage
```sh
# Using jiq with a pipe - outputs filtered result to STDOUT
cat data.json | jiq

# Using jiq with a file - outputs filtered result to STDOUT
jiq data.json

# Output only the jq query used during interactive session
jiq -q|--query-only data.json
cat data.json | jiq -q

# Display help information
jiq -h|--help
```

### Interactive Usage
1. Type the `jq` filter expression in the input field at the bottom
2. As you type, the filtered JSON data will be displayed in real-time in the main area
3. The input field supports VIM keybindings for editing:
   - Start in insert mode for immediate typing
   - Press `ESC` to enter normal mode for VIM navigation
   - Press `ESC` again in normal mode to exit `jiq`
4. Press `Tab` to switch focus between the input field and the results area
5. When focused on results, navigate using `jless` keybindings
6. Press `Tab` to return to the input field from results view

## Features
- **Real-time filtering**: See results instantly as you type your `jq` query
- **VIM keybindings**: Full VIM support in the query input field
- **Interactive navigation**: Browse results with `jless` keybindings
- **Flexible output**: Get filtered results or extract the query for use in scripts
- **Stateless**: No configuration files or history - works independently

