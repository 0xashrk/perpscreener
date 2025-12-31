use crate::business_logic::features::{FeatureSnapshot, Pivot, PivotKind};
use crate::models::candle::Candle;
use crate::models::patterns::{PatternClassification, PatternSignalType};

use super::AdvancedDetectedPattern;

const FIB_TOLERANCE_PCT: f64 = 0.005;

pub fn detect_advanced_patterns(
    candles: &[Candle],
    features: Option<&FeatureSnapshot>,
) -> Vec<AdvancedDetectedPattern> {
    let mut results = Vec::new();
    let Some(features) = features else {
        return results;
    };

    results.extend(detect_fibonacci(candles, &features.pivots));
    results.extend(detect_elliott(&features.pivots));
    results.extend(detect_fractals(candles));

    results
}

fn detect_fibonacci(candles: &[Candle], pivots: &[Pivot]) -> Vec<AdvancedDetectedPattern> {
    let mut results = Vec::new();
    let Some(current) = candles.last() else {
        return results;
    };
    let Some((low, high, direction)) = latest_swing(pivots) else {
        return results;
    };

    let diff = high - low;
    if diff.abs() <= f64::EPSILON {
        return results;
    }

    let levels = [
        ("Fibonacci 38.2% Retracement", 0.382),
        ("Fibonacci 50% Retracement", 0.5),
        ("Fibonacci 61.8% Retracement", 0.618),
    ];

    for (name, ratio) in levels {
        let level = match direction {
            TrendDirection::Up => high - ratio * diff,
            TrendDirection::Down => low + ratio * diff,
        };

        let distance = ((current.close - level) / current.close).abs();
        if distance <= FIB_TOLERANCE_PCT {
            results.push(advanced_pattern(
                name,
                PatternClassification::Neutral,
                PatternSignalType::KeyLevel,
                "fibonacci_retracement",
                "pivot_swing",
                vec![
                    "swing=last_opposite_pivots".to_string(),
                    "tolerance=0.5%".to_string(),
                ],
                10,
            ));
        }
    }

    results
}

fn detect_elliott(pivots: &[Pivot]) -> Vec<AdvancedDetectedPattern> {
    let mut results = Vec::new();
    let pivots = sorted_pivots(pivots);
    if pivots.len() >= 6 {
        let recent = &pivots[pivots.len() - 6..];
        if is_wave_sequence(recent, TrendDirection::Up) {
            results.push(advanced_pattern(
                "Elliott Wave 1-2-3-4-5 (Up)",
                PatternClassification::Bullish,
                PatternSignalType::Impulse,
                "elliott_wave",
                "pivots",
                vec!["min_pivots=6".to_string(), "trend=up".to_string()],
                30,
            ));
        }
        if is_wave_sequence(recent, TrendDirection::Down) {
            results.push(advanced_pattern(
                "Elliott Wave 1-2-3-4-5 (Down)",
                PatternClassification::Bearish,
                PatternSignalType::Impulse,
                "elliott_wave",
                "pivots",
                vec!["min_pivots=6".to_string(), "trend=down".to_string()],
                30,
            ));
        }
    }

    if pivots.len() >= 4 {
        let recent = &pivots[pivots.len() - 4..];
        if is_correction_sequence(recent) {
            results.push(advanced_pattern(
                "Elliott Wave A-B-C",
                PatternClassification::Neutral,
                PatternSignalType::Correction,
                "elliott_wave",
                "pivots",
                vec!["min_pivots=4".to_string(), "pattern=abc".to_string()],
                20,
            ));
        }
    }

    results
}

fn detect_fractals(candles: &[Candle]) -> Vec<AdvancedDetectedPattern> {
    let mut results = Vec::new();
    if candles.len() < 5 {
        return results;
    }

    let idx = candles.len() - 3;
    let center = &candles[idx];
    let left2 = &candles[idx - 2];
    let left1 = &candles[idx - 1];
    let right1 = &candles[idx + 1];
    let right2 = &candles[idx + 2];

    if center.high > left2.high
        && center.high > left1.high
        && center.high > right1.high
        && center.high > right2.high
    {
        results.push(advanced_pattern(
            "Williams Fractal (Up)",
            PatternClassification::Bearish,
            PatternSignalType::Reversal,
            "williams_fractal",
            "five_bar",
            vec!["center=high".to_string(), "window=5".to_string()],
            5,
        ));
    }

    if center.low < left2.low
        && center.low < left1.low
        && center.low < right1.low
        && center.low < right2.low
    {
        results.push(advanced_pattern(
            "Williams Fractal (Down)",
            PatternClassification::Bullish,
            PatternSignalType::Reversal,
            "williams_fractal",
            "five_bar",
            vec!["center=low".to_string(), "window=5".to_string()],
            5,
        ));
    }

    results
}

#[derive(Debug, Clone, Copy)]
enum TrendDirection {
    Up,
    Down,
}

fn latest_swing(pivots: &[Pivot]) -> Option<(f64, f64, TrendDirection)> {
    let pivots = sorted_pivots(pivots);
    if pivots.len() < 2 {
        return None;
    }

    let last = pivots.last()?;
    let prev = pivots.iter().rev().skip(1).find(|pivot| pivot.kind != last.kind)?;

    match (prev.kind, last.kind) {
        (PivotKind::Low, PivotKind::High) => Some((prev.price, last.price, TrendDirection::Up)),
        (PivotKind::High, PivotKind::Low) => Some((last.price, prev.price, TrendDirection::Down)),
        _ => None,
    }
}

fn sorted_pivots(pivots: &[Pivot]) -> Vec<Pivot> {
    let mut sorted = pivots.to_vec();
    sorted.sort_by_key(|pivot| pivot.index);
    sorted
}

fn is_wave_sequence(pivots: &[Pivot], direction: TrendDirection) -> bool {
    if pivots.len() != 6 {
        return false;
    }
    if !alternating(pivots) {
        return false;
    }

    match direction {
        TrendDirection::Up => {
            pivots[0].kind == PivotKind::Low
                && pivots[1].kind == PivotKind::High
                && pivots[2].price > pivots[0].price
                && pivots[3].price > pivots[1].price
                && pivots[4].price > pivots[2].price
                && pivots[5].price > pivots[3].price
        }
        TrendDirection::Down => {
            pivots[0].kind == PivotKind::High
                && pivots[1].kind == PivotKind::Low
                && pivots[2].price < pivots[0].price
                && pivots[3].price < pivots[1].price
                && pivots[4].price < pivots[2].price
                && pivots[5].price < pivots[3].price
        }
    }
}

fn is_correction_sequence(pivots: &[Pivot]) -> bool {
    if pivots.len() != 4 {
        return false;
    }
    alternating(pivots)
}

fn alternating(pivots: &[Pivot]) -> bool {
    pivots
        .windows(2)
        .all(|window| window[0].kind != window[1].kind)
}

fn advanced_pattern(
    name: &'static str,
    classification: PatternClassification,
    signal_type: PatternSignalType,
    method: &'static str,
    basis: &'static str,
    assumptions: Vec<String>,
    window: usize,
) -> AdvancedDetectedPattern {
    AdvancedDetectedPattern {
        pattern: name,
        category: "advanced",
        classification,
        signal_type,
        confidence: 0.6,
        window,
        method,
        basis,
        assumptions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_williams_fractal_up() {
        let candles = vec![
            candle(10.0, 12.0),
            candle(11.0, 13.0),
            candle(12.0, 15.0),
            candle(11.0, 13.0),
            candle(10.0, 12.0),
        ];
        let results = detect_fractals(&candles);
        assert!(results
            .iter()
            .any(|pattern| pattern.pattern == "Williams Fractal (Up)"));
    }

    fn candle(low: f64, high: f64) -> Candle {
        Candle {
            open_time: 0,
            close_time: 0,
            open: low,
            high,
            low,
            close: high,
            volume: 0.0,
            num_trades: 0,
            interval: None,
            symbol: None,
        }
    }
}
