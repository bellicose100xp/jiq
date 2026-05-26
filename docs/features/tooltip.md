---
title: Tooltip & overlays
parent: Features
nav_order: 11
description: Look up any jq function inline, see full error messages, and open the keybind reference.
---

# Tooltip and overlays

Contextual overlays that appear when you need them — function documentation inline, full error messages on demand, and a built-in keyboard reference.

<div class="before-after">
  <input type="radio" name="ba-tooltip" id="ba-tooltip-before" checked>
  <input type="radio" name="ba-tooltip" id="ba-tooltip-after">
  <div class="ba-header">
    <label for="ba-tooltip-before" class="ba-toggle">Without tooltip</label>
    <label for="ba-tooltip-after" class="ba-toggle">With tooltip</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">You can't remember how <code>group_by</code> works. Switch to a browser, search jq docs, find the right page.</p>
    <div class="ba-terminal">$ # Was it group_by(.key) or group_by(key) ?
$ # Does it return [[...], [...]] or {key: [...]} ?
$ # Open browser, search "jq group_by"...</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Put your cursor on <code>group_by</code>. The tooltip shows the signature and examples inline.</p>
    <div class="ba-terminal">Query: .users | <span style="color:#58a6ff">group_by</span>(.department)
  ┌─────────────────────────────────────────────┐
  │ group_by(path_exp)                          │
  │ Groups array elements by a common value.    │
  │ Returns: array of arrays                    │
  │ Example: [1,2,3,1] | group_by(.) → [[1,1]] │
  └─────────────────────────────────────────────┘</div>
  </div>
</div>

## Look up a function

Move your cursor onto any jq function name in the query input. A tooltip appears automatically showing:

- The function signature
- A brief description
- Return type information
- An inline example

Press <kbd>Ctrl</kbd>+<kbd>T</kbd> to toggle the tooltip on or off manually.

To disable auto-show, add to `~/.config/jiq/config.toml`:

```toml
[tooltip]
auto_show = false
```

## Read the full error message

When your query has a syntax error, jiq shows a brief indicator in the results title bar. To see the complete error:

Press <kbd>Ctrl</kbd>+<kbd>E</kbd> to toggle the error overlay.

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">Error overlay</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-error">jq: error (at &lt;stdin&gt;:1):</span></div>
    <div class="term-line"><span class="term-error">Cannot iterate over null (null)</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Ctrl+E to close  |  Ctrl+A for AI fix suggestions</span></div>
  </div>
</div>

The overlay sits above the results and disappears when you fix the error or press <kbd>Ctrl</kbd>+<kbd>E</kbd> again.

## Open the help reference

Press <kbd>F1</kbd> to open a multi-tab help popup showing every keybind organized by category. (<kbd>?</kbd> also works when the query input is not in typing mode.)

- Global keys
- Input (INSERT + NORMAL)
- Results pane
- History
- AI assistant
- Search
- Snippets

Click any tab header to switch, or press <kbd>Esc</kbd> to close.

## All keys

| Key | Action |
|---|---|
| `Ctrl+T` | Toggle function tooltip |
| `Ctrl+E` | Toggle error overlay |
| `F1` / `?` | Toggle help popup |
| `Esc` | Close any overlay |
