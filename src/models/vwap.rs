use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::models::interval::CandleInterval;

/// Supported VWAP timeframes accepted by the API.
pub const SUPPORTED_TIMEFRAMES: [&str; 5] = ["session", "4h", "1h", "weekly", "monthly"];

/// Comma-separated timeframe list parsed into enums.
#[derive(Debug, Clone, PartialEq, Eq, ToSchema)]
#[schema(
    value_type = String,
    example = "session,4h",
    description = "Comma-separated list of: session, 1h, 4h, weekly, monthly."
)]
pub struct TimeframeList(pub Vec<VwapTimeframe>);

impl TimeframeList {
    pub fn as_slice(&self) -> &[VwapTimeframe] {
        &self.0
    }
}

impl Default for TimeframeList {
    fn default() -> Self {
        default_timeframes()
    }
}

impl Serialize for TimeframeList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let joined = self
            .0
            .iter()
            .map(VwapTimeframe::as_str)
            .collect::<Vec<_>>()
            .join(",");
        serializer.serialize_str(&joined)
    }
}

impl<'de> Deserialize<'de> for TimeframeList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let parsed = parse_timeframes(&raw).map_err(serde::de::Error::custom)?;
        Ok(TimeframeList(parsed))
    }
}

/// Query parameters for VWAP streaming endpoints.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema, IntoParams)]
pub struct VwapStreamQuery {
    #[param(example = "BTC")]
    #[validate(length(min = 1, max = 24))]
    pub coin: String,
    #[serde(default = "default_timeframes")]
    #[param(
        example = "session,4h",
        value_type = String,
        description = "Comma-separated list of: session, 1h, 4h, weekly, monthly."
    )]
    pub timeframes: TimeframeList,
    #[serde(default = "default_bands")]
    #[param(example = true, default = true)]
    pub bands: bool,
    #[serde(default)]
    #[param(example = "1m")]
    pub interval: Option<CandleInterval>,
}

/// VWAP snapshot payload for SSE streaming.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VwapSnapshot {
    pub as_of_ms: u64,
    pub coin: String,
    pub current_price: f64,
    pub vwaps: Vec<VwapEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signals: Option<Vec<VwapSignal>>,
}

/// VWAP entry for a single timeframe.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VwapEntry {
    pub timeframe: VwapTimeframe,
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

/// VWAP signal entry for the UI.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VwapSignal {
    #[serde(rename = "type")]
    pub signal_type: String,
    pub timeframe: VwapTimeframe,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band: Option<String>,
}

/// VWAP timeframe selections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum VwapTimeframe {
    #[serde(rename = "session")]
    Session,
    #[serde(rename = "4h")]
    FourHour,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "weekly")]
    Weekly,
    #[serde(rename = "monthly")]
    Monthly,
}

impl VwapTimeframe {
    /// Convert the timeframe to its API string.
    pub fn as_str(&self) -> &'static str {
        match self {
            VwapTimeframe::Session => "session",
            VwapTimeframe::FourHour => "4h",
            VwapTimeframe::OneHour => "1h",
            VwapTimeframe::Weekly => "weekly",
            VwapTimeframe::Monthly => "monthly",
        }
    }
}

impl fmt::Display for VwapTimeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VwapTimeframe {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "session" => Ok(VwapTimeframe::Session),
            "4h" => Ok(VwapTimeframe::FourHour),
            "1h" => Ok(VwapTimeframe::OneHour),
            "weekly" => Ok(VwapTimeframe::Weekly),
            "monthly" => Ok(VwapTimeframe::Monthly),
            _ => Err(invalid_timeframe_message()),
        }
    }
}

/// Parse a comma-separated timeframe list.
fn parse_timeframes(input: &str) -> Result<Vec<VwapTimeframe>, String> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for raw in input.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let timeframe = VwapTimeframe::from_str(trimmed)?;
        if seen.insert(timeframe) {
            result.push(timeframe);
        }
    }

    if result.is_empty() {
        return Err(invalid_timeframe_message());
    }

    Ok(result)
}

/// Default timeframe selection for VWAP streaming.
pub fn default_timeframes() -> TimeframeList {
    TimeframeList(vec![VwapTimeframe::Session, VwapTimeframe::FourHour])
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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn timeframe_list_accepts_defaults() {
        let parsed: TimeframeList = serde_json::from_str("\"session,4h\"").unwrap();
        assert_eq!(parsed.0.len(), 2);
        assert_eq!(parsed.0[0], VwapTimeframe::Session);
        assert_eq!(parsed.0[1], VwapTimeframe::FourHour);
    }

    #[test]
    fn timeframe_list_rejects_unknown() {
        let error = serde_json::from_str::<TimeframeList>("\"session,unknown\"")
            .unwrap_err()
            .to_string();
        assert!(error.contains("timeframes must be one of"));
    }

    #[test]
    fn vwap_stream_query_validates() {
        let query = VwapStreamQuery {
            coin: "BTC".to_string(),
            timeframes: default_timeframes(),
            bands: true,
            interval: None,
        };
        assert!(query.validate().is_ok());
    }
}
