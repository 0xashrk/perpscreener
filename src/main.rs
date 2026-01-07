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
use crate::business_logic::features::FeatureConfig;
use crate::handlers::advanced_patterns::{get_advanced_patterns, get_advanced_patterns_stream};
use crate::handlers::chart::{get_chart_snapshot, get_chart_stream};
use crate::handlers::double_top::{get_double_top_status, get_double_top_stream};
use crate::handlers::health::health;
use crate::handlers::pattern_lifecycle::{get_pattern_lifecycle, get_pattern_lifecycle_stream};
use crate::handlers::pattern_registry::get_pattern_registry;
use crate::handlers::patterns::{get_patterns, get_patterns_stream};
use crate::handlers::vwap::{get_vwap_snapshot, get_vwap_stream};
use crate::models::candle::Candle;
use crate::models::chart::{ChartSnapshot, ChartStreamQuery};
use crate::models::double_top::{CoinPatternStatus, DoubleTopResponse};
use crate::models::health::HealthResponse;
use crate::models::interval::CandleInterval;
use crate::models::patterns::{
    AdvancedPatternDetection, AdvancedPatternResponse, CoinList, IntervalList,
    PatternClassification, PatternDetection, PatternLifecycleEntry, PatternLifecycleSnapshot,
    PatternLifecycleState, PatternQuery, PatternRegistryEntry, PatternRegistryResponse,
    PatternResponse, PatternSignalType, PatternSummary, PatternSummarySignal,
};
use crate::models::vwap::{VwapEntry, VwapSignal, VwapSnapshot, VwapStreamQuery, VwapTimeframe};
use crate::services::advanced_pattern_monitor::{
    AdvancedPatternMonitor, AdvancedPatternMonitorConfig,
};
use crate::services::advanced_pattern_state::AdvancedPatternStateInner;
use crate::services::candle_ingestion::{CandleIngestionConfig, CandleIngestionService};
use crate::services::candle_store::CandleStoreInner;
use crate::services::core_pattern_monitor::{CorePatternMonitor, CorePatternMonitorConfig};
use crate::services::core_pattern_state::CorePatternStateInner;
use crate::services::feature_store::FeatureStoreInner;
use crate::services::hyperliquid::HyperliquidClient;
use crate::services::monitor::MonitorService;
use crate::services::pattern_lifecycle_monitor::{
    PatternLifecycleMonitor, PatternLifecycleMonitorConfig,
};
use crate::services::pattern_lifecycle_state::PatternLifecycleStateInner;
use crate::services::pattern_state::{PatternStateInner, SharedPatternState};
use crate::services::token_store::TokenStore;
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health,
        handlers::double_top::get_double_top_status,
        handlers::double_top::get_double_top_stream,
        handlers::patterns::get_patterns,
        handlers::patterns::get_patterns_stream,
        handlers::pattern_registry::get_pattern_registry,
        handlers::pattern_lifecycle::get_pattern_lifecycle,
        handlers::pattern_lifecycle::get_pattern_lifecycle_stream,
        handlers::advanced_patterns::get_advanced_patterns,
        handlers::advanced_patterns::get_advanced_patterns_stream,
        handlers::chart::get_chart_stream,
        handlers::chart::get_chart_snapshot,
        handlers::vwap::get_vwap_stream,
        handlers::vwap::get_vwap_snapshot
    ),
    components(schemas(
        HealthResponse,
        DoubleTopResponse,
        CoinPatternStatus,
        ChartSnapshot,
        ChartStreamQuery,
        CandleInterval,
        VwapSnapshot,
        VwapEntry,
        VwapSignal,
        VwapStreamQuery,
        VwapTimeframe,
        Candle,
        PatternQuery,
        PatternResponse,
        PatternDetection,
        PatternSummary,
        PatternSummarySignal,
        PatternLifecycleSnapshot,
        PatternLifecycleEntry,
        PatternLifecycleState,
        PatternRegistryEntry,
        PatternRegistryResponse,
        AdvancedPatternResponse,
        AdvancedPatternDetection,
        PatternClassification,
        PatternSignalType,
        CoinList,
        IntervalList,
        errors::ErrorResponse
    ))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let _log_guard = init_logging();

    // Load tokens from SQLite database
    let token_store = match TokenStore::open() {
        Ok(store) => store,
        Err(e) => {
            tracing::error!("Failed to open token database: {}", e);
            return;
        }
    };
    let coins = match token_store.get_tokens() {
        Ok(tokens) => {
            tracing::info!("Loaded {} tokens from database: {:?}", tokens.len(), tokens);
            tokens
        }
        Err(e) => {
            tracing::error!("Failed to load tokens from database: {}", e);
            return;
        }
    };

    // Shared state for pattern detection status
    let (broadcaster, _receiver) = tokio::sync::broadcast::channel(16);
    let pattern_state: SharedPatternState = Arc::new(PatternStateInner {
        patterns: RwLock::new(Vec::new()),
        broadcaster,
    });
    let ingestion_config = CandleIngestionConfig::new(coins.clone());
    let candle_store = Arc::new(CandleStoreInner::new(ingestion_config.max_candles));
    let core_pattern_intervals = ingestion_config.intervals.clone();
    let feature_store = Arc::new(FeatureStoreInner::new(FeatureConfig::default()));
    let core_pattern_state = Arc::new(CorePatternStateInner::new());
    let advanced_pattern_state = Arc::new(AdvancedPatternStateInner::new());
    let pattern_lifecycle_state = Arc::new(PatternLifecycleStateInner::new());
    let hyperliquid = Arc::new(HyperliquidClient::new());
    let app_state = AppState {
        pattern_state: pattern_state.clone(),
        core_pattern_state: core_pattern_state.clone(),
        advanced_pattern_state: advanced_pattern_state.clone(),
        pattern_lifecycle_state: pattern_lifecycle_state.clone(),
        candle_store: candle_store.clone(),
        feature_store: feature_store.clone(),
        hyperliquid: hyperliquid.clone(),
    };

    // Start candle ingestion FIRST and wait for warmup before starting monitors
    let ingestion_store = candle_store.clone();
    let ingestion_features = feature_store.clone();

    let mut ingestion = CandleIngestionService::new(
        hyperliquid.clone(),
        ingestion_store,
        ingestion_features,
        ingestion_config.clone(),
    );

    tracing::info!("Starting candle ingestion warmup...");
    if let Err(err) = ingestion.warmup().await {
        tracing::error!("Candle ingestion warmup failed: {}", err);
        return;
    }
    tracing::info!("Candle ingestion warmup complete.");

    // Now spawn ingestion to run in background
    tokio::spawn(async move {
        tracing::info!("Candle ingestion active.");
        ingestion.run().await;
    });

    // Start double top monitoring in background
    let config = DoubleTopConfig::default();
    let monitor_state = pattern_state.clone();
    let monitor_hyperliquid = hyperliquid.clone();

    tokio::spawn(async move {
        let mut monitor = MonitorService::new(monitor_hyperliquid, coins, config, monitor_state);

        tracing::info!("Starting double top detection warmup...");
        if let Err(e) = monitor.warmup().await {
            tracing::error!("Warmup failed: {}", e);
            return;
        }

        tracing::info!("Double top detection active, monitoring every 60s");
        monitor.run().await;
    });

    let core_pattern_monitor = CorePatternMonitor::new(
        candle_store.clone(),
        feature_store.clone(),
        core_pattern_state.clone(),
        CorePatternMonitorConfig::new(ingestion_config.coins.clone(), core_pattern_intervals),
    );

    tokio::spawn(async move {
        core_pattern_monitor.run().await;
    });

    let advanced_pattern_monitor = AdvancedPatternMonitor::new(
        candle_store.clone(),
        feature_store.clone(),
        advanced_pattern_state.clone(),
        AdvancedPatternMonitorConfig::new(
            ingestion_config.coins.clone(),
            ingestion_config.intervals.clone(),
        ),
    );

    tokio::spawn(async move {
        advanced_pattern_monitor.run().await;
    });

    let lifecycle_monitor = PatternLifecycleMonitor::new(
        candle_store.clone(),
        feature_store.clone(),
        pattern_lifecycle_state.clone(),
        PatternLifecycleMonitorConfig::new(
            ingestion_config.coins.clone(),
            ingestion_config.intervals.clone(),
        ),
    );

    tokio::spawn(async move {
        lifecycle_monitor.run().await;
    });

    // Start web server
    let double_top_routes = Router::new()
        .route("/", get(get_double_top_status))
        .route("/stream", get(get_double_top_stream));
    let chart_routes = Router::new()
        .route("/", get(get_chart_snapshot))
        .route("/stream", get(get_chart_stream));
    let vwap_routes = Router::new()
        .route("/", get(get_vwap_snapshot))
        .route("/stream", get(get_vwap_stream));
    let pattern_routes = Router::new()
        .route("/", get(get_patterns))
        .route("/stream", get(get_patterns_stream))
        .route("/registry", get(get_pattern_registry))
        .route("/lifecycle", get(get_pattern_lifecycle))
        .route("/lifecycle/stream", get(get_pattern_lifecycle_stream));
    let advanced_pattern_routes = Router::new()
        .route("/", get(get_advanced_patterns))
        .route("/stream", get(get_advanced_patterns_stream));

    let app = Router::new()
        .route("/health", get(health))
        .nest("/double-top", double_top_routes)
        .nest("/patterns", pattern_routes)
        .nest("/patterns/advanced", advanced_pattern_routes)
        .nest("/chart", chart_routes)
        .nest("/vwap", vwap_routes)
        .with_state(app_state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:30001")
        .await
        .unwrap();
    tracing::info!("Server running on http://localhost:30001");
    tracing::info!("Swagger UI: http://localhost:30001/swagger-ui");
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
