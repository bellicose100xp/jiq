---
title: Path-at-cursor
parent: Features
nav_order: 1
description: Drill in, step back, and walk the JSON tree with > < * ^ } chords.
---

# Path-at-cursor

The results pane title bar shows the live jq path of the value under the cursor. Five chords rewrite the query in place: drill in, step back, iterate, step up, wrap.

---

## The path display

The title bar shows the jq path of the value pretty-printed on the cursor row:

<div class="tui-mockup with-title" data-title="Results pane title">
<pre>╭─ Results · Array [50 objects] · .users[2].profile.email ──╮
│ [                                                          │
│   {                                                        │
│     "name": "alice",                                       │
│     "profile": {                                           │
│       "email": "alice@example.com"   ← cursor here         │
│     }                                                      │
│   },                                                       │
│   ...                                                      │
╰────────────────────────────────────────────────────────────╯</pre>
</div>

Head-truncates with `…` on overflow. Hidden for multi-document streams where the parsed-value layout doesn't line up with the rendered output.

{: .note }
> Simple ASCII identifiers use `.field`; everything else (CJK, emoji, hyphens, digit-start, special chars) uses bracket notation `.["field"]`.

---

## The five chords

All chords act on the results pane. Press <kbd>Shift</kbd>+<kbd>Tab</kbd> to focus it first.

### <kbd>&gt;</kbd> — drill in

Pipe-composes the path under the cursor onto the current query.

<div class="io-pair">
  <div>
    <div class="io-label">Before</div>
    <div class="io-block">Query: .users[]
Cursor on: "email": "alice@example.com"
Path: .users[0].email</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing &gt;</div>
    <div class="io-block">Query: .users[0].email
Result: "alice@example.com"</div>
  </div>
</div>

When the current query is empty or just `.`, replaces it outright. Each `>` pushes a snapshot onto the undo ring for `<`.

---

### <kbd>&lt;</kbd> — step back

Pops the most recent snapshot off the undo ring, restoring the prior query, cursor row, and scroll position.

<div class="io-pair">
  <div>
    <div class="io-label">Before</div>
    <div class="io-block">Query: .users[0].profile.email
(after two &gt; drill-ins)</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing &lt;</div>
    <div class="io-block">Query: .users[0].profile
(restored from snapshot)</div>
  </div>
</div>

{: .note }
> <kbd>&lt;</kbd> always pops the most recent `>`-snapshot. Manual edits between drill-ins are discarded by the pop. The `< back` hint appears only when the undo ring is non-empty.

---

### <kbd>*</kbd> — iterate

Replaces the last array index with `[]` to fan out across all elements at that level.

<div class="io-pair">
  <div>
    <div class="io-label">Before</div>
    <div class="io-block">Query: .users[2].tags[1]
Result: "rust"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing *</div>
    <div class="io-block">Query: .users[2].tags[]
Result: "rust"
        "tui"
        "json"</div>
  </div>
</div>

Pushes onto the undo ring; <kbd>&lt;</kbd> reverses it.

---

### <kbd>^</kbd> — parent

Drops the last step from the trailing path segment of the typed query.

<div class="io-pair">
  <div>
    <div class="io-label">Before</div>
    <div class="io-block">Query: .users[0].name
Result: "alice"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing ^</div>
    <div class="io-block">Query: .users[0]
Result: { "name": "alice", ... }</div>
  </div>
</div>

Pipe-aware — chains across `|`. Repeated presses walk all the way up: `.users[0].name` → `.users[0]` → `.users` → `.`.

{: .tip }
> `^` is ring-free — it doesn't push to the undo stack.

---

### <kbd>}</kbd> — wrap

Wraps the cursor's leaf as a single-entry object literal so the result includes the key alongside the value.

<div class="io-pair">
  <div>
    <div class="io-label">Before</div>
    <div class="io-block">Query: .users[0].name
Result: "alice"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing }</div>
    <div class="io-block">Query: .users[0] | {name}
Result: { "name": "alice" }</div>
  </div>
</div>

For keys that need bracket notation, jiq emits the long-form `{(key): .[key]}` shape automatically.

---

## A worked drill chain

From `.` on `users.json`: walk into the second user, fan out their tags, step back up.

<div class="drill-chain">
  <div class="step">.</div>
  <div class="arrow">&gt;</div>
  <div class="step">.users</div>
  <div class="arrow">&gt;</div>
  <div class="step">.users[2]</div>
  <div class="arrow">&gt;</div>
  <div class="step">.users[2].tags[0]</div>
  <div class="arrow">*</div>
  <div class="step">.users[2].tags[]</div>
  <div class="arrow">^</div>
  <div class="step">.users[2]</div>
  <div class="arrow">&lt;</div>
  <div class="step active">.users[2].tags[]</div>
</div>

`<` after `^` lands back on the iterated query — `<` only pops `>`-snapshots, skipping the ring-free `^` step.

---

## Search-mode interaction

When the search overlay is open (<kbd>Ctrl</kbd>+<kbd>F</kbd> or <kbd>/</kbd>):

- <kbd>&gt;</kbd>, <kbd>*</kbd>, <kbd>}</kbd> act on the current match's row, not the cursor row, and close the overlay.
- <kbd>&lt;</kbd> behaves identically inside or outside search.

<div class="io-pair">
  <div>
    <div class="io-label">In search mode</div>
    <div class="io-block">Search: "alice@"
Match row: "email": "alice@example.com"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing &gt;</div>
    <div class="io-block">Query: .users[0].email
Search closed
Result: "alice@example.com"</div>
  </div>
</div>

---

## Bottom-border hints

<div class="tui-mockup">
<pre>╰─ &gt; value · &lt; back · * iterate · ^ parent · } wrap ─╯</pre>
</div>

`< back` is shown only when the undo ring is non-empty. The other four are always visible while the results pane is focused.

---

## Quick reference

| Key | Action | Pushes to undo ring? |
|-----|--------|----------------------|
| `>` | Drill into the value at the cursor (pipe-compose path) | Yes |
| `<` | Step back to prior query (only after at least one `>`) | Pops |
| `*` | Iterate over the nearest array (`[N]` → `[]`) | Yes |
| `^` | Step up one level (drop the last path step) | No |
| `}` | Wrap the cursor's leaf as a single-entry object | Yes |
{: .shortcuts }

All five work in search mode, where `>` `*` `}` operate on the current match's row.
