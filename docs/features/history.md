---
title: Query history
parent: Features
nav_order: 6
description: Searchable history of every successful query with quick cycling, fuzzy filter, and per-entry delete.
---

# Query history
{: .no_toc }

[Features](./) · [Quick reference](../quick-reference)
{: .fs-3 }

1. TOC
{:toc}

---

## What it stores

Every **successful** query — one that produced output, not a syntax error — is appended to history. The list is persisted across sessions, so closing and re-opening jiq keeps your last 1000 queries available.

Storage location depends on your platform:

| Platform | Path |
|---|---|
| Linux | `~/.local/share/jiq/history` |
| macOS | `~/Library/Application Support/jiq/history` |
| Windows | `%APPDATA%\jiq\history` |

Capacity is the **last 1000 entries**. Older queries are dropped as new ones come in.

---

## Quick cycling

Cycle through history **without leaving the input field**:

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>P</kbd> | Previous (older) query |
| <kbd>Ctrl</kbd>+<kbd>N</kbd> | Next (newer) query |
{: .shortcuts }

Each press replaces the current input with the adjacent history entry. Useful when you remember "I had something like this two queries ago" and don't want to break flow to open the popup.

---

## History popup

For fuzzy search across the whole history, open the popup:

- <kbd>Ctrl</kbd>+<kbd>R</kbd> — from any pane
- <kbd>↑</kbd> — from the input field in NORMAL mode

Entries are rendered with full jq syntax highlighting so you can scan the list visually:

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

Type characters to fuzzy-filter the list down to matches. The selected entry (`▸`) is the one that will be applied on <kbd>Enter</kbd>.

---

## Delete from history

{: .note }
> **Two ways to remove an entry.** Press <kbd>Ctrl</kbd>+<kbd>D</kbd> to delete the currently selected row, or hover any row with the mouse to reveal a clickable `✕` button on the right side and click it to delete that one. The popup auto-closes when you delete the last entry. Deletes are persisted to the on-disk history file immediately.

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
