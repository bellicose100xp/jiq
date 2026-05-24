---
title: Path-at-cursor
parent: Features
nav_order: 1
description: Drill in, step back, and walk the JSON tree with > < * ^ } chords.
---

# Path-at-cursor

The results pane title shows the live jq path of the value pretty-printed on the cursor row. Five keys on the results pane rewrite the input query in place — the result re-runs as if you'd typed the new query yourself.

Focus the results pane with <kbd>Shift</kbd>+<kbd>Tab</kbd>, then move the cursor (<kbd>j</kbd> / <kbd>k</kbd>) to the value you want to drill into.

<div class="tui-mockup with-title" data-title="Results pane title">
<pre>╭─ Results · Array [50 objects] · .users[2].profile.email ──╮
│ [                                                          │
│   {                                                        │
│     "name": "alice",                                       │
│     "profile": {                                           │
│       "email": "alice@example.com"  ← cursor               │
│     }                                                      │
│   },                                                       │
│   ...                                                      │
╰────────────────────────────────────────────────────────────╯
╰─ &gt; value · &lt; back · * iterate · ^ parent · } wrap ─────────╯</pre>
</div>

The path uses `.field` for simple ASCII identifiers and `.["field"]` for keys with hyphens, digits, spaces, or non-ASCII characters. Long paths head-truncate with `…`. The path span is hidden when the result is a multi-document stream.

## The five chords

<kbd>&gt;</kbd> pipe-composes the cursor row's path onto the current query. If the query is empty or `.`, replaces it outright.

<div class="io-pair">
  <div>
    <div class="io-label">Query · cursor on .users[0].email</div>
    <div class="io-block">.users[]</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After &gt;</div>
    <div class="io-block">.users[] | .users[0].email</div>
  </div>
</div>

<kbd>&lt;</kbd> pops the most recent <kbd>&gt;</kbd> / <kbd>*</kbd> / <kbd>}</kbd> snapshot, restoring the prior query, cursor row, and scroll position. Always pops — manual edits between drill-ins are discarded.

<kbd>*</kbd> replaces the rightmost array index in the cursor row's path with `[]` to fan out across that level.

<div class="io-pair">
  <div>
    <div class="io-label">Cursor on .users[2].tags[1]</div>
    <div class="io-block">.users[2].tags[1]
&quot;rust&quot;</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After *</div>
    <div class="io-block">.users[2].tags[]
&quot;rust&quot;
&quot;tui&quot;
&quot;json&quot;</div>
  </div>
</div>

<kbd>^</kbd> drops the last step from the trailing path segment of the typed query. Pipe-aware — only operates on the segment after the last `|`. Doesn't push to the undo ring.

<div class="drill-chain">
  <div class="step">.users[0].name</div>
  <div class="arrow">^</div>
  <div class="step">.users[0]</div>
  <div class="arrow">^</div>
  <div class="step">.users</div>
  <div class="arrow">^</div>
  <div class="step active">.</div>
</div>

<kbd>}</kbd> wraps the cursor's leaf as a single-entry object so the result includes the key alongside the value.

<div class="io-pair">
  <div>
    <div class="io-label">Cursor on .users[0].name</div>
    <div class="io-block">.users[0].name
&quot;alice&quot;</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After }</div>
    <div class="io-block">.users[0] | {name}
{ &quot;name&quot;: &quot;alice&quot; }</div>
  </div>
</div>

For keys that need bracket notation, jiq emits `<parent> | {"foo-bar": .["foo-bar"]}` instead.

## In search mode

When the search overlay is open, <kbd>&gt;</kbd>, <kbd>*</kbd>, and <kbd>}</kbd> act on the current match's row (not the result-pane cursor) and close the search overlay on success. <kbd>&lt;</kbd> and <kbd>^</kbd> behave identically inside or outside search.

## Shortcuts

| Key | Action | Undo ring |
|---|---|---|
| <kbd>&gt;</kbd> | Drill in — pipe-compose cursor row's path | push |
| <kbd>&lt;</kbd> | Step back — pop most recent snapshot | pop |
| <kbd>*</kbd> | Iterate — replace last `[N]` with `[]` | push |
| <kbd>^</kbd> | Parent — drop last path step (pipe-aware) | — |
| <kbd>}</kbd> | Wrap — leaf as `<parent> \| {key}` | push |

{: .shortcuts }
