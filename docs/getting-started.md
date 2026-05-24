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

That's it. Everything else ships with jiq.

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

## Three ways to feed JSON to jiq

```bash
# 1. From a file
jiq data.json

# 2. From stdin
cat data.json | jiq
echo '{"name": "Alice", "age": 30}' | jiq
curl https://api.example.com/data | jiq

# 3. From the clipboard (no args, no pipe)
jiq
```

The clipboard path tries the OS clipboard first, falls back to OSC 52 over SSH, and drops into the [paste-recovery view](./features/clipboard) if all of that fails. Read the full clipboard page for SSH/tmux specifics.

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

The TUI opens in **INSERT mode** (cyan border on the input). Try this:

1. Type `.users[]`. Watch the results pane stream all three user objects.
2. Add ` | select(.active)`. Bob disappears in real time.
3. Add ` | .email`. Just two emails left.
4. Press <kbd>Tab</kbd> to step into the **Results pane**.
5. Move your cursor onto `alice@example.com` with <kbd>j</kbd> / <kbd>k</kbd>.
6. Press <kbd>Enter</kbd> to print the filtered JSON to stdout, or <kbd>Ctrl</kbd>+<kbd>Q</kbd> to print just the query string.

That's the loop. The rest of jiq makes the loop faster.

---

## Speed up the loop — five things to learn next

| Time spent | Read this |
|:---|:---|
| 30 seconds | The [Quick reference](./quick-reference) — keybind cheat sheet |
| 2 minutes | [Path-at-cursor](./features/path-at-cursor) — single-keystroke drill-in / step-back |
| 2 minutes | [Autocomplete](./features/autocomplete) — `Tab` accepts whatever your data actually has |
| 5 minutes | [VIM editing](./features/vim-editing) — `ci\|` to refactor a single pipe stage |
| 5 minutes | [AI assistant](./features/ai-assistant) — natural-language → jq query |

---

## Workflow patterns

### Build a query, then use it in a script

```bash
# Open jiq, build the query interactively, exit with Ctrl+Q
QUERY=$(curl -s https://api.example.com/data | jiq)

# Now reuse it
curl -s https://api.example.com/data | jq "$QUERY"
```

### Pipe almost anything

```bash
# One-line POJO inspection
echo '{"a":1,"b":[2,3,4]}' | jiq

# kubectl
kubectl get pods -o json | jiq

# AWS CLI
aws ec2 describe-instances | jiq

# Web API
curl -s https://httpbin.org/json | jiq
```

### Recover from an unexpected paste

```bash
# Copy something to clipboard, then:
jiq

# If the clipboard had non-JSON, the paste-recovery view opens.
# Paste your JSON, press Enter, you're in.
```

---

## Modes at a glance

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

## Next up

- [Quick reference](./quick-reference) — every keybind on one page.
- [Features](./features/) — deep-dive per feature.
- [Configuration](./configuration) — `config.toml` reference.
- [Troubleshooting](./troubleshooting) — debug logs, known limitations.
