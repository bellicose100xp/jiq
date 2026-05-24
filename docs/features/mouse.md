---
title: Mouse support
parent: Features
nav_order: 7
description: Click to focus, scroll, drag-select, and apply suggestions — full mouse interaction across the TUI.
---

# Mouse support
{: .no_toc }

<details open markdown="block">
  <summary>On this page</summary>
  {: .text-delta }
- TOC
{:toc}
</details>

---

jiq is keyboard-first, but every common mouse gesture is wired up: wheel-scroll, click-to-focus, click-and-drag selection, double-click to apply suggestions.

## Per-pane behavior

### Query input

- **Click** → focuses the input and positions the cursor at the click location.
- **Mouse wheel** → horizontal scroll through long queries.

<div class="tui-mockup with-title" data-title="Click anywhere in the query to position the cursor">
<pre>
╭─ Query ─────────────────────────────────────────────────────╮
│ .users[] | select(.active) | { name, email }                │
│                       ▲                                     │
│                       click here → cursor jumps here        │
╰─────────────────────────────────────────────────────────────╯
</pre>
</div>

### Results pane

- **Click** → focuses the results pane.
- **Mouse wheel** → vertical scroll.
- **Click + drag** → multi-line visual selection. Release, then press <kbd>y</kbd> to copy.
- **Scrollbar** (right edge) → click and drag the thumb to scroll.

<div class="tui-mockup with-title" data-title="Right-edge scrollbar — click and drag the filled segment">
<pre>
╭─ Results ──────────────────╮▲
│ {                          ││
│   "users": [               │█
│     { "name": "alice" },   │█
│     { "name": "bob" },     │█
│     ...                    ││
│   ]                        ││
│ }                          │▼
╰────────────────────────────╯
</pre>
</div>

The filled segment (█) reflects scroll position. Click the track to jump, drag the thumb to scrub.

### Autocomplete dropdown

- **Click** → selects a suggestion (highlights it).
- **Double-click** → applies the suggestion (same as <kbd>Tab</kbd>).

<div class="tui-mockup with-title" data-title="Double-click any row to apply">
<pre>
.users[] | .|
            ╭─────────────────────╮
            │ name      [string]  │   ← single-click highlights
            │ email     [string]  │   ← double-click applies
            │ active    [boolean] │
            │ tags      [array]   │
            ╰─────────────────────╯
</pre>
</div>

### AI assistant popup

- **Click** → selects a suggestion.
- **Double-click** → applies it (same as <kbd>Enter</kbd> or <kbd>Alt</kbd>+<kbd>1</kbd>…<kbd>5</kbd>).

### History popup

- **Click** → selects an entry.
- **Double-click** → applies it.
- **Hover** over any row → reveals an <code>✕</code> delete button on the right; click it to remove just that entry from history.

<div class="tui-mockup with-title" data-title="Hover reveals the ✕ delete button">
<pre>
╭─ History ────────────────────────────────────────────────╮
│  .users[] | select(.active)                              │
│  .items | length                                       ✕ │  ← hovered row
│  [.events[] | .timestamp] | sort                         │
│  .users | map(.email)                                    │
╰─ Enter Select • Ctrl+D Delete • Esc Close ──────────────╯
</pre>
</div>

### Snippets popup

- **Click** → selects a snippet.
- **Double-click** → applies it.

### Help popup

- **Click on a tab** → switches between sections (Global, Input, Results, etc.).

---

## Edge cases

{: .note }
> - Clicking to focus the results pane **closes any open popup** (history, snippets, help).
> - During an active search, **clicking the results pane confirms the search** — same as <kbd>Tab</kbd>. Matches stay highlighted; navigate with <kbd>n</kbd> / <kbd>N</kbd>.

---

## Terminal compatibility

Most modern terminals forward mouse events out of the box (iTerm2, Alacritty, Ghostty, kitty, WezTerm, foot, Windows Terminal, macOS/GNOME/KDE built-ins). Exceptions:

- Older `screen` versions need `mousetrack on` in `.screenrc`.
- `tmux` has mouse on by default since 2.1; older setups need `set -g mouse on`.
- Some SSH multiplexers strip mouse events — check the terminal's mouse-forwarding setting.

Keyboard shortcuts cover every mouse action — see the [quick reference](../quick-reference).
