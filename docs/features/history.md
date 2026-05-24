---
title: Query history
parent: Features
nav_order: 6
description: Searchable history of every successful query with quick cycling, fuzzy filter, and per-entry delete.
---

# Query history
{: .no_toc }

1. TOC
{:toc}

---

## What it stores

Every successful query (one that produced output, not a syntax error) is appended to history and persisted across sessions. Capacity: last 1000 entries. Storage location:

| Platform | Path |
|---|---|
| Linux | `~/.local/share/jiq/history` |
| macOS | `~/Library/Application Support/jiq/history` |
| Windows | `%APPDATA%\jiq\history` |

---

## Quick cycling

Cycle through history without leaving the input field. Each press replaces the current input with the adjacent entry.

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>P</kbd> | Previous (older) query |
| <kbd>Ctrl</kbd>+<kbd>N</kbd> | Next (newer) query |
{: .shortcuts }

---

## History popup

Open with <kbd>Ctrl</kbd>+<kbd>R</kbd> from any pane, or <kbd>↑</kbd> from the input field in NORMAL mode. Entries are rendered with jq syntax highlighting:

```
╭─ History ────────────────────────────────────────────────╮
│ Filter: select                                           │
│                                                          │
│ ▸ .users[] | select(.active == true)              ✕      │
│   .items[] | select(.price > 100) | .name         ✕      │
│   [.events[] | select(.type == "click")] | length ✕      │
│   .data | map(select(.tier == "gold"))            ✕      │
│                                                          │
│ Enter Select · Ctrl+D Delete · Esc Close                 │
╰──────────────────────────────────────────────────────────╯
```

Type to fuzzy-filter. The selected entry (`▸`) is applied on <kbd>Enter</kbd>.

---

## Delete from history

Press <kbd>Ctrl</kbd>+<kbd>D</kbd> to delete the selected row, or hover a row to reveal a clickable `✕` button. The popup auto-closes when the last entry is deleted. Deletes persist to disk immediately.

---

## All shortcuts

### Quick cycling

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>P</kbd> | Previous (older) query |
| <kbd>Ctrl</kbd>+<kbd>N</kbd> | Next (newer) query |
{: .shortcuts }

### Popup

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>R</kbd> | Open history popup |
| <kbd>↑</kbd> (NORMAL mode) | Open history popup |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate entries |
| Type characters | Fuzzy search filter |
| <kbd>Enter</kbd> / <kbd>Tab</kbd> | Apply selected entry and close |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Delete selected entry |
| Click <kbd>✕</kbd> | Delete entry under mouse |
| <kbd>Esc</kbd> | Close without selecting |
{: .shortcuts }
