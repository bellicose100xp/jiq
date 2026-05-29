# Performance Improvement Plan

Selected from the 10-researcher synthesis. Verified against the 27-cell
matrix in `baselines.md`. Each entry is independently shippable.

Reorder freely as priorities shift. Tick boxes when done. After landing
each fix, re-run the matrix and update `baselines.md` with the new
numbers so the impact is recorded.

## Selected ideas

### 1. [ ] Pre-warm OnceLock caches at startup

- **Where:** `src/query/worker/thread.rs::worker_loop`
- **What:** Immediately call `executor.json_input_parsed()` and
  `executor.all_field_names()` on a background thread when the worker
  starts, so they're warm before the user types.
- **Targets:** first-keystroke spike of 125 ms (wide_large) / 48 ms
  (deep_large) / 40 ms (keys_large)
- **Effort:** small (~10 lines)
- **Risk:** near-zero (OnceLock is already thread-safe)
- **Workloads:** all
- **Consensus:** 10 of 10 researchers

### 2. [ ] Substring-fast field-name index for autocomplete

- **Where:** new `src/autocomplete/field_index.rs`; hook into
  `src/query/executor.rs` next to `all_field_names`; rewire
  `get_all_field_suggestions` in `src/autocomplete/context.rs`.
- **What:** Build at first-touch a single pre-lowercased byte haystack
  of all field names separated by `\x01` (or `\xFF`) sentinels, plus a
  `Vec<u32>` of name-start offsets and a parallel `Vec<Arc<str>>` of
  original-cased names. Per keystroke: lowercase partial once, run
  `memchr::memmem::Finder::find_iter` with early termination at K=20-64
  hits, map offsets back to names via binary search, dedupe.
- **Targets:** keys_large `inject_key_event` p50 13 ms → ~0.5 ms;
  p99 77 ms → ~2-5 ms. The only user-perceptible typing lag in the matrix.
- **Preserves:** substring + case-insensitive semantics exactly.
- **Effort:** small-medium (~150 lines new + tests)
- **Risk:** low — sentinel collision avoided by `\xFF` (invalid UTF-8 start
  byte); Unicode case-folding behavior matches current `to_lowercase()`.
- **Memory:** ~5 MB extra; no new deps (memchr already in tree).
- **Workloads:** keys (huge), all (small win)
- **Consensus:** 10 of 10 researchers
- **Detailed design:** synthesized from a 10-researcher batch — see
  `substring_autocomplete_research.md` (when fork completes).

### 3. [ ] Skip `prepare_json_for_context` when AI is disabled

- **Where:** `src/query/query_state.rs::process_response` (near the
  `last_successful_result_for_context` build).
- **What:** Currently every successful query runs `prepare_json_for_context`
  (minify + truncate the JSON for the AI sidebar) regardless of whether
  AI is configured. For users without AI, this is pure waste on the hot
  path between "worker delivered result" and "main thread renders."
  Gate the call on `app.ai.configured`. Build lazily on first AI request
  if the user toggles AI on mid-session.
- **Targets:** estimated 5-30 ms per query saved on deep_large for users
  without AI. Not separately timed in baselines so impact must be
  measured directly.
- **Effort:** trivial (~5-10 lines).
- **Risk:** trivial — handle the AI-toggled-on-mid-session case via an
  `OnceLock` or similar.
- **Workloads:** all (most impactful on deep_large).
- **Consensus:** 4 of 10 researchers.

## Maybe pile (need to perf-test before committing)

### M0. Pluggable jaq engine (opt-in alternative to jq subprocess)

- **Where:** new `JqEngine` trait in `src/query/`; current jq subprocess
  becomes one impl; jaq becomes a second impl. CLI flag `--engine jaq`
  (or config `query.engine = "jaq"`). Default stays on jq.
- **What:** jaq is a Rust library that evaluates jq queries in-process
  and returns `serde_json::Value` directly. Switching to it eliminates:
  the jq subprocess + spawn cost, the entire ANSI roundtrip
  (`strip_ansi_codes`, `ansi_into_text`, `parse_ansi_to_rendered_lines`),
  and the redundant `parse_and_detect_type` (we already have the Value).
  jiq formats and colors the Value itself, emitting ratatui spans
  directly with no parsing roundtrip.
- **Pretty-printing:** follow jnv's pattern — use
  `serde_json::to_string_pretty` (with `preserve_order`), accept that
  output formatting is "very close to jq but not byte-identical."
  Number-formatting and Unicode-escaping edges differ slightly.
  Acceptable because **jiq is an interactive TUI, not a piping tool** —
  users who need byte-identical jq output use the default jq engine.
- **Coloring:** preserve our current brighter, bolder palette exactly.
  Today the palette is set in `src/query/executor.rs::JQ_COLORS`
  (truecolor RGB + bold on arrays/objects/keys). With jaq we walk the
  Value tree and emit the same RGB values via
  `Style::default().fg(Color::Rgb(r, g, b)).add_modifier(Modifier::BOLD)`.
  Same colors, expressed in ratatui's API instead of ANSI codes. Search
  highlight overlay continues to work unchanged on the resulting spans.
- **Targets:** every shape, big.
  - wide_large: jq_wait 412-895 ms → jaq probably 200-500 ms (50-70%
    of jq for typical queries per benchmarks).
  - deep_large: 250 ms post-jq pipeline collapses to ~30-50 ms direct
    Value-to-spans walk.
  - keys_large: jq_wait 251 ms similarly halves; preprocessing tail
    (300+ ms p95) eliminated.
  - ANSI parsing problem disappears — no need for M1.
- **Effort:** ~3-4 days (revised down from "1 week" once we decided we
  don't need a jq-compatible pretty-printer). Engine trait + jaq
  adapter + walk-Value-to-spans + integration tests.
- **Pin to current jaq v3 line** (Cargo dep `jaq = "3"`). Latest major
  version, where new bug fixes and features land. Note v3 dropped
  direct `serde_json::Value` interop (#424), so the adapter needs a
  small bridge between jaq's native value type and `serde_json::Value`
  for jiq's autocomplete / path-at-cursor / AI-context consumers.
  ~50-100 lines, well-contained.
- **Documented compat caveats** for `--engine jaq`:
  - Missing/divergent builtins: `=~`/`!~` regex (#425), `scan()` `g`
    flag (#287), `halt_error/0` (#236), float-formatting edges (#232),
    `@base64d` newlines (#282).
  - Streaming/memory: jq streams results, jaq materializes. Hard cap
    on multi-GB pipelines.
  - Output formatting: subtle differences from jq (whitespace, number
    formatting, Unicode escaping). Fine for interactive use.
- **Risk:**
  - Single-maintainer upstream (Färber wrote 2030 of 2267 commits;
    next contributor 237). Major-version migrations can stall.
  - We own jaq bugs in this code path until upstream fixes them.
  - Builtin-coverage gap means a small percentage of queries that
    work in jq won't work in jaq.
- **Why pluggable, not default:** keeps the jq subprocess as the
  default — full compatibility, exactly today's behavior. Users who
  want max speed and can live with jaq's documented boundary opt in
  via `--engine jaq`.
- **Adoption signal:** 20+ embedders including Amazon Ion CLI, Netdata,
  Databend, Redpanda, MetalBear, AllenAI, MongoDB Labs, jnv. 753 commits
  in past year. Active issue tracker.
- **Why still "maybe":** real change with ongoing dual-maintenance.
  Worth a prototype against our fixtures to confirm the perf-win
  numbers before committing.
- **Detailed adoption + complaint research:**
  `tests/perf_fixtures/jaq_adoption_research.md`

### M1. Lazy viewport-only ANSI parsing

- **Where:** `src/query/worker/preprocess.rs::parse_ansi_to_rendered_lines`,
  `src/results/results_render.rs`.
- **What:** Today the worker eagerly parses the entire jq output into
  `Vec<RenderedLine>`, even though the renderer only ever shows ~50
  lines. Keep the raw `Arc<String>` output plus a cheap `Vec<u32>` of
  newline byte offsets (built via `memchr(b'\n')`). Parse ANSI on-demand
  only for the visible viewport at render time. Search/highlight code
  needs to be adapted to look up by byte ranges instead of pre-parsed
  spans.
- **Targets:** deep_large `ansi_into_text` (113 ms p50) +
  `parse_ansi_to_rendered_lines` (123 ms p50) collapse to ~5 ms
  per-frame viewport parse. keys_large p95 (~300 ms) similarly drops.
- **Why this over the SGR-parser-rewrite:** keeps `ansi-to-tui` as the
  single robust parser everywhere — no risk of misrendering jq's future
  output or pre-colored input. We make jiq faster by parsing **less**,
  not parsing **faster**.
- **Effort:** medium-large — touches preprocess, render, search,
  cursor-highlight code paths.
- **Risks:** search/highlighting and copy-line currently assume a
  fully-parsed `RenderedLine`. They'd need either a per-line memo or a
  byte-range adapter. Memory: ~8 MB extra for line-offset Vec on 1M
  lines (acceptable).
- **Why "maybe":** the savings only land if search/highlighting can be
  cleanly retrofitted; otherwise complexity grows fast. Need a small
  prototype to confirm the refactor is contained before committing.
- **Consensus:** 5 of 10 researchers (the ones who specifically warned
  against rewriting the SGR parser).

