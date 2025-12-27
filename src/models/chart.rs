use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::{Validate, ValidationError};

use crate::models::candle::Candle;

pub const SUPPORTED_INTERVALS: [&str; 14] = [
    "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d", "3d", "1w", "1M",
];

#[derive(Debug, Clone, Deserialize, Validate, ToSchema, IntoParams)]
pub struct ChartStreamQuery {
    #[validate(length(min = 1, max = 24))]
    #[param(example = "BTC")]
    pub coin: String,
    /// Candle interval. Supported: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 8h, 12h, 1d, 3d, 1w, 1M.
    #[validate(custom(function = "validate_interval"))]
    #[param(example = "15m")]
    pub interval: String,
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 5000))]
    #[param(example = 200, default = 200)]
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ChartSnapshot {
    pub as_of_ms: u64,
    pub coin: String,
    pub interval: String,
    pub candles: Vec<Candle>,
}

pub fn interval_ms(interval: &str) -> Option<u64> {
    match interval {
        "1m" => Some(60_000),
        "3m" => Some(180_000),
        "5m" => Some(300_000),
        "15m" => Some(900_000),
        "30m" => Some(1_800_000),
        "1h" => Some(3_600_000),
        "2h" => Some(7_200_000),
        "4h" => Some(14_400_000),
        "8h" => Some(28_800_000),
        "12h" => Some(43_200_000),
        "1d" => Some(86_400_000),
        "3d" => Some(259_200_000),
        "1w" => Some(604_800_000),
        "1M" => Some(2_592_000_000),
        _ => None,
    }
}

pub fn validate_interval(value: &str) -> Result<(), ValidationError> {
    if interval_ms(value).is_some() {
        return Ok(());
    }

    let mut error = ValidationError::new("unsupported_interval");
    error.message = Some(
        format!(
            "interval must be one of: {}",
            SUPPORTED_INTERVALS.join(", ")
        )
        .into(),
    );
    Err(error)
}

fn default_limit() -> usize {
    200
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn interval_ms_supports_all_intervals() {
        for interval in SUPPORTED_INTERVALS {
            assert!(
                interval_ms(interval).is_some(),
                "missing interval: {}",
                interval
            );
        }
    }

    #[test]
    fn interval_ms_rejects_unknown() {
        assert!(interval_ms("10m").is_none());
    }

    #[test]
    fn validate_interval_rejects_unknown() {
        let error = validate_interval("10m").unwrap_err();
        assert_eq!(error.code, "unsupported_interval");
    }

    #[test]
    fn chart_stream_query_requires_coin_and_limit_bounds() {
        let mut query = ChartStreamQuery {
            coin: "".to_string(),
            interval: "1m".to_string(),
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
