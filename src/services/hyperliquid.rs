use serde::{Deserialize, Serialize};
use thiserror::Error;

use reqwest::StatusCode;

use crate::models::candle::Candle;

const HYPERLIQUID_API_URL: &str = "https://api.hyperliquid.xyz/info";

#[derive(Debug, Error)]
pub enum HyperliquidError {
    #[error("hyperliquid request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("hyperliquid status {status}: {body}")]
    Status { status: StatusCode, body: String },
    #[error("hyperliquid decode error ({status}): {body}")]
    Decode {
        status: StatusCode,
        body: String,
        #[source]
        source: serde_json::Error,
    },
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

#[derive(Debug, Deserialize)]
struct CandleEnvelope {
    candles: Option<Vec<Candle>>,
    data: Option<Vec<Candle>>,
    result: Option<Vec<Candle>>,
    error: Option<String>,
    message: Option<String>,
}

/// HTTP client wrapper for Hyperliquid candle endpoints.
#[derive(Clone)]
pub struct HyperliquidClient {
    client: reqwest::Client,
}

impl HyperliquidClient {
    /// Create a new Hyperliquid client.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch candles for a coin within a time range.
    pub async fn fetch_candles(
        &self,
        coin: &str,
        interval: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<Candle>, HyperliquidError> {
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
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(HyperliquidError::Status { status, body });
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&body).map_err(|source| HyperliquidError::Decode {
                status,
                body: body.clone(),
                source,
            })?;

        match parsed {
            serde_json::Value::Array(_) => serde_json::from_value(parsed).map_err(|source| {
                HyperliquidError::Decode {
                    status,
                    body,
                    source,
                }
            }),
            serde_json::Value::Object(_) => {
                let envelope: CandleEnvelope =
                    serde_json::from_str(&body).map_err(|source| HyperliquidError::Decode {
                        status,
                        body: body.clone(),
                        source,
                    })?;
                if let Some(message) = envelope
                    .error
                    .or(envelope.message)
                    .filter(|value| !value.is_empty())
                {
                    return Err(HyperliquidError::Status { status, body: message });
                }
                if let Some(candles) = envelope
                    .candles
                    .or(envelope.data)
                    .or(envelope.result)
                {
                    return Ok(candles);
                }
                Err(HyperliquidError::Status {
                    status,
                    body: format!("missing candle array: {}", body),
                })
            }
            _ => Err(HyperliquidError::Status {
                status,
                body: format!("unexpected JSON payload: {}", body),
            }),
        }
    }

    /// Fetch historical candles for warmup (last N minutes of 1m candles).
    pub async fn fetch_warmup_candles(
        &self,
        coin: &str,
        warmup_candles: usize,
    ) -> Result<Vec<Candle>, HyperliquidError> {
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
