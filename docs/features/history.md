---
title: Query history
parent: Features
nav_order: 6
description: Every successful query is saved automatically. Recall any previous query instantly.
---

# Query history

Every query that produces output is saved automatically — recall any previous query without retyping it.

<div class="before-after">
  <input type="radio" name="ba-history" id="ba-history-before" checked>
  <input type="radio" name="ba-history" id="ba-history-after">
  <div class="ba-header">
    <label for="ba-history-before" class="ba-toggle">Without history</label>
    <label for="ba-history-after" class="ba-toggle">With history</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">You ran a useful query ten queries ago. Now you need it again but can't remember the exact syntax.</p>
    <div class="ba-terminal">$ # Was it .users[] | select(.age > 30) | .name ?
$ # Or .users[] | select(.age >= 30) | {name, email} ?
$ # Try again from scratch...</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Press Ctrl+R, type a fragment, select the one you want.</p>
    <div class="ba-terminal">Query: <span style="color:#58a6ff">.users[] | select(.age >= 30) | {name, email}</span>
       recalled from history in 2 seconds</div>
  </div>
</div>

## Cycle through recent queries

Without opening any popup, step through your history one query at a time:

- Press <kbd>Ctrl</kbd>+<kbd>P</kbd> to go back (older)
- Press <kbd>Ctrl</kbd>+<kbd>N</kbd> to go forward (newer)

The query replaces your current input. Results update immediately.

## Search your full history

For deeper recall:

1. Press <kbd>Ctrl</kbd>+<kbd>R</kbd> to open the history popup. (<kbd>Up</kbd> also works when you're not in the middle of typing.)
2. Type any fragment — the list filters by fuzzy match.
3. Use <kbd>Up</kbd> / <kbd>Down</kbd> to highlight an entry.
4. Press <kbd>Enter</kbd> or <kbd>Tab</kbd> to apply it.

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">History popup</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Filter:</span> <span class="term-highlight">select</span><span class="term-cursor"></span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-output">  .users[] | select(.active) | .email</span></div>
    <div class="term-line"><span class="term-highlight">&#9656; .users[] | select(.age >= 30) | {name, email}</span></div>
    <div class="term-line"><span class="term-output">  .items[] | select(.price > 100)</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Enter Apply  Ctrl+D Delete  Esc Close</span></div>
  </div>
</div>

## Delete a history entry

In the history popup, highlight an entry and press <kbd>Ctrl</kbd>+<kbd>D</kbd> to remove it. You can also hover a row to reveal the delete button and click it.

## Where history is stored

Up to 1,000 queries are saved (duplicates deduplicated). The file location depends on your OS:

| OS | Path |
|---|---|
| Linux | `~/.local/share/jiq/history` |
| macOS | `~/Library/Application Support/jiq/history` |
| Windows | `%APPDATA%\jiq\history` |

## All keys

### Quick cycling (no popup)

| Key | Action |
|---|---|
| `Ctrl+P` | Previous (older) query |
| `Ctrl+N` | Next (newer) query |

### History popup

| Key | Action |
|---|---|
| `Ctrl+R` / `Up` | Open popup |
| `Up` / `Down` | Navigate entries |
| Type | Fuzzy filter |
| `Enter` / `Tab` | Apply selected |
| `Ctrl+D` | Delete selected entry |
| Click delete button | Delete entry under mouse |
| `Esc` | Close without selecting |
