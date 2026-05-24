---
title: VIM editing
parent: Features
nav_order: 9
description: VIM motions, operators, text objects, undo/redo in the query input.
---

# VIM editing

The query input has two modes: **INSERT** (default, cyan border) and **NORMAL** (yellow border).

| Key | Effect |
|---|---|
| <kbd>Esc</kbd> | INSERT → NORMAL |
| <kbd>i</kbd> | NORMAL → INSERT at cursor |
| <kbd>a</kbd> | NORMAL → INSERT after cursor |
| <kbd>I</kbd> | NORMAL → INSERT at line start |
| <kbd>A</kbd> | NORMAL → INSERT at line end |
{: .shortcuts }

## Motion

| Key | Move to |
|---|---|
| <kbd>h</kbd> <kbd>l</kbd> | One char left / right |
| <kbd>0</kbd> <kbd>^</kbd> <kbd>Home</kbd> | Line start |
| <kbd>$</kbd> <kbd>End</kbd> | Line end |
| <kbd>w</kbd> | Next word start |
| <kbd>b</kbd> | Previous word start |
| <kbd>e</kbd> | Word end |
{: .shortcuts }

## Character search

| Key | Effect |
|---|---|
| <kbd>f</kbd>{c} | Forward to next `{c}` |
| <kbd>F</kbd>{c} | Backward to previous `{c}` |
| <kbd>t</kbd>{c} | Forward to char before `{c}` |
| <kbd>T</kbd>{c} | Backward to char after `{c}` |
| <kbd>;</kbd> | Repeat last search |
| <kbd>,</kbd> | Repeat in opposite direction |
{: .shortcuts }

## Edit

| Key | Effect |
|---|---|
| <kbd>x</kbd> | Delete char at cursor |
| <kbd>X</kbd> | Delete char before cursor |
| <kbd>D</kbd> | Delete to line end |
| <kbd>C</kbd> | Change to line end (delete + INSERT) |
| <kbd>u</kbd> | Undo |
| <kbd>Ctrl</kbd>+<kbd>r</kbd> | Redo |
| <kbd>yy</kbd> | Yank line to clipboard |
{: .shortcuts }

## Operator + motion

<kbd>d</kbd> deletes, <kbd>c</kbd> changes (delete + INSERT). Both accept any motion or character search:

| Key | Effect |
|---|---|
| <kbd>d</kbd><kbd>w</kbd> <kbd>d</kbd><kbd>b</kbd> <kbd>d</kbd><kbd>e</kbd> | Delete word forward / back / end |
| <kbd>d</kbd><kbd>$</kbd> <kbd>d</kbd><kbd>0</kbd> <kbd>d</kbd><kbd>^</kbd> | Delete to line end / start |
| <kbd>d</kbd><kbd>d</kbd> | Delete entire line |
| <kbd>d</kbd><kbd>f</kbd>{c} <kbd>d</kbd><kbd>t</kbd>{c} | Delete to / till char (also <kbd>F</kbd> <kbd>T</kbd>) |
| <kbd>c</kbd><kbd>w</kbd> <kbd>c</kbd><kbd>$</kbd> <kbd>c</kbd><kbd>c</kbd> | Change variants of the above |
{: .shortcuts }

## Text objects

<kbd>d</kbd><kbd>i</kbd>{t} deletes inside, <kbd>d</kbd><kbd>a</kbd>{t} deletes around (including delimiters). <kbd>c</kbd><kbd>i</kbd> / <kbd>c</kbd><kbd>a</kbd> are the change variants.

| Target | Selects |
|---|---|
| <kbd>w</kbd> | Word |
| <kbd>"</kbd> <kbd>'</kbd> <kbd>`</kbd> | Inside matching quote |
| <kbd>(</kbd> <kbd>)</kbd> <kbd>b</kbd> | Inside parentheses |
| <kbd>[</kbd> <kbd>]</kbd> | Inside brackets |
| <kbd>{</kbd> <kbd>}</kbd> <kbd>B</kbd> | Inside braces |
| <kbd>\|</kbd> | **Pipe segment** (jq-aware) |
{: .shortcuts }

The pipe-segment object is unique to jiq. It treats `|` as a separator and acts on the segment under the cursor:

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

`da|` also removes one adjacent pipe so the segment is gone cleanly:

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

## NORMAL-mode shortcuts

| Key | Effect |
|---|---|
| <kbd>/</kbd> | Open results search |
| <kbd>?</kbd> | Open help popup |
{: .shortcuts }

Every edit re-runs the query immediately.
