use std::cmp::Ordering;

use anyhow::{Context, Result};
use chrono::{DateTime, Timelike, Utc};

use crate::client::Candle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Flat,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Up => "UP",
            Direction::Down => "DOWN",
            Direction::Flat => "FLAT",
        }
    }
}

#[derive(Debug)]
pub struct Streaks {
    pub current: (Direction, usize),
    pub longest_up: usize,
    pub longest_down: usize,
}

pub struct MomentumResult {
    pub coin: String,
    pub start_time: DateTime<Utc>,
    pub now: DateTime<Utc>,
    pub price_to_beat: f64,
    pub current_price: f64,
    pub direction_vs_open: Direction,
    pub delta_price: f64,
    pub delta_pct: f64,
    pub ret_5m: Option<f64>,
    pub trend_5m: Direction,
    pub ret_15m: Option<f64>,
    pub trend_15m: Direction,
    pub trend_regime: &'static str,
    pub strength: u64,
    pub target_band: Option<(f64, f64)>,
    pub streaks: Streaks,
    pub vol_1m: Option<f64>,
    pub window_high: f64,
    pub window_low: f64,
    pub range_pct: f64,
    pub data_quality: String,
    pub agreement: &'static str,
}

pub fn floor_to_hour(dt: DateTime<Utc>) -> DateTime<Utc> {
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
        let penalty = (v * 5_000.0).min(30.0);
        strength = (strength - penalty).max(0.0);
    }

    strength.round() as u64
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

pub fn format_pct(v: f64) -> String {
    format!("{:.4}%", v * 100.0)
}

/// Run momentum analysis on a set of 1m candles for a single asset.
pub fn compute_momentum(
    coin: &str,
    candles: Vec<Candle>,
    now: DateTime<Utc>,
    start_time: DateTime<Utc>,
) -> Result<MomentumResult> {
    let start_ms =
        u64::try_from(start_time.timestamp_millis()).context("start time millis negative")?;
    let now_ms = u64::try_from(now.timestamp_millis()).context("now millis negative")?;

    let mut sorted = candles;
    sorted.sort_by_key(|c| c.open_time);

    let window: Vec<Candle> = sorted
        .into_iter()
        .filter(|c| c.open_time >= start_ms && c.open_time <= now_ms)
        .collect();

    if window.is_empty() {
        anyhow::bail!("no candles in current hour for {}", coin);
    }

    let elapsed_minutes = ((now_ms - start_ms) / 60_000) as usize;
    let expected_candles = elapsed_minutes + 1;

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
    let dir = direction_vs_open(current_price, price_to_beat);

    let ret_5m = ret_over_minutes(&window, 5);
    let ret_15m = ret_over_minutes(&window, 15);
    let t5 = trend_label(ret_5m);
    let t15 = trend_label(ret_15m);

    let regime: &'static str = match (t5, t15) {
        (Direction::Flat, Direction::Flat) => "DRIFT/FLAT",
        (a, b) if a == b && a != Direction::Flat => "TRENDING",
        _ => "CHOPPY",
    };

    let vol_series = log_return_series(&window);
    let vol_1m = stddev(&vol_series);
    let strength = trend_strength(ret_5m, ret_15m, vol_1m, regime);

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

    let agreement: &'static str = match (dir, regime, t5) {
        (Direction::Up, "TRENDING", Direction::Up) => "CONTINUATION UP",
        (Direction::Down, "TRENDING", Direction::Down) => "CONTINUATION DOWN",
        (Direction::Up, _, Direction::Down) => "PULLBACK RISK",
        (Direction::Down, _, Direction::Up) => "RECLAIM RISK",
        (_, "CHOPPY", _) => "RANGE/FAKEOUTS",
        _ => "NEUTRAL",
    };

    Ok(MomentumResult {
        coin: coin.to_string(),
        start_time,
        now,
        price_to_beat,
        current_price,
        direction_vs_open: dir,
        delta_price,
        delta_pct,
        ret_5m,
        trend_5m: t5,
        ret_15m,
        trend_15m: t15,
        trend_regime: regime,
        strength,
        target_band,
        streaks,
        vol_1m,
        window_high,
        window_low,
        range_pct,
        data_quality,
        agreement,
    })
}
