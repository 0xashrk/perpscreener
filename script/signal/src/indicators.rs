use crate::client::Candle;

#[allow(dead_code)]
pub struct BollingerBands {
    pub upper: f64,
    pub lower: f64,
    pub mid: f64,
    pub width: f64,
}

/// Bollinger Bands on candle closes. `mult` is typically 2.0.
pub fn compute_bb(candles: &[Candle], period: usize, mult: f64) -> Option<BollingerBands> {
    if candles.len() < period {
        return None;
    }
    let closes: Vec<f64> = candles[candles.len() - period..].iter().map(|c| c.c).collect();
    let mean = closes.iter().sum::<f64>() / period as f64;
    let var = closes.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / (period as f64 - 1.0);
    let std = var.sqrt();
    let upper = mean + mult * std;
    let lower = mean - mult * std;
    Some(BollingerBands {
        upper,
        lower,
        mid: mean,
        width: if mean > 0.0 {
            (upper - lower) / mean
        } else {
            0.0
        },
    })
}

/// RSI(period) on candle closes. Returns 0-100.
pub fn compute_rsi(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period + 1 {
        return None;
    }
    let recent = &candles[candles.len() - period - 1..];
    let mut gains = 0.0f64;
    let mut losses = 0.0f64;
    for i in 1..recent.len() {
        let change = recent[i].c - recent[i - 1].c;
        if change > 0.0 {
            gains += change;
        } else {
            losses += change.abs();
        }
    }
    let avg_gain = gains / period as f64;
    let avg_loss = losses / period as f64;
    if avg_loss == 0.0 {
        return Some(100.0);
    }
    let rs = avg_gain / avg_loss;
    Some(100.0 - 100.0 / (1.0 + rs))
}
