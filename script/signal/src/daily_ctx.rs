use crate::client::Candle;

pub struct DailyContext {
    /// 20-day highest high.
    pub daily_high: f64,
    /// 20-day lowest low.
    pub daily_low: f64,
    /// Price is within 3% of daily high (resistance zone).
    pub near_resistance: bool,
    /// Price is within 3% of daily low (support zone).
    pub near_support: bool,
    /// Distance from daily high as percentage (negative = below).
    pub pct_from_high: f64,
    /// Distance from daily low as percentage (positive = above).
    pub pct_from_low: f64,
}

/// Compute daily structure from 1d candles.
/// `candles_1d` should be closed daily candles sorted ascending.
pub fn compute_daily(candles_1d: &[Candle], current_price: f64) -> Option<DailyContext> {
    if candles_1d.len() < 5 {
        return None;
    }

    // Use up to last 20 closed daily candles.
    let n = candles_1d.len().min(20);
    let recent = &candles_1d[candles_1d.len() - n..];

    let daily_high = recent.iter().map(|c| c.h).fold(f64::MIN, f64::max);
    let daily_low = recent.iter().map(|c| c.l).fold(f64::MAX, f64::min);

    let pct_from_high = if daily_high > 0.0 {
        (current_price - daily_high) / daily_high
    } else {
        0.0
    };
    let pct_from_low = if daily_low > 0.0 {
        (current_price - daily_low) / daily_low
    } else {
        0.0
    };

    Some(DailyContext {
        daily_high,
        daily_low,
        near_resistance: pct_from_high > -0.03, // within 3% of high
        near_support: pct_from_low < 0.03,       // within 3% of low
        pct_from_high,
        pct_from_low,
    })
}

pub struct VolumeContext {
    /// Current volume / SMA(20) volume. >1 = above average, <1 = below.
    pub vol_ratio: f64,
    /// Volume is declining (ratio < 0.8).
    pub vol_declining: bool,
    /// Volume confirms move (ratio > 1.5).
    pub vol_confirms: bool,
}

/// Compute volume trend from 15m candles.
pub fn compute_volume(candles_15m: &[Candle]) -> Option<VolumeContext> {
    if candles_15m.len() < 21 {
        return None;
    }

    // SMA(20) of volume over the last 20 candles (excluding the most recent).
    let window = &candles_15m[candles_15m.len() - 21..candles_15m.len() - 1];
    let vol_sma: f64 = window.iter().map(|c| c.v).sum::<f64>() / 20.0;

    if vol_sma <= 0.0 {
        return None;
    }

    let current_vol = candles_15m.last()?.v;
    let vol_ratio = current_vol / vol_sma;

    Some(VolumeContext {
        vol_ratio,
        vol_declining: vol_ratio < 0.8,
        vol_confirms: vol_ratio > 1.5,
    })
}
