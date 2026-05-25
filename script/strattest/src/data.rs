use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

const HL_API: &str = "https://api.hyperliquid.xyz/info";

#[derive(Debug, Deserialize, Clone)]
pub struct Candle {
    pub t: u64,
    #[serde(rename = "T")]
    #[allow(dead_code)]
    pub t_close: u64,
    #[serde(deserialize_with = "de_f64")]
    pub o: f64,
    #[serde(deserialize_with = "de_f64")]
    pub h: f64,
    #[serde(deserialize_with = "de_f64")]
    pub l: f64,
    #[serde(deserialize_with = "de_f64")]
    pub c: f64,
    #[serde(deserialize_with = "de_f64")]
    #[allow(dead_code)]
    pub v: f64,
    #[allow(dead_code)]
    pub n: u32,
}

pub fn de_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = f64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("number or string")
        }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<f64, E> {
            Ok(v)
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<f64, E> {
            Ok(v as f64)
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<f64, E> {
            Ok(v as f64)
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<f64, E> {
            v.parse().map_err(E::custom)
        }
    }
    deserializer.deserialize_any(V)
}

#[derive(Serialize)]
struct CandleReq {
    #[serde(rename = "type")]
    req_type: String,
    req: CandleReqInner,
}

#[derive(Serialize)]
struct CandleReqInner {
    coin: String,
    interval: String,
    #[serde(rename = "startTime")]
    start_time: u64,
    #[serde(rename = "endTime")]
    end_time: u64,
}

async fn fetch_chunk(
    client: &Client,
    coin: &str,
    interval: &str,
    start: u64,
    end: u64,
) -> Result<Vec<Candle>> {
    let req = CandleReq {
        req_type: "candleSnapshot".to_string(),
        req: CandleReqInner {
            coin: coin.to_string(),
            interval: interval.to_string(),
            start_time: start,
            end_time: end,
        },
    };
    // Retry with backoff on rate limits.
    for attempt in 0..5u32 {
        let resp = client
            .post(HL_API)
            .json(&req)
            .send()
            .await
            .context("candle request failed")?;
        let status = resp.status();
        let body = resp.text().await.context("candle read failed")?;
        if status.as_u16() == 429 {
            let wait = std::time::Duration::from_millis(500 * 2u64.pow(attempt));
            tokio::time::sleep(wait).await;
            continue;
        }
        if !status.is_success() {
            anyhow::bail!("candle API {}: {}", status, body);
        }
        let candles: Vec<Candle> =
            serde_json::from_str(&body).context("candle parse failed")?;
        return Ok(candles);
    }
    anyhow::bail!("candle API: rate limited after 5 retries")
}

/// Fetch all candles for a period, paginating through the HL API (500 per request).
pub async fn fetch_all_candles(
    client: &Client,
    coin: &str,
    interval: &str,
    start_ms: u64,
    end_ms: u64,
) -> Result<Vec<Candle>> {
    let interval_ms: u64 = match interval {
        "1m" => 60_000,
        "5m" => 300_000,
        "15m" => 900_000,
        "1h" => 3_600_000,
        "4h" => 14_400_000,
        _ => anyhow::bail!("unsupported interval: {}", interval),
    };

    let chunk_ms = 500 * interval_ms;
    let mut chunks = Vec::new();
    let mut cursor = start_ms;
    while cursor < end_ms {
        let chunk_end = (cursor + chunk_ms).min(end_ms);
        chunks.push((cursor, chunk_end));
        cursor = chunk_end;
    }

    let total = chunks.len();
    let mut all = Vec::new();

    // Sequential with pacing to avoid rate limits on large fetches.
    for (i, (start, end)) in chunks.into_iter().enumerate() {
        let candles = fetch_chunk(client, coin, interval, start, end).await?;
        all.extend(candles);
        if (i + 1) % 50 == 0 || i + 1 == total {
            eprint!("\r  {}/{} chunks fetched...", i + 1, total);
        }
        // Pace: ~100ms between requests to stay under rate limits.
        if i + 1 < total {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    eprintln!();

    all.sort_by_key(|c| c.t);
    all.dedup_by_key(|c| c.t);
    Ok(all)
}
