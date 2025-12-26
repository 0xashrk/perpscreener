use serde::{Deserialize, Serialize};

const HYPERLIQUID_API_URL: &str = "https://api.hyperliquid.xyz/info";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    /// Candle open time (epoch ms)
    #[serde(rename = "t")]
    pub open_time: u64,
    /// Candle close time (epoch ms)
    #[serde(rename = "T")]
    pub close_time: u64,
    /// Open price
    #[serde(rename = "o", deserialize_with = "deserialize_string_to_f64")]
    pub open: f64,
    /// High price
    #[serde(rename = "h", deserialize_with = "deserialize_string_to_f64")]
    pub high: f64,
    /// Low price
    #[serde(rename = "l", deserialize_with = "deserialize_string_to_f64")]
    pub low: f64,
    /// Close price
    #[serde(rename = "c", deserialize_with = "deserialize_string_to_f64")]
    pub close: f64,
    /// Volume
    #[serde(rename = "v", deserialize_with = "deserialize_string_to_f64")]
    pub volume: f64,
    /// Number of trades
    #[serde(rename = "n")]
    pub num_trades: u64,
}

fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

#[derive(Debug, Serialize)]
struct CandleRequest {
    #[serde(rename = "type")]
    request_type: String,
    req: CandleRequestInner,
}

#[derive(Debug, Serialize)]
struct CandleRequestInner {
    coin: String,
    interval: String,
    #[serde(rename = "startTime")]
    start_time: u64,
    #[serde(rename = "endTime")]
    end_time: u64,
}

pub struct HyperliquidClient {
    client: reqwest::Client,
}

impl HyperliquidClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch candles for a coin within a time range
    pub async fn fetch_candles(
        &self,
        coin: &str,
        interval: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<Candle>, reqwest::Error> {
        let request = CandleRequest {
            request_type: "candleSnapshot".to_string(),
            req: CandleRequestInner {
                coin: coin.to_string(),
                interval: interval.to_string(),
                start_time,
                end_time,
            },
        };

        let response = self
            .client
            .post(HYPERLIQUID_API_URL)
            .json(&request)
            .send()
            .await?
            .json::<Vec<Candle>>()
            .await?;

        Ok(response)
    }

    /// Fetch historical candles for warmup (fetches last N minutes of 1m candles)
    pub async fn fetch_warmup_candles(
        &self,
        coin: &str,
        warmup_candles: usize,
    ) -> Result<Vec<Candle>, reqwest::Error> {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let interval_ms = 60_000u64; // 1 minute
        let start_time = now - (warmup_candles as u64 * interval_ms);

        self.fetch_candles(coin, "1m", start_time, now).await
    }
}

impl Default for HyperliquidClient {
    fn default() -> Self {
        Self::new()
    }
}
