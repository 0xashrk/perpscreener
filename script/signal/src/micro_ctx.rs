use crate::client::Candle;
use crate::vwap::VwapContext;

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

#[allow(dead_code)]
pub struct MicroContext {
    pub current_price: f64,
    pub price_vs_vwap: Direction,
    pub vwap_delta_pct: f64,
    pub trend_1c: Direction,
    pub trend_4c: Direction,
    pub trend_regime: &'static str,
    pub strength: u64,
    pub agreement: &'static str,
    pub vol: Option<f64>,
    pub streak_dir: Direction,
    pub streak_len: usize,
}

/// Compute micro context from 15m candles with VWAP-based agreement.
pub fn compute_micro(candles_15m: &[Candle], vwap: &VwapContext) -> Option<MicroContext> {
    if candles_15m.is_empty() {
        return None;
    }

    let price = candles_15m.last()?.c;

    let pvw = if vwap.price_vs_vwap > 0.0001 {
        Direction::Up
    } else if vwap.price_vs_vwap < -0.0001 {
        Direction::Down
    } else {
        Direction::Flat
    };

    let ret_1c = ret_over(candles_15m, 1);
    let ret_4c = ret_over(candles_15m, 4);
    let t1 = trend_label(ret_1c);
    let t4 = trend_label(ret_4c);

    let regime: &'static str = match (t1, t4) {
        (Direction::Flat, Direction::Flat) => "DRIFT/FLAT",
        (a, b) if a == b && a != Direction::Flat => "TRENDING",
        _ => "CHOPPY",
    };

    let vol = log_ret_stddev(candles_15m);
    let strength = trend_strength(ret_1c, ret_4c, vol, regime);
    let (streak_dir, streak_len) = compute_streaks(candles_15m);

    let agreement: &'static str = match (pvw, regime, t1) {
        (Direction::Up, "TRENDING", Direction::Up) => "CONTINUATION UP",
        (Direction::Down, "TRENDING", Direction::Down) => "CONTINUATION DOWN",
        (Direction::Up, _, Direction::Down) => "PULLBACK RISK",
        (Direction::Down, _, Direction::Up) => "RECLAIM RISK",
        (_, "CHOPPY", _) => "RANGE/FAKEOUTS",
        _ => "NEUTRAL",
    };

    Some(MicroContext {
        current_price: price,
        price_vs_vwap: pvw,
        vwap_delta_pct: vwap.price_vs_vwap,
        trend_1c: t1,
        trend_4c: t4,
        trend_regime: regime,
        strength,
        agreement,
        vol,
        streak_dir,
        streak_len,
    })
}

fn ret_over(candles: &[Candle], n: usize) -> Option<f64> {
    if candles.len() <= n {
        return None;
    }
    let last = candles.last()?;
    let prior = candles.get(candles.len() - n - 1)?;
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

fn log_ret_stddev(candles: &[Candle]) -> Option<f64> {
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

fn compute_streaks(candles: &[Candle]) -> (Direction, usize) {
    let mut dir = Direction::Flat;
    let mut len = 0usize;
    for c in candles {
        let d = if c.c > c.o {
            Direction::Up
        } else if c.c < c.o {
            Direction::Down
        } else {
            Direction::Flat
        };
        if d == dir {
            len += 1;
        } else {
            dir = d;
            len = 1;
        }
    }
    (dir, len)
}

fn trend_strength(ret1: Option<f64>, ret4: Option<f64>, vol: Option<f64>, regime: &str) -> u64 {
    let mag = match (ret1, ret4) {
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
