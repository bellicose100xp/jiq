# jaq adoption survey & complaint analysis

Compiled 2026-05-29 by direct GitHub research (no sub-agents — directive
forbade them). Sources: github.com/01mf02/jaq, code-search across crates
embedding `jaq-core`, downstream issue trackers.

## 1. Adoption — selected applications using jaq today

| Application | Domain | jaq version pinned | Status |
|---|---|---|---|
| **nushell** | shell | (via `jaq-core` references in supply-chain configs) | embedded |
| **mirrord** (metalbear-co) | k8s dev tooling | jaq adapter crate present | active |
| **netdata** | observability | netflow-plugin crate | active |
| **redpanda** | streaming platform | rust transform-sdk example | example/SDK |
| **databend** | data warehouse | top-level Cargo dep | active |
| **ynqa/jnv** | TUI jq frontend (closest analog to jiq) | jaq-core 2.2.1, jaq-json 1.1.3, jaq-std 2.1.2 | **stuck on v2; v3 upgrade open as #115** |
| **dts** (martinohmann) | data-transform CLI | top-level dep | active |
| **slumber** | HTTP request runner | core crate dep | active |
| **eww** | wayland widgets | top-level dep | active |
| **ockam** | secure-channel toolkit | api crate dep | active |
| **grafbase-sdk** | edge GraphQL SDK | top-level dep | active |
| **dolma** (allenai) | LLM data prep | top-level dep | active |
| **openapi-tui** | TUI OpenAPI viewer | top-level dep | active |
| **healthscript** (rhombusgg) | HTTP healthcheck DSL | top-level dep | active |
| **Daft** (Eventual-Inc) | dataframes | json-functions crate | active |
| **scnr** (shindan-io) | scanner core | top-level dep | active |
| **Ripple** (rdkcentral) | platform middleware | core/main dep | active |
| **kaish**, **aq**, **yq** (rivet-dev), **jqc** (muleyuck), **rsigma** (timescale) | small CLIs | direct dep | active |
| **amazon-ion/ion-cli** | Amazon Ion CLI | adopted via PR #193 | "What an excellent library `jaq` is." (jobarr-amzn, AWS Ion team) |

That's 20+ live embedders, including teams at Amazon, Netdata, Databend,
Redpanda, MetalBear, AllenAI, MongoDB Labs, and the broader Rust CLI
ecosystem. Not a niche project.

## 2. Top complaints — by category

Surveyed jaq's own issue tracker plus cross-repo "jaq" issues.

### A. Breaking-change pain across major versions (top frequency)
- **#424 — `serde_json` interop dropped in v3.0.** Author removed
  `TryFrom<&jaq_json::Val> for serde_json::Value` in PR #381 because
  embedders were over-converting. Real downstream pain: jnv hasn't
  finished upgrading to v3 yet (#115 still open). For jiq specifically,
  this is the *exact boundary* we'd sit at — we'd have to write our
  own conversion or stay on v2. **Most material complaint for our use
  case.**
- v2→v3 took half a year of alpha/beta/gamma; downstream upgrade PRs
  are still landing months after the v3 release.

### B. Missing builtins / behavioral differences (medium frequency)
- `is_match` `=~` and `!~` operators (#425, open).
- `scan()` default `g` flag missing (#287, open).
- `halt_error/0` missing, `halt_error/1` differs (#236, open).
- Float and negative-number formatting differences (#232, open).
- Some `@base64d` newline handling (#282, open).
- Nested regex captures (#77, open).
- jq 1.7 features (`pick`, `abs`) — **closed (#112), shipped.**
- `--raw-output0` — **closed (#101), shipped.**
- jq 1.8 SQL builtins (`INDEX`/`IN`/`GROUP_BY`/`UNIQUE_BY`) — present
  in jaq-std (no open issues complaining of absence).

### C. Memory / streaming for huge data (low frequency)
- #276 "How to handle data which does not fit into memory?" — open,
  philosophical. jaq materializes; jq streams.

### D. Error message divergence
- Mentioned in passing across multiple issues but rarely the primary
  complaint.

## 3. Compatibility gap snapshot (jq 1.8 parity)

| Feature | Status |
|---|---|
| `pick/1` | shipped (#112 closed) |
| `abs/0` | shipped (#112 closed) |
| `--raw-output0` | shipped (#101 closed) |
| `INDEX`, `IN`, `GROUP_BY`, `UNIQUE_BY`, `MIN_BY`, `MAX_BY` (1.7/1.8) | shipped via jaq-std |
| `is_match`, `=~`, `!~` regex operators | **missing** (#425 open) |
| `scan()` default global flag | **differs** (#287 open) |
| `halt_error/0` | **missing** (#236 open) |
| Float/negative formatting edge cases | **differs** (#232 open) |
| `@base64d` newline handling | **differs** (#282 open) |
| Streaming over data > RAM | **not supported** (#276 open) |

## 4. Maturity verdict

- **First release:** December 2020. ~5.5 years old.
- **Stars:** 3,618. **Forks:** 114.
- **Commit activity (last 52 weeks):** 753 commits.
- **Last push:** 2026-05-26 (this week).
- **Releases:** v2.0 (Nov 2024), v2.1 (Jan 2025), v2.2 (Apr 2025),
  v2.3 (Jul 2025), v3.0-alpha (Nov 2025), v3.0-gamma (Mar 2026), v3.0
  (Mar 2026). About a release every 2-3 months.
- **Bus factor:** Michael Färber wrote 2030 commits; the next
  contributor wrote 237 (kammerchorinnsbruck), then 37 (kklingenberg),
  then 28, 22, 13, 12 etc. **Effectively single-maintainer**, but
  with active outside contributions and an engaged issue tracker.
- Major releases ship breaking API changes that take months to
  propagate downstream (jnv example).

**Verdict:** active, mature, but single-maintainer with frequent
breaking changes between majors. Not abandoned. Adoption is broad
enough that a regression would hit dozens of downstream projects.

## 5. Recommendation for jiq

**Yes — ship pluggable jaq engine, with caveats.** The adoption picture
is solid, the project is alive, and the perf upside (eliminating the
ANSI roundtrip and the jq subprocess) is exactly what the matrix says
costs us most.

**Caveats to bake into the docs and the implementation:**

1. **Pin to a v2 line for first release.** v3.0 dropped serde_json
   interop and downstream is still untangling that. Pinning v2.x
   (jaq-core 2.x, jaq-json 1.x, jaq-std 2.x — same as jnv ships today)
   gets us a stable base with mature serde_json support. Plan a
   separate v3 migration once the interop story stabilizes.

2. **Document known divergences in `--engine jaq`'s help text.** At
   minimum: `=~`/`!~` regex operators, `scan()` `g` flag, `halt_error/0`,
   numeric formatting edge cases, `@base64d` newlines. Users who hit
   any of these get a "switch back to default jq engine" hint.

3. **Don't make jaq the default.** Compatibility surface is real;
   default stays on jq subprocess. `--engine jaq` (or
   `query.engine = "jaq"`) is opt-in for users who want the speed and
   accept the boundary.

4. **Streaming caveat.** jaq materializes the full input. For users
   piping multi-GB JSON, default jq streams; jaq doesn't. Document
   this as a hard limit.

5. **Plan for breakage on jaq major upgrades.** Track the v2 → v3
   migration cost based on jnv's experience (still in progress months
   after release). Budget at least a week of compat work per major.

The ergonomics of integration are the real cost — not the perf or the
correctness. The jq compat surface jaq doesn't quite cover is small
enough that "warn and let user opt out" is a viable strategy. Shipping
this is defensible.
