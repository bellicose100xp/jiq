---
title: Results pane
parent: Features
nav_order: 1
description: Navigate the live output, zoom into nested values, select and copy lines.
---

# Results pane

Stop manually typing long jq paths to explore nested JSON — navigate the output visually and let jiq build the path for you.

<div class="before-after">
  <input type="radio" name="ba-results" id="ba-results-before" checked>
  <input type="radio" name="ba-results" id="ba-results-after">
  <div class="ba-header">
    <label for="ba-results-before" class="ba-toggle">Without jiq</label>
    <label for="ba-results-after" class="ba-toggle">With jiq</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">You guess at paths, hit errors, retype, try again:</p>
    <div class="ba-terminal">$ jq '.data.users[0].profile.settings.notifications' data.json
null
$ jq '.data.users[0].profile.preferences.notifications' data.json
null
$ jq '.data.users[0].settings.notifications' data.json
{"email": true, "sms": false}</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Move the cursor to the row you want, press <kbd>&gt;</kbd>:</p>
    <div class="ba-terminal">Query: .data.users[0]
       ▸ "name": "Alice"
       ▸ "email": "alice@co.dev"
  ──── ▸ "settings": {              ← cursor here
       ▸   "notifications": {...}

Press &gt;  →  query becomes .data.users[0].settings

Press &gt;  →  query becomes .data.users[0].settings.notifications

Press &lt;  →  back to .data.users[0].settings</div>
  </div>
</div>

To focus the results pane, press <kbd>Shift</kbd>+<kbd>Tab</kbd> or click it.

---

## Navigate the output

Use <kbd>j</kbd> and <kbd>k</kbd> (or arrow keys) to move the cursor one line at a time.

| To move | Press |
|---|---|
| 1 line | `j` `k` `↑` `↓` |
| 10 lines | `J` `K` |
| Half page | `Ctrl+d` `Ctrl+u` `PgDn` `PgUp` |
| First line | `g` `Home` |
| Last line | `G` `End` |
| 1 column left/right | `h` `l` `←` `→` |
| 10 columns left/right | `H` `L` |
| Left edge | `0` |
| Right edge | `$` |

The title bar shows the result type and the jq path of the value on the cursor row.

---

## Zoom into a nested value

When you see a nested object or array you want to inspect, move the cursor to that row and press <kbd>&gt;</kbd>.

jiq appends the path of that value to your current query and re-runs it. The output now shows only that piece of data.

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Zooming into nested values</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-highlight">.</span></div>
    <div class="term-line"><span class="term-dim">Cursor on:</span> <span class="term-output">"users": [...]</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-key">&gt;</span> <span class="term-dim">pressed</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-highlight">.users</span></div>
    <div class="term-line"><span class="term-dim">Cursor on:</span> <span class="term-output">{"name": "Alice", ...}</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-key">&gt;</span> <span class="term-dim">pressed</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-highlight">.users[0]</span></div>
  </div>
</div>

<div class="io-pair">
  <div>
    <div class="io-label">Query with cursor on email field</div>
    <div class="io-block">.</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing &gt;</div>
    <div class="io-block">.users[0].email</div>
  </div>
</div>

---

## Step back to the previous query

Press <kbd>&lt;</kbd> to undo the last zoom. jiq restores the previous query, cursor position, and scroll offset.

A clickable `[ < Back ]` badge also appears on the top-left of the results pane border whenever there is something to step back to. Click it to undo the last zoom — same effect as the keyboard chord.

You can zoom in multiple levels — each `>` is remembered, and each `<` steps back one level.

<div class="drill-chain">
  <div class="step">.users[0].email</div>
  <div class="arrow">&lt;</div>
  <div class="step">.users[0]</div>
  <div class="arrow">&lt;</div>
  <div class="step">.users</div>
  <div class="arrow">&lt;</div>
  <div class="step active">.</div>
</div>

---

## Expand an array

When the cursor is on an element inside an array and you want to see all elements at once, press <kbd>*</kbd>.

jiq replaces the array index in the path with `[]`, showing every element.

<div class="io-pair">
  <div>
    <div class="io-label">Cursor on .users[2].tags[1]</div>
    <div class="io-block">"rust"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing *</div>
    <div class="io-block">.users[2].tags[]
"rust"
"tui"
"json"</div>
  </div>
</div>

---

## Step up one level

Press <kbd>^</kbd> to remove the last segment from the path — moving up the hierarchy.

<div class="drill-chain">
  <div class="step">.users[0].name</div>
  <div class="arrow">^</div>
  <div class="step">.users[0]</div>
  <div class="arrow">^</div>
  <div class="step">.users</div>
  <div class="arrow">^</div>
  <div class="step active">.</div>
</div>

Unlike `<`, pressing `^` does not push to the history ring — you cannot `<` back through it.

---

## Show a value alongside its key

When a value like `"alice"` is useful but you also want to see its key, move the cursor to that value and press <kbd>}</kbd>.

jiq rewrites the query to wrap the value in an object: `.users[0].name` becomes `.users[0] | {name}`, showing `{"name": "alice"}`.

<div class="io-pair">
  <div>
    <div class="io-label">Cursor on .users[0].name</div>
    <div class="io-block">"alice"</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing }</div>
    <div class="io-block">.users[0] | {name}
{"name": "alice"}</div>
  </div>
</div>

---

## Walk between siblings

When the cursor is on a child of an object or array, hop to the next or previous sibling without scrolling line by line:

- Press <kbd>]</kbd> to jump to the next sibling
- Press <kbd>[</kbd> to jump to the previous sibling

The cursor lands on the sibling's row — the query is not rewritten. Wraps around at the boundaries.

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Sibling navigation</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Object keys:</span></div>
    <div class="term-line"><span class="term-output">  "users": [...]      </span> <span class="term-dim">← cursor</span></div>
    <div class="term-line"><span class="term-output">  "meta": {...}</span></div>
    <div class="term-line"><span class="term-output">  "config": {...}</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-key">]</span> <span class="term-dim">→ cursor jumps to</span> <span class="term-highlight">"meta"</span></div>
    <div class="term-line"><span class="term-key">]</span> <span class="term-dim">→ cursor jumps to</span> <span class="term-highlight">"config"</span></div>
    <div class="term-line"><span class="term-key">]</span> <span class="term-dim">→ wraps to</span> <span class="term-highlight">"users"</span></div>
  </div>
</div>

Use this to scan an object's keys or array elements quickly, then press `>` to zoom into the one you want.

---

## Select and copy specific lines

To copy only part of the output:

<div class="step-flow">
  <div class="step-item done">
    <div class="step-circle">1</div>
    <div class="step-text">Press <kbd>v</kbd> to enter visual mode</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item done">
    <div class="step-circle">2</div>
    <div class="step-text">Use <kbd>j</kbd>/<kbd>k</kbd> to extend selection</div>
    <div class="step-connector"></div>
  </div>
  <div class="step-item active">
    <div class="step-circle">3</div>
    <div class="step-text">Press <kbd>y</kbd> to copy to clipboard</div>
  </div>
</div>

To copy the entire result without selecting, press <kbd>Ctrl</kbd>+<kbd>Y</kbd> or <kbd>Ctrl</kbd>+<kbd>O</kbd> from anywhere.

---

## Read the status indicators

| Indicator | What it means |
|---|---|
| `L1-20/100 (0%)` (top-right border) | Line/position indicator: visible line range, total lines, and scroll percentage. It lives in the top-right corner of the results border so it stays visible even when the AI or help box overlays the bottom of the screen. During an active search the match count takes this slot instead. |
| `Syntax Error` | The query has a syntax error; the previous result stays visible |
| `No Results` | The query is valid but produces no output |
| `No Matches` | A search is active but nothing matched |
| Execution time in yellow | The query took 200ms-1s |
| Execution time in red | The query took over 1s |

---

## All keys

| Key | Action |
|---|---|
| `j` `k` `↑` `↓` | Move cursor 1 line |
| `J` `K` | Move 10 lines |
| `Ctrl+d` `PgDn` | Half page down |
| `Ctrl+u` `PgUp` | Half page up |
| `g` `Home` | First line |
| `G` `End` | Last line |
| `h` `l` `←` `→` | Scroll 1 column |
| `H` `L` | Scroll 10 columns |
| `0` | Left edge |
| `$` | Right edge |
| `>` | Zoom into value at cursor |
| `<` | Step back to previous query |
| `*` | Expand array at cursor |
| `^` | Remove last path segment |
| `}` | Wrap leaf value as `{key}` object |
| `]` `[` | Jump cursor to next / previous sibling (wraps) |
| `v` `V` | Enter visual line selection |
| `y` | Copy selection (or full result if none) |
