use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use validator::Validate;

use crate::errors::AppError;
use crate::models::interval::{interval_ms, SUPPORTED_INTERVALS};
use crate::models::vwap::{parse_timeframes, VwapSnapshot, VwapStreamQuery, VwapTimeframe};
use crate::services::vwap::{ensure_timeframes_covered, VwapService};
use crate::state::AppState;

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
    Query(query): Query<VwapStreamQuery>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    query
        .validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let timeframes = parse_timeframes(&query.timeframes).map_err(AppError::Validation)?;
    let (interval, interval_ms) = resolve_interval(&query, &timeframes)?;

    let now_ms = chrono::Utc::now().timestamp_millis() as u64;
    ensure_timeframes_covered(&timeframes, interval_ms, now_ms).map_err(AppError::Validation)?;

    let request = VwapStreamRequest {
        coin: query.coin,
        interval,
        interval_ms,
        timeframes,
        bands: query.bands,
    };

    let service = VwapService::new(state.hyperliquid.clone());
    let stream = vwap_stream(service, request);

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

struct VwapStreamRequest {
    coin: String,
    interval: String,
    interval_ms: u64,
    timeframes: Vec<VwapTimeframe>,
    bands: bool,
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
    if let Err(error) = send_snapshot(&service, &request, &tx).await {
        tracing::error!("vwap snapshot error: {}", error);
        return;
    }

    let mut ticker = tokio::time::interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

        if let Err(error) = send_snapshot(&service, &request, &tx).await {
            tracing::error!("vwap snapshot error: {}", error);
            break;
        }
    }
}

async fn send_snapshot(
    service: &VwapService,
    request: &VwapStreamRequest,
    tx: &mpsc::Sender<Result<Event, Infallible>>,
) -> Result<(), AppError> {
    let snapshot = service
        .fetch_snapshot(
            &request.coin,
            &request.interval,
            request.interval_ms,
            &request.timeframes,
            request.bands,
        )
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;

    let event = snapshot_event(&snapshot)?;

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

fn resolve_interval(
    query: &VwapStreamQuery,
    timeframes: &[VwapTimeframe],
) -> Result<(String, u64), AppError> {
    let interval = match query.interval.as_deref() {
        Some(interval) => interval.to_string(),
        None => {
            if timeframes
                .iter()
                .any(|tf| matches!(tf, VwapTimeframe::Weekly | VwapTimeframe::Monthly))
            {
                "1h".to_string()
            } else {
                "1m".to_string()
            }
        }
    };

    let interval_ms = interval_ms(&interval).ok_or_else(|| {
        AppError::Validation(format!(
            "interval must be one of: {}",
            SUPPORTED_INTERVALS.join(", ")
        ))
    })?;

    Ok((interval, interval_ms))
}
