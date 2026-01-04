use crate::models::candle::Candle;

use super::candlesticks_bearish;
use super::candlesticks_bullish;
use super::DetectedPattern;

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
    confidence: f64,
) -> DetectedPattern {
    DetectedPattern {
        pattern: name,
        category: "candlestick_reversal",
        classification,
        signal_type: crate::models::patterns::PatternSignalType::Reversal,
        confidence: confidence.clamp(0.1, 0.98),
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
    slice
        .iter()
        .map(|c| c.high)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
}

pub(super) fn min_low(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    slice
        .iter()
        .map(|c| c.low)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
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
    if !last.close.is_finite() {
        return None;
    }
    let low = slice
        .iter()
        .map(|c| c.low)
        .filter(|value| value.is_finite())
        .min_by(|a, b| a.partial_cmp(b).unwrap())?;
    let high = slice
        .iter()
        .map(|c| c.high)
        .filter(|value| value.is_finite())
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

pub(super) fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

pub(super) fn pattern_confidence(base: f64, scores: &[f64]) -> f64 {
    if scores.is_empty() {
        return base;
    }
    let sum = scores.iter().copied().sum::<f64>();
    let avg = sum / scores.len() as f64;
    base + 0.35 * clamp01(avg)
}

pub(super) fn body_ratio(candle: &Candle) -> f64 {
    let range_value = range(candle).abs().max(f64::EPSILON);
    clamp01(body(candle).abs() / range_value)
}

pub(super) fn upper_wick_ratio(candle: &Candle) -> f64 {
    let range_value = range(candle).abs().max(f64::EPSILON);
    let upper = candle.high - candle.open.max(candle.close);
    clamp01(upper / range_value)
}

pub(super) fn lower_wick_ratio(candle: &Candle) -> f64 {
    let range_value = range(candle).abs().max(f64::EPSILON);
    let lower = candle.open.min(candle.close) - candle.low;
    clamp01(lower / range_value)
}

pub(super) fn range_score(range_value: f64, avg_range: f64) -> f64 {
    if avg_range.abs() <= f64::EPSILON {
        return 0.0;
    }
    clamp01(range_value.abs() / (avg_range.abs() * 1.5))
}

pub(super) fn trend_score(start: f64, end: f64) -> f64 {
    if start.abs() <= f64::EPSILON {
        return 0.0;
    }
    let pct = ((end - start) / start).abs();
    clamp01(pct / 0.05)
}

pub(super) fn scaled_score(value: f64, scale: f64) -> f64 {
    if scale.abs() <= f64::EPSILON {
        0.0
    } else {
        clamp01(value / scale)
    }
}

pub(super) fn proximity_score(delta: f64, scale: f64) -> f64 {
    if scale.abs() <= f64::EPSILON {
        0.0
    } else {
        clamp01(1.0 - (delta.abs() / scale))
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
        let pattern = build_pattern("Hammer", PatternClassification::Bullish, 1, 0.7);
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

    #[test]
    fn pattern_confidence_increases_with_scores() {
        let low = pattern_confidence(0.55, &[0.1, 0.2]);
        let high = pattern_confidence(0.55, &[0.8, 0.9]);
        assert!(high > low);
        assert!(high <= 0.98);
    }
}
