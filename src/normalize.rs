//! Provider payloads → FHIR R4 Observations (LOINC-coded).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Raw provider series point (Vital-style payload shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderPoint {
    pub date: NaiveDate,
    /// provider metric key: hr | hrv | sleep_hours | steps
    pub metric: String,
    pub value: f64,
}

/// LOINC assignments per provider metric.
pub fn loinc_for(metric: &str) -> Option<(&'static str, &'static str, &'static str)> {
    // (code, display, unit)
    match metric {
        "hr" => Some(("8867-4", "Heart rate", "bpm")),
        "hrv" => Some(("80404-7", "Heart rate variability", "ms")),
        "sleep_hours" => Some(("93831-6", "Sleep duration", "h")),
        "steps" => Some(("55423-8", "Number of steps", "steps")),
        _ => None,
    }
}

/// FHIR R4 Observation (the fields this app models — kept local to keep
/// this repo standalone; fhir-r4-explorer owns the full crate).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub resource_type: String,
    pub status: String,
    pub code: CodingWrap,
    pub effective_date_time: NaiveDate,
    pub value_quantity: Quantity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodingWrap {
    pub coding: [Coding; 1],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coding {
    pub system: String,
    pub code: String,
    pub display: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

/// Normalize one provider point to a FHIR observation; unknown metrics
/// are rejected (not silently mapped).
pub fn to_observation(point: &ProviderPoint) -> Result<Observation, NormalizeError> {
    let (code, display, unit) =
        loinc_for(&point.metric).ok_or_else(|| NormalizeError::UnknownMetric {
            metric: point.metric.clone(),
        })?;
    Ok(Observation {
        resource_type: "Observation".into(),
        status: "final".into(),
        code: CodingWrap {
            coding: [Coding {
                system: "http://loinc.org".into(),
                code: code.into(),
                display: display.into(),
            }],
        },
        effective_date_time: point.date,
        value_quantity: Quantity {
            value: point.value,
            unit: unit.into(),
        },
    })
}

#[derive(Debug, thiserror::Error)]
pub enum NormalizeError {
    #[error("unknown provider metric: {metric}")]
    UnknownMetric { metric: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_map_to_loinc() {
        let point = ProviderPoint {
            date: NaiveDate::from_ymd_opt(2026, 9, 1).expect("date"),
            metric: "hr".into(),
            value: 62.0,
        };
        let obs = to_observation(&point).expect("normalized");
        assert_eq!(obs.code.coding[0].code, "8867-4");
        assert_eq!(obs.value_quantity.unit, "bpm");
        assert_eq!(obs.status, "final");
    }

    #[test]
    fn unknown_metric_rejected() {
        let point = ProviderPoint {
            date: NaiveDate::from_ymd_opt(2026, 9, 1).expect("date"),
            metric: "mood".into(),
            value: 3.0,
        };
        let err = to_observation(&point).unwrap_err();
        assert!(err.to_string().contains("mood"));
    }
}
