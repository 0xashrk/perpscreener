use crate::models::candle::Candle;

use super::types::{VolatilityPoint, VolatilitySnapshot};

pub fn compute_volatility(candles: &[Candle], window: usize) -> Option<VolatilitySnapshot> {
    if window == 0 || candles.len() < window + 1 {
        return None;
    }

    let returns = closes_to_returns(candles);
    if returns.len() < window {
        return None;
    }

    let mut values = Vec::new();
    for (idx, slice) in returns.windows(window).enumerate() {
        let value = stddev(slice);
        let close_time = candles[idx + window].close_time;
        values.push(VolatilityPoint { close_time, value });
    }

    if values.is_empty() {
        None
    } else {
        Some(VolatilitySnapshot { window, values })
    }
}

fn closes_to_returns(candles: &[Candle]) -> Vec<f64> {
    let mut returns = Vec::with_capacity(candles.len().saturating_sub(1));
    for pair in candles.windows(2) {
        let prev = pair[0].close;
        let current = pair[1].close;
        let ret = if prev.abs() > f64::EPSILON {
            (current - prev) / prev
        } else {
            0.0
        };
        returns.push(ret);
    }
    returns
}

fn stddev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let diff = value - mean;
            diff * diff
        })
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(close: f64, time: u64) -> Candle {
        Candle {
            open_time: time,
            close_time: time,
            open: close,
            high: close,
            low: close,
            close,
            volume: 0.0,
            num_trades: 0,
            interval: None,
            symbol: None,
        }
    }

    #[test]
    fn computes_volatility_from_returns() {
        let candles = vec![
            candle(100.0, 0),
            candle(110.0, 1),
            candle(100.0, 2),
            candle(110.0, 3),
            candle(100.0, 4),
        ];

        let snapshot = compute_volatility(&candles, 3).expect("vol snapshot");
        assert_eq!(snapshot.window, 3);
        assert!(!snapshot.values.is_empty());
        assert!(snapshot.values[0].value > 0.0);
    }
}
