use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::models::interval::CandleInterval;

/// Comma-separated coin list parsed into normalized symbols.
#[derive(Debug, Clone, PartialEq, Eq, ToSchema)]
#[schema(
    value_type = String,
    example = "BTC,ETH",
    description = "Comma-separated list of coin symbols."
)]
pub struct CoinList(pub Vec<String>);

impl CoinList {
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

impl Serialize for CoinList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let joined = self.0.join(",");
        serializer.serialize_str(&joined)
    }
}

impl<'de> Deserialize<'de> for CoinList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let parsed = parse_coin_list(&raw).map_err(serde::de::Error::custom)?;
        Ok(CoinList(parsed))
    }
}

/// Comma-separated interval list parsed into enums.
#[derive(Debug, Clone, PartialEq, Eq, ToSchema)]
#[schema(
    value_type = String,
    example = "1m,15m",
    description = "Comma-separated list of supported intervals."
)]
pub struct IntervalList(pub Vec<CandleInterval>);

impl IntervalList {
    pub fn as_slice(&self) -> &[CandleInterval] {
        &self.0
    }
}

impl Serialize for IntervalList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let joined = self
            .0
            .iter()
            .map(CandleInterval::as_str)
            .collect::<Vec<_>>()
            .join(",");
        serializer.serialize_str(&joined)
    }
}

impl<'de> Deserialize<'de> for IntervalList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let parsed = parse_interval_list(&raw).map_err(serde::de::Error::custom)?;
        Ok(IntervalList(parsed))
    }
}

/// Query parameters for core pattern snapshots.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema, IntoParams)]
pub struct PatternQuery {
    /// Comma-separated list of coin symbols.
    #[serde(default)]
    #[param(example = "BTC,ETH", value_type = String)]
    pub coins: Option<CoinList>,
    /// Comma-separated list of supported intervals.
    #[serde(default)]
    #[param(example = "1m,15m", value_type = String)]
    pub intervals: Option<IntervalList>,
    /// Max patterns per coin/timeframe.
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 200))]
    #[param(example = 25, default = 25)]
    pub limit: usize,
    /// Only return detections after this timestamp.
    #[serde(default)]
    #[param(example = 1735689600000_i64)]
    pub since_ms: Option<u64>,
}

/// Core pattern snapshot response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PatternResponse {
    pub as_of_ms: u64,
    pub detections: Vec<PatternDetection>,
    pub summaries: Vec<PatternSummary>,
}

/// Core pattern detection payload.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PatternDetection {
    pub coin: String,
    pub interval: CandleInterval,
    pub pattern: String,
    pub category: String,
    pub classification: PatternClassification,
    pub signal_type: PatternSignalType,
    pub confidence: f64,
    pub detected_at_ms: u64,
    pub window_start_ms: u64,
    pub window_end_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Aggregated summary of pattern signals per coin/timeframe.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PatternSummary {
    pub coin: String,
    pub interval: CandleInterval,
    pub bullish_score: f64,
    pub bearish_score: f64,
    pub neutral_score: f64,
    pub top_signals: Vec<PatternSummarySignal>,
}

/// Highest-weighted signals contributing to the summary.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PatternSummarySignal {
    pub pattern: String,
    pub classification: PatternClassification,
    pub confidence: f64,
}

/// Advanced pattern detection payload with heuristic context.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdvancedPatternDetection {
    #[serde(flatten)]
    pub detection: PatternDetection,
    pub method: String,
    pub basis: String,
    pub assumptions: Vec<String>,
}

/// Advanced pattern snapshot response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdvancedPatternResponse {
    pub as_of_ms: u64,
    pub detections: Vec<AdvancedPatternDetection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum PatternClassification {
    #[serde(rename = "bullish")]
    Bullish,
    #[serde(rename = "bearish")]
    Bearish,
    #[serde(rename = "neutral")]
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum PatternSignalType {
    #[serde(rename = "reversal")]
    Reversal,
    #[serde(rename = "continuation")]
    Continuation,
    #[serde(rename = "trend")]
    Trend,
    #[serde(rename = "range")]
    Range,
    #[serde(rename = "key_level")]
    KeyLevel,
    #[serde(rename = "impulse")]
    Impulse,
    #[serde(rename = "correction")]
    Correction,
}

fn parse_coin_list(input: &str) -> Result<Vec<String>, String> {
    let mut seen = HashSet::new();
    let mut coins = Vec::new();

    for raw in input.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let normalized = trimmed.to_uppercase();
        if seen.insert(normalized.clone()) {
            coins.push(normalized);
        }
    }

    if coins.is_empty() {
        return Err("coins must include at least one symbol".to_string());
    }

    Ok(coins)
}

fn parse_interval_list(input: &str) -> Result<Vec<CandleInterval>, String> {
    let mut seen = HashSet::new();
    let mut intervals = Vec::new();

    for raw in input.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let interval = CandleInterval::from_str(trimmed)
            .map_err(|_| invalid_interval_message())?;
        if seen.insert(interval) {
            intervals.push(interval);
        }
    }

    if intervals.is_empty() {
        return Err(invalid_interval_message());
    }

    Ok(intervals)
}

fn default_limit() -> usize {
    25
}

fn invalid_interval_message() -> String {
    format!(
        "intervals must be one of: {}",
        crate::models::interval::SUPPORTED_INTERVALS.join(", ")
    )
}

impl fmt::Display for PatternClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            PatternClassification::Bullish => "bullish",
            PatternClassification::Bearish => "bearish",
            PatternClassification::Neutral => "neutral",
        };
        f.write_str(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_list_parses_and_normalizes() {
        let parsed = parse_coin_list("btc, ETH,btc").expect("coin list");
        assert_eq!(parsed, vec!["BTC".to_string(), "ETH".to_string()]);
    }

    #[test]
    fn interval_list_rejects_unknown() {
        let err = parse_interval_list("10m").unwrap_err();
        assert!(err.contains("intervals must be one of"));
    }

    #[test]
    fn interval_list_parses_known_values() {
        let parsed = parse_interval_list("1m,15m").expect("interval list");
        assert_eq!(parsed, vec![CandleInterval::OneMinute, CandleInterval::FifteenMinutes]);
    }
}
