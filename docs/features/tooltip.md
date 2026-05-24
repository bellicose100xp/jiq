---
title: Tooltip & overlays
parent: Features
nav_order: 11
description: Look up any jq function, read full error messages, and open the keybind reference.
---

# Tooltip & overlays

Three overlays are available while you work: a function tooltip, an error overlay, and a help popup.

## Look up a jq function

Position your cursor on any jq function name in the query input. A tooltip appears showing the function's signature, a description, and usage examples.

<div class="tui-mockup">
<pre>Query: .users | map(.name)
                     ^ cursor on map

  ┌─ map ─────────────────────────────────┐
  │ map(f)  apply f to each array element │
  │                                       │
  │ Examples:                             │
  │   .users | map(.name)                 │
  │   .nums | map(. * 2)                  │
  └───────────────────────────────────────┘
</pre>
</div>

Press **Ctrl+T** to turn auto-show on or off. When auto-show is on, the tooltip appears whenever your cursor is on a known function; when it's off, the tooltip only appears when you press Ctrl+T explicitly.

To configure auto-show in `~/.config/jiq/config.toml`:

```toml
[tooltip]
auto_show = true   # default true
```

## Read the full error message

When a query fails, the input border turns red and the title shows `⚠ Syntax Error`. The abbreviated error is visible there, but the full jq error message is hidden until you need it.

Press **Ctrl+E** to open the error overlay:

<div class="tui-mockup">
<pre>┌─ Error ─────────────────────────────────┐
│ jq: error: syntax error, unexpected '|' │
│ at &lt;top-level&gt;, line 1:                 │
│   .users | | map(.name)                 │
└─────────────────────────────────────────┘
</pre>
</div>

The previous successful result stays visible behind the overlay so you can keep reading the output while you fix the query. Press **Ctrl+E** again to dismiss.

**Ctrl+E** is a no-op when there is no error.

## Open the keybind reference

Press **F1** or **?** to open the help popup. It has seven tabs — Global, Input, Result, History, AI, Search, Snippet — covering every available key.

The active tab is selected automatically based on what's currently open (for example, if the history popup is open, the History tab is shown).

### Navigate the help popup

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Next / previous tab |
| `h` `l` `←` `→` | Previous / next tab |
| `1` … `7` | Jump to tab N |
| `j` `k` `↑` `↓` | Scroll the content |
| `J` `K` | Scroll 10 lines |
| `Ctrl+d` `Ctrl+u` | Page down / up |
| `g` `G` | Jump to top / bottom |
| `Esc` `q` | Close |

Scroll position is remembered per tab — switching tabs and back returns you to where you were.
