---
title: Tooltip & overlays
parent: Features
nav_order: 11
description: Function tooltip, error overlay, and the help popup.
---

# Tooltip & overlays

Three input-anchored overlays.

## Inline overlays

### Function tooltip — <kbd>Ctrl</kbd>+<kbd>T</kbd>

When the cursor is on a jq function or operator, jiq shows the signature, a short description, and example uses. Toggling <kbd>Ctrl</kbd>+<kbd>T</kbd> turns auto-show on or off.

<div class="tui-mockup">
<pre>Query: .users | map(.name)
                     ^ cursor

  ┌─ map ─────────────────────────────────┐
  │ map(f)  apply f to each array element │
  │                                       │
  │ Examples:                             │
  │   .users | map(.name)                 │
  │   .nums | map(. * 2)                  │
  └───────────────────────────────────────┘
</pre>
</div>

Functions take priority over operators. Move the cursor off the function or operator and the tooltip dismisses.

### Error overlay — <kbd>Ctrl</kbd>+<kbd>E</kbd>

When the current query fails, the input border turns red and the title shows `Syntax Error`. The full jq error stays hidden until you press <kbd>Ctrl</kbd>+<kbd>E</kbd>. <kbd>Ctrl</kbd>+<kbd>E</kbd> only toggles when the current query has an error; otherwise it's a no-op.

<div class="tui-mockup">
<pre>┌─ Error ─────────────────────────────────┐
│ jq: error: syntax error, unexpected '|' │
│ at &lt;top-level&gt;, line 1:                 │
│   .users | | map(.name)                 │
└─────────────────────────────────────────┘
</pre>
</div>

The previous successful result stays visible behind the overlay so you can keep reading while you fix the query. <kbd>Ctrl</kbd>+<kbd>E</kbd> again to dismiss.

## Help popup — <kbd>F1</kbd> or <kbd>?</kbd>

Tabbed reference for every keybind. Tabs: Global, Input, Result, History, AI, Search, Snippet. The active tab is auto-selected based on what's currently open.

| Key | Effect |
|---|---|
| <kbd>F1</kbd> <kbd>?</kbd> | Open / close popup |
| <kbd>Tab</kbd> <kbd>Shift</kbd>+<kbd>Tab</kbd> | Next / previous tab |
| <kbd>h</kbd> <kbd>l</kbd> <kbd>←</kbd> <kbd>→</kbd> | Previous / next tab |
| <kbd>1</kbd>..<kbd>7</kbd> | Jump to tab N |
| <kbd>j</kbd> <kbd>k</kbd> <kbd>↑</kbd> <kbd>↓</kbd> | Scroll line |
| <kbd>J</kbd> <kbd>K</kbd> | Scroll 10 lines |
| <kbd>Ctrl</kbd>+<kbd>d</kbd> <kbd>Ctrl</kbd>+<kbd>u</kbd> | Page down / up |
| <kbd>g</kbd> <kbd>G</kbd> | Jump to top / bottom |
| <kbd>Esc</kbd> <kbd>q</kbd> | Close |
{: .shortcuts }

Scroll position is remembered per-tab.
