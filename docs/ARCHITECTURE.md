# Architecture

## Modules

```
src/
├── provider.rs    # Provider enum: Fixture | Vital
├── normalize.rs   # ProviderPoint → FHIR R4 Observation (LOINC)
├── trends.rs      # split-half mean direction (pure)
├── routes.rs      # sync / observations / trends
└── state.rs       # AppState + per-user store
```

## Provider abstraction (src/provider.rs)

One enum, two backends, same `fetch(http, start, end)` contract:

- **Fixture** — committed synthetic JSON (30 days × 4 metrics; hr steady
  noise, hrv declining ~0.5/day, sleep steady, steps noisy). Tests and
  the offline demo run entirely against it.
- **Vital** — the real aggregation API shape: per-metric
  `/v2/timeseries/{user_id}/{metric}/grouped` with `start_date` /
  `end_date` and bearer auth. Scaffold pulls `hr` + `hrv`; more metrics
  are a loop change, not a redesign.

## FHIR normalization (src/normalize.rs)

| Provider metric | LOINC | Display | Unit |
|-----------------|-------|---------|------|
| `hr` | 8867-4 | Heart rate | bpm |
| `hrv` | 80404-7 | Heart rate variability | ms |
| `sleep_hours` | 93831-6 | Sleep duration | h |
| `steps` | 55423-8 | Number of steps | steps |

Unknown metrics return `UnknownMetric { metric }` — **rejected, never
silently mapped**. Observations are emitted with `status: final`,
`http://loinc.org` coding, and `effectiveDateTime`; the shapes mirror
the full fhir-models crate in app #3 (kept local so this repo stays
standalone — a documented tradeoff).

## Store (src/state.rs)

Per user, per LOINC code, chronological observation vectors
(RwLock-guarded). Sync appends; reads are sorted at insert time.

## Trends (src/trends.rs)

Split-half mean comparison: first half vs second half mean of each
series; < 5% relative change = Flat, otherwise Rising/Falling. Deliberately
simple — Phase 3 hands the series to the biomarker app's drift engine
(z-scores, EWMA, Theil–Sen) instead of duplicating it.

## Design decisions

| Decision | Why |
|----------|-----|
| Fixture provider in-repo | Full-pipeline tests with zero accounts; the Vital backend compiles against the same contract |
| Reject unknown metrics | Silent metric mapping is how data quality dies in health pipelines |
| Local FHIR structs | Standalone repo beats a cross-workspace path dependency at scaffold scale |
