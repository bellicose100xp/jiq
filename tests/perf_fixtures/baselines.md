# Performance Baselines

Metadata + analysis for the perf-matrix baseline. Raw per-cell numbers
live in [`baseline_matrix.txt`](baseline_matrix.txt) — that's the source
of truth, parseable by `tests/perf_fixtures/run_matrix.sh`. This file
holds *what's not in the matrix*: when it was captured, on what code,
how to interpret it, and what we learned from it.

## Current baseline

```yaml
captured_at: 2026-05-29
branch: perf
commit: b733125
hardware: Apple Silicon (M-series)
build: cargo build --release
matrix_file: tests/perf_fixtures/baseline_matrix.txt
runner: tests/perf_fixtures/run_matrix.sh
```

## Methodology

```yaml
shapes: [wide, deep, keys]
sizes: [small, medium, large]
runs_per_cell: 3
total_cells: 27
driver:
  bench_scripts:
    wide: tests/perf_scripts/typical.script
    deep: tests/perf_scripts/deep_drilling.script
    keys: tests/perf_scripts/autocomplete_burst.script
  pty_wrapper: script(1)
cell_completion: poll for "PERF SUMMARY" in /tmp/jiq-debug.log
cell_timeout_caps:
  small: 30s
  medium: 30s
  large: 120s
streaming_metric: inject_key_event p99
delta_thresholds:
  win: <= -20%
  regression: >= +20%
  flat: anywhere in between
```

`run_matrix.sh` always finishes all 27 cells; thresholds are
informational only. No early exit on regression.

## Headline metrics (run 1, from baseline_matrix.txt)

```yaml
wide_large:
  inject_key_event_p99_ms: 124.99
  jq_wait_p95_ms: 895
deep_large:
  inject_key_event_p99_ms: 66.96
  jq_wait_p95_ms: 694
  ansi_into_text_p50_ms: 113
  parse_ansi_to_rendered_lines_p50_ms: 123
keys_large:
  inject_key_event_p99_ms: 77.59
  inject_key_event_p50_ms: 13
```

## Cross-shape analysis

Bottleneck is shape-dependent, not absolute:

- **wide arrays** — `jq_wait` dominates (~895 ms p95 on `wide_large`).
  Post-jq pipeline cost is 3+ orders of magnitude smaller. Engine swap
  or query-side caching is the only lever.
- **deep trees** — post-jq pipeline matters. ANSI parsing alone is
  ~250 ms on a 1M-line nested tree (`ansi_into_text` + `parse_ansi_to_rendered_lines`),
  vs `jq_wait` 700 ms. Lazy viewport-only parsing is the lever.
- **many-key objects** — per-keystroke autocomplete dominates user-felt
  latency. `keys_large` `inject_key_event` p99 = 77 ms is user-visible
  typing lag. Substring autocomplete index is the lever.

No single "the bottleneck" to fix.

## jaq port — measured, parked

```yaml
status: parked indefinitely
branch: perf-jaq
commit: ad48fff
results_vs_jq_baseline:
  wide_large_total_query_p95: 1.2x faster (895ms -> ~750ms)
  deep_large_total_query_p95: 2.4x faster (694ms -> ~293ms)
  keys_large_total_query_p95: parity (~257ms)
  keys_large_inject_key_event_p99: parity (77.7ms)
decision_rationale: |
  Numbers don't justify replacing a working engine. The user-felt
  typing-lag bottleneck (keys_large p99) is engine-agnostic; jaq doesn't
  fix it. See "jiq — Perf Findings & jaq Decision" in obsidian for the
  full reasoning and the parsed-`Val` caching optimization that would
  need to land before revisiting.
```

## Refreshing the baseline

When a real perf win lands and we want subsequent runs to compare
against the new floor:

```sh
cargo build --release
tests/perf_fixtures/run_matrix.sh /dev/null    # capture without comparison
cp tests/perf_fixtures/matrix_latest.txt tests/perf_fixtures/baseline_matrix.txt
```

Then update the `captured_at`, `branch`, `commit`, and headline metrics
blocks above. Don't transcribe per-cell tables — `baseline_matrix.txt`
is the source of truth.
