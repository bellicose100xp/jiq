---
title: Mouse support
parent: Features
nav_order: 7
description: Click to focus, scroll, drag-select, and apply suggestions — full mouse interaction across the TUI.
---

# Mouse support
{: .no_toc }

[Features](./) · [Quick reference](../quick-reference)

<details open markdown="block">
  <summary>On this page</summary>
  {: .text-delta }
- TOC
{:toc}
</details>

---

jiq is keyboard-first, but every common mouse gesture is wired up. Useful for casual exploration, live demos, and switching focus between the input and the results pane without thinking about <kbd>Shift</kbd>+<kbd>Tab</kbd>. Wheel-scroll the result, click into a long query to fix a typo, double-click an autocomplete suggestion — they all work.

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
- **Click + drag** → multi-line visual selection. Release the mouse to end the selection, then press <kbd>y</kbd> to copy the selected lines to the clipboard.
- **Scrollbar** (rendered on the right edge) → click and drag the thumb to scroll.

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

The filled segment (█) reflects the current scroll position within the full result. Click anywhere on the track to jump there, or drag the thumb to scrub through the output.

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
> - Clicking to switch focus to the results pane **closes any open popup** (history, snippets, help). This keeps the interaction predictable: a click "commits" you to the results pane.
> - During an active search, **clicking the results pane confirms the search** — the same effect as pressing <kbd>Tab</kbd> in the search bar. The matches stay highlighted; you continue navigating with <kbd>n</kbd> / <kbd>N</kbd>.

---

## Terminal compatibility

{: .tip }
Most modern terminals (iTerm2, Alacritty, Ghostty, kitty, WezTerm, foot, Windows Terminal, the macOS / GNOME / KDE built-ins) forward mouse events out of the box. A few do not:
>
> - Older `screen` versions need `mousetrack on` in `.screenrc`.
> - `tmux` has it on by default since 2.1, but very old setups may need `set -g mouse on`.
> - Some SSH multiplexers strip mouse events; if clicks don't register over SSH, check the terminal's mouse-forwarding setting.

If your terminal is in this group, the keyboard shortcuts cover everything mouse interaction does — see the [quick reference](../quick-reference).
