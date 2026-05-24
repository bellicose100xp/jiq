---
title: Results pane
parent: Features
nav_order: 10
description: Cursor navigation, horizontal scrolling, visual line selection, and execution-time / type indicators on the output pane.
---

# Results pane
{: .no_toc }

[Features](./) · [Quick reference](../quick-reference) · [Path-at-cursor](./path-at-cursor)
{: .fs-3 .fw-300 }

The results pane is the bottom (or right, depending on layout) window that shows the output of your jq query. It's not just a viewer — it's where you scroll, drill, search, and copy.
{: .fs-5 .fw-300 }

<details markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
- TOC
{:toc}
</details>

---

## Anatomy

<div class="tui-mockup" markdown="0">
<pre>
╭─ Results · Array [50 objects] · .users[2].profile.email ─────╮
│ [                                                           ▲│
│   {                                                          │
│     "name": "alice",                                         █
│     "email": "alice@example.com",                            █
│     ...                                                      │
│   }                                                          │
│ ]                                                            ▼│
│ 42ms ✓                                                       │
╰── j/k Move · v Select · y Copy · > Drill · Ctrl+F Find ──────╯
</pre>
</div>

| Region | What it shows |
|---|---|
| **Title — left** | Pane name (`Results`). |
| **Title — middle** | Type/count badge — see [Type badges](#type-badges) below. |
| **Title — right** | Live [path-at-cursor](./path-at-cursor) for the value pretty-printed on the cursor row (head-truncated with `…` if it overflows). |
| **Body** | Pretty-printed, syntax-highlighted JSON output of the current query. |
| **Right edge** | Scrollbar — the filled `█` block shows your position; arrows at top/bottom (`▲`/`▼`) indicate scrollable. |
| **Bottom-left** | [Execution time](#execution-time) of the last query. |
| **Bottom border** | Context-sensitive shortcut hints — what you can do *right now* in the current state. |

---

## Type badges

The middle of the title shows what jq returned. The badge updates on every successful run.

| Badge | Meaning |
|---|---|
| `Array [50 objects]` | A JSON array; the count and inner type of its elements. |
| `Stream [3 values]` | Multiple top-level documents separated by newlines (JSONL/NDJSON, or `jq` producing a value stream). |
| `Object` | A single JSON object. |
| `String` / `Number` / `Boolean` / `Null` | A scalar result. |

### Status badges

When something is off, the badge is replaced with a status indicator:

| Badge | When |
|---|---|
| {: .badge .badge-yellow } **`⚠ Syntax Error`** | The current query string isn't valid jq. The pane keeps the last successful output but dims it, so you can edit your way out without losing context. |
| {: .badge .badge-purple } **`∅ No Results`** | The query is valid but produced no output (e.g., `select(...)` matched nothing). |
| {: .badge .badge-red } **`⚠ No Matches`** | [Search](./search) is active and the current search query doesn't match anything in the visible results. |

---

## Execution time

The bottom-left corner shows how long the last query took, color-coded so you can spot expensive operations at a glance.

| Range | Color | Example |
|---|---|---|
| < 200ms | {: .badge .badge-green } **green** | `42ms ✓` |
| 200ms - 1s | {: .badge .badge-yellow } **yellow** | `680ms` |
| > 1s | {: .badge .badge-red } **red** | `1.2s` |

This is the actual execution time of jq plus jiq's preprocessing — it does **not** include the keystroke debounce delay, so you're seeing the true cost of the query, not the typing pause.

---

## Cursor navigation

The cursor in the results pane is a **row marker** — it highlights one line at a time and follows wherever you move it. There's no glyph cursor; the row itself is the cursor. The marker is the anchor used by [`>` drill-in](./path-at-cursor), [`v`/`V` selection](#visual-line-selection), and [search](./search) jumps.

Switch focus to the results pane with <kbd>Shift</kbd>+<kbd>Tab</kbd>, or click the pane.

| Key | Action |
|---|---|
| <kbd>j</kbd> / <kbd>k</kbd> / <kbd>↑</kbd> / <kbd>↓</kbd> | Move cursor 1 line |
| <kbd>J</kbd> / <kbd>K</kbd> | Move cursor 10 lines |
| <kbd>Ctrl</kbd>+<kbd>d</kbd> / <kbd>PgDn</kbd> | Half page down (also works from the input field) |
| <kbd>Ctrl</kbd>+<kbd>u</kbd> / <kbd>PgUp</kbd> | Half page up (also works from the input field) |
| <kbd>g</kbd> / <kbd>Home</kbd> | Jump to top |
| <kbd>G</kbd> / <kbd>End</kbd> | Jump to bottom |
{: .shortcuts }

---

## Horizontal scrolling

For wide objects and long string values that don't fit, scroll horizontally instead of wrapping.

| Key | Action |
|---|---|
| <kbd>h</kbd> / <kbd>l</kbd> / <kbd>←</kbd> / <kbd>→</kbd> | Scroll 1 column |
| <kbd>H</kbd> / <kbd>L</kbd> | Scroll 10 columns |
| <kbd>0</kbd> | Jump to left edge |
| <kbd>$</kbd> | Jump to right edge |
{: .shortcuts }

---

## Visual line selection

Pick contiguous lines and copy them — useful for grabbing one record from an array result without copying the whole pane.

| Key | Action |
|---|---|
| <kbd>v</kbd> / <kbd>V</kbd> | Enter visual line selection mode |
| <kbd>j</kbd> / <kbd>k</kbd> / <kbd>↑</kbd> / <kbd>↓</kbd> | Extend selection up or down |
| <kbd>y</kbd> | Yank selected lines to clipboard |
| <kbd>Esc</kbd> / <kbd>v</kbd> / <kbd>V</kbd> | Exit visual mode |
| **Mouse** click + drag | Same effect — drag across rows to multi-select |
{: .shortcuts }

{: .tip }
> Visual selection grabs the **rendered** text — including indentation and quotes — exactly as it appears. If you want the structured value as JSON, drill in with [`>`](./path-at-cursor) and copy from there with <kbd>Ctrl</kbd>+<kbd>O</kbd> instead.

---

## Path-at-cursor chords

The cursor's path is live in the title bar. From the results pane, these chords rewrite your typed query so the existing async pipeline picks up the change:

| Key | Action |
|---|---|
| <kbd>&gt;</kbd> | Drill into the value at the cursor |
| <kbd>&lt;</kbd> | Step back to the prior query |
| <kbd>*</kbd> | Iterate the nearest array level |
| <kbd>^</kbd> | Step up one level (drop the last path step) |
| <kbd>}</kbd> | Wrap the cursor's leaf as a single-entry object |
{: .shortcuts }

See [Path-at-cursor](./path-at-cursor) for the full walkthrough with examples.

---

## JSONL auto-detection

{: .note }
> When the input contains multiple top-level JSON values separated by newlines (JSONL / NDJSON), jiq wraps them in an array automatically so `jq` can process them as a single document. The type badge shows **`Stream [3 values]`** so you can tell the original was a stream rather than an array. Works for both file input and piped stdin — no flag needed.

```bash
# Each line is a separate JSON object — jiq detects and wraps automatically.
cat events.jsonl | jiq
```

```
{"event": "login",  "user": "alice"}
{"event": "logout", "user": "alice"}
{"event": "login",  "user": "bob"}
```
