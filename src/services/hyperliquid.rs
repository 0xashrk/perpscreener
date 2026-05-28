use serde::{Deserialize, Serialize};
use thiserror::Error;

use reqwest::StatusCode;

use crate::models::candle::Candle;
use crate::models::orderbook::{L2BookLevel, L2BookSnapshot};

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
            serde_json::Value::Array(_) => {
                serde_json::from_value(parsed).map_err(|source| HyperliquidError::Decode {
                    status,
                    body,
                    source,
                })
            }
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
                    return Err(HyperliquidError::Status {
                        status,
                        body: message,
                    });
                }
                if let Some(candles) = envelope.candles.or(envelope.data).or(envelope.result) {
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
    #[allow(dead_code)]
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

// L2 Book types
#[derive(Debug, Serialize)]
struct L2BookRequest {
    #[serde(rename = "type")]
    request_type: String,
    coin: String,
    #[serde(rename = "nSigFigs", skip_serializing_if = "Option::is_none")]
    n_sig_figs: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mantissa: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct L2BookRawResponse {
    coin: String,
    time: u64,
    levels: Vec<Vec<L2BookRawLevel>>,
}

#[derive(Debug, Deserialize)]
struct L2BookRawLevel {
    px: String,
    sz: String,
    n: u32,
}

impl HyperliquidClient {
    /// Fetch L2 order book snapshot for a coin.
    pub async fn fetch_l2_book(
        &self,
        coin: &str,
        n_sig_figs: Option<u8>,
        mantissa: Option<u8>,
    ) -> Result<L2BookSnapshot, HyperliquidError> {
        let request = L2BookRequest {
            request_type: "l2Book".to_string(),
            coin: coin.to_string(),
            n_sig_figs,
            mantissa,
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

        let raw: L2BookRawResponse =
            serde_json::from_str(&body).map_err(|source| HyperliquidError::Decode {
                status,
                body: body.clone(),
                source,
            })?;

        // Convert raw levels to typed levels
        let bids: Vec<L2BookLevel> = raw
            .levels
            .first()
            .map(|levels| {
                levels
                    .iter()
                    .map(|l| L2BookLevel {
                        px: l.px.clone(),
                        sz: l.sz.clone(),
                        n: l.n,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let asks: Vec<L2BookLevel> = raw
            .levels
            .get(1)
            .map(|levels| {
                levels
                    .iter()
                    .map(|l| L2BookLevel {
                        px: l.px.clone(),
                        sz: l.sz.clone(),
                        n: l.n,
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(L2BookSnapshot {
            coin: raw.coin,
            time: raw.time,
            levels: (bids, asks),
        })
    }
}
