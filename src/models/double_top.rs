use serde::Serialize;
use utoipa::ToSchema;

use crate::business_logic::double_top::PatternState;

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
