use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Supported candle intervals for Hyperliquid.
pub const SUPPORTED_INTERVALS: [&str; 14] = [
    "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d", "3d", "1w", "1M",
];

/// Candle intervals supported by Hyperliquid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum CandleInterval {
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "3m")]
    ThreeMinutes,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "30m")]
    ThirtyMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "2h")]
    TwoHours,
    #[serde(rename = "4h")]
    FourHours,
    #[serde(rename = "8h")]
    EightHours,
    #[serde(rename = "12h")]
    TwelveHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "3d")]
    ThreeDays,
    #[serde(rename = "1w")]
    OneWeek,
    #[serde(rename = "1M")]
    OneMonth,
}

impl CandleInterval {
    pub const fn as_str(&self) -> &'static str {
        match self {
            CandleInterval::OneMinute => "1m",
            CandleInterval::ThreeMinutes => "3m",
            CandleInterval::FiveMinutes => "5m",
            CandleInterval::FifteenMinutes => "15m",
            CandleInterval::ThirtyMinutes => "30m",
            CandleInterval::OneHour => "1h",
            CandleInterval::TwoHours => "2h",
            CandleInterval::FourHours => "4h",
            CandleInterval::EightHours => "8h",
            CandleInterval::TwelveHours => "12h",
            CandleInterval::OneDay => "1d",
            CandleInterval::ThreeDays => "3d",
            CandleInterval::OneWeek => "1w",
            CandleInterval::OneMonth => "1M",
        }
    }

    pub const fn ms(&self) -> u64 {
        match self {
            CandleInterval::OneMinute => 60_000,
            CandleInterval::ThreeMinutes => 180_000,
            CandleInterval::FiveMinutes => 300_000,
            CandleInterval::FifteenMinutes => 900_000,
            CandleInterval::ThirtyMinutes => 1_800_000,
            CandleInterval::OneHour => 3_600_000,
            CandleInterval::TwoHours => 7_200_000,
            CandleInterval::FourHours => 14_400_000,
            CandleInterval::EightHours => 28_800_000,
            CandleInterval::TwelveHours => 43_200_000,
            CandleInterval::OneDay => 86_400_000,
            CandleInterval::ThreeDays => 259_200_000,
            CandleInterval::OneWeek => 604_800_000,
            CandleInterval::OneMonth => 2_592_000_000,
        }
    }
}

impl fmt::Display for CandleInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CandleInterval {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "1m" => Ok(CandleInterval::OneMinute),
            "3m" => Ok(CandleInterval::ThreeMinutes),
            "5m" => Ok(CandleInterval::FiveMinutes),
            "15m" => Ok(CandleInterval::FifteenMinutes),
            "30m" => Ok(CandleInterval::ThirtyMinutes),
            "1h" => Ok(CandleInterval::OneHour),
            "2h" => Ok(CandleInterval::TwoHours),
            "4h" => Ok(CandleInterval::FourHours),
            "8h" => Ok(CandleInterval::EightHours),
            "12h" => Ok(CandleInterval::TwelveHours),
            "1d" => Ok(CandleInterval::OneDay),
            "3d" => Ok(CandleInterval::ThreeDays),
            "1w" => Ok(CandleInterval::OneWeek),
            "1M" => Ok(CandleInterval::OneMonth),
            _ => Err(invalid_interval_message()),
        }
    }
}

/// Convert a supported interval string to milliseconds.
pub fn interval_ms(interval: &str) -> Option<u64> {
    CandleInterval::from_str(interval).ok().map(|interval| interval.ms())
}

fn invalid_interval_message() -> String {
    format!(
        "interval must be one of: {}",
        SUPPORTED_INTERVALS.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn candle_interval_parses_known_values() {
        let interval = CandleInterval::from_str("1h").unwrap();
        assert_eq!(interval, CandleInterval::OneHour);
        assert_eq!(interval.ms(), 3_600_000);
    }

    #[test]
    fn candle_interval_rejects_unknown_values() {
        let error = CandleInterval::from_str("10m").unwrap_err();
        assert!(error.contains("interval must be one of"));
    }
}
