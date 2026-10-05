//! Provider enum: `Fixture` (committed JSON, offline) or `Vital` (the
//! real aggregation API when configured). Same data shape either way —
//! the Vital pattern: one client, many wearables behind it.

use std::path::PathBuf;

use chrono::NaiveDate;
use serde::Deserialize;

use crate::normalize::ProviderPoint;

#[derive(Debug, Clone)]
pub enum Provider {
    /// Committed fixture JSON (synthetic Oura-ish series).
    Fixture { path: PathBuf },
    /// Vital aggregation API (sandbox + API key when configured).
    Vital {
        base_url: String,
        api_key: String,
        user_id: String,
    },
}

#[derive(Debug, Deserialize)]
struct ProviderResponse {
    #[serde(default)]
    points: Vec<RawPoint>,
}

#[derive(Debug, Deserialize)]
struct RawPoint {
    date: NaiveDate,
    metric: String,
    value: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("io error: {0}")]
    Io(String),
    #[error("bad provider response: {0}")]
    BadResponse(String),
}

impl Provider {
    /// Pull a date range of series points for the connected user.
    pub async fn fetch(
        &self,
        http: &reqwest::Client,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ProviderPoint>, ProviderError> {
        match self {
            Provider::Fixture { path } => {
                let raw = std::fs::read_to_string(path)
                    .map_err(|e| ProviderError::Io(format!("{}: {e}", path.display())))?;
                let response: ProviderResponse = serde_json::from_str(&raw)
                    .map_err(|e| ProviderError::BadResponse(e.to_string()))?;
                Ok(response
                    .points
                    .into_iter()
                    .filter(|p| p.date >= start && p.date <= end)
                    .map(|p| ProviderPoint {
                        date: p.date,
                        metric: p.metric,
                        value: p.value,
                    })
                    .collect())
            }
            Provider::Vital {
                base_url,
                api_key,
                user_id,
            } => {
                // Vital timeseries shape: /v2/timeseries/{user_id}/{metric}/grouped
                // — pulled per metric; scaffold pulls hr + hrv.
                let mut points = Vec::new();
                for metric in ["hr", "hrv"] {
                    let response: serde_json::Value = http
                        .get(format!(
                            "{base_url}/v2/timeseries/{user_id}/{metric}/grouped"
                        ))
                        .query(&[
                            ("start_date", start.to_string()),
                            ("end_date", end.to_string()),
                        ])
                        .header("Authorization", format!("Bearer {api_key}"))
                        .send()
                        .await
                        .map_err(|e| ProviderError::BadResponse(e.to_string()))?
                        .error_for_status()
                        .map_err(|e| ProviderError::BadResponse(e.to_string()))?
                        .json()
                        .await
                        .map_err(|e| ProviderError::BadResponse(e.to_string()))?;
                    if let Some(list) = response["data"].as_array() {
                        for item in list {
                            if let (Some(date), Some(value)) = (
                                item["date"]
                                    .as_str()
                                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
                                item["value"].as_f64().or_else(|| item["avg"].as_f64()),
                            ) {
                                points.push(ProviderPoint {
                                    date,
                                    metric: metric.to_string(),
                                    value,
                                });
                            }
                        }
                    }
                }
                Ok(points)
            }
        }
    }
}
