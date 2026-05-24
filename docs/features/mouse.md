---
title: Mouse
parent: Features
nav_order: 7
description: Click to focus, scroll any pane, drag-select in results, click suggestions and history rows.
---

# Mouse

Every pane responds to the mouse. Wheel scrolls the pane under the cursor; left-click focuses or selects.

## Per-pane behavior

### Input field

- **Click** when unfocused: takes focus, switches to INSERT mode.
- **Click** when focused: positions the text cursor at the click column.
- **Wheel**: horizontal scroll through the query.

### Results pane

- **Click**: focuses the pane and moves the row cursor to the clicked line. If a search is active and unconfirmed, the click also confirms it.
- **Drag** while in visual mode (<kbd>v</kbd> / <kbd>V</kbd>): extends the selection to the row under the mouse.
- **Wheel**: scrolls the pane by 3 lines.

### Autocomplete dropdown, AI popup, snippet list

- **Click a row**: selects that suggestion / item.
- **Wheel**: scrolls the list.

For AI suggestions, clicking a row both selects and applies it.

### History popup

- **Click a row**: applies that query and closes the popup.
- **Click `✕`** on a hovered row: deletes that entry from history.
- **Wheel**: scrolls the list.

### Help popup

- **Click a tab in the tab bar**: switches sections.
- **Wheel**: scrolls the active section.
- **Click outside the popup**: closes it.

### Search bar

- **Click** while a confirmed search is active: returns to edit mode so you can refine the pattern.

## Hover

Hovering changes which row jiq considers "active":

- Results pane: highlights the row under the cursor.
- AI / snippet / help popups: visually previews the row under the cursor.
- History popup: reveals the `✕` delete button on the hovered row.

## Shortcuts

| Action | Mouse |
|---|---|
| Focus a pane | Left-click anywhere inside it |
| Position cursor in input | Click while input is focused |
| Move row cursor in results | Click on any line |
| Scroll | Wheel up / down |
| Drag-select in results | <kbd>v</kbd> first, then click + drag |
| Apply AI suggestion | Click the suggestion |
| Apply history entry | Click the entry |
| Delete history entry | Click `✕` on the hovered row |
| Switch help tab | Click the tab |
{: .shortcuts }
