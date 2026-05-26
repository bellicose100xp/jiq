---
title: Vim editing
parent: Features
nav_order: 7
description: Full Vim motions, operators, and text objects in the query input — or just use INSERT mode as a regular text field.
---

# Vim editing

Every Vim motion and operator works in the query input — or ignore it entirely and just type in INSERT mode like a regular text field.

<div class="before-after">
  <input type="radio" name="ba-vim" id="ba-vim-before" checked>
  <input type="radio" name="ba-vim" id="ba-vim-after">
  <div class="ba-header">
    <label for="ba-vim-before" class="ba-toggle">Without Vim motions</label>
    <label for="ba-vim-after" class="ba-toggle">With Vim motions</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">You want to change <code>select(.active)</code> to <code>select(.name)</code> in your query.</p>
    <div class="ba-terminal">.users[] | select(.active) | .email

Hold Backspace x6 to delete "active"
Type "name"
(or: click to position, shift-select, delete, type)

~12 keystrokes</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Same edit — put cursor anywhere on <code>active</code>, then use a Vim text object.</p>
    <div class="ba-terminal">.users[] | select(.active) | .email
                     ^^^^^^ cursor on "active"

Press: ciw        (change inner word)
Type:  name

3 keystrokes total</div>
  </div>
</div>

## Two modes

jiq's query input has two modes, shown by the border color:

<div class="mode-demo">
  <div class="mode-badge insert">
    <div class="mode-label">INSERT</div>
    <div class="mode-content">Type normally. Every character edits the query and re-runs jq in real time. This is the default mode on launch.</div>
  </div>
  <div class="mode-badge normal">
    <div class="mode-label">NORMAL</div>
    <div class="mode-content">Navigate and restructure the query with motions and operators. Nothing is inserted until you enter INSERT mode.</div>
  </div>
</div>

## Switch between modes

| From | To | Press |
|---|---|---|
| INSERT | NORMAL | <kbd>Esc</kbd> |
| NORMAL | INSERT at cursor | `i` |
| NORMAL | INSERT after cursor | `a` |
| NORMAL | INSERT at line start | `I` |
| NORMAL | INSERT at line end | `A` |

## Move through the query

### Basic movement

| Key | Motion |
|---|---|
| `h` / `l` | One character left / right |
| `w` | Next word start |
| `b` | Previous word start |
| `e` | End of current word |
| `0` / `^` | Line start |
| `$` | Line end |

### Jump to a character

| Key | Motion |
|---|---|
| `f{c}` | Forward to character |
| `F{c}` | Backward to character |
| `t{c}` | Forward, stop before character |
| `T{c}` | Backward, stop after character |
| `;` | Repeat last char search (same direction) |
| `,` | Repeat last char search (opposite direction) |

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Character search example</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-output">.users[] | select(.active) | .name</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Cursor at start. Press</span> <span class="term-highlight">f|</span> <span class="term-dim">to jump to first pipe:</span></div>
    <div class="term-line"><span class="term-output">.users[] </span><span class="term-highlight">|</span><span class="term-output"> select(.active) | .name</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Press</span> <span class="term-highlight">;</span> <span class="term-dim">to repeat — jumps to second pipe:</span></div>
    <div class="term-line"><span class="term-output">.users[] | select(.active) </span><span class="term-highlight">|</span><span class="term-output"> .name</span></div>
  </div>
</div>

## Delete or change part of the query

Operators combine with motions: `d` deletes, `c` deletes and enters INSERT mode.

### Operator + motion

| Key | Action |
|---|---|
| `dw` / `db` / `de` | Delete word forward / back / to end |
| `d$` / `d0` / `d^` | Delete to line end / start |
| `dd` / `D` | Delete entire line / to end of line |
| `df{c}` / `dF{c}` | Delete through character forward / backward |
| `dt{c}` / `dT{c}` | Delete until character forward / backward |
| `cw` / `cb` / `ce` | Change word forward / back / to end |
| `c$` / `c0` / `c^` / `cc` / `C` | Change to end / start / entire line |
| `cf{c}` / `cF{c}` | Change through character forward / backward |
| `ct{c}` / `cT{c}` | Change until character forward / backward |

### Single-character edits

| Key | Action |
|---|---|
| `x` | Delete character at cursor |
| `X` | Delete character before cursor |

### Text objects

Text objects select a region based on structure. Prefix with `d` to delete or `c` to change.

| Object | Scope | What it selects |
|---|---|---|
| `iw` / `aw` | Word | Inner word / word + surrounding space |
| `i"` / `a"` | Double quotes | Inside quotes / including quotes |
| `i'` / `a'` | Single quotes | Inside quotes / including quotes |
| `` i` `` / `` a` `` | Backticks | Inside ticks / including ticks |
| `i(` / `a(` | Parentheses | Inside parens / including parens |
| `i[` / `a[` | Brackets | Inside brackets / including brackets |
| `i{` / `a{` | Braces | Inside braces / including braces |
| `i\|` / `a\|` | Pipe segment | Inside pipe / including one pipe |

## Work with pipe segments

The `i|` and `a|` text objects are jq-specific — they treat `|` as a delimiter, just like quotes or brackets work in Vim.

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Pipe segment text object</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Query:</span> <span class="term-output">.users[] | select(.active) | .name</span></div>
    <div class="term-line"><span class="term-dim">Cursor on "select"</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-highlight">ci|</span> <span class="term-dim">selects:</span> <span class="term-output"> select(.active) </span></div>
    <div class="term-line"><span class="term-dim">Type replacement:</span> <span class="term-highlight">map(.email)</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Result:</span> <span class="term-success">.users[] | map(.email) | .name</span></div>
  </div>
</div>

Use `da|` to delete the pipe segment including one pipe delimiter — useful for removing an entire stage from a pipeline.

## Undo and redo

| Key | Action |
|---|---|
| `u` | Undo last change |
| <kbd>Ctrl</kbd>+<kbd>R</kbd> | Redo |

Undo tracks every edit. Multiple undos walk back through the full change history.

## INSERT mode shortcuts

While in INSERT mode, these shortcuts are available without switching to NORMAL:

| Key | Action |
|---|---|
| <kbd>Esc</kbd> | Switch to NORMAL mode |
| <kbd>Up</kbd> / <kbd>Ctrl</kbd>+<kbd>R</kbd> | Open history popup |
| <kbd>Ctrl</kbd>+<kbd>P</kbd> / <kbd>Ctrl</kbd>+<kbd>N</kbd> | Previous / next query in history |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> / <kbd>Ctrl</kbd>+<kbd>U</kbd> | Scroll results half page down / up |

## All keys (NORMAL mode)

| Key | Action |
|---|---|
| `i` / `a` / `I` / `A` | Enter INSERT mode (at/after cursor, line start/end) |
| `h` / `l` | Move left / right |
| `0` / `^` / `$` | Line start / first non-space / line end |
| `w` / `b` / `e` | Word forward / back / end |
| `f{c}` / `F{c}` / `t{c}` / `T{c}` | Find / till character |
| `;` / `,` | Repeat / reverse last char search |
| `x` / `X` | Delete char at / before cursor |
| `dd` / `D` | Delete line / to end |
| `dw` / `cw` / `ciw` | Delete / change word |
| `df{c}` / `dt{c}` / `cf{c}` / `ct{c}` | Delete / change to / till char |
| `di"` / `ci"` / `di(` / `ci(` / etc. | Delete / change inside quotes / brackets |
| `di\|` / `ci\|` / `da\|` / `ca\|` | Delete / change inside / around pipe segment |
| `u` | Undo |
| <kbd>Ctrl</kbd>+<kbd>R</kbd> | Redo |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> / <kbd>Ctrl</kbd>+<kbd>U</kbd> | Scroll results half page down / up |
