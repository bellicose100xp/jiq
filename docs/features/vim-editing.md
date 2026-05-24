---
title: Vim editing
parent: Features
nav_order: 9
description: Edit queries with Vim motions, operators, and text objects.
---

# Vim editing

The query input has two modes. **INSERT** mode (cyan border) works like a normal text field — just type. **NORMAL** mode (yellow border) gives you Vim navigation and editing commands.

If you don't use Vim, you can ignore this page entirely. INSERT mode is the default and works without any Vim knowledge.

<div class="mode-demo" markdown="0">
  <div class="mode-badge insert">
    <div class="mode-label">INSERT</div>
    <div class="mode-content">.users[] | select(.ac█)</div>
  </div>
  <div class="mode-badge normal">
    <div class="mode-label">NORMAL</div>
    <div class="mode-content">.users[] | select(.active)</div>
  </div>
</div>

## Switch between modes

Press **Esc** to enter NORMAL mode. Press **i**, **a**, **I**, or **A** to return to INSERT.

| Key | Goes to INSERT at |
|---|---|
| `i` | Cursor position |
| `a` | After the cursor |
| `I` | Start of line |
| `A` | End of line |

## Move through the query

In NORMAL mode, use these keys to position the cursor:

| Key | Moves to |
|---|---|
| `h` `l` `←` `→` | One character left / right |
| `0` `^` `Home` | Start of line |
| `$` `End` | End of line |
| `w` | Next word start |
| `b` | Previous word start |
| `e` | Word end |

## Delete and change text

**Delete** with `d` + a motion. **Change** (delete then switch to INSERT) with `c` + a motion.

| Keys | Effect |
|---|---|
| `dw` `db` `de` | Delete word forward / back / end |
| `d$` `d0` | Delete to end / start of line |
| `dd` `D` | Delete entire line / to end |
| `cw` `c$` `cc` `C` | Change word / to end / line |
| `x` | Delete character at cursor |
| `X` | Delete character before cursor |

## Use text objects

Text objects let you act on a whole region without positioning exactly. Use `di{t}` (inside) or `da{t}` (around, including delimiters) with any of:

| Text object | Selects |
|---|---|
| `w` | Word |
| `"` `'` `` ` `` | Inside matching quote |
| `(` `)` `b` | Inside parentheses |
| `[` `]` | Inside brackets |
| `{` `}` `B` | Inside braces |
| `\|` | **Pipe segment** — the jq-specific one |

### The pipe segment

The `|` text object treats each `|` in your query as a separator and acts on the segment under the cursor. This is especially useful in jq where you build queries by chaining pipe steps.

<div class="io-pair">
  <div>
    <div class="io-label">Before (cursor on <code>map</code>)</div>
    <div class="io-block">.users[] | map(.name) | sort</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After <kbd>d</kbd><kbd>i</kbd><kbd>|</kbd></div>
    <div class="io-block">.users[] |  | sort</div>
  </div>
</div>

`da|` also removes one adjacent pipe so the step is gone cleanly:

<div class="io-pair">
  <div>
    <div class="io-label">Before</div>
    <div class="io-block">.users[] | map(.name) | sort</div>
  </div>
  <div class="io-arrow">→</div>
  <div>
    <div class="io-label">After <kbd>d</kbd><kbd>a</kbd><kbd>|</kbd></div>
    <div class="io-block">.users[] | sort</div>
  </div>
</div>

## Search for a character

Jump to a specific character in the query:

| Key | Jumps to |
|---|---|
| `f{c}` | Next occurrence of character `c` |
| `F{c}` | Previous occurrence of character `c` |
| `t{c}` | One before the next `c` |
| `T{c}` | One after the previous `c` |
| `;` | Repeat the last jump |
| `,` | Repeat in the opposite direction |

## Undo and redo

| Key | Effect |
|---|---|
| `u` | Undo |
| `Ctrl+r` | Redo |

## Open search or help from NORMAL mode

| Key | Effect |
|---|---|
| `/` | Open [search in results](./search) |
| `?` | Open the help popup |
