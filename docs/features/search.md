---
title: Search in results
parent: Features
nav_order: 5
description: Find a specific value in the rendered output and step through every match.
---

# Search in results

Find any value in your output instantly and jump between every match.

<div class="before-after">
  <input type="radio" name="ba-search" id="ba-search-before" checked>
  <input type="radio" name="ba-search" id="ba-search-after">
  <div class="ba-header">
    <label for="ba-search-before" class="ba-toggle">Without search</label>
    <label for="ba-search-after" class="ba-toggle">With search</label>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Scrolling through 500 lines of JSON output trying to find a specific email address. j, j, j, j... maybe it was further up? k, k, k...</p>
    <div class="ba-terminal">$ jiq users.json
# Type: .users[]
# Now scroll through 500 results looking for "alice"
# j j j j j j j j j j j j j j j j j...
# Did I miss it? k k k k k k...</div>
  </div>
  <div class="ba-state">
    <p class="ba-caption">Press Ctrl+F, type your term, every match highlights instantly. n/N jumps between them.</p>
    <div class="ba-terminal">$ jiq users.json
# Type: .users[]
# Press Ctrl+F, type "alice"
# Instantly: "2/7 matches" — cursor jumps to first match
# Press n → next match, N → previous match</div>
  </div>
</div>

---

## Find a value

Press <kbd>Ctrl</kbd>+<kbd>F</kbd> from anywhere in jiq. A search bar appears at the bottom of the output area. (You can also press <kbd>/</kbd> when the output area is active.)

Type your term — matches highlight in real time as you type. The counter in the title bar shows your position (e.g., `2/7`).

<div class="animated-terminal">
  <div class="terminal-chrome">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
    <span class="terminal-title">search active</span>
  </div>
  <div class="terminal-body">
    <div class="term-line"><span class="term-dim">Results - 2/7 matches</span></div>
    <div class="term-line"><span class="term-output">[</span></div>
    <div class="term-line"><span class="term-output">  {</span></div>
    <div class="term-line"><span class="term-output">    "email": "</span><span class="term-highlight">alice</span><span class="term-output">@example.com",</span></div>
    <div class="term-line"><span class="term-output">    "name": "</span><span class="term-highlight">Alice</span><span class="term-output"> Johnson",</span></div>
    <div class="term-line"><span class="term-output">    "role": "admin"</span></div>
    <div class="term-line"><span class="term-output">  },</span></div>
    <div class="term-line"><span class="term-output">  ...</span></div>
    <div class="term-line">&nbsp;</div>
    <div class="term-line"><span class="term-dim">Search:</span> <span class="term-highlight">alice</span><span class="term-cursor"></span></div>
  </div>
</div>

Press <kbd>Enter</kbd> to confirm and jump to the next match.

Search is case-insensitive — typing `alice` matches `Alice`, `ALICE`, and `alice`.

## Step through matches

After confirming your search:

| To move | Press |
|---|---|
| Next match | <kbd>n</kbd> or <kbd>Enter</kbd> |
| Previous match | <kbd>N</kbd> or <kbd>Shift</kbd>+<kbd>Enter</kbd> |

Navigation wraps around — pressing <kbd>n</kbd> on the last match jumps back to the first.

## Refine your search

To change the search term after confirming, press <kbd>Tab</kbd>, <kbd>Ctrl</kbd>+<kbd>F</kbd>, or <kbd>/</kbd> to return to editing mode. Type a new term and press <kbd>Enter</kbd> to confirm again.

## Navigate to a match's value

Once you've found the match you want, you can filter the output down to just that value:

1. Step to the match with <kbd>n</kbd> / <kbd>N</kbd>.
2. Press <kbd>&gt;</kbd> to zoom in — jiq rewrites your query to show only that piece of data.

This closes the search bar. Press <kbd>&lt;</kbd> afterward to return to where you were. (See [Results pane](./results-pane) for more on navigating into nested values.)

## Close search

Press <kbd>Esc</kbd> to close the search bar and clear all highlights.

---

## All keys

| Key | Action |
|---|---|
| `Ctrl+F` | Open search |
| `/` | Open search (when output area is active) |
| Type | Filter matches in real time |
| `Enter` | Confirm and jump to next match |
| `n` / `Enter` | Next match |
| `N` / `Shift+Enter` | Previous match |
| `Tab` | Edit the search term again |
| `Ctrl+F` / `/` | Edit the search term again |
| `>` | Zoom into the matched value (rewrites query) |
| `*` `}` | Transform the matched row (see [Results pane](./results-pane)) |
| `]` `[` | Jump to next / previous sibling of the matched row |
| `Esc` | Close search |
