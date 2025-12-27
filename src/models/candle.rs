use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Candle {
    /// Candle open time (epoch ms)
    #[serde(rename = "t")]
    #[schema(rename = "t")]
    pub open_time: u64,
    /// Candle close time (epoch ms)
    #[serde(rename = "T")]
    #[schema(rename = "T")]
    pub close_time: u64,
    /// Open price
    #[serde(rename = "o", deserialize_with = "deserialize_string_to_f64")]
    #[schema(rename = "o")]
    pub open: f64,
    /// High price
    #[serde(rename = "h", deserialize_with = "deserialize_string_to_f64")]
    #[schema(rename = "h")]
    pub high: f64,
    /// Low price
    #[serde(rename = "l", deserialize_with = "deserialize_string_to_f64")]
    #[schema(rename = "l")]
    pub low: f64,
    /// Close price
    #[serde(rename = "c", deserialize_with = "deserialize_string_to_f64")]
    #[schema(rename = "c")]
    pub close: f64,
    /// Volume
    #[serde(rename = "v", deserialize_with = "deserialize_string_to_f64")]
    #[schema(rename = "v")]
    pub volume: f64,
    /// Number of trades
    #[serde(rename = "n")]
    #[schema(rename = "n")]
    pub num_trades: u64,
    /// Candle interval (optional if upstream omits it)
    #[serde(rename = "i")]
    #[schema(rename = "i")]
    pub interval: Option<String>,
    /// Candle symbol (optional if upstream omits it)
    #[serde(rename = "s")]
    #[schema(rename = "s")]
    pub symbol: Option<String>,
}

fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}
