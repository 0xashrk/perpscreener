use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use tokio_stream::StreamExt;

use crate::errors::AppError;
use crate::handlers::query::ValidatedQuery;
use crate::models::patterns::{AdvancedPatternDetection, AdvancedPatternResponse, PatternQuery};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/patterns/advanced",
    params(PatternQuery),
    responses(
        (status = 200, description = "Advanced pattern detections", body = AdvancedPatternResponse)
    )
)]
/// Return the latest advanced pattern detections (Fibonacci, Elliott, fractals).
pub async fn get_advanced_patterns(
    State(state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<PatternQuery>,
) -> Result<Json<AdvancedPatternResponse>, AppError> {
    let detections = state.advanced_pattern_state.detections.read().await.clone();
    let filtered = filter_advanced(detections, &query);
    let trimmed = limit_per_group(filtered, query.limit);

    Ok(Json(AdvancedPatternResponse {
        as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
        detections: trimmed,
    }))
}

#[utoipa::path(
    get,
    path = "/patterns/advanced/stream",
    responses(
        (status = 200, description = "SSE stream of advanced pattern snapshots", content_type = "text/event-stream")
    )
)]
/// Stream advanced pattern snapshots over SSE.
pub async fn get_advanced_patterns_stream(
    State(state): State<AppState>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let initial_detections = state.advanced_pattern_state.detections.read().await.clone();
    let initial_snapshot = AdvancedPatternResponse {
        as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
        detections: initial_detections,
    };

    let initial_events = match snapshot_event(initial_snapshot) {
        Some(event) => vec![Ok(event)],
        None => Vec::new(),
    };
    let initial_stream = tokio_stream::iter(initial_events);

    let rx = state.advanced_pattern_state.broadcaster.subscribe();
    let broadcast_stream = BroadcastStream::new(rx).filter_map(|message| match message {
        Ok(snapshot) => snapshot_event(snapshot).map(Ok),
        Err(BroadcastStreamRecvError::Lagged(_)) => None,
    });

    let stream = initial_stream.chain(broadcast_stream);

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

fn filter_advanced(
    detections: Vec<AdvancedPatternDetection>,
    query: &PatternQuery,
) -> Vec<AdvancedPatternDetection> {
    let coins: Option<HashSet<String>> = query
        .coins
        .as_ref()
        .map(|list| list.as_slice().iter().cloned().collect());
    let intervals: Option<HashSet<_>> = query
        .intervals
        .as_ref()
        .map(|list| list.as_slice().iter().copied().collect());

    detections
        .into_iter()
        .filter(|detection| {
            if let Some(ref coins) = coins {
                if !coins.contains(&detection.detection.coin.to_uppercase()) {
                    return false;
                }
            }
            if let Some(ref intervals) = intervals {
                if !intervals.contains(&detection.detection.interval) {
                    return false;
                }
            }
            if let Some(since_ms) = query.since_ms {
                if detection.detection.detected_at_ms < since_ms {
                    return false;
                }
            }
            true
        })
        .collect()
}

fn limit_per_group(
    detections: Vec<AdvancedPatternDetection>,
    limit: usize,
) -> Vec<AdvancedPatternDetection> {
    let mut grouped: HashMap<(String, crate::models::interval::CandleInterval), Vec<AdvancedPatternDetection>> =
        HashMap::new();

    for detection in detections {
        grouped
            .entry((detection.detection.coin.clone(), detection.detection.interval))
            .or_default()
            .push(detection);
    }

    let mut trimmed = Vec::new();
    for (_, mut group) in grouped {
        group.sort_by(|a, b| b.detection.detected_at_ms.cmp(&a.detection.detected_at_ms));
        group.truncate(limit);
        trimmed.extend(group);
    }

    trimmed.sort_by(|a, b| {
        a.detection
            .coin
            .cmp(&b.detection.coin)
            .then_with(|| a.detection.interval.as_str().cmp(b.detection.interval.as_str()))
            .then_with(|| b.detection.detected_at_ms.cmp(&a.detection.detected_at_ms))
    });

    trimmed
}

fn snapshot_event(snapshot: AdvancedPatternResponse) -> Option<Event> {
    let data = serde_json::to_string(&snapshot).ok()?;
    Some(
        Event::default()
            .event("snapshot")
            .id(snapshot.as_of_ms.to_string())
            .data(data),
    )
}
