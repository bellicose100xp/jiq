---
title: Path-at-cursor
parent: Features
nav_order: 1
description: Drill in, step back, and walk the JSON tree with > < * ^ } chords.
---

# Path-at-cursor

[← Features](./) · [Quick reference](../quick-reference)
{: .fs-3 .text-grey-dk-100 }

When you're exploring deeply nested JSON, manually typing something like `.users[2].profile.email` is friction — you have to read the result, count indices, and retype the path correctly. jiq shows that path live in the results pane title bar as you move the cursor, and lets you commit to it with a single keystroke. Five chords on the results pane (and inside search) rewrite the query for you; the existing async pipeline picks the change up and re-runs `jq` exactly as if you had typed it.

---

## The path display

The results pane title bar shows the **live jq path** of the value pretty-printed on the cursor row:

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

The path span fills the available top-border space and head-truncates with `…` only when it would overflow. The display is hidden for multi-document streams (where the parsed-value layout doesn't line up with the rendered output).

{: .note }
> Path emission follows jq's own rules: simple ASCII identifiers use `.field`, everything else (CJK, emoji, hyphens, digit-start, special chars) uses bracket notation `.["field"]`.

---

## The five chords

All chords act on the **results pane**. Press <kbd>Shift</kbd>+<kbd>Tab</kbd> to focus it first.

### <kbd>&gt;</kbd> — drill in

Pipe-compose the path under the cursor onto the current query.

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

When the current query is empty or just `.`, the chord replaces it outright instead of pipe-composing. Each `>` pushes a snapshot onto the undo ring so you can step back with `<`.

---

### <kbd>&lt;</kbd> — step back

Pop the most recent snapshot off the undo ring, restoring the prior query, cursor row, and scroll position.

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
> <kbd>&lt;</kbd> always pops the most recent `>`-snapshot. If you manually edit the textarea between drill-ins, those intermediate edits are discarded by the pop — the mental model is a clean "undo last `>`". The hint `< back` only appears when the undo ring is non-empty.

---

### <kbd>*</kbd> — iterate

Replace the last array index in the path with `[]` to fan out across all elements at that level.

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

Pushes onto the undo ring, so <kbd>&lt;</kbd> reverses it.

---

### <kbd>^</kbd> — parent

Step up one level by parsing the trailing path segment of the typed query and dropping the last step.

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

Pipe-aware — chains across `|`, so a `>` followed by `^` cleanly walks back through a drill chain. Repeated presses walk all the way up: `.users[0].name` → `.users[0]` → `.users` → `.`.

{: .tip }
> `^` is **ring-free**: it doesn't push to the undo stack. Use it for free-form traversal, and keep the `>` / `<` ring for branching exploration.

---

### <kbd>}</kbd> — wrap

Wrap the cursor's leaf as a single-entry object literal, so the result includes the key alongside the value.

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

Starting from a query of `.` on a `users.json` file, walk into the second user, fan out their tags, then step back up:

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

Notice how `<` after `^` lands you back on the iterated query — `<` only ever pops `>`-snapshots, so it skips over the ring-free `^` step.

---

## Search-mode interaction

When the search overlay is open (<kbd>Ctrl</kbd>+<kbd>F</kbd> or <kbd>/</kbd>):

- <kbd>&gt;</kbd>, <kbd>*</kbd>, <kbd>}</kbd> operate on the **current match's row**, not the cursor row, and close the search overlay.
- <kbd>&lt;</kbd> works identically inside or outside search.
- This lets you find a value by text, then drill into it without leaving the keyboard.

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

The results pane shows live hints along the bottom border so the chords are always discoverable:

<div class="tui-mockup">
<pre>╰─ &gt; value · &lt; back · * iterate · ^ parent · } wrap ─╯</pre>
</div>

`< back` is only shown when the undo ring is non-empty. The other four are always visible while the results pane is focused.

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

All five also work in search mode, where `>` `*` `}` operate on the current match's row.
