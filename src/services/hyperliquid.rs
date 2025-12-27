use serde::Serialize;

use crate::models::candle::Candle;

const HYPERLIQUID_API_URL: &str = "https://api.hyperliquid.xyz/info";

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

#[derive(Clone)]
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
