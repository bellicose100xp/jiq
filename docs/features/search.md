---
title: Search in results
parent: Features
nav_order: 5
description: Find and step through matches in the rendered result. Case-insensitive, full-text, with live highlight.
---

# Search in results

A case-insensitive substring search over the rendered result text. The search bar opens at the bottom of the results pane and runs on every keystroke.

Open with <kbd>Ctrl</kbd>+<kbd>F</kbd> from anywhere, or <kbd>/</kbd> from the results pane. Close with <kbd>Esc</kbd>.

<div class="tui-mockup with-title" data-title="Ctrl+F — search active">
<pre>╭─ Results · 2/7 ─────────────────────────────────────╮
│ [                                                    │
│   {                                                  │
│     "email": "alice@example.com",  ← match 1         │
│     "email": "bob@example.com"     ← match 2 (cur)   │
│     ...                                              │
│ ]                                                    │
╰──────────────────────────────────────────────────────╯
╭─ Search ─────────────────────────────────────────────╮
│ /example                                             │
╰── Enter Next · N Prev · Esc Close ──────────────────╯</pre>
</div>

The match counter `2/7` shows where you are in the match list. The current match is brighter than the rest. As you type, the cursor scrolls to the first match; if there are no matches, the pane resets to the top.

## Two modes

**Editing.** You're typing the query; matches re-compute on every keystroke. <kbd>Enter</kbd> or <kbd>Tab</kbd> confirms.

**Confirmed.** <kbd>n</kbd> / <kbd>N</kbd> step through matches and wrap around. <kbd>Tab</kbd>, <kbd>Ctrl</kbd>+<kbd>F</kbd>, or <kbd>/</kbd> drops back to editing. All [results-pane navigation](./results-pane) (<kbd>j</kbd>, <kbd>k</kbd>, <kbd>g</kbd>, <kbd>G</kbd>, scroll, etc.) is delegated through.

## Drill into a match

<kbd>&gt;</kbd>, <kbd>*</kbd>, and <kbd>}</kbd> while search is open act on the **current match's row** instead of the cursor row, and close the overlay on success. Search for a value, drill into it, keep going.

<div class="io-pair">
  <div>
    <div class="io-label">Search · current match on alice@</div>
    <div class="io-block">.users
"email": "alice@example.com"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After &gt;</div>
    <div class="io-block">.users | .users[0].email
"alice@example.com"</div>
  </div>
</div>

See [Results pane](./results-pane#drill-chords) for the full chord set.

## Shortcuts

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Open search (any pane) |
| <kbd>/</kbd> | Open search (from results pane) |
| <kbd>Enter</kbd> / <kbd>Tab</kbd> | Confirm; jump to current match |
| <kbd>n</kbd> / <kbd>Enter</kbd> | Next match |
| <kbd>N</kbd> / <kbd>Shift</kbd>+<kbd>Enter</kbd> | Previous match |
| <kbd>Tab</kbd> (confirmed) | Back to editing |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> / <kbd>/</kbd> (confirmed) | Back to editing |
| <kbd>&gt;</kbd> <kbd>*</kbd> <kbd>}</kbd> | Drill into current match's row |
| <kbd>&lt;</kbd> <kbd>^</kbd> | Drill back / parent (no overlay close) |
| <kbd>Esc</kbd> | Close search |

{: .shortcuts }
