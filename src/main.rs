mod business_logic;
mod errors;
mod handlers;
mod models;
mod services;
mod state;

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::business_logic::config::DoubleTopConfig;
use crate::handlers::chart::{get_chart_snapshot, get_chart_stream};
use crate::handlers::double_top::{get_double_top_status, get_double_top_stream};
use crate::handlers::health::health;
use crate::models::candle::Candle;
use crate::models::chart::{ChartSnapshot, ChartStreamQuery};
use crate::models::double_top::{CoinPatternStatus, DoubleTopResponse};
use crate::models::health::HealthResponse;
use crate::services::hyperliquid::HyperliquidClient;
use crate::services::monitor::MonitorService;
use crate::services::pattern_state::{PatternStateInner, SharedPatternState};
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health,
        handlers::double_top::get_double_top_status,
        handlers::double_top::get_double_top_stream,
        handlers::chart::get_chart_stream,
        handlers::chart::get_chart_snapshot
    ),
    components(schemas(
        HealthResponse,
        DoubleTopResponse,
        CoinPatternStatus,
        ChartSnapshot,
        ChartStreamQuery,
        Candle,
        errors::ErrorResponse
    ))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let _log_guard = init_logging();
    // Shared state for pattern detection status
    let (broadcaster, _receiver) = tokio::sync::broadcast::channel(16);
    let pattern_state: SharedPatternState = Arc::new(PatternStateInner {
        patterns: RwLock::new(Vec::new()),
        broadcaster,
    });
    let app_state = AppState {
        pattern_state: pattern_state.clone(),
        hyperliquid: Arc::new(HyperliquidClient::new()),
    };

    // Start double top monitoring in background
    let coins = vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()];
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
    let double_top_routes = Router::new()
        .route("/", get(get_double_top_status))
        .route("/stream", get(get_double_top_stream));
    let chart_routes = Router::new()
        .route("/", get(get_chart_snapshot))
        .route("/stream", get(get_chart_stream));

    let app = Router::new()
        .route("/health", get(health))
        .nest("/double-top", double_top_routes)
        .nest("/chart", chart_routes)
        .with_state(app_state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server running on http://localhost:3000");
    tracing::info!("Swagger UI: http://localhost:3000/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}

fn init_logging() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::never(".", "dev.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "perpscreener=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false),
        )
        .init();

    guard
}
