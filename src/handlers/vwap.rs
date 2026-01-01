use std::convert::Infallible;
use std::time::Duration;

use crate::errors::AppError;
use crate::handlers::query::ValidatedQuery;
use crate::models::interval::CandleInterval;
use crate::models::vwap::{VwapSnapshot, VwapStreamQuery, VwapTimeframe};
use crate::services::vwap::{ensure_timeframes_covered, VwapError, VwapService};
use crate::state::AppState;
use axum::{
    extract::State,
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

const SNAPSHOT_INTERVAL_SECS: u64 = 60;
const HEARTBEAT_THRESHOLD_MS: u64 = 90_000;
const HEARTBEAT_CHECK_SECS: u64 = 30;

#[utoipa::path(
    get,
    path = "/vwap/stream",
    params(VwapStreamQuery),
    responses(
        (status = 200, description = "SSE stream of VWAP snapshots", content_type = "text/event-stream"),
        (status = 400, description = "Invalid request", body = crate::errors::ErrorResponse)
    )
)]
/// Stream VWAP snapshots over SSE.
pub async fn get_vwap_stream(
    State(state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<VwapStreamQuery>,
    headers: HeaderMap,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let timeframes = query.timeframes.as_slice();
    let interval = resolve_interval(&query, timeframes)?;
    let interval_ms = interval.ms();

    let now_ms = chrono::Utc::now().timestamp_millis() as u64;
    ensure_timeframes_covered(timeframes, interval_ms, now_ms).map_err(AppError::Validation)?;

    let last_event_id = parse_last_event_id(&headers);

    let request = VwapStreamRequest {
        coin: query.coin,
        interval,
        timeframes: timeframes.to_vec(),
        bands: query.bands,
        last_event_id,
    };

    let service = VwapService::new(state.hyperliquid.clone());
    let stream = vwap_stream(service, request);

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

#[utoipa::path(
    get,
    path = "/vwap",
    params(VwapStreamQuery),
    responses(
        (status = 200, description = "VWAP snapshot", body = VwapSnapshot),
        (status = 400, description = "Invalid request", body = crate::errors::ErrorResponse)
    )
)]
/// Return a VWAP snapshot for the requested timeframes.
pub async fn get_vwap_snapshot(
    State(state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<VwapStreamQuery>,
) -> Result<Json<VwapSnapshot>, AppError> {
    let timeframes = query.timeframes.as_slice();
    let interval = resolve_interval(&query, timeframes)?;
    let interval_ms = interval.ms();

    let now_ms = chrono::Utc::now().timestamp_millis() as u64;
    ensure_timeframes_covered(timeframes, interval_ms, now_ms).map_err(AppError::Validation)?;

    let service = VwapService::new(state.hyperliquid.clone());
    let snapshot = service
        .fetch_snapshot(&query.coin, interval, timeframes, query.bands)
        .await
        .map_err(map_vwap_error)?;

    Ok(Json(snapshot))
}

struct VwapStreamRequest {
    coin: String,
    interval: CandleInterval,
    timeframes: Vec<VwapTimeframe>,
    bands: bool,
    last_event_id: Option<u64>,
}

fn vwap_stream(
    service: VwapService,
    request: VwapStreamRequest,
) -> ReceiverStream<Result<Event, Infallible>> {
    let (tx, rx) = mpsc::channel(16);
    tokio::spawn(run_vwap_stream(service, request, tx));
    ReceiverStream::new(rx)
}

async fn run_vwap_stream(
    service: VwapService,
    request: VwapStreamRequest,
    tx: mpsc::Sender<Result<Event, Infallible>>,
) {
    if let Some(last_event_id) = request.last_event_id {
        tracing::debug!(last_event_id, "vwap stream reconnect");
    }

    let mut last_snapshot_ms = match send_snapshot(&service, &request, &tx).await {
        Ok(as_of_ms) => as_of_ms,
        Err(error) => {
            tracing::error!("vwap snapshot error: {}", error);
            return;
        }
    };
    let mut last_heartbeat_ms = last_snapshot_ms;

    let mut snapshot_ticker = tokio::time::interval(Duration::from_secs(SNAPSHOT_INTERVAL_SECS));
    let mut heartbeat_ticker = tokio::time::interval(Duration::from_secs(HEARTBEAT_CHECK_SECS));

    loop {
        tokio::select! {
            _ = snapshot_ticker.tick() => {
                match send_snapshot(&service, &request, &tx).await {
                    Ok(as_of_ms) => {
                        last_snapshot_ms = as_of_ms;
                        last_heartbeat_ms = as_of_ms;
                    }
                    Err(error) => {
                        tracing::error!("vwap snapshot error: {}", error);
                        break;
                    }
                }
            }
            _ = heartbeat_ticker.tick() => {
                let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                if should_emit_heartbeat(last_snapshot_ms, last_heartbeat_ms, now_ms) {
                    if let Err(error) = send_heartbeat(now_ms, &tx).await {
                        tracing::error!("vwap heartbeat error: {}", error);
                        break;
                    }
                    last_heartbeat_ms = now_ms;
                }
            }
        }
    }
}

async fn send_snapshot(
    service: &VwapService,
    request: &VwapStreamRequest,
    tx: &mpsc::Sender<Result<Event, Infallible>>,
) -> Result<u64, AppError> {
    let snapshot = service
        .fetch_snapshot(
            &request.coin,
            request.interval,
            &request.timeframes,
            request.bands,
        )
        .await
        .map_err(map_vwap_error)?;

    let event = snapshot_event(&snapshot)?;

    tx.send(Ok(event))
        .await
        .map_err(|_| AppError::Internal("vwap stream closed".to_string()))?;
    Ok(snapshot.as_of_ms)
}

#[derive(Serialize)]
struct HeartbeatPayload {
    as_of_ms: u64,
}

async fn send_heartbeat(
    as_of_ms: u64,
    tx: &mpsc::Sender<Result<Event, Infallible>>,
) -> Result<(), AppError> {
    let event = heartbeat_event(as_of_ms)?;

    tx.send(Ok(event))
        .await
        .map_err(|_| AppError::Internal("vwap stream closed".to_string()))?;
    Ok(())
}

fn snapshot_event(snapshot: &VwapSnapshot) -> Result<Event, AppError> {
    let data =
        serde_json::to_string(snapshot).map_err(|err| AppError::Internal(err.to_string()))?;
    Ok(Event::default()
        .event("snapshot")
        .id(snapshot.as_of_ms.to_string())
        .data(data))
}

fn heartbeat_event(as_of_ms: u64) -> Result<Event, AppError> {
    let payload = HeartbeatPayload { as_of_ms };
    let data =
        serde_json::to_string(&payload).map_err(|err| AppError::Internal(err.to_string()))?;
    Ok(Event::default()
        .event("heartbeat")
        .id(as_of_ms.to_string())
        .data(data))
}

fn should_emit_heartbeat(last_snapshot_ms: u64, last_heartbeat_ms: u64, now_ms: u64) -> bool {
    now_ms.saturating_sub(last_snapshot_ms) >= HEARTBEAT_THRESHOLD_MS
        && now_ms.saturating_sub(last_heartbeat_ms) >= HEARTBEAT_THRESHOLD_MS
}

fn parse_last_event_id(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
}

fn map_vwap_error(error: VwapError) -> AppError {
    match error {
        VwapError::InvalidCoin { coin } => AppError::Validation(format!("invalid coin: {}", coin)),
        VwapError::NoClosedCandles => AppError::Upstream("no closed candles available".to_string()),
        VwapError::NoVwapData { timeframe } => {
            AppError::Upstream(format!("no vwap data for timeframe {}", timeframe))
        }
        VwapError::Upstream { message } => AppError::Upstream(message),
    }
}

fn resolve_interval(
    query: &VwapStreamQuery,
    timeframes: &[VwapTimeframe],
) -> Result<CandleInterval, AppError> {
    let interval = match query.interval {
        Some(interval) => interval,
        None => {
            if timeframes
                .iter()
                .any(|tf| matches!(tf, VwapTimeframe::Weekly | VwapTimeframe::Monthly))
            {
                CandleInterval::OneHour
            } else {
                CandleInterval::OneMinute
            }
        }
    };

    Ok(interval)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::vwap::TimeframeList;
    use axum::http::HeaderValue;

    #[test]
    fn parse_last_event_id_reads_header() {
        let mut headers = HeaderMap::new();
        headers.insert("last-event-id", HeaderValue::from_static("12345"));
        assert_eq!(parse_last_event_id(&headers), Some(12345));
    }

    #[test]
    fn parse_last_event_id_ignores_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert("last-event-id", HeaderValue::from_static("nope"));
        assert_eq!(parse_last_event_id(&headers), None);
    }

    #[test]
    fn should_emit_heartbeat_after_threshold() {
        let now_ms = 100_000;
        assert!(should_emit_heartbeat(0, 0, now_ms));
        assert!(!should_emit_heartbeat(
            now_ms - 60_000,
            now_ms - 60_000,
            now_ms
        ));
        assert!(!should_emit_heartbeat(0, 50_000, now_ms));
    }

    #[test]
    fn map_vwap_error_invalid_coin_returns_validation() {
        let error = map_vwap_error(VwapError::InvalidCoin {
            coin: "NOPE".to_string(),
        });
        match error {
            AppError::Validation(message) => assert!(message.contains("invalid coin")),
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn resolve_interval_defaults_to_1m_for_intraday() {
        let query = VwapStreamQuery {
            coin: "BTC".to_string(),
            timeframes: TimeframeList(vec![VwapTimeframe::Session, VwapTimeframe::FourHour]),
            bands: true,
            interval: None,
        };
        let interval = resolve_interval(&query, query.timeframes.as_slice()).unwrap();
        assert_eq!(interval, CandleInterval::OneMinute);
    }

    #[test]
    fn resolve_interval_defaults_to_1h_for_swing() {
        let query = VwapStreamQuery {
            coin: "BTC".to_string(),
            timeframes: TimeframeList(vec![VwapTimeframe::Session, VwapTimeframe::Weekly]),
            bands: true,
            interval: None,
        };
        let interval = resolve_interval(&query, query.timeframes.as_slice()).unwrap();
        assert_eq!(interval, CandleInterval::OneHour);
    }
}
