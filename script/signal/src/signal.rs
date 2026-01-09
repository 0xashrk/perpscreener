//! Signal scanner for HL_ALPHA recipe.
//!
//! Evaluates trading signals against live backend data.

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(name = "signal", about = "Evaluate HL_ALPHA trading signals")]
struct Args {
    /// Asset symbol (e.g., BTC, ETH)
    #[arg(long)]
    coin: String,

    /// Backend base URL
    #[arg(long, default_value = "http://localhost:30001")]
    backend: String,

    /// Profile override: auto, aggressive, balanced, conservative
    #[arg(long, default_value = "auto")]
    profile: String,
}

// ---------------------------------------------------------------------------
// Backend response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ChartSnapshot {
    #[allow(dead_code)]
    as_of_ms: u64,
    #[allow(dead_code)]
    coin: String,
    #[allow(dead_code)]
    interval: String,
    candles: Vec<Candle>,
}

#[derive(Debug, Deserialize)]
struct Candle {
    #[serde(rename = "t")]
    #[allow(dead_code)]
    open_time: u64,
    #[serde(rename = "T")]
    #[allow(dead_code)]
    close_time: u64,
    #[serde(rename = "o")]
    #[allow(dead_code)]
    open: f64,
    #[serde(rename = "h")]
    high: f64,
    #[serde(rename = "l")]
    low: f64,
    #[serde(rename = "c")]
    close: f64,
    #[serde(rename = "v")]
    #[allow(dead_code)]
    volume: f64,
    #[serde(rename = "n")]
    #[allow(dead_code)]
    num_trades: u64,
}

#[derive(Debug, Deserialize)]
struct L2BookSnapshot {
    #[allow(dead_code)]
    coin: String,
    #[allow(dead_code)]
    time: u64,
    levels: (Vec<L2BookLevel>, Vec<L2BookLevel>),
}

#[derive(Debug, Deserialize)]
struct L2BookLevel {
    px: String,
    sz: String,
    #[allow(dead_code)]
    n: u32,
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct SignalOutput {
    coin: String,
    timestamp: String,
    profile: String,
    indicators: Indicators,
    signals: Signals,
    recommendation: String,
}

#[derive(Debug, Serialize)]
struct Indicators {
    mid: f64,
    sma20: f64,
    sma50: f64,
    trend: String,
    donchian_hi: f64,
    donchian_lo: f64,
    atr: f64,
    atr_pct: f64,
    spread: f64,
    ob_imbalance: f64,
}

#[derive(Debug, Serialize)]
struct Signals {
    strong_long: bool,
    strong_short: bool,
}

// ---------------------------------------------------------------------------
// Profile params
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct ProfileParams {
    ob_long: f64,
    ob_short: f64,
    spread_max: f64,
}

impl ProfileParams {
    fn aggressive() -> Self {
        Self {
            ob_long: 1.05,
            ob_short: 0.95,
            spread_max: 0.0012,
        }
    }

    fn balanced() -> Self {
        Self {
            ob_long: 1.10,
            ob_short: 0.90,
            spread_max: 0.0008,
        }
    }

    fn conservative() -> Self {
        Self {
            ob_long: 1.15,
            ob_short: 0.87,
            spread_max: 0.0006,
        }
    }
}

// ---------------------------------------------------------------------------
// API client
// ---------------------------------------------------------------------------

struct BackendClient {
    base_url: String,
    client: reqwest::Client,
}

impl BackendClient {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn fetch_candles(&self, coin: &str, interval: &str, limit: usize) -> Result<Vec<Candle>> {
        let url = format!(
            "{}/chart?coin={}&interval={}&limit={}",
            self.base_url, coin, interval, limit
        );

        let resp: ChartSnapshot = self
            .client
            .get(&url)
            .send()
            .await
            .context("failed to fetch candles")?
            .json()
            .await
            .context("failed to parse candle response")?;

        Ok(resp.candles)
    }

    async fn fetch_orderbook(&self, coin: &str) -> Result<L2BookSnapshot> {
        let url = format!("{}/orderbook?coin={}", self.base_url, coin);

        let resp: L2BookSnapshot = self
            .client
            .get(&url)
            .send()
            .await
            .context("failed to fetch orderbook")?
            .json()
            .await
            .context("failed to parse orderbook response")?;

        Ok(resp)
    }
}

// ---------------------------------------------------------------------------
// Indicator calculations
// ---------------------------------------------------------------------------

fn calc_sma(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }
    let sum: f64 = candles.iter().rev().take(period).map(|c| c.close).sum();
    Some(sum / period as f64)
}

fn calc_donchian(candles: &[Candle], length: usize) -> Option<(f64, f64)> {
    if candles.len() < length {
        return None;
    }
    let window: Vec<&Candle> = candles.iter().rev().take(length).collect();
    let hi = window.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
    let lo = window.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
    Some((hi, lo))
}

fn calc_atr(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period + 1 {
        return None;
    }

    let mut tr_values = Vec::with_capacity(period);
    let recent: Vec<&Candle> = candles.iter().rev().take(period + 1).collect();

    for i in 0..period {
        let curr = recent[i];
        let prev = recent[i + 1];
        let tr = (curr.high - curr.low)
            .max((curr.high - prev.close).abs())
            .max((curr.low - prev.close).abs());
        tr_values.push(tr);
    }

    Some(tr_values.iter().sum::<f64>() / period as f64)
}

fn calc_orderbook_metrics(book: &L2BookSnapshot) -> Option<(f64, f64, f64)> {
    let (bids, asks) = &book.levels;

    if bids.is_empty() || asks.is_empty() {
        return None;
    }

    let best_bid: f64 = bids[0].px.parse().ok()?;
    let best_ask: f64 = asks[0].px.parse().ok()?;
    let mid = (best_bid + best_ask) / 2.0;
    let spread = (best_ask - best_bid) / mid;

    let bid_size: f64 = bids
        .iter()
        .take(10)
        .filter_map(|l| l.sz.parse::<f64>().ok())
        .sum();
    let ask_size: f64 = asks
        .iter()
        .take(10)
        .filter_map(|l| l.sz.parse::<f64>().ok())
        .sum();

    let imbalance = if ask_size > 0.0 {
        bid_size / ask_size
    } else {
        1.0
    };

    Some((mid, spread, imbalance))
}

// ---------------------------------------------------------------------------
// Profile selection
// ---------------------------------------------------------------------------

fn select_profile(profile_arg: &str, atr_pct: f64, spread: f64, trend_strength: f64) -> (String, ProfileParams) {
    match profile_arg.to_lowercase().as_str() {
        "aggressive" | "agg" => ("aggressive".to_string(), ProfileParams::aggressive()),
        "balanced" | "bal" => ("balanced".to_string(), ProfileParams::balanced()),
        "conservative" | "con" => ("conservative".to_string(), ProfileParams::conservative()),
        _ => {
            // Auto selection logic from recipe
            if atr_pct >= 0.06 || spread >= 0.0014 {
                ("conservative".to_string(), ProfileParams::conservative())
            } else if trend_strength >= 0.003 && (0.015..=0.05).contains(&atr_pct) && spread <= 0.0011 {
                ("aggressive".to_string(), ProfileParams::aggressive())
            } else {
                ("balanced".to_string(), ProfileParams::balanced())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = BackendClient::new(&args.backend);

    // Fetch data in parallel
    let (candles_4h, candles_1h, orderbook) = tokio::try_join!(
        client.fetch_candles(&args.coin, "4h", 60),
        client.fetch_candles(&args.coin, "1h", 45),
        client.fetch_orderbook(&args.coin),
    )?;

    // Drop most recent candle (use closed only)
    let closed_4h: Vec<Candle> = candles_4h.into_iter().rev().skip(1).rev().collect();
    let closed_1h: Vec<Candle> = candles_1h.into_iter().rev().skip(1).rev().collect();

    // Validate candle counts
    if closed_4h.len() < 55 {
        anyhow::bail!(
            "insufficient 4h candles: need 55, got {}",
            closed_4h.len()
        );
    }
    if closed_1h.len() < 40 {
        anyhow::bail!(
            "insufficient 1h candles: need 40, got {}",
            closed_1h.len()
        );
    }

    // Calculate indicators
    let sma20 = calc_sma(&closed_4h, 20).context("failed to calc SMA20")?;
    let sma50 = calc_sma(&closed_4h, 50).context("failed to calc SMA50")?;
    let (don_hi, don_lo) = calc_donchian(&closed_1h, 20).context("failed to calc Donchian")?;
    let atr = calc_atr(&closed_1h, 14).context("failed to calc ATR")?;
    let (mid, spread, ob_imbalance) =
        calc_orderbook_metrics(&orderbook).context("failed to calc orderbook metrics")?;

    let atr_pct = atr / mid;
    let trend_strength = (sma20 - sma50).abs() / mid;

    // Trend
    let bull = sma20 > sma50;
    let trend = if bull { "bull" } else { "bear" };

    // Profile selection
    let (profile_name, params) = select_profile(&args.profile, atr_pct, spread, trend_strength);

    // Signal evaluation
    let strong_long = bull && mid > don_hi && ob_imbalance >= params.ob_long && spread <= params.spread_max;
    let strong_short = !bull && mid < don_lo && ob_imbalance <= params.ob_short && spread <= params.spread_max;

    let recommendation = if strong_long {
        "LONG"
    } else if strong_short {
        "SHORT"
    } else {
        "NONE"
    };

    // Build output
    let output = SignalOutput {
        coin: args.coin,
        timestamp: Utc::now().to_rfc3339(),
        profile: profile_name,
        indicators: Indicators {
            mid,
            sma20,
            sma50,
            trend: trend.to_string(),
            donchian_hi: don_hi,
            donchian_lo: don_lo,
            atr,
            atr_pct,
            spread,
            ob_imbalance,
        },
        signals: Signals {
            strong_long,
            strong_short,
        },
        recommendation: recommendation.to_string(),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
