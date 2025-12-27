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
use crate::models::chart::{interval_ms, ChartSnapshot, ChartStreamQuery, SUPPORTED_INTERVALS};
use crate::services::chart::ChartService;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/chart/stream",
    params(ChartStreamQuery),
    responses(
        (status = 200, description = "SSE stream of candle snapshots", content_type = "text/event-stream"),
        (status = 400, description = "Invalid request", body = crate::errors::ErrorResponse)
    )
)]
pub async fn get_chart_stream(
    State(state): State<AppState>,
    Query(query): Query<ChartStreamQuery>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    query
        .validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let poll_interval_ms = interval_ms(&query.interval).ok_or_else(|| {
        AppError::Validation(format!(
            "interval must be one of: {}",
            SUPPORTED_INTERVALS.join(", ")
        ))
    })?;
    let poll_interval = Duration::from_millis(poll_interval_ms);

    let service = ChartService::new(state.hyperliquid.clone());
    let stream = chart_stream(service, query, poll_interval);

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

fn chart_stream(
    service: ChartService,
    query: ChartStreamQuery,
    poll_interval: Duration,
) -> ReceiverStream<Result<Event, Infallible>> {
    let (tx, rx) = mpsc::channel(16);
    tokio::spawn(run_chart_stream(service, query, poll_interval, tx));
    ReceiverStream::new(rx)
}

async fn run_chart_stream(
    service: ChartService,
    query: ChartStreamQuery,
    poll_interval: Duration,
    tx: mpsc::Sender<Result<Event, Infallible>>,
) {
    if let Err(error) = send_snapshot(&service, &query, &tx).await {
        tracing::error!("chart snapshot error: {}", error);
        return;
    }

    let mut ticker = tokio::time::interval(poll_interval);

    loop {
        ticker.tick().await;

        if let Err(error) = send_snapshot(&service, &query, &tx).await {
            tracing::error!("chart snapshot error: {}", error);
            break;
        }
    }
}

async fn send_snapshot(
    service: &ChartService,
    query: &ChartStreamQuery,
    tx: &mpsc::Sender<Result<Event, Infallible>>,
) -> Result<(), AppError> {
    let snapshot = service
        .fetch_snapshot(&query.coin, &query.interval, query.limit)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;

    let event = snapshot_event(&snapshot)?;

    tx.send(Ok(event))
        .await
        .map_err(|_| AppError::Internal("chart stream closed".to_string()))?;
    Ok(())
}

fn snapshot_event(snapshot: &ChartSnapshot) -> Result<Event, AppError> {
    let data =
        serde_json::to_string(snapshot).map_err(|err| AppError::Internal(err.to_string()))?;
    Ok(Event::default()
        .event("snapshot")
        .id(snapshot.as_of_ms.to_string())
        .data(data))
}
