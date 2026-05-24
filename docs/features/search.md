---
title: Search in results
parent: Features
nav_order: 5
description: Find and navigate text matches in JSON output with live highlighting and clear no-match feedback.
---

# Search in results
{: .no_toc }

<details markdown="1">
<summary>Table of contents</summary>

1. TOC
{:toc}

</details>

---

## What it does

Live, case-insensitive full-text search over the rendered output. Type to filter, press <kbd>Enter</kbd> to commit, then walk matches with <kbd>n</kbd> / <kbd>N</kbd> (or <kbd>Enter</kbd> / <kbd>Shift</kbd>+<kbd>Enter</kbd>). Match count and position appear in the results pane title.

Path-at-cursor chords (`>`, `*`, `}`) operate on the **current match** while search is active. See [Path-at-cursor](./path-at-cursor) for the full chord set.

<div class="tui-mockup">
<pre>
╭─ Results · 3/12 matches ─────────────────────────────╮
│   "name": "alice@<span style="background:#4a3a1e;color:#ffd76e">example.com</span>",      ◀ match 1        │
│   "email": "bob@<span style="background:#4a3a1e;color:#ffd76e">example.com</span>",       ◀ match 2        │
│   "name": "carol@<span style="background:#4a3a1e;color:#ffd76e">example.com</span>",      ◀ match 3 ▶      │
│   ...                                                │
╰──────────────────────────────────────────────────────╯
╭─ Search ─────────────────────────────────────────────╮
│ /example.com                                         │
╰── Enter Confirm · n Next · N Prev · Esc Close ───────╯
</pre>
</div>

---

## Three states

### Editing

Trigger with <kbd>Ctrl</kbd>+<kbd>F</kbd> from anywhere, or <kbd>/</kbd> from the results pane (and from NORMAL mode in the input). Typing live-filters and re-highlights every keystroke.

If the current query matches nothing, the results pane dims and shows a <span class="badge badge-yellow">⚠ No Matches</span> badge in the title, alongside <span class="badge badge-red">⚠ Syntax Error</span> and <span class="badge badge-purple">∅ No Results</span>. The viewport resets to the top.

### Navigating

After <kbd>Enter</kbd>, the cursor jumps to the next match. From there:

- <kbd>n</kbd> or <kbd>Enter</kbd> — next match
- <kbd>N</kbd> or <kbd>Shift</kbd>+<kbd>Enter</kbd> — previous match
- <kbd>Ctrl</kbd>+<kbd>F</kbd> or <kbd>/</kbd> — re-enter edit mode

Highlighting persists across cursor movement.

### Closed

<kbd>Esc</kbd> closes the search overlay, clears highlights, and restores full brightness.

---

## Drill-in during search

While search is active, path-at-cursor chords anchor on the **current match's** row instead of the cursor's row:

- <kbd>&gt;</kbd> — drill into the value at the current match. Closes the search overlay.
- <kbd>&#42;</kbd> — iterate the nearest array level around the current match.
- <kbd>}</kbd> — wrap the current match's leaf as a single-entry object.
- <kbd>&lt;</kbd> — steps back through the drill ring (identical inside or outside search).

Two-keystroke workflow: <kbd>/</kbd>`pattern`<kbd>Enter</kbd><kbd>&gt;</kbd>.

---

## Mouse

While editing, clicking the results pane confirms the search and switches focus — same as <kbd>Tab</kbd>.

---

## Shortcuts

{: .shortcuts }
| Key | Action |
|-----|--------|
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Open search from any pane |
| <kbd>/</kbd> | Open search from results pane (or NORMAL mode in input) |
| <kbd>Enter</kbd> | Confirm and jump to next match |
| <kbd>n</kbd> / <kbd>Enter</kbd> | Next match |
| <kbd>N</kbd> / <kbd>Shift</kbd>+<kbd>Enter</kbd> | Previous match |
| <kbd>Tab</kbd> | Bidirectional toggle between search bar and results pane |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> / <kbd>/</kbd> | Re-enter edit mode (while navigating) |
| <kbd>Esc</kbd> | Close search and clear highlights |

---

{: .note }
> Search is **case-insensitive** and matches against the **rendered output** — post-jq, post-pretty-print — not the raw input.
