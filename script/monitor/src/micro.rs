use std::cmp::Ordering;

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

/// Snapshot of intrahour momentum state at a given moment.
pub struct MicroSnapshot {
    pub price: f64,
    pub direction_vs_open: Direction,
    pub delta_pct: f64,
    pub trend_5m: Direction,
    pub trend_15m: Direction,
    pub trend_regime: &'static str,
    pub strength: u64,
    pub agreement: &'static str,
    pub vol_1m: Option<f64>,
}

pub fn floor_to_hour(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.date_naive()
        .and_hms_opt(dt.hour(), 0, 0)
        .unwrap()
        .and_local_timezone(Utc)
        .unwrap()
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
    let prior = candles.get(candles.len() - minutes - 1)?;
    Some(last.c / prior.c - 1.0)
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

fn log_return_stddev(candles: &[Candle]) -> Option<f64> {
    let rets: Vec<f64> = candles
        .windows(2)
        .filter_map(|w| {
            if w[0].c <= 0.0 || w[1].c <= 0.0 {
                return None;
            }
            Some((w[1].c / w[0].c).ln())
        })
        .collect();
    if rets.len() < 2 {
        return None;
    }
    let mean = rets.iter().sum::<f64>() / rets.len() as f64;
    let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (rets.len() as f64 - 1.0);
    Some(var.sqrt())
}

fn trend_strength(ret5: Option<f64>, ret15: Option<f64>, vol: Option<f64>, regime: &str) -> u64 {
    let mag = match (ret5, ret15) {
        (Some(a), Some(b)) => (a.abs() + b.abs()) / 2.0,
        (Some(a), None) | (None, Some(a)) => a.abs(),
        _ => 0.0,
    };
    let mut s = (mag * 10_000.0).clamp(0.0, 100.0);
    if regime == "TRENDING" {
        s = (s + 10.0).min(100.0);
    } else if regime == "CHOPPY" {
        s = (s - 15.0).max(0.0);
    }
    if let Some(v) = vol {
        s = (s - (v * 5_000.0).min(30.0)).max(0.0);
    }
    s.round() as u64
}

pub fn snapshot(candles: &[Candle], now: DateTime<Utc>) -> Option<MicroSnapshot> {
    let start_time = floor_to_hour(now);
    let start_ms = start_time.timestamp_millis() as u64;
    let now_ms = now.timestamp_millis() as u64;

    let mut window: Vec<Candle> = candles
        .iter()
        .filter(|c| c.t >= start_ms && c.t <= now_ms)
        .cloned()
        .collect();
    window.sort_by_key(|c| c.t);

    if window.is_empty() {
        return None;
    }

    let first = &window[0];
    let last = &window[window.len() - 1];
    let price = last.c;
    let delta_pct = (price - first.o) / first.o;
    let dir = direction_vs_open(price, first.o);

    let ret_5m = ret_over_minutes(&window, 5);
    let ret_15m = ret_over_minutes(&window, 15);
    let t5 = trend_label(ret_5m);
    let t15 = trend_label(ret_15m);

    let regime: &'static str = match (t5, t15) {
        (Direction::Flat, Direction::Flat) => "DRIFT/FLAT",
        (a, b) if a == b && a != Direction::Flat => "TRENDING",
        _ => "CHOPPY",
    };

    let vol_1m = log_return_stddev(&window);
    let strength = trend_strength(ret_5m, ret_15m, vol_1m, regime);

    let agreement: &'static str = match (dir, regime, t5) {
        (Direction::Up, "TRENDING", Direction::Up) => "CONTINUATION UP",
        (Direction::Down, "TRENDING", Direction::Down) => "CONTINUATION DOWN",
        (Direction::Up, _, Direction::Down) => "PULLBACK RISK",
        (Direction::Down, _, Direction::Up) => "RECLAIM RISK",
        (_, "CHOPPY", _) => "RANGE/FAKEOUTS",
        _ => "NEUTRAL",
    };

    Some(MicroSnapshot {
        price,
        direction_vs_open: dir,
        delta_pct,
        trend_5m: t5,
        trend_15m: t15,
        trend_regime: regime,
        strength,
        agreement,
        vol_1m,
    })
}
