use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct Candle {
    #[serde(rename = "t")]
    pub open_time: u64,
    #[serde(rename = "o", deserialize_with = "deserialize_string_to_f64")]
    pub open: f64,
    #[serde(rename = "h", deserialize_with = "deserialize_string_to_f64")]
    pub high: f64,
    #[serde(rename = "l", deserialize_with = "deserialize_string_to_f64")]
    pub low: f64,
    #[serde(rename = "c", deserialize_with = "deserialize_string_to_f64")]
    pub close: f64,
}

pub fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrNumber;

    impl<'de> serde::de::Visitor<'de> for StringOrNumber {
        type Value = f64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_str<E>(self, value: &str) -> Result<f64, E>
        where
            E: serde::de::Error,
        {
            value.parse::<f64>().map_err(E::custom)
        }

        fn visit_string<E>(self, value: String) -> Result<f64, E>
        where
            E: serde::de::Error,
        {
            self.visit_str(&value)
        }

        fn visit_f64<E>(self, value: f64) -> Result<f64, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<f64, E>
        where
            E: serde::de::Error,
        {
            Ok(value as f64)
        }

        fn visit_u64<E>(self, value: u64) -> Result<f64, E>
        where
            E: serde::de::Error,
        {
            Ok(value as f64)
        }
    }

    deserializer.deserialize_any(StringOrNumber)
}

#[derive(Debug, Deserialize, Clone)]
struct ChartSnapshot {
    candles: Vec<Candle>,
}

pub struct BackendClient {
    base_url: String,
    client: Client,
}

impl BackendClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    pub async fn fetch_candles(&self, coin: &str, limit: usize) -> Result<Vec<Candle>> {
        let url = format!(
            "{}/chart?coin={}&interval=1m&limit={}",
            self.base_url, coin, limit
        );

        let snapshot: ChartSnapshot = self
            .client
            .get(&url)
            .send()
            .await
            .context("failed to fetch candles")?
            .error_for_status()
            .context("chart request failed")?
            .json()
            .await
            .context("failed to parse candles response")?;

        if snapshot.candles.is_empty() {
            return Err(anyhow!("no candles returned"));
        }

        Ok(snapshot.candles)
    }
}

const HYPERLIQUID_API_URL: &str = "https://api.hyperliquid.xyz/info";

#[derive(Debug, Serialize)]
struct HlCandleRequest {
    #[serde(rename = "type")]
    request_type: String,
    req: HlCandleRequestInner,
}

#[derive(Debug, Serialize)]
struct HlCandleRequestInner {
    coin: String,
    interval: String,
    #[serde(rename = "startTime")]
    start_time: u64,
    #[serde(rename = "endTime")]
    end_time: u64,
}

pub async fn fetch_hl_candles(
    client: &Client,
    coin: &str,
    start_time: u64,
    end_time: u64,
) -> Result<Vec<Candle>> {
    let request = HlCandleRequest {
        request_type: "candleSnapshot".to_string(),
        req: HlCandleRequestInner {
            coin: coin.to_string(),
            interval: "1m".to_string(),
            start_time,
            end_time,
        },
    };

    let resp = client
        .post(HYPERLIQUID_API_URL)
        .json(&request)
        .send()
        .await
        .context("failed to send hyperliquid request")?;
    let status = resp.status();
    let body = resp
        .text()
        .await
        .context("failed to read hyperliquid body")?;
    if !status.is_success() {
        return Err(anyhow!("hyperliquid status {}: {}", status, body));
    }

    let parsed: Value = serde_json::from_str(&body)
        .with_context(|| format!("failed to parse hyperliquid JSON: {}", body))?;

    if let Value::Array(_) = parsed {
        let candles: Vec<Candle> = serde_json::from_value(parsed)
            .with_context(|| "failed to decode hyperliquid candle array")?;
        return Ok(candles);
    }

    if let Value::Object(map) = parsed {
        if let Some(err) = map.get("error").or_else(|| map.get("message")) {
            return Err(anyhow!("hyperliquid error: {}", err));
        }
        if let Some(arr) = map
            .get("candles")
            .or_else(|| map.get("data"))
            .or_else(|| map.get("result"))
        {
            let candles: Vec<Candle> = serde_json::from_value(arr.clone())
                .with_context(|| "failed to decode hyperliquid candle payload")?;
            return Ok(candles);
        }
    }

    Err(anyhow!("unexpected hyperliquid response: {}", body))
}

// -- metaAndAssetCtxs for discovering top assets by volume --

#[derive(Debug, Deserialize)]
struct MetaResponse {
    universe: Vec<AssetInfo>,
}

#[derive(Debug, Deserialize)]
struct AssetInfo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct AssetCtx {
    #[serde(rename = "dayNtlVlm", deserialize_with = "deserialize_string_to_f64")]
    day_ntl_vlm: f64,
}

/// Fetch the top `n` perp assets by 24h notional volume from Hyperliquid.
pub async fn fetch_top_assets(client: &Client, n: usize) -> Result<Vec<String>> {
    let body = serde_json::json!({"type": "metaAndAssetCtxs"});
    let resp = client
        .post(HYPERLIQUID_API_URL)
        .json(&body)
        .send()
        .await
        .context("failed to fetch metaAndAssetCtxs")?;

    let parsed: Value = resp
        .json()
        .await
        .context("failed to parse metaAndAssetCtxs")?;

    let arr = parsed
        .as_array()
        .context("metaAndAssetCtxs: expected array")?;
    if arr.len() < 2 {
        return Err(anyhow!("metaAndAssetCtxs: expected 2 elements, got {}", arr.len()));
    }

    let meta: MetaResponse = serde_json::from_value(arr[0].clone())
        .context("failed to parse meta universe")?;
    let ctxs: Vec<AssetCtx> = serde_json::from_value(arr[1].clone())
        .context("failed to parse asset contexts")?;

    let mut assets: Vec<(String, f64)> = meta
        .universe
        .into_iter()
        .zip(ctxs.into_iter())
        .map(|(info, ctx)| (info.name, ctx.day_ntl_vlm))
        .collect();

    assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    assets.truncate(n);

    Ok(assets.into_iter().map(|(name, _)| name).collect())
}
