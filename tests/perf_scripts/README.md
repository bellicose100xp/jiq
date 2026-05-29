# Performance Scripts

Deterministic input scripts replayed by `jiq --bench-script <path>` for
performance measurement. Each script simulates one common interaction
pattern so baseline-vs-fix comparisons are reproducible.

## Format

One directive per line. `#` starts a comment.

| Directive       | Effect                                                       |
|-----------------|--------------------------------------------------------------|
| `text <chars>`  | Type each char as a Char key event                           |
| `key <name>`    | Press a special key (see below)                              |
| `wait <ms>`     | Sleep N milliseconds before the next directive               |

Special key names: `enter`, `esc` / `escape`, `tab`, `backspace`, `space`,
`up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`,
`delete` / `del`, `f1` … `f12`, or any single character.

Modifier prefix: `ctrl-`, `alt-`, `shift-` (e.g. `ctrl-c`, `alt-enter`).

After the last directive runs, jiq triggers a clean shutdown so the perf
summary dumps to `/tmp/jiq-debug.log`.

## Running

Always against a release build of jiq:

```sh
cargo build --release
./target/release/jiq --debug \
    --bench-script tests/perf_scripts/typical.script \
    < tests/perf_fixtures/wide_medium.json
tail -50 /tmp/jiq-debug.log
```

Run each script three times to average out machine noise (OS scheduling,
cache state, frequency scaling). Compare median values across runs.

## Scripts

- **typical.script** — realistic mixed session: type a query, scroll,
  search, navigate matches, exit. Covers debounce, jq, render, scroll,
  search in one pass.
- **heavy_search.script** — search-focused: type, search a common
  substring, walk through 50+ matches. Stresses the search update path
  on large content.
- **scroll_navigation.script** — scroll-only: load, page through the
  whole result with PageDown / Down. Stresses path-at-cursor and render.
- **deep_drilling.script** — path-at-cursor focused: cursor through a
  deeply nested structure with `[` / `]` drills. Use against `deep.json`.
- **autocomplete_burst.script** — autocomplete-focused: type query
  fragments that trigger frequent suggestion regeneration. Use against
  `keys.json`.

## Designing new scripts

- Use `wait` between actions that should fire the 150ms query debounce
  individually (e.g. `wait 200` before pressing Enter to commit a query).
- Without `wait`, characters typed back-to-back collapse into a single
  debounced execution — useful for measuring cancellation paths.
- Avoid mouse events; jiq's perf-relevant work is all keyboard-driven and
  the bench-script format intentionally has no mouse directive.
