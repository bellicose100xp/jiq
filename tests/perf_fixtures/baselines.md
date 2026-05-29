# Performance Baselines

Snapshot of perf-matrix measurements for jiq. Updated whenever a perf
fix lands and we re-run the matrix. Comparing future runs against these
numbers tells us whether a fix actually helped.

This file holds *measurements*. The README in this directory holds the
*system docs*. Don't mix them.

## Methodology

- Hardware: Apple Silicon (M-series).
- Build: `cargo build --release`.
- Matrix: 3 shapes × 3 sizes × 3 runs per cell = 27 cells.
- Driver: `tests/perf_scripts/<script>.script` replayed via
  `--bench-script <path>` inside `script(1)` (PTY required for raw mode).
- Bench-script runner enters drain mode after the last directive; only
  quits when `has_pending_query()` is false (cap 30s) so worker-thread
  preprocessing timers complete before the perf summary dumps.
- Each cell run: timeout 30s for small/medium, 90s for large.

## Captured baseline (2026-05-29, branch `perf`)

All 27 cells reported all 11 instrumented timers cleanly. Numbers below
are p50 / p95 from run 1 of 3 in each cell (runs were within noise). Full
data lives at `/home/chahcha/.claude/jobs/f1848398/perf_matrix_clean.txt`.

### wide × small (10k lines, 224 KB)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 10.07 ms | 10.08 ms |
| jq_spawn                        | 636 µs   | 651 µs   |
| ansi_into_text                  | 1.39 ms  | 1.43 ms  |
| parse_ansi_to_rendered_lines    | 1.49 ms  | 1.54 ms  |
| parse_and_detect_type           | 308 µs   | 313 µs   |
| strip_ansi_codes                | 143 µs   | 147 µs   |
| compute_line_metrics            | 68 µs    | 69 µs    |
| json_input_parsed_first_touch   | 1.36 ms  | 1.36 ms  |
| all_field_names_first_touch     | 9.7 µs   | 9.7 µs   |
| search_update                   | 135 µs   | 212 µs   |
| inject_key_event                | 3.8 µs   | 297 µs   |
| path_at_line                    | 10.5 µs  | 34.8 µs  |

### wide × medium (100k lines, 2.3 MB)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 50.30 ms | 100.61 ms|
| jq_spawn                        | 641 µs   | 644 µs   |
| ansi_into_text                  | 1.39 ms  | —        |
| parse_ansi_to_rendered_lines    | 1.49 ms  | —        |
| parse_and_detect_type           | 296 µs   | —        |
| strip_ansi_codes                | 150 µs   | —        |
| compute_line_metrics            | 74 µs    | —        |
| json_input_parsed_first_touch   | 11.94 ms | —        |
| inject_key_event                | 4.0 µs   | 11.97 ms |

### wide × large (1M lines, 23 MB)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 412 ms   | 895 ms   |
| jq_spawn                        | 639 µs   | 678 µs   |
| ansi_into_text                  | 1.38 ms  | 1.38 ms  |
| parse_ansi_to_rendered_lines    | 1.48 ms  | 1.48 ms  |
| parse_and_detect_type           | 296 µs   | 305 µs   |
| strip_ansi_codes                | 138 µs   | 145 µs   |
| compute_line_metrics            | 68 µs    | 69 µs    |
| json_input_parsed_first_touch   | 124.95 ms| 124.95 ms|
| inject_key_event                | 3.85 µs  | **125 ms p99** |
| path_at_line                    | 8.9 µs   | 12.2 µs  |

### deep × small (10k lines, 600 KB nested)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 20.13 ms | 20.13 ms |
| ansi_into_text                  | 10.54 ms | 10.54 ms |
| parse_ansi_to_rendered_lines    | 11.57 ms | 11.57 ms |
| parse_and_detect_type           | 1.91 ms  | —        |
| strip_ansi_codes                | 1.13 ms  | —        |
| compute_line_metrics            | 814 µs   | —        |
| json_input_parsed_first_touch   | 1.15 ms  | —        |
| all_field_names_first_touch     | 316 µs   | —        |
| inject_key_event                | 160 ns   | 1.48 ms p99 |

### deep × medium (100k lines, 4.4 MB nested)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 110.65 ms| 110.66 ms|
| ansi_into_text                  | 65.34 ms | 65.34 ms |
| parse_ansi_to_rendered_lines    | 71.72 ms | 71.72 ms |
| parse_and_detect_type           | 12.06 ms | —        |
| strip_ansi_codes                | 7.26 ms  | —        |
| compute_line_metrics            | 5.15 ms  | —        |
| json_input_parsed_first_touch   | 7.23 ms  | —        |
| all_field_names_first_touch     | 2.25 ms  | —        |
| inject_key_event                | 142 ns   | 9.49 ms p99 |

### deep × large (1M lines, 32 MB nested)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 412 ms   | 694 ms   |
| ansi_into_text                  | **113 ms** | 116 ms |
| parse_ansi_to_rendered_lines    | **123 ms** | 127 ms |
| parse_and_detect_type           | 22.5 ms  | 23.2 ms  |
| strip_ansi_codes                | 11.78 ms | 12.45 ms |
| compute_line_metrics            | 8.93 ms  | 9.35 ms  |
| json_input_parsed_first_touch   | 48.40 ms | —        |
| all_field_names_first_touch     | 18.54 ms | —        |
| inject_key_event                | 1.40 µs  | **67 ms p99** |

### keys × small (1k fields, 55 KB)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 10.07 ms | 10.08 ms |
| ansi_into_text                  | 2.91 µs  | —        |
| parse_ansi_to_rendered_lines    | 3.25 µs  | —        |
| parse_and_detect_type           | 1.39 µs  | —        |
| strip_ansi_codes                | 711 ns   | —        |
| inject_key_event                | 127 µs   | 178 µs   |
| all_field_names_first_touch     | 154 µs   | —        |
| json_input_parsed_first_touch   | 421 µs   | —        |

### keys × medium (10k fields, 554 KB)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 20.13 ms | 30.19 ms |
| ansi_into_text                  | 2.81 µs  | —        |
| parse_ansi_to_rendered_lines    | 3.17 µs  | —        |
| parse_and_detect_type           | 1.24 µs  | —        |
| inject_key_event                | 1.26 ms  | 1.77 ms  |
| all_field_names_first_touch     | 1.39 ms  | —        |
| json_input_parsed_first_touch   | 3.47 ms  | —        |

### keys × large (100k fields, 5.6 MB)

| Operation                       | p50      | p95      |
|---------------------------------|----------|----------|
| jq_wait                         | 241 ms   | 251 ms   |
| ansi_into_text                  | 3 µs     | **297 ms** |
| parse_ansi_to_rendered_lines    | 3.5 µs   | **329 ms** |
| parse_and_detect_type           | 1.3 µs   | 61 ms    |
| strip_ansi_codes                | 712 ns   | 28 ms    |
| compute_line_metrics            | 891 ns   | 7.95 ms  |
| inject_key_event                | **13 ms**| 20 ms / **77 ms p99** |
| all_field_names_first_touch     | 23.33 ms | —        |
| json_input_parsed_first_touch   | 40.37 ms | —        |

> The bimodal p50 vs p95 on `keys_large` is because two queries fired in
> the cell — one tiny (autocomplete cache hit), one large (preprocessing
> the whole field set). p95 captures the heavy one.

## Cross-shape summary

The bottleneck is shape-dependent, not absolute:

- **wide arrays**: jq dominates (~895 ms p95 large) by 3+ orders of
  magnitude over post-jq processing.
- **deep trees**: post-jq pipeline matters. ANSI parsing alone is ~250 ms
  on a 1M-line nested tree, vs jq's 700 ms — comparable cost.
- **many-key objects**: per-keystroke autocomplete dominates the
  user-perceived experience. p99 of 77 ms per keystroke on 100k fields is
  user-visible lag.

There is no single "the bottleneck" to fix.

## How to refresh

Re-run the matrix and overwrite this file:

```sh
/home/chahcha/.claude/jobs/f1848398/run_full_matrix.sh
# wait ~22 minutes
# then transcribe new numbers into this file
```

Update the date and branch line at the top.
