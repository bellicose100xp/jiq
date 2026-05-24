---
title: Search in results
parent: Features
nav_order: 5
description: Find and navigate text matches in JSON output with live highlighting and clear no-match feedback.
---

# Search in results
{: .no_toc }

[Features](./){: .btn .btn-outline .fs-3 .mr-2 }
[Quick reference](../quick-reference){: .btn .btn-outline .fs-3 .mr-2 }
[Path-at-cursor](./path-at-cursor){: .btn .btn-outline .fs-3 }

<details markdown="1">
<summary>Table of contents</summary>

1. TOC
{:toc}

</details>

---

## What it does

Live, case-insensitive full-text search over the rendered output. Type to filter, press <kbd>Enter</kbd> to commit, then walk the matches with <kbd>n</kbd> / <kbd>N</kbd> (or <kbd>Enter</kbd> / <kbd>Shift</kbd>+<kbd>Enter</kbd>). The current match count and position appear right in the results pane title.

Path-at-cursor chords (`>`, `*`, `}`) operate on the **current match** while search is active — so you can find a value, drill into it, and continue exploring without leaving the keyboard. See [Path-at-cursor](./path-at-cursor) for the full chord set.

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

If the current query matches **nothing**, the results pane dims and shows a <span class="badge badge-yellow">⚠ No Matches</span> badge in the title — visually consistent with the existing <span class="badge badge-red">⚠ Syntax Error</span> and <span class="badge badge-purple">∅ No Results</span> badges. The viewport resets to the top so you don't sit on a stale partial-match scroll position.

### Navigating

After <kbd>Enter</kbd>, the cursor jumps to the next match. From there:

- <kbd>n</kbd> or <kbd>Enter</kbd> — next match
- <kbd>N</kbd> or <kbd>Shift</kbd>+<kbd>Enter</kbd> — previous match
- <kbd>Ctrl</kbd>+<kbd>F</kbd> or <kbd>/</kbd> — re-enter edit mode (refine the query)

Highlighting persists across cursor movement — moving onto a row that contains a match keeps the match highlighted, which used to drop in earlier versions.

### Closed

<kbd>Esc</kbd> closes the search overlay, clears highlights, and restores the results pane to full brightness.

---

## Drill-in during search

While search is active, the path-at-cursor chords switch their anchor from "the cursor's row" to "the **current match's** row":

- <kbd>&gt;</kbd> — drill into the value at the current match. Closes the search overlay.
- <kbd>&#42;</kbd> — iterate the nearest array level around the current match.
- <kbd>}</kbd> — wrap the current match's leaf as a single-entry object.
- <kbd>&lt;</kbd> — works identically inside or outside search (steps back through the drill ring).

This makes "find the thing, then go look at it" a two-keystroke workflow: <kbd>/</kbd>`pattern`<kbd>Enter</kbd><kbd>&gt;</kbd>.

---

## Mouse

While search is editing, clicking the results pane **confirms** the search (and switches focus there) — the same outcome as pressing <kbd>Tab</kbd>. So if you've typed enough to find what you wanted, you can just click it.

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
> Search is **case-insensitive** and matches against the **rendered output** — post-jq, post-pretty-print — not the raw input. Whitespace, key/value formatting, and comma punctuation in the rendered output are all fair game.
