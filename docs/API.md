# API Reference

Base URL: `http://localhost:8009`.

## Health

```bash
curl localhost:8009/health
```

```json
{ "status": "ok", "version": "0.1.0" }
```

## Sync (provider → FHIR → store)

```bash
curl -X POST localhost:8009/api/v1/users/ada/sync \
  -H 'content-type: application/json' -d '{"days": 30}'
```

```json
{
  "user_id": "ada", "window_days": 30,
  "pulled": 120, "normalized": 120, "skipped_unknown_metrics": 0
}
```

## Observations (FHIR R4 out)

```bash
curl 'localhost:8009/api/v1/users/ada/observations?loinc=8867-4'
```

```json
{
  "user_id": "ada",
  "available_codes": ["55423-8", "80404-7", "8867-4", "93831-6"],
  "observations": [ {
    "resourceType": "Observation",
    "status": "final",
    "code": { "coding": [ { "system": "http://loinc.org",
              "code": "8867-4", "display": "Heart rate" } ] },
    "effectiveDateTime": "2026-09-06",
    "valueQuantity": { "value": 58.4, "unit": "bpm" }
  } ]
}
```

Omit `loinc` for all series merged chronologically.

## Trends

```bash
curl localhost:8009/api/v1/users/ada/trends
```

```json
{ "user_id": "ada", "trends": [
  { "loinc": "80404-7", "n": 30, "trend": "falling",
    "latest": 33.2, "unit": "ms" },
  { "loinc": "8867-4", "n": 30, "trend": "flat",
    "latest": 58.1, "unit": "bpm" } ] }
```

## Errors

| Status | Meaning |
|--------|---------|
| `404` | user has no data — run sync first |
| `502` | provider error (fixture missing / Vital unreachable) |
