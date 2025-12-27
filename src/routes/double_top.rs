use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use tokio_stream::StreamExt;
use utoipa::ToSchema;

use crate::business_logic::double_top::PatternState;

/// Shared state for pattern detection status
pub type SharedPatternState = Arc<PatternStateInner>;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CoinPatternStatus {
    pub coin: String,
    pub state: String,
    pub peak1_price: Option<f64>,
    pub neckline_price: Option<f64>,
    pub peak2_price: Option<f64>,
    pub is_warmed_up: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DoubleTopResponse {
    pub patterns: Vec<CoinPatternStatus>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PatternSnapshot {
    pub as_of_ms: u64,
    pub patterns: Vec<CoinPatternStatus>,
}

#[derive(Debug)]
pub struct PatternStateInner {
    pub patterns: RwLock<Vec<CoinPatternStatus>>,
    pub broadcaster: broadcast::Sender<PatternSnapshot>,
}

impl From<PatternState> for String {
    fn from(state: PatternState) -> Self {
        match state {
            PatternState::Watching => "WATCHING".to_string(),
            PatternState::PeakFound => "PEAK_FOUND".to_string(),
            PatternState::TroughFound => "TROUGH_FOUND".to_string(),
            PatternState::Forming => "FORMING".to_string(),
            PatternState::Confirmed => "CONFIRMED".to_string(),
            PatternState::Invalidated => "INVALIDATED".to_string(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/double-top",
    responses(
        (status = 200, description = "Double top pattern status for all coins", body = DoubleTopResponse)
    )
)]
pub async fn get_double_top_status(
    State(state): State<SharedPatternState>,
) -> Json<DoubleTopResponse> {
    let patterns = state.patterns.read().await.clone();
    Json(DoubleTopResponse { patterns })
}

#[utoipa::path(
    get,
    path = "/double-top/stream",
    responses(
        (status = 200, description = "SSE stream of double top pattern snapshots")
    )
)]
pub async fn get_double_top_stream(
    State(state): State<SharedPatternState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let initial_patterns = state.patterns.read().await.clone();
    let initial_snapshot = PatternSnapshot {
        as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
        patterns: initial_patterns,
    };

    let initial_events = match snapshot_event(initial_snapshot) {
        Some(event) => vec![Ok(event)],
        None => Vec::new(),
    };
    let initial_stream = tokio_stream::iter(initial_events);

    let rx = state.broadcaster.subscribe();
    let broadcast_stream = BroadcastStream::new(rx).filter_map(|message| {
        match message {
            Ok(snapshot) => snapshot_event(snapshot).map(Ok),
            Err(BroadcastStreamRecvError::Lagged(_)) => None,
        }
    });

    let stream = initial_stream.chain(broadcast_stream);

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

fn snapshot_event(snapshot: PatternSnapshot) -> Option<Event> {
    let data = serde_json::to_string(&snapshot).ok()?;
    Some(
        Event::default()
            .event("snapshot")
            .id(snapshot.as_of_ms.to_string())
            .data(data),
    )
}
