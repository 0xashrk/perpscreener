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
    candles: &[Candle],
    name: &'static str,
    classification: crate::models::patterns::PatternClassification,
    window: usize,
) -> DetectedPattern {
    DetectedPattern {
        pattern: name,
        category: "candlestick_reversal",
        classification,
        signal_type: crate::models::patterns::PatternSignalType::Reversal,
        confidence: candlestick_confidence(candles, window),
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

fn candlestick_confidence(candles: &[Candle], window: usize) -> f64 {
    let Some(last) = candle(candles, 0) else {
        return 0.6;
    };
    let range_value = range(last).abs();
    if range_value <= f64::EPSILON {
        return 0.55;
    }
    let body_value = body(last).abs();
    let body_ratio = (body_value / range_value).clamp(0.0, 1.0);
    let extremity = (body_ratio - 0.5).abs() * 2.0;
    let mut avg_window = window.max(5).min(10);
    avg_window = avg_window.min(candles.len()).max(1);
    let avg_range = avg_high_low_diff(candles, avg_window, 0).unwrap_or(range_value);
    let range_ratio = if avg_range > f64::EPSILON {
        range_value / avg_range
    } else {
        1.0
    };
    let range_score = ((range_ratio - 0.5) / 1.5).clamp(0.0, 1.0);
    let window_score = (window.min(5) as f64) / 5.0;

    let confidence = 0.45 + 0.25 * window_score + 0.2 * extremity + 0.25 * range_score;
    confidence.clamp(0.45, 0.92)
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
        let candles = vec![candle(10.0, 12.0, 12.0, 9.0)];
        let pattern = build_pattern(&candles, "Hammer", PatternClassification::Bullish, 1);
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
    fn candlestick_confidence_varies_with_range() {
        let mut candles = vec![candle(10.0, 10.5, 10.5, 9.5); 10];
        let base = candlestick_confidence(&candles, 1);

        candles[9] = candle(10.0, 11.5, 12.0, 9.0);
        let boosted = candlestick_confidence(&candles, 1);

        assert!(boosted > base);
    }

    #[test]
    fn candlestick_confidence_stays_in_bounds() {
        let candles = vec![candle(10.0, 12.0, 12.0, 9.0); 5];
        let confidence = candlestick_confidence(&candles, 3);
        assert!(confidence >= 0.45);
        assert!(confidence <= 0.92);
    }
}
