use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;

use crate::business_logic::double_top::PatternState;

/// Shared state for pattern detection status
pub type SharedPatternState = Arc<RwLock<Vec<CoinPatternStatus>>>;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CoinPatternStatus {
    pub coin: String,
    pub state: String,
    pub peak1_price: Option<f64>,
    pub neckline_price: Option<f64>,
    pub peak2_price: Option<f64>,
    pub is_warmed_up: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DoubleTopResponse {
    pub patterns: Vec<CoinPatternStatus>,
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
    let patterns = state.read().await.clone();
    Json(DoubleTopResponse { patterns })
}
