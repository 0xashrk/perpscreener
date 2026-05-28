use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const HL_API: &str = "https://api.hyperliquid.xyz/info";

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
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

pub async fn fetch_candles(
    client: &Client,
    coin: &str,
    start: u64,
    end: u64,
) -> Result<Vec<Candle>> {
    let req = CandleReq {
        req_type: "candleSnapshot".to_string(),
        req: CandleReqInner {
            coin: coin.to_string(),
            interval: "1m".to_string(),
            start_time: start,
            end_time: end,
        },
    };
    let resp = client
        .post(HL_API)
        .json(&req)
        .send()
        .await
        .context("candle request failed")?;
    let status = resp.status();
    let body = resp.text().await.context("candle read failed")?;
    if !status.is_success() {
        anyhow::bail!("candle API {}: {}", status, body);
    }
    let candles: Vec<Candle> =
        serde_json::from_str(&body).context("candle parse failed")?;
    Ok(candles)
}
