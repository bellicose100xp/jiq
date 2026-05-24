---
title: Results pane
parent: Features
nav_order: 1
description: Cursor navigation, drill chords, visual line selection, and result-type indicators on the output pane.
---

# Results pane

The bottom pane that renders the jq output. Focus it with <kbd>Shift</kbd>+<kbd>Tab</kbd> or click; release with <kbd>Tab</kbd> / <kbd>BackTab</kbd> / <kbd>i</kbd> (the last drops back into INSERT mode).

The title shows the result type and the live jq path of the value pretty-printed on the cursor row. Bottom-left shows query execution time. Bottom border shows context hints for the chords available right now. A scrollbar appears on the right when content overflows.

<div class="tui-mockup with-title" data-title="Results pane — focused, cursor on row 4">
<pre>╭─ Array [50] · .users[2].email ────────────╮▲
│ [                                          │█
│   { "name": "alice", ... },                │█
│   { "name": "bob",   "email": "..." }    ← │█
│   ...                                      │█
│ ]                                          ││
│ 42ms                                       │▼
╰─ &gt; value · * iterate · ^ parent · } wrap ─╯</pre>
</div>

The path uses `.field` for simple ASCII identifiers and `.["field"]` for keys with hyphens, digits, spaces, or non-ASCII. Long paths head-truncate with `…`. The path span is hidden for multi-document streams.

When the query has a syntax error the title flips to `⚠ Syntax Error` and the previous result stays rendered (dimmed). Valid query with no output: `∅ No Results`. Search with no matches: `⚠ No Matches`. Execution time turns yellow at 200ms and red at 1s.

## Navigation

The cursor is a row marker. It anchors drill chords and visual selection.

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
| <kbd>v</kbd> <kbd>V</kbd> | Visual line mode |
| <kbd>y</kbd> | Yank selection (or full result if no selection) |

{: .shortcuts }

## Drill chords

Five keys rewrite the input query in place. The result re-runs as if you'd typed it.

<kbd>&gt;</kbd> pipe-composes the cursor row's path onto the current query. Empty or `.` query: replaces it outright.

<div class="io-pair">
  <div>
    <div class="io-label">Query · cursor on .users[0].email</div>
    <div class="io-block">.</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After &gt;</div>
    <div class="io-block">.users[0].email</div>
  </div>
</div>

<kbd>*</kbd> replaces the rightmost array index in the cursor row's path with `[]` to fan out across that level.

<div class="io-pair">
  <div>
    <div class="io-label">Cursor on .users[2].tags[1]</div>
    <div class="io-block">.
"rust"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After *</div>
    <div class="io-block">.users[2].tags[]
"rust"
"tui"
"json"</div>
  </div>
</div>

<kbd>^</kbd> drops the last step from the trailing path segment. Pipe-aware — only acts on the segment after the last `|`.

<div class="drill-chain">
  <div class="step">.users[0].name</div>
  <div class="arrow">^</div>
  <div class="step">.users[0]</div>
  <div class="arrow">^</div>
  <div class="step">.users</div>
  <div class="arrow">^</div>
  <div class="step active">.</div>
</div>

<kbd>}</kbd> wraps the leaf as a single-entry object so the result includes the key alongside the value.

<div class="io-pair">
  <div>
    <div class="io-label">Cursor on .users[0].name</div>
    <div class="io-block">.
"alice"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After }</div>
    <div class="io-block">.users[0] | {name}
{ "name": "alice" }</div>
  </div>
</div>

<kbd>&lt;</kbd> pops the most recent <kbd>&gt;</kbd> / <kbd>*</kbd> / <kbd>}</kbd> snapshot, restoring the prior query, cursor row, and scroll. Manual edits between drill-ins are discarded by the pop. <kbd>^</kbd> doesn't push to the ring.

In search mode, <kbd>&gt;</kbd> / <kbd>*</kbd> / <kbd>}</kbd> act on the current match's row (not the cursor) and close the overlay. <kbd>&lt;</kbd> and <kbd>^</kbd> behave identically inside or outside search.

| Key | Action | Undo ring |
|---|---|---|
| <kbd>&gt;</kbd> | Drill in — pipe-compose cursor row's path | push |
| <kbd>&lt;</kbd> | Step back — pop most recent snapshot | pop |
| <kbd>*</kbd> | Iterate — replace last `[N]` with `[]` | push |
| <kbd>^</kbd> | Parent — drop last path step | — |
| <kbd>}</kbd> | Wrap leaf as `<parent> \| {key}` | push |

{: .shortcuts }
