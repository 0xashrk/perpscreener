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
use crate::models::patterns::{PatternLifecycleEntry, PatternLifecycleSnapshot, PatternQuery};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/patterns/lifecycle",
    params(PatternQuery),
    responses(
        (status = 200, description = "Pattern lifecycle snapshot", body = PatternLifecycleSnapshot)
    )
)]
/// Return the latest per-pattern lifecycle snapshot.
pub async fn get_pattern_lifecycle(
    State(state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<PatternQuery>,
) -> Result<Json<PatternLifecycleSnapshot>, AppError> {
    let entries = state.pattern_lifecycle_state.entries.read().await.clone();
    let filtered = filter_entries(entries, &query);
    let trimmed = limit_per_group(filtered, query.limit);

    Ok(Json(PatternLifecycleSnapshot {
        as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
        entries: trimmed,
    }))
}

#[utoipa::path(
    get,
    path = "/patterns/lifecycle/stream",
    responses(
        (status = 200, description = "SSE stream of pattern lifecycle snapshots", content_type = "text/event-stream")
    )
)]
/// Stream lifecycle snapshots over SSE.
pub async fn get_pattern_lifecycle_stream(
    State(state): State<AppState>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let initial_entries = state.pattern_lifecycle_state.entries.read().await.clone();
    let initial_snapshot = PatternLifecycleSnapshot {
        as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
        entries: initial_entries,
    };

    let initial_events = match snapshot_event(initial_snapshot) {
        Some(event) => vec![Ok(event)],
        None => Vec::new(),
    };
    let initial_stream = tokio_stream::iter(initial_events);

    let rx = state.pattern_lifecycle_state.broadcaster.subscribe();
    let broadcast_stream = BroadcastStream::new(rx).filter_map(|message| match message {
        Ok(snapshot) => snapshot_event(snapshot).map(Ok),
        Err(BroadcastStreamRecvError::Lagged(_)) => None,
    });

    let stream = initial_stream.chain(broadcast_stream);
    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

fn filter_entries(
    entries: Vec<PatternLifecycleEntry>,
    query: &PatternQuery,
) -> Vec<PatternLifecycleEntry> {
    let coins: Option<HashSet<String>> = query.coins.as_ref().map(|list| {
        list.as_slice()
            .iter()
            .map(|coin| coin.to_uppercase())
            .collect()
    });
    let intervals: Option<HashSet<_>> = query
        .intervals
        .as_ref()
        .map(|list| list.as_slice().iter().copied().collect());

    entries
        .into_iter()
        .filter(|entry| {
            if let Some(ref coins) = coins {
                if !coins.contains(&entry.coin.to_uppercase()) {
                    return false;
                }
            }
            if let Some(ref intervals) = intervals {
                if !intervals.contains(&entry.interval) {
                    return false;
                }
            }
            true
        })
        .collect()
}

fn limit_per_group(entries: Vec<PatternLifecycleEntry>, limit: usize) -> Vec<PatternLifecycleEntry> {
    let mut grouped: HashMap<(String, crate::models::interval::CandleInterval), Vec<PatternLifecycleEntry>> =
        HashMap::new();

    for entry in entries {
        grouped
            .entry((entry.coin.clone(), entry.interval))
            .or_default()
            .push(entry);
    }

    let mut trimmed = Vec::new();
    for (_, mut group) in grouped {
        group.sort_by(|a, b| b.last_updated_ms.cmp(&a.last_updated_ms));
        group.truncate(limit);
        trimmed.extend(group);
    }

    trimmed.sort_by(|a, b| {
        a.coin
            .cmp(&b.coin)
            .then_with(|| a.interval.as_str().cmp(b.interval.as_str()))
            .then_with(|| b.last_updated_ms.cmp(&a.last_updated_ms))
    });

    trimmed
}

fn snapshot_event(snapshot: PatternLifecycleSnapshot) -> Option<Event> {
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
    use crate::models::patterns::{PatternClassification, PatternLifecycleState, PatternSignalType};

    fn entry(coin: &str, interval: CandleInterval, last_updated_ms: u64) -> PatternLifecycleEntry {
        PatternLifecycleEntry {
            coin: coin.to_string(),
            interval,
            pattern: "Hammer".to_string(),
            category: "candlestick_reversal".to_string(),
            classification: PatternClassification::Bullish,
            signal_type: PatternSignalType::Reversal,
            state: PatternLifecycleState::Confirmed,
            confidence: 0.7,
            state_since_ms: last_updated_ms,
            last_updated_ms,
            window_start_ms: 0,
            window_end_ms: 0,
            notes: None,
        }
    }

    #[test]
    fn limit_per_group_respects_limit() {
        let entries = vec![
            entry("BTC", CandleInterval::OneMinute, 3),
            entry("BTC", CandleInterval::OneMinute, 2),
            entry("BTC", CandleInterval::OneMinute, 1),
            entry("ETH", CandleInterval::OneMinute, 4),
        ];

        let trimmed = limit_per_group(entries, 2);
        let btc_count = trimmed
            .iter()
            .filter(|d| d.coin == "BTC" && d.interval == CandleInterval::OneMinute)
            .count();

        assert_eq!(btc_count, 2);
    }
}
