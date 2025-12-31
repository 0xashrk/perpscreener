use crate::models::candle::Candle;

use super::DetectedPattern;

mod candlesticks_bullish;
mod candlesticks_bearish;

pub fn detect_candlestick_patterns(candles: &[Candle]) -> Vec<DetectedPattern> {
    if candles.is_empty() {
        return Vec::new();
    }

    let mut detections = Vec::new();
    detections.extend(candlesticks_bullish::detect(candles));
    detections.extend(candlesticks_bearish::detect(candles));
    detections
}

pub(super) fn build_pattern(
    name: &'static str,
    classification: crate::models::patterns::PatternClassification,
    window: usize,
) -> DetectedPattern {
    DetectedPattern {
        pattern: name,
        category: "candlestick_reversal",
        classification,
        signal_type: crate::models::patterns::PatternSignalType::Reversal,
        confidence: confidence_for_window(window),
        window,
        notes: None,
    }
}

pub(super) fn candle(candles: &[Candle], offset: usize) -> Option<&Candle> {
    if candles.len() <= offset {
        return None;
    }
    let idx = candles.len().saturating_sub(1).saturating_sub(offset);
    candles.get(idx)
}

pub(super) fn body(candle: &Candle) -> f64 {
    (candle.close - candle.open).abs()
}

pub(super) fn range(candle: &Candle) -> f64 {
    candle.high - candle.low
}

pub(super) fn avg_high_low_diff(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let avg_high = avg_high(candles, window, offset)?;
    let avg_low = avg_low(candles, window, offset)?;
    Some(avg_high - avg_low)
}

pub(super) fn avg_high(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    let sum = slice.iter().map(|c| c.high).sum::<f64>();
    Some(sum / window as f64)
}

pub(super) fn avg_low(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    let sum = slice.iter().map(|c| c.low).sum::<f64>();
    Some(sum / window as f64)
}

pub(super) fn max_high(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    slice.iter().map(|c| c.high).max_by(|a, b| a.partial_cmp(b).unwrap())
}

pub(super) fn min_low(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    slice.iter().map(|c| c.low).min_by(|a, b| a.partial_cmp(b).unwrap())
}

pub(super) fn max_open(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    slice
        .iter()
        .map(|c| c.open)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
}

pub(super) fn min_open(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    slice
        .iter()
        .map(|c| c.open)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
}

pub(super) fn stochastic(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    let last = slice.last()?;
    let low = slice
        .iter()
        .map(|c| c.low)
        .min_by(|a, b| a.partial_cmp(b).unwrap())?;
    let high = slice
        .iter()
        .map(|c| c.high)
        .max_by(|a, b| a.partial_cmp(b).unwrap())?;
    if (high - low).abs() <= f64::EPSILON {
        return None;
    }
    Some((last.close - low) / (high - low) * 100.0)
}

pub(super) fn approx_eq(a: f64, b: f64) -> bool {
    let tol = 1e-6_f64.max(1e-6 * b.abs());
    (a - b).abs() <= tol
}

fn confidence_for_window(window: usize) -> f64 {
    match window {
        1 => 0.65,
        2 => 0.7,
        3 => 0.75,
        4 => 0.8,
        5 => 0.85,
        _ => 0.7,
    }
}

fn window_slice(candles: &[Candle], window: usize, offset: usize) -> Option<&[Candle]> {
    if window == 0 || candles.len() <= offset {
        return None;
    }

    let end = candles.len().saturating_sub(1).saturating_sub(offset);
    if end + 1 < window {
        return None;
    }
    let start = end + 1 - window;
    candles.get(start..=end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::patterns::{PatternClassification, PatternSignalType};

    fn candle(open: f64, close: f64, high: f64, low: f64) -> Candle {
        Candle {
            open_time: 0,
            close_time: 0,
            open,
            high,
            low,
            close,
            volume: 0.0,
            num_trades: 0,
            interval: None,
            symbol: None,
        }
    }

    #[test]
    fn detect_candlestick_patterns_returns_empty_for_no_candles() {
        let results = detect_candlestick_patterns(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn build_pattern_sets_defaults() {
        let pattern = build_pattern("Hammer", PatternClassification::Bullish, 1);
        assert_eq!(pattern.category, "candlestick_reversal");
        assert_eq!(pattern.signal_type, PatternSignalType::Reversal);
    }

    #[test]
    fn stochastic_returns_value() {
        let candles = vec![
            candle(10.0, 12.0, 12.0, 9.0),
            candle(12.0, 11.0, 13.0, 10.0),
            candle(11.0, 14.0, 14.0, 11.0),
        ];
        let value = stochastic(&candles, 3, 0).expect("stochastic");
        assert!(value > 0.0);
    }
}
