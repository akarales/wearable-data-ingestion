//! Integration tests over the fixture provider: sync → FHIR out → trends.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use wearable_data_ingestion::AppState;
use wearable_data_ingestion::provider::Provider;
use wearable_data_ingestion::routes;

fn app() -> axum::Router {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/wearable_series.json"
    );
    let state = AppState::with_provider(Provider::Fixture { path: path.into() });
    routes::router(state)
}

async fn request_on(
    router: axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let payload = body.map(|b| b.to_string()).unwrap_or_default();
    let response = router
        .oneshot(builder.body(Body::from(payload)).expect("builds"))
        .await
        .expect("request");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (status, serde_json::from_slice(&bytes).expect("json"))
}

// One instance per test so sync-then-read flows share the store.
async fn synced_app() -> axum::Router {
    let router = app();
    let _ = request_on(
        router.clone(),
        "POST",
        "/api/v1/users/ada/sync",
        Some(json!({ "days": 30 })),
    )
    .await;
    router
}

#[tokio::test]
async fn health_ok() {
    let (status, body) = request_on(app(), "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn sync_normalizes_fixture_to_fhir() {
    let (status, body) = request_on(
        app(),
        "POST",
        "/api/v1/users/ada/sync",
        Some(json!({ "days": 30 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["normalized"], 120);
    assert_eq!(body["skipped_unknown_metrics"], 0);
}

#[tokio::test]
async fn observations_are_fhir_shaped() {
    let router = synced_app().await;
    let (status, body) = request_on(
        router,
        "GET",
        "/api/v1/users/ada/observations?loinc=8867-4",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let observations = body["observations"].as_array().expect("observations");
    assert_eq!(observations.len(), 30);
    assert_eq!(observations[0]["resourceType"], "Observation");
    assert_eq!(
        observations[0]["code"]["coding"][0]["system"],
        "http://loinc.org"
    );
    assert_eq!(observations[0]["status"], "final");
}

#[tokio::test]
async fn trends_report_split_half_directions() {
    let router = synced_app().await;
    let (status, body) = request_on(router, "GET", "/api/v1/users/ada/trends", None).await;
    assert_eq!(status, StatusCode::OK);
    let trends = body["trends"].as_array().expect("trends");
    let by_code = |code: &str| {
        trends
            .iter()
            .find(|t| t["loinc"] == code)
            .expect("code present")
    };
    // hrv declines ~0.5/day in the fixture
    assert_eq!(by_code("80404-7")["trend"], "falling");
    // hr is steady noise
    assert_eq!(by_code("8867-4")["trend"], "flat");
}

#[tokio::test]
async fn unsynced_user_is_404() {
    let (status, _) = request_on(app(), "GET", "/api/v1/users/ghost/trends", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
