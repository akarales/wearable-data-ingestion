# AGENTS.md

## Commands

```bash
cargo run                  # :8009 (fixture provider)
cargo test -q              # 10 tests (5 unit + 5 integration)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Environment

`APP_PORT` (8009) · `APP_VITAL_API_KEY` / `APP_VITAL_USER_ID` /
`APP_VITAL_BASE_URL` (unset → fixture provider)

## Conventions

- Conventional commits; hygiene hook strips AI attribution
- New metrics: add the LOINC mapping in `normalize.rs::loinc_for` +
  a test — never pass unknown metrics through
- The fixture (`tests/fixtures/wearable_series.json`) is regenerated
  deterministically relative to today for the 30-day window tests
- Deps ≥7 days old (BEST_PRACTICES/INDEX.md)
