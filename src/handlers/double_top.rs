use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use tokio_stream::StreamExt;

use crate::errors::AppError;
use crate::models::double_top::{DoubleTopResponse, PatternSnapshot};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/double-top",
    responses(
        (status = 200, description = "Double top pattern status for all coins", body = DoubleTopResponse)
    )
)]
pub async fn get_double_top_status(
    State(state): State<AppState>,
) -> Result<Json<DoubleTopResponse>, AppError> {
    let patterns = state.pattern_state.patterns.read().await.clone();
    Ok(Json(DoubleTopResponse { patterns }))
}

#[utoipa::path(
    get,
    path = "/double-top/stream",
    responses(
        (status = 200, description = "SSE stream of double top pattern snapshots", content_type = "text/event-stream")
    )
)]
pub async fn get_double_top_stream(
    State(state): State<AppState>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let initial_patterns = state.pattern_state.patterns.read().await.clone();
    let initial_snapshot = PatternSnapshot {
        as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
        patterns: initial_patterns,
    };

    let initial_events = match snapshot_event(initial_snapshot) {
        Some(event) => vec![Ok(event)],
        None => Vec::new(),
    };
    let initial_stream = tokio_stream::iter(initial_events);

    let rx = state.pattern_state.broadcaster.subscribe();
    let broadcast_stream = BroadcastStream::new(rx).filter_map(|message| match message {
        Ok(snapshot) => snapshot_event(snapshot).map(Ok),
        Err(BroadcastStreamRecvError::Lagged(_)) => None,
    });

    let stream = initial_stream.chain(broadcast_stream);

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
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
