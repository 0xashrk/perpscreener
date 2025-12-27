use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::{Validate, ValidationError};

use crate::models::interval::{interval_ms, SUPPORTED_INTERVALS};

pub const SUPPORTED_TIMEFRAMES: [&str; 5] = ["session", "4h", "1h", "weekly", "monthly"];

#[derive(Debug, Clone, Deserialize, Validate, ToSchema, IntoParams)]
pub struct VwapStreamQuery {
    #[param(example = "BTC")]
    #[validate(length(min = 1, max = 24))]
    pub coin: String,
    #[serde(default = "default_timeframes")]
    #[param(example = "session,4h")]
    #[validate(custom(function = "validate_timeframes"))]
    pub timeframes: String,
    #[serde(default = "default_bands")]
    #[param(example = true, default = true)]
    pub bands: bool,
    #[serde(default)]
    #[param(example = "1m")]
    #[validate(custom(function = "validate_interval_opt"))]
    pub interval: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VwapSnapshot {
    pub as_of_ms: u64,
    pub coin: String,
    pub current_price: f64,
    pub vwaps: Vec<VwapEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signals: Option<Vec<VwapSignal>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VwapEntry {
    pub timeframe: String,
    pub anchor_time_ms: u64,
    pub vwap: f64,
    pub cumulative_volume: f64,
    pub distance_pct: f64,
    pub position: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper_band_1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower_band_1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper_band_2: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower_band_2: Option<f64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VwapSignal {
    #[serde(rename = "type")]
    pub signal_type: String,
    pub timeframe: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VwapTimeframe {
    Session,
    FourHour,
    OneHour,
    Weekly,
    Monthly,
}

impl VwapTimeframe {
    pub fn as_str(&self) -> &'static str {
        match self {
            VwapTimeframe::Session => "session",
            VwapTimeframe::FourHour => "4h",
            VwapTimeframe::OneHour => "1h",
            VwapTimeframe::Weekly => "weekly",
            VwapTimeframe::Monthly => "monthly",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "session" => Some(VwapTimeframe::Session),
            "4h" => Some(VwapTimeframe::FourHour),
            "1h" => Some(VwapTimeframe::OneHour),
            "weekly" => Some(VwapTimeframe::Weekly),
            "monthly" => Some(VwapTimeframe::Monthly),
            _ => None,
        }
    }
}

pub fn parse_timeframes(input: &str) -> Result<Vec<VwapTimeframe>, String> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for raw in input.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let timeframe = VwapTimeframe::parse(trimmed).ok_or_else(invalid_timeframe_message)?;
        if seen.insert(timeframe) {
            result.push(timeframe);
        }
    }

    if result.is_empty() {
        return Err(invalid_timeframe_message());
    }

    Ok(result)
}

pub fn default_timeframes() -> String {
    "session,4h".to_string()
}

fn default_bands() -> bool {
    true
}

fn invalid_timeframe_message() -> String {
    format!(
        "timeframes must be one of: {}",
        SUPPORTED_TIMEFRAMES.join(", ")
    )
}

pub fn validate_timeframes(value: &str) -> Result<(), ValidationError> {
    parse_timeframes(value).map(|_| ()).map_err(|message| {
        let mut error = ValidationError::new("unsupported_timeframe");
        error.message = Some(message.into());
        error
    })
}

pub fn validate_interval_opt(interval: &String) -> Result<(), ValidationError> {
    if interval_ms(interval).is_some() {
        Ok(())
    } else {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn parse_timeframes_accepts_defaults() {
        let parsed = parse_timeframes("session,4h").unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], VwapTimeframe::Session);
        assert_eq!(parsed[1], VwapTimeframe::FourHour);
    }

    #[test]
    fn parse_timeframes_rejects_unknown() {
        let error = parse_timeframes("session,unknown").unwrap_err();
        assert!(error.contains("timeframes must be one of"));
    }

    #[test]
    fn vwap_stream_query_validates() {
        let query = VwapStreamQuery {
            coin: "BTC".to_string(),
            timeframes: "session,4h".to_string(),
            bands: true,
            interval: None,
        };
        assert!(query.validate().is_ok());
    }
}
