# Development Guide

Machine-facing commands live in [AGENTS.md](../AGENTS.md).

## Prerequisites

Rust 1.96, cargo. Nothing else for the fixture provider (tests and demo
run offline); Vital sandbox credentials only for the real backend.

## Daily loop

```bash
cargo run               # :8009 — fixture provider
cargo test -q          # 10 tests — no network
cargo clippy --all-targets -- -D warnings
```

## Vital mode

```bash
APP_VITAL_API_KEY=… APP_VITAL_USER_ID=… cargo run
```

## The fixture

`tests/fixtures/wearable_series.json` is generated **relative to today**
(30 days ending now) with a fixed seed — the sync-window tests depend on
the data landing inside the window. If the fixture ages out (someone runs
tests after the window passes), regenerate:

```bash
# deterministic: hr steady, hrv -0.5/day, sleep steady, steps noisy
# see tests/ingestion.rs for the expected shapes
```

The generator parameters are pinned in the test expectations (120
points, hrv falling, hr flat) — regenerate with the same seed or update
the tests together.

## Adding a metric

1. Add the LOINC mapping in `src/normalize.rs::loinc_for`
2. Extend the fixture generator with the new series
3. Update the unit test (metrics map to LOINC; unknown metrics rejected)

Never pass an unmapped metric through — the rejection path is the
feature.

## Gotchas learned here

- Stateful flow tests need ONE router instance (sync-then-read shares
  the store) — clone the router per request
- Port 8009

## Conventions

Conventional commits; hygiene hook strips AI attribution.
