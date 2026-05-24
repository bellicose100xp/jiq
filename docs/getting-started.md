---
title: Getting started
nav_order: 2
description: Install jiq, run your first query, learn the loop in two minutes.
---

# Getting started
{: .no_toc }

<details open markdown="block">
  <summary>On this page</summary>
  {: .text-delta }
- TOC
{: toc }
</details>

---

## Requirements

- **`jq`** — the JSON processor jiq runs under the hood. [Install jq](https://jqlang.org/download/) (`jq` 1.6+ works; 1.8.1+ recommended).

---

## Install

### Script (macOS / Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/bellicose100xp/jiq/releases/latest/download/jiq-installer.sh | sh
```

### Homebrew (macOS)

```bash
brew install bellicose100xp/tap/jiq
```

### Cargo

```bash
cargo install jiq
```

### Pre-built binary

Pick a binary from the [releases page](https://github.com/bellicose100xp/jiq/releases/latest), drop it on `$PATH`.

### From source

```bash
git clone https://github.com/bellicose100xp/jiq
cd jiq
cargo build --release
sudo cp target/release/jiq /usr/local/bin/
```

---

## Input

```bash
# From a file
jiq data.json

# From stdin
cat data.json | jiq
echo '{"name": "Alice", "age": 30}' | jiq
curl https://api.example.com/data | jiq

# From the clipboard, with paste-box fallback
jiq
```

With no file and no piped stdin, jiq reads from the clipboard. If the clipboard is empty or not valid JSON, the in-app paste box opens — paste, press <kbd>Enter</kbd>. See [Clipboard & paste recovery](./features/clipboard) for SSH/tmux specifics.

---

## Your first query

Suppose you have `users.json`:

```json
{
  "users": [
    { "name": "alice", "age": 30, "active": true,  "email": "alice@example.com" },
    { "name": "bob",   "age": 22, "active": false, "email": "bob@example.com" },
    { "name": "carol", "age": 45, "active": true,  "email": "carol@example.com" }
  ]
}
```

Launch jiq:

```bash
jiq users.json
```

The TUI opens in **INSERT mode** (cyan border on the input).

1. Type `.users[]` — the results pane streams all three user objects.
2. Add ` | select(.active)` — Bob disappears.
3. Add ` | .email` — two emails left.
4. <kbd>Tab</kbd> into the **Results pane**.
5. Move the cursor onto `alice@example.com` with <kbd>j</kbd> / <kbd>k</kbd>.
6. <kbd>Enter</kbd> prints the filtered JSON to stdout. <kbd>Ctrl</kbd>+<kbd>Q</kbd> prints just the query string.

---

## Next

| Time | Read |
|:---|:---|
| 30s | [Quick reference](./quick-reference) — keybind cheat sheet |
| 2m | [Path-at-cursor](./features/path-at-cursor) — drill-in / step-back |
| 2m | [Autocomplete](./features/autocomplete) — `Tab` to accept |
| 5m | [VIM editing](./features/vim-editing) — `ci\|` to edit one pipe stage |
| 5m | [AI assistant](./features/ai-assistant) — natural-language → jq |

---

## Pipe anything

```bash
echo '{"a":1,"b":[2,3,4]}' | jiq
kubectl get pods -o json | jiq
aws ec2 describe-instances | jiq
curl -s https://httpbin.org/json | jiq
```

Build a query interactively, exit with <kbd>Ctrl</kbd>+<kbd>Q</kbd>, reuse the string:

```bash
QUERY=$(curl -s https://api.example.com/data | jiq)
curl -s https://api.example.com/data | jq "$QUERY"
```

---

## Modes

| State | Border | What's typing? |
|:---|:---|:---|
| **INSERT mode** (default) | cyan | Edits the jq query — every keystroke re-runs |
| **NORMAL mode** | yellow | VIM motions / operators / text objects |
| **Syntax error** | red | Border flips red; previous output stays visible |
| **Search** | — | `/` or <kbd>Ctrl</kbd>+<kbd>F</kbd> opens the search bar at the bottom |

Toggle INSERT ↔ NORMAL with <kbd>Esc</kbd> and `i` / `a` / `I` / `A`.

---

## Output options

| Key | Behavior |
|:---|:---|
| <kbd>Enter</kbd> | Exit, print **filtered JSON** to stdout |
| <kbd>Ctrl</kbd>+<kbd>Q</kbd> | Exit, print **just the query string** to stdout |
| <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Copy current pane (focus-aware) to clipboard, stay in app |
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | Copy results to clipboard regardless of focus |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> / <kbd>q</kbd> | Quit silently |

{: .shortcuts }

---

## More

- [Quick reference](./quick-reference) — every keybind.
- [Features](./features/) — deep-dive per feature.
- [Configuration](./configuration) — `config.toml`.
- [Troubleshooting](./troubleshooting) — debug logs, known limitations.
