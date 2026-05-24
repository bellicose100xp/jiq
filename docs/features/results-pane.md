---
title: Results pane
parent: Features
nav_order: 1
description: Navigate the live output, drill into nested values, select and copy lines.
---

# Results pane

The results pane shows your jq output, updated live as you type. Beyond reading results, you can navigate them with the keyboard, drill into nested values with a single key, and copy specific lines — all without leaving jiq or rewriting your query manually.

To focus the results pane, press **Shift+Tab** or click it. Press **Tab** to return to the query input.

## Navigate the output

Use **j** and **k** (or the arrow keys) to move the cursor one line at a time.

| To move | Press |
|---|---|
| 1 line | `j` `k` `↑` `↓` |
| 10 lines | `J` `K` |
| Half page | `Ctrl+d` `Ctrl+u` |
| First line | `g` `Home` |
| Last line | `G` `End` |
| 1 column left/right | `h` `l` `←` `→` |
| 10 columns left/right | `H` `L` |
| Left edge | `0` |
| Right edge of cursor line | `$` |

The title bar shows the result type and the jq path of the value on the cursor row. The bottom border shows which drill keys are available for the current row.

## Drill into a nested value

When you see a nested object or array you want to inspect, you don't need to type out its path manually.

1. Move the cursor to that row with `j` / `k`.
2. Press **`>`**.

jiq appends the path of that value to your current query and re-runs it. The results pane now shows only that part of the data.

<div class="io-pair">
  <div>
    <div class="io-label">Query · cursor on the email field</div>
    <div class="io-block">.</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After pressing ></div>
    <div class="io-block">.users[0].email</div>
  </div>
</div>

## Step back out

Press **`<`** to undo the last drill. jiq restores the previous query, cursor position, and scroll offset.

You can drill in multiple levels — each `>` push is remembered, and each `<` steps back one level.

## Expand an array

When the cursor is on an element inside an array and you want to see all elements at once:

1. Move the cursor to any element of the array.
2. Press **`*`**.

jiq replaces the array index in the path with `[]`, showing all elements.

<div class="io-pair">
  <div>
    <div class="io-label">Cursor on .users[2].tags[1]</div>
    <div class="io-block">.
"rust"</div>
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

## Step up one level

Press **`^`** to remove the last segment from the path — moving from `.users[0].name` to `.users[0]`, then to `.users`, then to `.`.

<div class="drill-chain">
  <div class="step">.users[0].name</div>
  <div class="arrow">^</div>
  <div class="step">.users[0]</div>
  <div class="arrow">^</div>
  <div class="step">.users</div>
  <div class="arrow">^</div>
  <div class="step active">.</div>
</div>

Unlike `<`, pressing `^` does not push to the history ring — you can't `<` back through it.

## Show a value alongside its key

When a value like `"alice"` is useful but you also want to see its key:

1. Move the cursor to that value.
2. Press **`}`**.

jiq rewrites the query to wrap the value in an object: `.users[0].name` becomes `.users[0] | {name}`, showing `{"name": "alice"}`.

## Select and copy specific lines

To copy only part of the output rather than everything:

1. Press **`v`** (or **`V`**) to enter visual selection mode.
2. Use `j` / `k` to extend the selection up or down.
3. Press **`y`** to copy the selected lines to your clipboard.

To copy the entire result without selecting, press **Ctrl+Y** or **Ctrl+O** from anywhere.

## Read the status indicators

| Indicator | What it means |
|---|---|
| `⚠ Syntax Error` | The query has a syntax error; the previous result stays visible |
| `∅ No Results` | The query is valid but produces no output |
| `⚠ No Matches` | A search is active but nothing matched |
| Execution time in yellow | The query took 200ms–1s |
| Execution time in red | The query took over 1s |

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
| `$` | Right edge of cursor line |
| `>` | Drill into value at cursor |
| `<` | Step back to previous query |
| `*` | Expand array at cursor |
| `^` | Remove last path segment |
| `}` | Wrap leaf value as `{key}` object |
| `v` `V` | Enter visual line selection |
| `y` | Copy selection (or full result if none) |
