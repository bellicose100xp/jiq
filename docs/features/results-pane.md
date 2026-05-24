---
title: Results pane
parent: Features
nav_order: 10
description: Cursor navigation, horizontal scrolling, visual line selection, and the result-type and execution-time indicators on the output pane.
---

# Results pane

The bottom pane that renders the jq output. Focus it with <kbd>Shift</kbd>+<kbd>Tab</kbd> or by clicking; release with <kbd>Tab</kbd>, <kbd>BackTab</kbd>, or <kbd>i</kbd> (the latter also drops back into INSERT mode).

The title shows the result type, the path-at-cursor, and a vertical scrollbar when content overflows. The bottom-left shows query execution time. The bottom border shows context hints for the chords available right now.

<div class="tui-mockup with-title" data-title="Results pane — focused, with cursor on row 4">
<pre>╭─ Results · Array [50 objects] · .users[2].email ──────╮▲
│ [                                                      │█
│   { "name": "alice", ... },                            │█
│   { "name": "bob",   "email": "bob@example.com" }     ←│█
│   ...                                                  │█
│ ]                                                      ││
│ 42ms                                                   │▼
╰─ &gt; value · &lt; back · * iterate · ^ parent · } wrap ────╯</pre>
</div>

## Title bar

The title contains, left to right:

- **Result type.** `Array [N objects]`, `Array [N strings]`, `Array [N mixed]`, `Object`, `String`, `Number`, `Boolean`, `null`, or `Stream [N]` for jq iteration output.
- **Path-at-cursor.** The jq path of the value pretty-printed on the cursor row. See [path-at-cursor](./path-at-cursor).

When the query has a syntax error, the title flips to `⚠ Syntax Error` and the previous successful result stays rendered (dimmed). When the query is valid but produces nothing, the title shows `∅ No Results`. During search with no matches, `⚠ No Matches`.

The execution time badge color tracks duration: under 200ms uses the default border color, 200–999ms turns yellow, ≥1s turns red. Sub-second values render as `42ms`, longer ones as `1.2s`.

## Navigation

The cursor is a row marker. It anchors path-at-cursor, drill chords, and visual selection.

| Key | Action |
|---|---|
| <kbd>j</kbd> <kbd>k</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | Move cursor 1 line |
| <kbd>J</kbd> <kbd>K</kbd> | Move 10 lines |
| <kbd>Ctrl</kbd>+<kbd>d</kbd> <kbd>PgDn</kbd> | Half page down |
| <kbd>Ctrl</kbd>+<kbd>u</kbd> <kbd>PgUp</kbd> | Half page up |
| <kbd>g</kbd> <kbd>Home</kbd> | First line |
| <kbd>G</kbd> <kbd>End</kbd> | Last line |
| <kbd>h</kbd> <kbd>l</kbd> <kbd>←</kbd> <kbd>→</kbd> | Scroll 1 column |
| <kbd>H</kbd> <kbd>L</kbd> | Scroll 10 columns |
| <kbd>0</kbd> | Jump to column 0 |
| <kbd>$</kbd> | Jump to end of cursor line |

{: .shortcuts }

## Visual line selection

<kbd>v</kbd> or <kbd>V</kbd> enters visual-line mode (the cursor row is selected). Move the cursor to grow the selection up or down. <kbd>y</kbd> yanks the selected lines to the clipboard; <kbd>Esc</kbd> exits without copying. Click-and-drag with the mouse does the same thing.

| Key | Action |
|---|---|
| <kbd>v</kbd> <kbd>V</kbd> | Enter visual line mode |
| <kbd>j</kbd> <kbd>k</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | Extend selection |
| <kbd>y</kbd> | Yank selection to clipboard |
| <kbd>Esc</kbd> | Exit visual mode |

{: .shortcuts }

## Other

| Key | Action |
|---|---|
| <kbd>Tab</kbd> <kbd>BackTab</kbd> | Return to input |
| <kbd>i</kbd> | Return to input, INSERT mode |
| <kbd>/</kbd> | Open search |
| <kbd>?</kbd> | Open help |
| <kbd>y</kbd> | Yank result to clipboard (no visual selection: full result) |
| <kbd>&gt;</kbd> <kbd>&lt;</kbd> <kbd>*</kbd> <kbd>^</kbd> <kbd>}</kbd> | Drill chords — see [path-at-cursor](./path-at-cursor) |

{: .shortcuts }
