use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::models::candle::Candle;
use crate::models::interval::CandleInterval;

/// Query parameters for chart endpoints.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema, IntoParams)]
pub struct ChartStreamQuery {
    #[validate(length(min = 1, max = 24))]
    #[param(example = "BTC")]
    pub coin: String,
    /// Candle interval. Supported: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 8h, 12h, 1d, 3d, 1w, 1M.
    #[param(example = "15m")]
    pub interval: CandleInterval,
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 5000))]
    #[param(example = 200, default = 200)]
    pub limit: usize,
}

/// Candle snapshot payload for chart endpoints.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ChartSnapshot {
    pub as_of_ms: u64,
    pub coin: String,
    pub interval: CandleInterval,
    pub candles: Vec<Candle>,
}

fn default_limit() -> usize {
    200
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn chart_stream_query_requires_coin_and_limit_bounds() {
        let mut query = ChartStreamQuery {
            coin: "".to_string(),
            interval: CandleInterval::OneMinute,
            limit: 0,
        };
        assert!(query.validate().is_err());

        query.coin = "BTC".to_string();
        query.limit = 5001;
        assert!(query.validate().is_err());

        query.limit = 5000;
        assert!(query.validate().is_ok());
    }
}
