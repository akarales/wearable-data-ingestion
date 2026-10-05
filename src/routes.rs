//! Routes: sync (provider → FHIR), series (FHIR out), trends.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;
use crate::normalize::to_observation;
use crate::trends;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/users/{user_id}/sync", post(sync))
        .route("/api/v1/users/{user_id}/observations", get(series))
        .route("/api/v1/users/{user_id}/trends", get(user_trends))
        .with_state(state)
}

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    /// Lookback window in days (default 30).
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 {
    30
}

/// Pull from the provider (fixture offline / Vital when configured),
/// normalize to FHIR R4 Observations, store per user.
pub async fn sync(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<SyncRequest>,
) -> Response {
    let days = request.days.clamp(1, 365) as i64;
    let end = Utc::now().date_naive();
    let start = end - chrono::Duration::days(days);

    let points = match state
        .provider
        .fetch(&reqwest::Client::new(), start, end)
        .await
    {
        Ok(points) => points,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("provider: {err}") })),
            )
                .into_response();
        }
    };

    let mut normalized = Vec::with_capacity(points.len());
    let mut skipped = 0usize;
    for point in &points {
        match to_observation(point) {
            Ok(observation) => normalized.push(observation),
            Err(_) => skipped += 1,
        }
    }
    let count = normalized.len();
    state.store.insert(&user_id, &normalized);

    (
        StatusCode::OK,
        Json(json!({
            "user_id": user_id,
            "window_days": days,
            "pulled": points.len(),
            "normalized": count,
            "skipped_unknown_metrics": skipped,
        })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct SeriesQuery {
    pub loinc: Option<String>,
}

pub async fn series(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<SeriesQuery>,
) -> Response {
    let observations = state.store.observations(&user_id, params.loinc.as_deref());
    if observations.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("no observations for {user_id} — run sync first") })),
        )
            .into_response();
    }
    let codes: Vec<String> = state.store.codes(&user_id);
    (
        StatusCode::OK,
        Json(json!({
            "user_id": user_id,
            "available_codes": codes,
            "observations": observations,
        })),
    )
        .into_response()
}

pub async fn user_trends(State(state): State<AppState>, Path(user_id): Path<String>) -> Response {
    let codes = state.store.codes(&user_id);
    if codes.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("no data for {user_id} — run sync first") })),
        )
            .into_response();
    }
    let reports: Vec<Value> = codes
        .iter()
        .map(|code| {
            let observations = state.store.observations(&user_id, Some(code));
            json!({
                "loinc": code,
                "n": observations.len(),
                "trend": trends::trend(&observations).map(|t| t.label()),
                "latest": observations.last().map(|o| o.value_quantity.value),
                "unit": observations.last().map(|o| o.value_quantity.unit.clone()),
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(json!({ "user_id": user_id, "trends": reports })),
    )
        .into_response()
}
