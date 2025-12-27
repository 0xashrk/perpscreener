use crate::models::candle::Candle;

/// VWAP computation result from a candle window.
pub struct VwapResult {
    pub vwap: f64,
    pub cumulative_volume: f64,
    pub stddev: f64,
}

/// Compute VWAP and simple dispersion metrics for a candle window.
pub fn compute_vwap(candles: &[Candle]) -> Option<VwapResult> {
    if candles.is_empty() {
        return None;
    }

    let mut sum_pv = 0.0;
    let mut sum_volume = 0.0;
    let mut sum_tp = 0.0;
    let mut sum_tp2 = 0.0;
    let mut count = 0usize;

    for candle in candles {
        let typical = (candle.high + candle.low + candle.close) / 3.0;
        sum_pv += typical * candle.volume;
        sum_volume += candle.volume;
        sum_tp += typical;
        sum_tp2 += typical * typical;
        count += 1;
    }

    if sum_volume <= 0.0 || count == 0 {
        return None;
    }

    let vwap = sum_pv / sum_volume;
    let mean = sum_tp / count as f64;
    let variance = (sum_tp2 / count as f64) - (mean * mean);
    let stddev = variance.max(0.0).sqrt();

    Some(VwapResult {
        vwap,
        cumulative_volume: sum_volume,
        stddev,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(high: f64, low: f64, close: f64, volume: f64) -> Candle {
        Candle {
            open_time: 0,
            close_time: 0,
            open: close,
            high,
            low,
            close,
            volume,
            num_trades: 0,
            interval: None,
            symbol: None,
        }
    }

    #[test]
    fn compute_vwap_returns_none_for_empty() {
        assert!(compute_vwap(&[]).is_none());
    }

    #[test]
    fn compute_vwap_calculates_expected_value() {
        let candles = vec![
            candle(110.0, 90.0, 100.0, 2.0),
            candle(120.0, 80.0, 100.0, 1.0),
        ];
        let result = compute_vwap(&candles).unwrap();
        assert!((result.vwap - 100.0).abs() < 0.0001);
        assert!((result.cumulative_volume - 3.0).abs() < 0.0001);
    }
}
