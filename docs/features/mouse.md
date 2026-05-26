---
title: Mouse
parent: Features
nav_order: 9
description: Use the mouse to focus panes, scroll, select output lines, and apply suggestions.
---

# Mouse support

jiq has two main areas: the query input (where you type) and the output area (where results appear). Both respond to the mouse — click, scroll, and drag work alongside keyboard shortcuts.

## What you can do with the mouse

| Gesture | Where | What happens |
|---|---|---|
| Click | Query input or output area | Make that area active for keyboard input |
| Click | Input field | Position cursor at click location |
| Click | Autocomplete | Highlight a suggestion |
| Double-click | Autocomplete | Apply a suggestion |
| Double-click | Output area row | Drill into the value on that row (same as <kbd>&gt;</kbd>) |
| Click + drag | Output area | Select multiple lines |
| Scroll wheel | Output area | Scroll vertically |
| Scroll wheel | Input field | Scroll horizontally through long queries |
| Hover | History popup row | Reveal the delete button |
| Click delete button | History popup | Delete that entry |
| Click | Help popup tab | Switch to that tab |
| Click | Scrollbar | Reposition the scroll thumb |
| Drag | Scrollbar thumb | Drag to scroll |

## Select and copy with the mouse

To copy specific output lines:

1. Click and drag across the lines you want in the output area.
2. The selected lines highlight as you drag.
3. Press <kbd>y</kbd> to copy the selection to your clipboard.

You can also start a selection with the keyboard by pressing `v`, then extend with `j`/`k`.

## Mouse and keyboard together

Mouse actions don't interfere with keyboard state. You can:

- Click a suggestion in autocomplete, then keep typing
- Scroll results with the wheel, then press `>` to zoom into the value under your cursor
- Drag-select lines, press `y` to copy, then continue editing the query

Whichever area you last clicked receives keyboard input. Click to switch, or use <kbd>Shift</kbd>+<kbd>Tab</kbd>.
