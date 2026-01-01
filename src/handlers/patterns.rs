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

use crate::business_logic::patterns::aggregation::{summarize_detections, PatternScoreWeights};
use crate::errors::AppError;
use crate::handlers::query::ValidatedQuery;
use crate::models::patterns::{PatternDetection, PatternQuery, PatternResponse};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/patterns",
    params(PatternQuery),
    responses(
        (status = 200, description = "Core pattern detections", body = PatternResponse)
    )
)]
/// Return the latest core pattern detections (candlesticks + gaps).
pub async fn get_patterns(
    State(state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<PatternQuery>,
) -> Result<Json<PatternResponse>, AppError> {
    let detections = state.core_pattern_state.detections.read().await.clone();
    let filtered = filter_detections(detections, &query);
    let trimmed = limit_per_group(filtered, query.limit);
    let summaries = summarize_detections(&trimmed, &PatternScoreWeights::default());

    Ok(Json(PatternResponse {
        as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
        detections: trimmed,
        summaries,
    }))
}

#[utoipa::path(
    get,
    path = "/patterns/stream",
    responses(
        (status = 200, description = "SSE stream of core pattern snapshots", content_type = "text/event-stream")
    )
)]
/// Stream core pattern snapshots over SSE.
pub async fn get_patterns_stream(
    State(state): State<AppState>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let initial_detections = state.core_pattern_state.detections.read().await.clone();
    let initial_summaries =
        summarize_detections(&initial_detections, &PatternScoreWeights::default());
    let initial_snapshot = PatternResponse {
        as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
        detections: initial_detections,
        summaries: initial_summaries,
    };

    let initial_events = match snapshot_event(initial_snapshot) {
        Some(event) => vec![Ok(event)],
        None => Vec::new(),
    };
    let initial_stream = tokio_stream::iter(initial_events);

    let rx = state.core_pattern_state.broadcaster.subscribe();
    let broadcast_stream = BroadcastStream::new(rx).filter_map(|message| match message {
        Ok(snapshot) => snapshot_event(snapshot).map(Ok),
        Err(BroadcastStreamRecvError::Lagged(_)) => None,
    });

    let stream = initial_stream.chain(broadcast_stream);

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

fn filter_detections(detections: Vec<PatternDetection>, query: &PatternQuery) -> Vec<PatternDetection> {
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
                if !coins.contains(&detection.coin.to_uppercase()) {
                    return false;
                }
            }
            if let Some(ref intervals) = intervals {
                if !intervals.contains(&detection.interval) {
                    return false;
                }
            }
            if let Some(since_ms) = query.since_ms {
                if detection.detected_at_ms < since_ms {
                    return false;
                }
            }
            true
        })
        .collect()
}

fn limit_per_group(detections: Vec<PatternDetection>, limit: usize) -> Vec<PatternDetection> {
    let mut grouped: HashMap<(String, crate::models::interval::CandleInterval), Vec<PatternDetection>> =
        HashMap::new();

    for detection in detections {
        grouped
            .entry((detection.coin.clone(), detection.interval))
            .or_default()
            .push(detection);
    }

    let mut trimmed = Vec::new();
    for (_, mut group) in grouped {
        group.sort_by(|a, b| b.detected_at_ms.cmp(&a.detected_at_ms));
        group.truncate(limit);
        trimmed.extend(group);
    }

    trimmed.sort_by(|a, b| {
        a.coin
            .cmp(&b.coin)
            .then_with(|| a.interval.as_str().cmp(b.interval.as_str()))
            .then_with(|| b.detected_at_ms.cmp(&a.detected_at_ms))
    });

    trimmed
}

fn snapshot_event(snapshot: PatternResponse) -> Option<Event> {
    let data = serde_json::to_string(&snapshot).ok()?;
    Some(
        Event::default()
            .event("snapshot")
            .id(snapshot.as_of_ms.to_string())
            .data(data),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::interval::CandleInterval;
    use crate::models::patterns::{PatternClassification, PatternSignalType};

    fn detection(coin: &str, interval: CandleInterval, detected_at_ms: u64) -> PatternDetection {
        PatternDetection {
            coin: coin.to_string(),
            interval,
            pattern: "Test".to_string(),
            category: "candlestick".to_string(),
            classification: PatternClassification::Bullish,
            signal_type: PatternSignalType::Reversal,
            confidence: 0.5,
            detected_at_ms,
            window_start_ms: detected_at_ms.saturating_sub(60_000),
            window_end_ms: detected_at_ms,
            notes: None,
        }
    }

    #[test]
    fn limit_per_group_respects_limit() {
        let detections = vec![
            detection("BTC", CandleInterval::OneMinute, 3),
            detection("BTC", CandleInterval::OneMinute, 2),
            detection("BTC", CandleInterval::OneMinute, 1),
            detection("ETH", CandleInterval::OneMinute, 4),
        ];

        let trimmed = limit_per_group(detections, 2);
        let btc_count = trimmed
            .iter()
            .filter(|d| d.coin == "BTC" && d.interval == CandleInterval::OneMinute)
            .count();

        assert_eq!(btc_count, 2);
    }
}
