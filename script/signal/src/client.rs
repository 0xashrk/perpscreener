use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const HL_API: &str = "https://api.hyperliquid.xyz/info";

// -- Candle ------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Candle {
    pub t: u64,
    #[serde(rename = "T")]
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
    pub v: f64,
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

// -- Candle fetching ---------------------------------------------------------

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
    let resp = client
        .post(HL_API)
        .json(&req)
        .send()
        .await
        .context("candle request failed")?;
    let status = resp.status();
    let body = resp.text().await.context("candle read failed")?;
    if !status.is_success() {
        anyhow::bail!("candle API {} {}: {}", coin, status, body);
    }
    let candles: Vec<Candle> = serde_json::from_str(&body)
        .with_context(|| format!("candle parse {} failed", coin))?;
    Ok(candles)
}

// -- L2 Book -----------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct L2BookResponse {
    pub levels: Vec<Vec<L2Level>>,
}

#[derive(Debug, Deserialize)]
pub struct L2Level {
    #[serde(deserialize_with = "de_f64")]
    pub px: f64,
    #[serde(deserialize_with = "de_f64")]
    pub sz: f64,
    #[allow(dead_code)]
    pub n: u32,
}

pub async fn fetch_l2_book(client: &Client, coin: &str) -> Result<L2BookResponse> {
    let body = serde_json::json!({"type": "l2Book", "coin": coin});
    let resp = client
        .post(HL_API)
        .json(&body)
        .send()
        .await
        .context("l2Book request failed")?;
    let status = resp.status();
    let text = resp.text().await.context("l2Book read failed")?;
    if !status.is_success() {
        anyhow::bail!("l2Book {} {}: {}", coin, status, text);
    }
    serde_json::from_str(&text).context("l2Book parse failed")
}

// -- Top assets by volume ----------------------------------------------------

#[derive(Deserialize)]
struct MetaResponse {
    universe: Vec<AssetInfo>,
}

#[derive(Deserialize)]
struct AssetInfo {
    name: String,
    #[serde(rename = "maxLeverage")]
    max_leverage: u32,
}

#[derive(Deserialize)]
struct AssetCtx {
    #[serde(rename = "dayNtlVlm", deserialize_with = "de_f64")]
    day_ntl_vlm: f64,
}

/// Asset name + max leverage allowed on HL.
pub struct AssetMeta {
    pub name: String,
    pub max_leverage: u32,
}

async fn fetch_meta(client: &Client) -> Result<Vec<(AssetInfo, AssetCtx)>> {
    let body = serde_json::json!({"type": "metaAndAssetCtxs"});
    let resp = client
        .post(HL_API)
        .json(&body)
        .send()
        .await
        .context("metaAndAssetCtxs failed")?;
    let parsed: Value = resp.json().await.context("metaAndAssetCtxs parse failed")?;
    let arr = parsed.as_array().context("expected array")?;
    if arr.len() < 2 {
        return Err(anyhow!("metaAndAssetCtxs: expected 2 elements"));
    }
    let meta: MetaResponse = serde_json::from_value(arr[0].clone())?;
    let ctxs: Vec<AssetCtx> = serde_json::from_value(arr[1].clone())?;
    Ok(meta.universe.into_iter().zip(ctxs).collect())
}

pub async fn fetch_top_assets(client: &Client, n: usize) -> Result<Vec<AssetMeta>> {
    let pairs = fetch_meta(client).await?;
    let mut assets: Vec<(AssetMeta, f64)> = pairs
        .into_iter()
        .map(|(info, ctx)| {
            (
                AssetMeta {
                    name: info.name,
                    max_leverage: info.max_leverage,
                },
                ctx.day_ntl_vlm,
            )
        })
        .collect();
    assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    assets.truncate(n);
    Ok(assets.into_iter().map(|(m, _)| m).collect())
}

pub async fn fetch_asset_max_leverage(client: &Client, coin: &str) -> Result<u32> {
    let pairs = fetch_meta(client).await?;
    for (info, _) in pairs {
        if info.name == coin {
            return Ok(info.max_leverage);
        }
    }
    Ok(50) // fallback
}
