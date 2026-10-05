//! Binary entry: fixture provider by default; Vital when configured.

use tower_http::trace::TraceLayer;
use wearable_data_ingestion::AppState;
use wearable_data_ingestion::provider::Provider;
use wearable_data_ingestion::routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,wearable_data_ingestion=debug".into()),
        )
        .init();

    let port: u16 = std::env::var("APP_PORT")
        .unwrap_or_else(|_| "8009".to_string())
        .parse()
        .expect("APP_PORT must be a valid port");

    let state = match (
        std::env::var("APP_VITAL_API_KEY"),
        std::env::var("APP_VITAL_USER_ID"),
    ) {
        (Ok(api_key), Ok(user_id)) => {
            let base_url = std::env::var("APP_VITAL_BASE_URL")
                .unwrap_or_else(|_| "https://api.tryvital.io".to_string());
            tracing::info!(%base_url, "provider: vital aggregation api");
            AppState::with_provider(Provider::Vital {
                base_url,
                api_key,
                user_id,
            })
        }
        _ => {
            tracing::info!("provider: committed fixture (offline)");
            AppState::with_fixture()
        }
    };

    let app = routes::router(state).layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(%port, "wearable data ingestion listening");
    axum::serve(listener, app).await?;
    Ok(())
}
