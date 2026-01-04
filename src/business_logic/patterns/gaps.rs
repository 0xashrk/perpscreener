use crate::models::candle::Candle;
use crate::models::patterns::{PatternClassification, PatternSignalType};

use super::DetectedPattern;

const VOLUME_WINDOW: usize = 20;
const RANGE_WINDOW: usize = 20;
const RANGE_BOUND_THRESHOLD: f64 = 0.02;
const BREAKAWAY_VOLUME: f64 = 1.5;
const RUNAWAY_VOLUME: f64 = 1.3;
const EXHAUSTION_VOLUME: f64 = 2.5;
const EXTENSION_THRESHOLD: f64 = 0.05;

pub fn detect_gap_patterns(candles: &[Candle]) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    let Some((gap, direction)) = detect_gap(candles) else {
        return results;
    };

    let volume_ratio = volume_ratio(candles);
    let range_bound = is_range_bound(candles);
    let trend = trend_direction(candles);
    let extension = trend_extension(candles);

    let (pattern, classification, signal_type) = if volume_ratio >= EXHAUSTION_VOLUME
        && trend_matches(direction, trend)
        && extension >= EXTENSION_THRESHOLD
    {
        match direction {
            GapDirection::Up => (
                "Exhaustion Gap (Up)",
                PatternClassification::Bearish,
                PatternSignalType::Reversal,
            ),
            GapDirection::Down => (
                "Exhaustion Gap (Down)",
                PatternClassification::Bullish,
                PatternSignalType::Reversal,
            ),
        }
    } else if range_bound && volume_ratio >= BREAKAWAY_VOLUME {
        match direction {
            GapDirection::Up => (
                "Breakaway Gap (Up)",
                PatternClassification::Bullish,
                PatternSignalType::Trend,
            ),
            GapDirection::Down => (
                "Breakaway Gap (Down)",
                PatternClassification::Bearish,
                PatternSignalType::Trend,
            ),
        }
    } else if trend_matches(direction, trend) && volume_ratio >= RUNAWAY_VOLUME {
        match direction {
            GapDirection::Up => (
                "Runaway Gap (Up)",
                PatternClassification::Bullish,
                PatternSignalType::Continuation,
            ),
            GapDirection::Down => (
                "Runaway Gap (Down)",
                PatternClassification::Bearish,
                PatternSignalType::Continuation,
            ),
        }
    } else {
        (
            "Common Gap",
            PatternClassification::Neutral,
            PatternSignalType::Range,
        )
    };

    results.push(DetectedPattern {
        pattern,
        category: "gap",
        classification,
        signal_type,
        confidence: gap_confidence(volume_ratio, gap.percent),
        window: 2,
        notes: Some(format!("gap_pct={:.2}%", gap.percent * 100.0)),
    });

    results
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GapDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
struct GapInfo {
    percent: f64,
}

fn detect_gap(candles: &[Candle]) -> Option<(GapInfo, GapDirection)> {
    if candles.len() < 2 {
        return None;
    }

    let prev = candles.get(candles.len() - 2)?;
    let current = candles.last()?;
    if prev.close.abs() <= f64::EPSILON {
        return None;
    }

    if current.low > prev.high {
        let gap_size = current.low - prev.high;
        let percent = gap_size / prev.close;
        return Some((GapInfo { percent }, GapDirection::Up));
    }

    if current.high < prev.low {
        let gap_size = prev.low - current.high;
        let percent = gap_size / prev.close;
        return Some((GapInfo { percent }, GapDirection::Down));
    }

    None
}

fn volume_ratio(candles: &[Candle]) -> f64 {
    if candles.len() < 2 {
        return 0.0;
    }
    let avg = average_volume(candles, VOLUME_WINDOW);
    if avg <= f64::EPSILON {
        return 0.0;
    }
    let current = candles.last().unwrap().volume;
    current / avg
}

fn average_volume(candles: &[Candle], window: usize) -> f64 {
    if candles.is_empty() {
        return 0.0;
    }
    let end = candles.len();
    let start = end.saturating_sub(window);
    let slice = &candles[start..end];
    let sum = slice.iter().map(|c| c.volume).sum::<f64>();
    sum / slice.len() as f64
}

fn is_range_bound(candles: &[Candle]) -> bool {
    if candles.len() < 2 {
        return false;
    }
    let end = candles.len();
    let start = end.saturating_sub(RANGE_WINDOW);
    let slice = &candles[start..end];
    let high = slice
        .iter()
        .map(|c| c.high)
        .filter(|value| value.is_finite())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);
    let low = slice
        .iter()
        .map(|c| c.low)
        .filter(|value| value.is_finite())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);
    let avg_close = slice.iter().map(|c| c.close).sum::<f64>() / slice.len() as f64;
    if avg_close.abs() <= f64::EPSILON {
        return false;
    }
    (high - low) / avg_close <= RANGE_BOUND_THRESHOLD
}

fn trend_direction(candles: &[Candle]) -> i8 {
    if candles.len() < 2 {
        return 0;
    }
    let end = candles.len() - 1;
    let start = end.saturating_sub(RANGE_WINDOW);
    let start_close = candles.get(start).map(|c| c.close).unwrap_or(0.0);
    let end_close = candles.last().map(|c| c.close).unwrap_or(0.0);
    if end_close > start_close {
        1
    } else if end_close < start_close {
        -1
    } else {
        0
    }
}

fn trend_extension(candles: &[Candle]) -> f64 {
    if candles.len() < 2 {
        return 0.0;
    }
    let end = candles.len() - 1;
    let start = end.saturating_sub(RANGE_WINDOW);
    let start_close = candles.get(start).map(|c| c.close).unwrap_or(0.0);
    let end_close = candles.last().map(|c| c.close).unwrap_or(0.0);
    if start_close.abs() <= f64::EPSILON {
        0.0
    } else {
        ((end_close - start_close) / start_close).abs()
    }
}

fn trend_matches(direction: GapDirection, trend: i8) -> bool {
    matches!(
        (direction, trend),
        (GapDirection::Up, 1) | (GapDirection::Down, -1)
    )
}

fn gap_confidence(volume_ratio: f64, gap_percent: f64) -> f64 {
    let base = if volume_ratio >= EXHAUSTION_VOLUME {
        0.8
    } else if volume_ratio >= BREAKAWAY_VOLUME {
        0.7
    } else if volume_ratio >= RUNAWAY_VOLUME {
        0.65
    } else {
        0.6
    };

    let magnitude_score = (gap_percent / 0.05).clamp(0.0, 1.0);
    (base + 0.15 * magnitude_score).clamp(0.55, 0.9)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(open: f64, close: f64, high: f64, low: f64, volume: f64) -> Candle {
        Candle {
            open_time: 0,
            close_time: 0,
            open,
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
    fn detects_up_gap() {
        let candles = vec![
            candle(10.0, 10.0, 11.0, 9.0, 100.0),
            candle(12.0, 12.5, 13.0, 12.1, 200.0),
        ];
        let detections = detect_gap_patterns(&candles);
        assert!(!detections.is_empty());
    }
}
