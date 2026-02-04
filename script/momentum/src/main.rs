use std::cmp::Ordering;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Timelike, Utc};
use clap::Parser;
use reqwest::Client;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(name = "momentum", about = "BTC intrahour momentum context (recipe)")]
struct Args {
    /// Asset symbol (e.g., BTC, ETH)
    #[arg(long)]
    coin: String,

    /// Backend base URL
    #[arg(long, default_value = "http://localhost:30001")]
    backend: String,

    /// Number of 1m candles to pull (must cover current hour)
    #[arg(long, default_value_t = 180)]
    limit: usize,
}

#[derive(Debug, Deserialize, Clone)]
struct ChartSnapshot {
    candles: Vec<Candle>,
}

#[derive(Debug, Deserialize, Clone)]
struct Candle {
    #[serde(rename = "t")]
    open_time: u64,
    #[serde(rename = "o")]
    open: f64,
    #[serde(rename = "h")]
    high: f64,
    #[serde(rename = "l")]
    low: f64,
    #[serde(rename = "c")]
    close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Flat,
}

impl Direction {
    fn as_str(self) -> &'static str {
        match self {
            Direction::Up => "UP",
            Direction::Down => "DOWN",
            Direction::Flat => "FLAT",
        }
    }
}

struct BackendClient {
    base_url: String,
    client: Client,
}

impl BackendClient {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    async fn fetch_candles(&self, coin: &str, limit: usize) -> Result<Vec<Candle>> {
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

fn floor_to_hour(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.date_naive()
        .and_hms_opt(dt.hour(), 0, 0)
        .unwrap()
        .and_local_timezone(Utc)
        .unwrap()
}

fn candle_direction(c: &Candle) -> Direction {
    if c.close > c.open {
        Direction::Up
    } else if c.close < c.open {
        Direction::Down
    } else {
        Direction::Flat
    }
}

fn direction_vs_open(current: f64, start: f64) -> Direction {
    match current.partial_cmp(&start).unwrap_or(Ordering::Equal) {
        Ordering::Greater => Direction::Up,
        Ordering::Less => Direction::Down,
        Ordering::Equal => Direction::Flat,
    }
}

fn ret_over_minutes(candles: &[Candle], minutes: usize) -> Option<f64> {
    if candles.len() <= minutes {
        return None;
    }
    let last = candles.last()?;
    let idx = candles.len().checked_sub(minutes + 1)?;
    let prior = candles.get(idx)?;
    Some(last.close / prior.close - 1.0)
}

fn trend_label(ret: Option<f64>) -> Direction {
    match ret {
        Some(r) if r.abs() >= 0.0002 => {
            if r > 0.0 {
                Direction::Up
            } else {
                Direction::Down
            }
        }
        _ => Direction::Flat,
    }
}

fn stddev(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let mean: f64 = values.iter().copied().sum::<f64>() / values.len() as f64;
    let var: f64 = values
        .iter()
        .map(|v| {
            let d = v - mean;
            d * d
        })
        .sum::<f64>()
        / (values.len() as f64 - 1.0);
    Some(var.sqrt())
}

fn log_return_series(candles: &[Candle]) -> Vec<f64> {
    candles
        .windows(2)
        .filter_map(|w| {
            let prev = &w[0];
            let curr = &w[1];
            if prev.close <= 0.0 || curr.close <= 0.0 {
                return None;
            }
            Some((curr.close / prev.close).ln())
        })
        .collect()
}

#[derive(Debug)]
struct Streaks {
    current: (Direction, usize),
    longest_up: usize,
    longest_down: usize,
}

fn compute_streaks(candles: &[Candle]) -> Option<Streaks> {
    if candles.is_empty() {
        return None;
    }

    let mut runs: Vec<(Direction, usize)> = Vec::new();
    for candle in candles {
        let dir = candle_direction(candle);
        match runs.last_mut() {
            Some((last_dir, len)) if *last_dir == dir => *len += 1,
            _ => runs.push((dir, 1)),
        }
    }

    let current = *runs.last()?;
    let mut longest_up = 0;
    let mut longest_down = 0;
    for (dir, len) in &runs {
        match dir {
            Direction::Up => longest_up = longest_up.max(*len),
            Direction::Down => longest_down = longest_down.max(*len),
            Direction::Flat => {}
        }
    }

    Some(Streaks {
        current,
        longest_up,
        longest_down,
    })
}

fn trend_strength(ret5: Option<f64>, ret15: Option<f64>, vol: Option<f64>, regime: &str) -> u64 {
    let mag = match (ret5, ret15) {
        (Some(a), Some(b)) => (a.abs() + b.abs()) / 2.0,
        (Some(a), None) | (None, Some(a)) => a.abs(),
        _ => 0.0,
    };

    let mut strength = (mag * 10_000.0).clamp(0.0, 100.0);

    if regime == "TRENDING" {
        strength = (strength + 10.0).min(100.0);
    } else if regime == "CHOPPY" {
        strength = (strength - 15.0).max(0.0);
    }

    if let Some(v) = vol {
        // Penalize realized volatility; typical vol ~0.001 => subtract ~5
        let penalty = (v * 5_000.0).min(30.0);
        strength = (strength - penalty).max(0.0);
    }

    strength.round() as u64
}

fn format_pct(v: f64) -> String {
    format!("{:.4}%", v * 100.0)
}

fn build_data_quality(alignment_ok: bool, has_gaps: bool, missing_candles: bool) -> String {
    let mut issues = Vec::new();
    if !alignment_ok {
        issues.push("alignment warning");
    }
    if has_gaps {
        issues.push("gaps");
    }
    if missing_candles {
        issues.push("missing candles");
    }

    if issues.is_empty() {
        "OK".to_string()
    } else {
        issues.join("; ")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = BackendClient::new(&args.backend);

    let now = Utc::now();
    let start_time = floor_to_hour(now);
    let start_ms = u64::try_from(start_time.timestamp_millis())
        .context("start time millis negative")?;
    let now_ms = u64::try_from(now.timestamp_millis()).context("now millis negative")?;

    let mut candles = client.fetch_candles(&args.coin, args.limit).await?;
    candles.sort_by_key(|c| c.open_time);

    let window: Vec<Candle> = candles
        .into_iter()
        .filter(|c| c.open_time >= start_ms && c.open_time <= now_ms)
        .collect();

    let elapsed_minutes = ((now_ms - start_ms) / 60_000) as usize;
    let expected_candles = elapsed_minutes + 1; // inclusive of the starting minute

    if window.len() < expected_candles {
        return Err(anyhow!(
            "insufficient candles in current hour: expected at least {}, got {}",
            expected_candles,
            window.len()
        ));
    }

    let first = &window[0];
    let last = &window[window.len() - 1];
    let alignment_ok = first.open_time == start_ms;

    let has_gaps = window
        .windows(2)
        .any(|w| w[1].open_time != w[0].open_time + 60_000);

    let price_to_beat = first.open;
    let current_price = last.close;
    let delta_price = current_price - price_to_beat;
    let delta_pct = delta_price / price_to_beat;
    let direction_vs_open = direction_vs_open(current_price, price_to_beat);

    let ret_5m = ret_over_minutes(&window, 5);
    let ret_15m = ret_over_minutes(&window, 15);

    let trend_5m = trend_label(ret_5m);
    let trend_15m = trend_label(ret_15m);

    let trend_regime = match (trend_5m, trend_15m) {
        (Direction::Flat, Direction::Flat) => "DRIFT/FLAT",
        (a, b) if a == b && a != Direction::Flat => "TRENDING",
        _ => "CHOPPY",
    };

    let vol_series = log_return_series(&window);
    let vol_1m = stddev(&vol_series);

    let strength = trend_strength(ret_5m, ret_15m, vol_1m, trend_regime);

    let proj_5m = ret_5m.map(|r| current_price * (1.0 + r));
    let proj_15m = ret_15m.map(|r| current_price * (1.0 + r));
    let target_band = match (proj_5m, proj_15m) {
        (Some(a), Some(b)) => Some((a.min(b), a.max(b))),
        _ => None,
    };

    let streaks = compute_streaks(&window).context("failed to compute streaks")?;

    let window_high = window.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let window_low = window.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let range_pct = (window_high - window_low) / price_to_beat;

    let missing_candles = window.len() < expected_candles;
    let data_quality = build_data_quality(alignment_ok, has_gaps, missing_candles);

    // Agreement signal per recipe
    let agreement = match (direction_vs_open, trend_regime, trend_5m) {
        (Direction::Up, "TRENDING", Direction::Up) => "CONTINUATION UP",
        (Direction::Down, "TRENDING", Direction::Down) => "CONTINUATION DOWN",
        (Direction::Up, _, Direction::Down) => "PULLBACK RISK (up hour, down micro)",
        (Direction::Down, _, Direction::Up) => "RECLAIM RISK (down hour, up micro)",
        (_, "CHOPPY", _) => "RANGE/FAKEOUTS LIKELY",
        _ => "NEUTRAL",
    };

    let mut out = String::new();

    writeln!(
        &mut out,
        "Vs hour-open: {} by {:.4} ({})",
        direction_vs_open.as_str(),
        delta_price,
        format_pct(delta_pct)
    )?;
    writeln!(
        &mut out,
        "Trend: 5m={} ({}), 15m={} ({}) → {} strength={}/100",
        trend_5m.as_str(),
        ret_5m.map(format_pct).unwrap_or_else(|| "n/a".to_string()),
        trend_15m.as_str(),
        ret_15m.map(format_pct).unwrap_or_else(|| "n/a".to_string()),
        trend_regime,
        strength
    )?;
    if let Some((lo, hi)) = target_band {
        writeln!(&mut out, "Target band (5–15m): {:.4} to {:.4}", lo, hi)?;
    } else {
        writeln!(&mut out, "Target band (5–15m): n/a")?;
    }
    writeln!(&mut out, "Agreement signal: {}", agreement)?;
    writeln!(&mut out)?;

    writeln!(&mut out, "| Field | Value |")?;
    writeln!(&mut out, "|---|---|")?;
    writeln!(&mut out, "| start_time_utc | {} |", start_time.to_rfc3339())?;
    writeln!(&mut out, "| now_utc | {} |", now.to_rfc3339())?;
    writeln!(
        &mut out,
        "| price_to_beat (open @ start) | {:.4} |",
        price_to_beat
    )?;
    writeln!(&mut out, "| current_price | {:.4} |", current_price)?;
    writeln!(
        &mut out,
        "| direction_vs_open | {} |",
        direction_vs_open.as_str()
    )?;
    writeln!(&mut out, "| delta_price | {:.4} |", delta_price)?;
    writeln!(&mut out, "| delta_pct | {} |", format_pct(delta_pct))?;
    writeln!(
        &mut out,
        "| ret_5m | {} |",
        ret_5m.map(format_pct).unwrap_or_else(|| "n/a".to_string())
    )?;
    writeln!(&mut out, "| trend_5m | {} |", trend_5m.as_str())?;
    writeln!(
        &mut out,
        "| ret_15m | {} |",
        ret_15m.map(format_pct).unwrap_or_else(|| "n/a".to_string())
    )?;
    writeln!(&mut out, "| trend_15m | {} |", trend_15m.as_str())?;
    writeln!(&mut out, "| trend_regime | {} |", trend_regime)?;
    writeln!(&mut out, "| trend_strength (0..100) | {} |", strength)?;
    match target_band {
        Some((lo, hi)) => writeln!(
            &mut out,
            "| target_band (5–15m) | [{:.4}, {:.4}] |",
            lo,
            hi
        )?,
        None => writeln!(&mut out, "| target_band (5–15m) | n/a |")?,
    }
    writeln!(
        &mut out,
        "| current_streak | {}×{} |",
        streaks.current.0.as_str(),
        streaks.current.1
    )?;
    writeln!(&mut out, "| longest_up_streak | {} |", streaks.longest_up)?;
    writeln!(&mut out, "| longest_down_streak | {} |", streaks.longest_down)?;
    writeln!(
        &mut out,
        "| vol_1m | {} |",
        vol_1m
            .map(|v| format!("{:.6}", v))
            .unwrap_or_else(|| "n/a".to_string())
    )?;
    writeln!(&mut out, "| window_high | {:.4} |", window_high)?;
    writeln!(&mut out, "| window_low | {:.4} |", window_low)?;
    writeln!(&mut out, "| range_pct | {} |", format_pct(range_pct))?;
    writeln!(&mut out, "| data_quality | {} |", data_quality)?;

    // Write output both to stdout and to file under script/momentum/momentum.txt
    print!("{}", out);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("momentum.txt");
    fs::write(&path, &out)
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}
