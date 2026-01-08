use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(name = "backtest")]
#[command(about = "Backtest tool for trading recipes")]
struct Args {
    /// Asset symbol (e.g., BTC, ETH)
    #[arg(short, long)]
    coin: String,

    /// Lookback period in hours
    #[arg(short = 'H', long, default_value = "12")]
    hours: u32,

    /// Candle interval for scanning (1m, 5m, 15m)
    #[arg(short, long, default_value = "1m")]
    scan_interval: String,

    /// Comma-separated SMA periods (calculated on 4h)
    #[arg(long, default_value = "20,50")]
    sma_periods: String,

    /// Donchian channel length (calculated on 1h)
    #[arg(long, default_value = "20")]
    donchian_len: u8,

    /// ATR period (calculated on 1h)
    #[arg(long, default_value = "14")]
    atr_period: u8,

    /// Include per-candle scan data in output
    #[arg(long, default_value = "false")]
    include_scans: bool,
}

// ============================================================================
// API Types
// ============================================================================

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

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct Candle {
    t: u64,      // open time
    #[serde(rename = "T")]
    t_close: u64, // close time
    #[serde(deserialize_with = "deserialize_f64")]
    o: f64,      // open
    #[serde(deserialize_with = "deserialize_f64")]
    h: f64,      // high
    #[serde(deserialize_with = "deserialize_f64")]
    l: f64,      // low
    #[serde(deserialize_with = "deserialize_f64")]
    c: f64,      // close
    #[serde(deserialize_with = "deserialize_f64")]
    v: f64,      // volume
    n: u32,      // number of trades
}

#[derive(Debug, Serialize)]
struct L2BookRequest {
    #[serde(rename = "type")]
    request_type: String,
    coin: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct L2BookResponse {
    coin: String,
    time: u64,
    levels: Vec<Vec<L2BookLevel>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct L2BookLevel {
    #[serde(deserialize_with = "deserialize_f64")]
    px: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    sz: f64,
    n: u32,
}

fn deserialize_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct F64Visitor;

    impl<'de> Visitor<'de> for F64Visitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number or string")
        }

        fn visit_f64<E>(self, v: f64) -> Result<f64, E> {
            Ok(v)
        }

        fn visit_i64<E>(self, v: i64) -> Result<f64, E> {
            Ok(v as f64)
        }

        fn visit_u64<E>(self, v: u64) -> Result<f64, E> {
            Ok(v as f64)
        }

        fn visit_str<E>(self, v: &str) -> Result<f64, E>
        where
            E: de::Error,
        {
            v.parse::<f64>().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(F64Visitor)
}

// ============================================================================
// Output Types
// ============================================================================

#[derive(Debug, Serialize)]
struct BacktestOutput {
    coin: String,
    generated_at: String,
    params: BacktestParams,
    data: DataCounts,
    orderbook: OrderbookData,
    indicators: Indicators,
    derived: DerivedValues,
    price_range: PriceRange,
    summary: Summary,
    #[serde(skip_serializing_if = "Option::is_none")]
    scans: Option<Vec<ScanPoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct BacktestParams {
    hours: u32,
    scan_interval: String,
    sma_periods: Vec<u32>,
    donchian_len: u8,
    atr_period: u8,
}

#[derive(Debug, Serialize)]
struct DataCounts {
    candles_scan: usize,
    candles_1h: usize,
    candles_4h: usize,
}

#[derive(Debug, Serialize)]
struct OrderbookData {
    time: u64,
    bid: f64,
    ask: f64,
    mid: f64,
    spread: f64,
    spread_pct: f64,
    ob_imbalance: f64,
}

#[derive(Debug, Serialize)]
struct Indicators {
    #[serde(flatten)]
    smas: HashMap<String, f64>,
    don_hi_1h: f64,
    don_lo_1h: f64,
    atr_1h: f64,
}

#[derive(Debug, Serialize)]
struct DerivedValues {
    bull: bool,
    trend_strength: f64,
    atr_pct: f64,
    current_vs_don_hi: f64,
    current_vs_don_lo: f64,
}

#[derive(Debug, Serialize)]
struct PriceRange {
    low: f64,
    high: f64,
    current: f64,
}

#[derive(Debug, Serialize)]
struct Summary {
    long_breakouts: usize,
    short_breakouts: usize,
    first_long_breakout_ts: Option<u64>,
    first_short_breakout_ts: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ScanPoint {
    ts: u64,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    don_hi: f64,
    don_lo: f64,
    breakout_long: bool,
    breakout_short: bool,
}

// ============================================================================
// API Client
// ============================================================================

const HYPERLIQUID_API_URL: &str = "https://api.hyperliquid.xyz/info";

async fn fetch_candles(
    client: &reqwest::Client,
    coin: &str,
    interval: &str,
    start_time: u64,
    end_time: u64,
) -> Result<Vec<Candle>> {
    let request = CandleRequest {
        request_type: "candleSnapshot".to_string(),
        req: CandleRequestInner {
            coin: coin.to_string(),
            interval: interval.to_string(),
            start_time,
            end_time,
        },
    };

    let response = client
        .post(HYPERLIQUID_API_URL)
        .json(&request)
        .send()
        .await
        .context("Failed to send candle request")?;

    let status = response.status();
    let body = response.text().await.context("Failed to read response")?;

    if !status.is_success() {
        anyhow::bail!("API error ({}): {}", status, body);
    }

    let candles: Vec<Candle> = serde_json::from_str(&body)
        .context(format!("Failed to parse candles: {}", body))?;

    Ok(candles)
}

async fn fetch_orderbook(client: &reqwest::Client, coin: &str) -> Result<L2BookResponse> {
    let request = L2BookRequest {
        request_type: "l2Book".to_string(),
        coin: coin.to_string(),
    };

    let response = client
        .post(HYPERLIQUID_API_URL)
        .json(&request)
        .send()
        .await
        .context("Failed to send orderbook request")?;

    let status = response.status();
    let body = response.text().await.context("Failed to read response")?;

    if !status.is_success() {
        anyhow::bail!("API error ({}): {}", status, body);
    }

    let orderbook: L2BookResponse = serde_json::from_str(&body)
        .context(format!("Failed to parse orderbook: {}", body))?;

    Ok(orderbook)
}

// ============================================================================
// Indicator Calculations
// ============================================================================

fn compute_sma(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period {
        return None;
    }
    let sum: f64 = closes[closes.len() - period..].iter().sum();
    Some(sum / period as f64)
}

fn compute_donchian(candles: &[Candle], length: usize) -> Option<(f64, f64)> {
    if candles.len() < length {
        return None;
    }
    let recent = &candles[candles.len() - length..];
    let hi = recent.iter().map(|c| c.h).fold(f64::MIN, f64::max);
    let lo = recent.iter().map(|c| c.l).fold(f64::MAX, f64::min);
    Some((hi, lo))
}

fn compute_atr(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period + 1 {
        return None;
    }

    let mut trs: Vec<f64> = Vec::with_capacity(candles.len() - 1);

    for i in 1..candles.len() {
        let h = candles[i].h;
        let l = candles[i].l;
        let pc = candles[i - 1].c;
        let tr = (h - l).max((h - pc).abs()).max((l - pc).abs());
        trs.push(tr);
    }

    if trs.len() < period {
        return None;
    }

    // Simple average for ATR (could use Wilder's smoothing for more accuracy)
    let sum: f64 = trs[trs.len() - period..].iter().sum();
    Some(sum / period as f64)
}

fn process_orderbook(ob: &L2BookResponse) -> OrderbookData {
    let bids = ob.levels.first().map(|v| v.as_slice()).unwrap_or(&[]);
    let asks = ob.levels.get(1).map(|v| v.as_slice()).unwrap_or(&[]);

    let bid = bids.first().map(|l| l.px).unwrap_or(0.0);
    let ask = asks.first().map(|l| l.px).unwrap_or(0.0);
    let mid = (bid + ask) / 2.0;
    let spread = ask - bid;
    let spread_pct = if mid > 0.0 { (spread / mid) * 100.0 } else { 0.0 };

    // OB imbalance: sum of bid sizes / sum of ask sizes (top 10 levels)
    let bid_sz: f64 = bids.iter().take(10).map(|l| l.sz).sum();
    let ask_sz: f64 = asks.iter().take(10).map(|l| l.sz).sum();
    let ob_imbalance = if ask_sz > 0.0 { bid_sz / ask_sz } else { 0.0 };

    OrderbookData {
        time: ob.time,
        bid,
        ask,
        mid,
        spread,
        spread_pct,
        ob_imbalance,
    }
}

// ============================================================================
// Scan Logic
// ============================================================================

fn scan_breakouts(
    scan_candles: &[Candle],
    hourly_candles: &[Candle],
    donchian_len: usize,
    start_ts: u64,
) -> (Vec<ScanPoint>, Summary) {
    let mut scans = Vec::new();
    let mut long_breakouts = 0usize;
    let mut short_breakouts = 0usize;
    let mut first_long_ts: Option<u64> = None;
    let mut first_short_ts: Option<u64> = None;

    for candle in scan_candles {
        if candle.t < start_ts {
            continue;
        }

        // Find 1h candles closed before this candle's timestamp
        // A candle is closed when current time > candle.t_close
        let closed_hourly: Vec<&Candle> = hourly_candles
            .iter()
            .filter(|h| h.t_close < candle.t)
            .collect();

        if closed_hourly.len() < donchian_len {
            continue;
        }

        // Calculate Donchian from closed hourly candles
        let recent: Vec<&Candle> = closed_hourly
            .iter()
            .rev()
            .take(donchian_len)
            .copied()
            .collect();

        let don_hi = recent.iter().map(|c| c.h).fold(f64::MIN, f64::max);
        let don_lo = recent.iter().map(|c| c.l).fold(f64::MAX, f64::min);

        let mid = (candle.h + candle.l) / 2.0;
        let breakout_long = mid > don_hi;
        let breakout_short = mid < don_lo;

        if breakout_long {
            long_breakouts += 1;
            if first_long_ts.is_none() {
                first_long_ts = Some(candle.t);
            }
        }
        if breakout_short {
            short_breakouts += 1;
            if first_short_ts.is_none() {
                first_short_ts = Some(candle.t);
            }
        }

        scans.push(ScanPoint {
            ts: candle.t,
            o: candle.o,
            h: candle.h,
            l: candle.l,
            c: candle.c,
            don_hi,
            don_lo,
            breakout_long,
            breakout_short,
        });
    }

    let summary = Summary {
        long_breakouts,
        short_breakouts,
        first_long_breakout_ts: first_long_ts,
        first_short_breakout_ts: first_short_ts,
    };

    (scans, summary)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse SMA periods
    let sma_periods: Vec<u32> = args
        .sma_periods
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let client = reqwest::Client::new();
    let now = Utc::now();
    let now_ms = now.timestamp_millis() as u64;

    // Calculate time ranges
    let scan_hours = args.hours as u64;
    let scan_start = now_ms - (scan_hours * 60 * 60 * 1000);

    // For 4h candles, need enough for largest SMA (14 days worth)
    let max_sma = sma_periods.iter().max().copied().unwrap_or(50) as u64;
    let candles_4h_start = now_ms - (max_sma * 4 * 60 * 60 * 1000 + 24 * 60 * 60 * 1000);

    // For 1h candles, need enough for Donchian + ATR + buffer (5 days)
    let candles_1h_start = now_ms - (5 * 24 * 60 * 60 * 1000);

    eprintln!("Fetching data for {}...", args.coin);

    // Fetch all data in parallel
    let (scan_result, hourly_result, four_hour_result, ob_result) = tokio::join!(
        fetch_candles(&client, &args.coin, &args.scan_interval, scan_start, now_ms),
        fetch_candles(&client, &args.coin, "1h", candles_1h_start, now_ms),
        fetch_candles(&client, &args.coin, "4h", candles_4h_start, now_ms),
        fetch_orderbook(&client, &args.coin),
    );

    let scan_candles = scan_result.context("Failed to fetch scan candles")?;
    let hourly_candles = hourly_result.context("Failed to fetch 1h candles")?;
    let four_hour_candles = four_hour_result.context("Failed to fetch 4h candles")?;
    let orderbook = ob_result.context("Failed to fetch orderbook")?;

    eprintln!(
        "Fetched: {} {} candles, {} 1h candles, {} 4h candles",
        scan_candles.len(),
        args.scan_interval,
        hourly_candles.len(),
        four_hour_candles.len()
    );

    // Drop most recent candle (not closed) for indicator calculations
    let closed_hourly: Vec<Candle> = if hourly_candles.len() > 1 {
        hourly_candles[..hourly_candles.len() - 1].to_vec()
    } else {
        hourly_candles.clone()
    };

    let closed_4h: Vec<Candle> = if four_hour_candles.len() > 1 {
        four_hour_candles[..four_hour_candles.len() - 1].to_vec()
    } else {
        four_hour_candles.clone()
    };

    // Calculate indicators
    let closes_4h: Vec<f64> = closed_4h.iter().map(|c| c.c).collect();

    let mut smas: HashMap<String, f64> = HashMap::new();
    for period in &sma_periods {
        if let Some(sma) = compute_sma(&closes_4h, *period as usize) {
            smas.insert(format!("sma{}_4h", period), sma);
        }
    }

    let (don_hi, don_lo) = compute_donchian(&closed_hourly, args.donchian_len as usize)
        .unwrap_or((0.0, 0.0));

    let atr = compute_atr(&closed_hourly, args.atr_period as usize).unwrap_or(0.0);

    // Process orderbook
    let ob_data = process_orderbook(&orderbook);

    // Calculate derived values
    let sma20 = smas.get("sma20_4h").copied().unwrap_or(0.0);
    let sma50 = smas.get("sma50_4h").copied().unwrap_or(0.0);
    let bull = sma20 > sma50;
    let trend_strength = if ob_data.mid > 0.0 {
        (sma20 - sma50).abs() / ob_data.mid
    } else {
        0.0
    };
    let atr_pct = if ob_data.mid > 0.0 { atr / ob_data.mid } else { 0.0 };
    let current_vs_don_hi = if don_hi > 0.0 {
        (ob_data.mid - don_hi) / don_hi
    } else {
        0.0
    };
    let current_vs_don_lo = if don_lo > 0.0 {
        (ob_data.mid - don_lo) / don_lo
    } else {
        0.0
    };

    // Scan for breakouts
    let (scans, summary) = scan_breakouts(
        &scan_candles,
        &hourly_candles,
        args.donchian_len as usize,
        scan_start,
    );

    // Calculate price range from scan candles
    let scan_after_start: Vec<&Candle> = scan_candles
        .iter()
        .filter(|c| c.t >= scan_start)
        .collect();

    let price_low = scan_after_start.iter().map(|c| c.l).fold(f64::MAX, f64::min);
    let price_high = scan_after_start.iter().map(|c| c.h).fold(f64::MIN, f64::max);
    let current_price = scan_candles.last().map(|c| c.c).unwrap_or(0.0);

    // Build output
    let output = BacktestOutput {
        coin: args.coin,
        generated_at: now.to_rfc3339(),
        params: BacktestParams {
            hours: args.hours,
            scan_interval: args.scan_interval,
            sma_periods,
            donchian_len: args.donchian_len,
            atr_period: args.atr_period,
        },
        data: DataCounts {
            candles_scan: scan_candles.len(),
            candles_1h: hourly_candles.len(),
            candles_4h: four_hour_candles.len(),
        },
        orderbook: ob_data,
        indicators: Indicators {
            smas,
            don_hi_1h: don_hi,
            don_lo_1h: don_lo,
            atr_1h: atr,
        },
        derived: DerivedValues {
            bull,
            trend_strength,
            atr_pct,
            current_vs_don_hi,
            current_vs_don_lo,
        },
        price_range: PriceRange {
            low: price_low,
            high: price_high,
            current: current_price,
        },
        summary,
        scans: if args.include_scans { Some(scans) } else { None },
        error: None,
    };

    // Print JSON output
    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);

    Ok(())
}
