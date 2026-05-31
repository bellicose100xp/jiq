---
title: Quick reference
nav_order: 3
description: One-page cheat sheet of every jiq keybind, grouped by mode and pane, with links to the deep-dive page for each feature.
---

# Quick reference

Every keybind in one place. Click any section header for the full guide.
{: .fs-5 .fw-300 }

## Global <span class="mode-indicator mode-indicator--global">ALL MODES</span>

| Key | Action |
|:---|:---|
| <kbd>F1</kbd> / <kbd>?</kbd> | Toggle help popup |
| <kbd>Shift</kbd>+<kbd>Tab</kbd> | Switch focus: input ↔ results |
| <kbd>Ctrl</kbd>+<kbd>Y</kbd> | Copy (focus-aware: query if input, results if results) |
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | Copy results (regardless of focus) |
| <kbd>Ctrl</kbd>+<kbd>W</kbd> | [Save result to file](./features/save) |
| <kbd>Ctrl</kbd>+<kbd>T</kbd> | Toggle [function tooltip](./features/tooltip) |
| <kbd>Ctrl</kbd>+<kbd>E</kbd> | Toggle [error overlay](./features/results-pane#decode-an-error) (plain-language jq errors) |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Toggle [AI assistant](./features/ai-assistant) |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | Open [snippets](./features/snippets) |
| <kbd>Ctrl</kbd>+<kbd>R</kbd> | Open [history popup](./features/history) |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Open [search in results](./features/search) |
| <kbd>Enter</kbd> | Exit and print filtered JSON |
| <kbd>Ctrl</kbd>+<kbd>Q</kbd> | Exit and print just the query string |
| <kbd>Ctrl</kbd>+<kbd>C</kbd> / <kbd>q</kbd> | Quit silently |

{: .shortcuts }

## [Input — INSERT mode](./features/vim-editing) <span class="mode-indicator mode-indicator--insert">INSERT</span>

Cyan border. Just type — every keystroke re-runs jq.

| Key | Action |
|:---|:---|
| Type chars | Edit query (real-time results) |
| <kbd>Tab</kbd> | Accept [autocomplete](./features/autocomplete) suggestion |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate autocomplete |
| <kbd>←</kbd> / <kbd>→</kbd> | Move cursor |
| <kbd>Home</kbd> / <kbd>End</kbd> | Line start / end |
| <kbd>Backspace</kbd> / <kbd>Delete</kbd> | Delete char |
| <kbd>Ctrl</kbd>+<kbd>P</kbd> / <kbd>Ctrl</kbd>+<kbd>N</kbd> | Cycle [history](./features/history) (older / newer) |
| <kbd>Ctrl</kbd>+<kbd>d</kbd> / <kbd>Ctrl</kbd>+<kbd>u</kbd> | Scroll results half page |
| <kbd>Esc</kbd> | NORMAL mode / close autocomplete |
| Mouse click | Position cursor |
| Mouse wheel | Horizontal scroll |
| Horizontal swipe | Horizontal pan (terminal-dependent) |

{: .shortcuts }

## [Input — NORMAL mode](./features/vim-editing) <span class="mode-indicator mode-indicator--normal">NORMAL</span>

Yellow border. Vim motions, operators, text objects, undo/redo. Toggle from INSERT with <kbd>Esc</kbd>.

### Navigation

| Key | Action |
|:---|:---|
| `h` `l` `←` `→` | Move 1 char |
| `0` `^` `Home` | Line start |
| `$` `End` | Line end |
| `w` | Next word start |
| `b` | Previous word start |
| `e` | Word end |

{: .shortcuts }

### Enter INSERT

| Key | Action |
|:---|:---|
| `i` | At cursor |
| `a` | After cursor |
| `I` | Line start |
| `A` | Line end |

{: .shortcuts }

### Edit

| Key | Action |
|:---|:---|
| `x` | Delete char at cursor |
| `X` | Delete char before |
| `u` | Undo |
| `Ctrl+r` | Redo |
| `yy` | Focus-aware copy |

{: .shortcuts }

### Character search

| Key | Action |
|:---|:---|
| `f{c}` | Find forward to char |
| `F{c}` | Find backward to char |
| `t{c}` | Till forward (stop before) |
| `T{c}` | Till backward (stop after) |
| `;` | Repeat in same direction |
| `,` | Repeat in opposite direction |

{: .shortcuts }

### Operators (delete + change)

| Key | Action |
|:---|:---|
| `dw` `db` `de` | Delete word fwd/back/end |
| `d$` `d0` `d^` | Delete to end / start |
| `dd` `D` | Delete line / to end |
| `df{c}` `dF{c}` `dt{c}` `dT{c}` | Delete to/till char |
| `cw` `cb` `ce` | Change word fwd/back/end |
| `c$` `c0` `c^` `cc` `C` | Change to end / start / line |
| `cf{c}` `cF{c}` `ct{c}` `cT{c}` | Change to/till char |

{: .shortcuts }

### Text objects

| Key | Action |
|:---|:---|
| `ciw` `diw` | Inner word |
| `ci"` `di"` `ci'` `di'` `ci\`` `di\`` | Inside quotes |
| `ci(` `di(` `ci[` `di[` `ci{` `di{` | Inside brackets |
| `ci\|` `di\|` | Inside pipe segment (jq-aware) |
| `ca"` `da"` etc. | Around quotes (incl. quotes) |
| `ca(` `da(` etc. | Around brackets (incl. brackets) |
| `ca\|` `da\|` | Around pipe segment (incl. one pipe) |

{: .shortcuts }

### Other

| Key | Action |
|:---|:---|
| `/` | Open [search](./features/search) |
| `Ctrl+d` `Ctrl+u` | Scroll results half page |

{: .shortcuts }

## [Results pane](./features/results-pane) <span class="mode-indicator mode-indicator--results">RESULTS</span>

Focus with <kbd>Shift</kbd>+<kbd>Tab</kbd> or click.

### Cursor

| Key | Action |
|:---|:---|
| `j` `k` `↑` `↓` | Move 1 line |
| `J` `K` | Move 10 lines |
| `Ctrl+d` `PgDn` | Half page down |
| `Ctrl+u` `PgUp` | Half page up |
| `g` `Home` | Top |
| `G` `End` | Bottom |

{: .shortcuts }

### [Query navigation](./features/results-pane)

| Key | Action |
|:---|:---|
| <kbd>&gt;</kbd> | Zoom into value at cursor |
| <kbd>&lt;</kbd> | Step back to prior query (or click the `[ < Back ]` badge on the results border) |
| <kbd>*</kbd> | Iterate nearest array (`[N]` → `[]`) |
| <kbd>^</kbd> | Step up one level |
| <kbd>}</kbd> | Wrap value as `{key}` object |
| <kbd>]</kbd> <kbd>[</kbd> | Jump to next / prev sibling (wraps) |

{: .shortcuts }

### Horizontal scroll

| Key | Action |
|:---|:---|
| `h` `l` `←` `→` | 1 column |
| `H` `L` | 10 columns |
| `0` | Left edge |
| `$` | Right edge |
| Horizontal swipe | Scroll left/right (terminal-dependent) |

{: .shortcuts }

### Visual line selection

| Key | Action |
|:---|:---|
| `v` `V` | Enter visual line mode |
| `j` `k` `↑` `↓` | Extend selection |
| `y` | Yank to clipboard |
| `Esc` `v` `V` | Exit |
| Click + drag | Select with mouse |

{: .shortcuts }

## [Search in results](./features/search) <span class="mode-indicator mode-indicator--search">SEARCH</span>

| Key | Action |
|:---|:---|
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Open from any pane |
| <kbd>/</kbd> | Open from results / NORMAL input |
| <kbd>Enter</kbd> | Confirm + jump to next |
| <kbd>n</kbd> / <kbd>Enter</kbd> | Next match |
| <kbd>N</kbd> / <kbd>Shift</kbd>+<kbd>Enter</kbd> | Previous match |
| <kbd>Tab</kbd> | Toggle search bar ↔ results |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> / <kbd>/</kbd> | Re-enter edit mode |
| <kbd>Esc</kbd> | Close |

{: .shortcuts }

Case-insensitive.

## [Query history](./features/history)

### Quick cycling (no popup)

| Key | Action |
|:---|:---|
| <kbd>Ctrl</kbd>+<kbd>P</kbd> | Previous (older) |
| <kbd>Ctrl</kbd>+<kbd>N</kbd> | Next (newer) |

{: .shortcuts }

### Popup

| Key | Action |
|:---|:---|
| <kbd>Ctrl</kbd>+<kbd>R</kbd> / <kbd>↑</kbd> (NORMAL) | Open |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate |
| Type chars | Fuzzy filter |
| <kbd>Enter</kbd> / <kbd>Tab</kbd> | Apply |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Delete selected |
| Click <kbd>✕</kbd> | Delete entry under mouse |
| <kbd>Esc</kbd> | Close |

{: .shortcuts }

## [Snippet library](./features/snippets)

### Browse mode

| Key | Action |
|:---|:---|
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | Open |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Navigate |
| Type chars | Fuzzy filter |
| <kbd>Enter</kbd> | Apply |
| <kbd>Ctrl</kbd>+<kbd>N</kbd> | New from current query |
| <kbd>Ctrl</kbd>+<kbd>E</kbd> | Edit selected |
| <kbd>Ctrl</kbd>+<kbd>R</kbd> | Replace selected's query with current input |
| <kbd>Ctrl</kbd>+<kbd>D</kbd> | Delete |
| <kbd>Esc</kbd> | Close |

{: .shortcuts }

### Create / edit mode

| Key | Action |
|:---|:---|
| <kbd>Tab</kbd> / <kbd>Shift</kbd>+<kbd>Tab</kbd> | Navigate fields |
| <kbd>Enter</kbd> | Save |
| <kbd>Esc</kbd> | Cancel |

{: .shortcuts }

## [AI assistant](./features/ai-assistant) <span class="mode-indicator mode-indicator--ai">AI</span>

| Key | Action |
|:---|:---|
| <kbd>Ctrl</kbd>+<kbd>A</kbd> | Toggle popup |
| <kbd>Alt</kbd>+<kbd>1</kbd>..<kbd>5</kbd> | Apply suggestion N |
| <kbd>Alt</kbd>+<kbd>↑</kbd> / <kbd>Alt</kbd>+<kbd>↓</kbd> | Navigate |
| <kbd>Alt</kbd>+<kbd>j</kbd> / <kbd>Alt</kbd>+<kbd>k</kbd> | Navigate (vim) |
| <kbd>Enter</kbd> | Apply selected |
| <kbd>Ctrl</kbd>+<kbd>A</kbd> / <kbd>Esc</kbd> | Close |

{: .shortcuts }

## [Mouse](./features/mouse)

| Gesture | Action |
|:---|:---|
| Click pane | Focus |
| Click + drag (results) | Multi-line visual selection |
| Double-click result row | Drill into value (same as <kbd>&gt;</kbd>) |
| Mouse wheel | Vertical scroll |
| Horizontal swipe (results / input) | Scroll left/right (terminal-dependent) |
| Click suggestion | Select |
| Double-click suggestion | Apply |
| Hover history row | Reveal `✕` delete button |
| Click help tab | Switch tab |
| Click scrollbar | Reposition / drag |

{: .shortcuts }

## [Source picker](./features/clipboard)

Shown on bare `jiq` launch (no flag, no piped stdin) when the clipboard contains a JSON object or array. Otherwise jiq drops straight to the paste editor.

| Key | Action |
|:---|:---|
| <kbd>↑</kbd> / <kbd>↓</kbd> / <kbd>Tab</kbd> | Toggle Clipboard ↔ Paste |
| <kbd>Enter</kbd> | Confirm highlighted source |
| <kbd>Esc</kbd> | Quit jiq |

{: .shortcuts }

## CLI flags

| Flag | Behavior |
|:---|:---|
| `jiq <file>` | Load from file |
| `cat <file> \| jiq` | Load from piped stdin |
| `jiq` | Smart picker (peeks clipboard at launch) |
| `jiq --clipboard` | Force clipboard auto-load, skip the picker |
| `jiq --paste` | Open paste editor immediately, no clipboard read |
| `jiq --debug` | Write debug log to `/tmp/jiq-debug.log` |

Combining `--clipboard` or `--paste` with piped stdin or a file argument exits non-zero with an "ambiguous input source" error.

## [Paste editor](./features/clipboard)

Full-screen text area with VIM editing. Opens via `--paste`, the picker's Paste option, or as a fallback when the clipboard isn't queryable.

| Key | Action |
|:---|:---|
| Paste (Cmd/Ctrl+Shift+V) | Insert JSON |
| <kbd>Enter</kbd> | Validate + load |
| <kbd>Ctrl</kbd>+<kbd>X</kbd> | Clear textarea |
| All NORMAL-mode VIM keys | Edit |
| <kbd>j</kbd> / <kbd>k</kbd> / <kbd>g</kbd> / <kbd>G</kbd> | Navigate lines |

{: .shortcuts }
