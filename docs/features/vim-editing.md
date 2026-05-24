---
title: VIM editing
parent: Features
nav_order: 9
description: Full vim motions, operators, text objects, character search, and undo/redo in the query input field — including pipe-aware text objects unique to jq.
---

# VIM editing
{: .no_toc }

[Features](./){: .btn .btn-outline .fs-3 .mr-2 }
[Quick reference](../quick-reference){: .btn .btn-outline .fs-3 }

The query input is a real vim-style editor: motions, operators, text objects, character search, and a proper undo ring. INSERT mode lets you type queries with live autocomplete; NORMAL mode lets you restructure them without retyping.

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
- TOC
{:toc}
</details>

---

## Two modes

- **INSERT mode** (cyan border): just type your query. Autocomplete, real-time results.
- **NORMAL mode** (yellow border): vim-style editing for fast structural changes.

Toggle with <kbd>Esc</kbd> to leave INSERT, and `i` / `a` / `I` / `A` to re-enter it. The mode indicator turns **red** when the current query has a syntax error, so you can spot a broken edit without looking at the results pane.

<div class="tui-mockup with-title" data-title="INSERT mode (cyan border)" markdown="0">
<pre>╭─ Input · INSERT ─────────────────────────╮
│ .users[] | select(.active)               │
╰──────────────────────────────────────────╯</pre>
</div>

<div class="tui-mockup with-title" data-title="NORMAL mode (yellow border)" markdown="0">
<pre>╭─ Input · NORMAL ─────────────────────────╮
│ .users[] | select(.active)               │
╰──────────────────────────────────────────╯</pre>
</div>

{: .note }
Border colors are theme-controlled — they shift with the configured theme but the mode-to-color mapping (INSERT cyan / NORMAL yellow / error red) stays consistent.

---

## jq-aware text objects

The standout text object is the **pipe segment**: `ci|` / `di|` / `ca|` / `da|` operate on everything between two `|` characters. Most jq edits are stage-by-stage refactors, and these chords let you swap one stage without touching anything else.

`ci|` / `di|` (**inside** the pipe segment): keeps both surrounding `|` and the spacing untouched.

`ca|` / `da|` (**around** the pipe segment): also eats **one** of the surrounding `|`, collapsing the stage out of the chain.

<div class="io-pair" markdown="0">
  <div>
    <div class="io-label">Before — cursor on <code>select(.active)</code></div>
    <div class="io-block">.users[] | select(.active) | .name</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After <code>di|</code></div>
    <div class="io-block">.users[] |  | .name</div>
  </div>
</div>

<div class="io-pair" markdown="0">
  <div>
    <div class="io-label">Before — cursor on <code>select(.active)</code></div>
    <div class="io-block">.users[] | select(.active) | .name</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After <code>da|</code></div>
    <div class="io-block">.users[]  | .name</div>
  </div>
</div>

The same operator+text-object grammar works on quotes (`ci"` / `da'`), brackets (`ci(` / `da[` / `ci{`), and backticks (`ci` `` ` ``). All are listed in the [Text objects](#text-objects) table below.

---

## Navigation
{: .shortcuts }

| Key | Action |
|-----|--------|
| `h` / `←` | Move left |
| `l` / `→` | Move right |
| `0` / `Home` | Line start |
| `^` | Line start (first non-blank — same as `0` for single-line query) |
| `$` / `End` | Line end |
| `w` | Next word start |
| `b` | Previous word start |
| `e` | Word end |

---

## Editing
{: .shortcuts }

| Key | Action |
|-----|--------|
| `i` | Enter INSERT at cursor |
| `a` | Enter INSERT after cursor |
| `I` | Enter INSERT at line start |
| `A` | Enter INSERT at line end |
| `x` | Delete char at cursor |
| `X` | Delete char before cursor |

---

## Character search
{: .shortcuts }

Move forward or backward to a specific character within the line.

| Key | Action |
|-----|--------|
| `f{char}` | Find forward to character |
| `F{char}` | Find backward to character |
| `t{char}` | Till forward (stop **before** character) |
| `T{char}` | Till backward (stop **after** character) |
| `;` | Repeat last `f`/`F`/`t`/`T` in same direction |
| `,` | Repeat last `f`/`F`/`t`/`T` in opposite direction |

Useful chord: `f|` jumps to the next pipe, then `ci|` edits the segment after it.

---

## Operators
{: .shortcuts }

Delete (`d`) and change (`c`) compose with motions and character search.

| Key | Action |
|-----|--------|
| `dw` / `db` / `de` | Delete word forward / backward / to end |
| `d$` / `d0` / `d^` | Delete to line end / start |
| `dd` | Delete entire line |
| `D` | Delete to end of line (same as `d$`) |
| `df{char}` / `dF{char}` | Delete to character forward / backward (inclusive) |
| `dt{char}` / `dT{char}` | Delete till character forward / backward (exclusive) |
| `cw` / `cb` / `ce` | Change word forward / backward / to end |
| `c$` / `c0` / `c^` | Change to line end / start |
| `cc` | Change entire line |
| `C` | Change to end of line (same as `c$`) |
| `cf{char}` / `cF{char}` | Change to character forward / backward (inclusive) |
| `ct{char}` / `cT{char}` | Change till character forward / backward (exclusive) |

---

## Text objects
{: .shortcuts }

`ci` / `di` = inside (excludes delimiters). `ca` / `da` = around (includes delimiters; for pipes, includes one surrounding `|`).

| Key | Action |
|-----|--------|
| `ciw` / `diw` | Change / delete inner word |
| `ci"` / `di"` | Change / delete inside double quotes |
| `ci'` / `di'` | Change / delete inside single quotes |
| `ci`` ` `` / `di`` ` `` | Change / delete inside backticks |
| `ci(` / `di(` | Change / delete inside `( )` |
| `ci[` / `di[` | Change / delete inside `[ ]` |
| `ci{` / `di{` | Change / delete inside `{ }` |
| `ci\|` / `di\|` | Change / delete inside pipe segment |
| `ca"` / `da"` | Change / delete around double quotes (includes quotes) |
| `ca'` / `da'` | Change / delete around single quotes (includes quotes) |
| `ca`` ` `` / `da`` ` `` | Change / delete around backticks (includes backticks) |
| `ca(` / `da(` | Change / delete around `( )` (includes brackets) |
| `ca[` / `da[` | Change / delete around `[ ]` (includes brackets) |
| `ca{` / `da{` | Change / delete around `{ }` (includes braces) |
| `ca\|` / `da\|` | Change / delete around pipe segment (includes one `\|`) |

---

## Undo / redo
{: .shortcuts }

| Key | Action |
|-----|--------|
| `u` | Undo |
| `Ctrl+r` | Redo |

The undo ring is per-session and survives mode toggles.

---

## Yank / copy
{: .shortcuts }

| Key | Action |
|-----|--------|
| `yy` | Copy current query (focus-aware: copies results when results pane focused) |

For a results-aware copy regardless of focus, see <kbd>Ctrl</kbd>+<kbd>O</kbd> in the [Quick reference](../quick-reference).

---

## Other from NORMAL mode
{: .shortcuts }

| Key | Action |
|-----|--------|
| `Ctrl+d` / `Ctrl+u` | Scroll **results pane** half page down / up (without leaving the input) |
| `/` | Open search in results |

Visual line selection is not available inside the input — it lives on the results pane. See [Results pane](./results-pane) for `v` / `V` / drag selection.

---

## Why this matters for jq

Most jq edits are pipe-segment edits: replace `select(...)` with a different filter, swap a `map(...)` for a `[ ]` comprehension, drop a stage entirely. The pipe-aware text objects (`ci|` / `da|`) make those edits one chord apiece.

For inner-function tweaks — changing `select(.active)` to `select(.active and .verified)` — `f(` then `ci(` walks straight to the predicate and clears it for retyping. The `f` / `t` family pairs naturally with jq's punctuation-heavy syntax.

The editor is built around mid-flow query refactoring, not general text editing — so motions and objects map directly to the structures jq queries are made of.
