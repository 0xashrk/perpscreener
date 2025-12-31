use std::collections::{HashMap, HashSet};

use axum::{extract::State, Json};

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
