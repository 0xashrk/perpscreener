mod routes;
mod services;
mod business_logic;

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::business_logic::config::DoubleTopConfig;
use crate::routes::double_top::{
    get_double_top_status, get_double_top_stream, CoinPatternStatus, DoubleTopResponse,
    PatternStateInner, SharedPatternState,
};
use crate::services::monitor::MonitorService;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        routes::double_top::get_double_top_status,
        routes::double_top::get_double_top_stream
    ),
    components(schemas(
        routes::health::HealthResponse,
        DoubleTopResponse,
        CoinPatternStatus
    ))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "perpscreener=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Shared state for pattern detection status
    let (broadcaster, _receiver) = tokio::sync::broadcast::channel(16);
    let pattern_state: SharedPatternState = Arc::new(PatternStateInner {
        patterns: RwLock::new(Vec::new()),
        broadcaster,
    });

    // Start double top monitoring in background
    let coins = vec![
        "BTC".to_string(),
        "ETH".to_string(),
        "SOL".to_string(),
    ];
    let config = DoubleTopConfig::default();
    let monitor_state = pattern_state.clone();

    tokio::spawn(async move {
        let mut monitor = MonitorService::new(coins, config, monitor_state);

        tracing::info!("Starting double top detection warmup...");
        if let Err(e) = monitor.warmup().await {
            tracing::error!("Warmup failed: {}", e);
            return;
        }

        tracing::info!("Double top detection active, monitoring every 60s");
        monitor.run().await;
    });

    // Start web server
    let app = Router::new()
        .route("/health", get(routes::health::health))
        .route("/double-top", get(get_double_top_status))
        .route("/double-top/stream", get(get_double_top_stream))
        .with_state(pattern_state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server running on http://localhost:3000");
    tracing::info!("Swagger UI: http://localhost:3000/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}
