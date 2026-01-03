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

    let price_ref = current.close.abs().max(1.0);

    for (name, ratio) in levels {
        let level = match direction {
            TrendDirection::Up => high - ratio * diff,
            TrendDirection::Down => low + ratio * diff,
        };

        let distance = ((current.close - level) / price_ref).abs();
        if distance <= FIB_TOLERANCE_PCT {
            let confidence = fib_confidence(distance, diff.abs(), price_ref);
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
                confidence,
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
            let confidence = wave_confidence(recent);
            results.push(advanced_pattern(
                "Elliott Wave 1-2-3-4-5 (Up)",
                PatternClassification::Bullish,
                PatternSignalType::Impulse,
                "elliott_wave",
                "pivots",
                vec!["min_pivots=6".to_string(), "trend=up".to_string()],
                30,
                confidence,
            ));
        }
        if is_wave_sequence(recent, TrendDirection::Down) {
            let confidence = wave_confidence(recent);
            results.push(advanced_pattern(
                "Elliott Wave 1-2-3-4-5 (Down)",
                PatternClassification::Bearish,
                PatternSignalType::Impulse,
                "elliott_wave",
                "pivots",
                vec!["min_pivots=6".to_string(), "trend=down".to_string()],
                30,
                confidence,
            ));
        }
    }

    if pivots.len() >= 4 {
        let recent = &pivots[pivots.len() - 4..];
        if is_correction_sequence(recent) {
            let confidence = wave_confidence(recent);
            results.push(advanced_pattern(
                "Elliott Wave A-B-C",
                PatternClassification::Neutral,
                PatternSignalType::Correction,
                "elliott_wave",
                "pivots",
                vec!["min_pivots=4".to_string(), "pattern=abc".to_string()],
                20,
                confidence,
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
        let confidence = fractal_confidence(candles, idx, TrendDirection::Up);
        results.push(advanced_pattern(
            "Williams Fractal (Up)",
            PatternClassification::Bearish,
            PatternSignalType::Reversal,
            "williams_fractal",
            "five_bar",
            vec!["center=high".to_string(), "window=5".to_string()],
            5,
            confidence,
        ));
    }

    if center.low < left2.low
        && center.low < left1.low
        && center.low < right1.low
        && center.low < right2.low
    {
        let confidence = fractal_confidence(candles, idx, TrendDirection::Down);
        results.push(advanced_pattern(
            "Williams Fractal (Down)",
            PatternClassification::Bullish,
            PatternSignalType::Reversal,
            "williams_fractal",
            "five_bar",
            vec!["center=low".to_string(), "window=5".to_string()],
            5,
            confidence,
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
    let prev = pivots
        .iter()
        .rev()
        .skip(1)
        .find(|pivot| pivot.kind != last.kind)?;

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
    confidence: f64,
) -> AdvancedDetectedPattern {
    AdvancedDetectedPattern {
        pattern: name,
        category: "advanced",
        classification,
        signal_type,
        confidence: confidence.clamp(0.55, 0.9),
        window,
        method,
        basis,
        assumptions,
    }
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn fib_confidence(distance_pct: f64, swing_size: f64, price_ref: f64) -> f64 {
    let proximity = 1.0 - clamp01(distance_pct / FIB_TOLERANCE_PCT);
    let swing_score = if price_ref <= f64::EPSILON {
        0.0
    } else {
        clamp01(swing_size / price_ref / 0.05)
    };
    let confidence = 0.55 + 0.25 * proximity + 0.15 * swing_score;
    confidence.clamp(0.55, 0.9)
}

fn wave_confidence(pivots: &[Pivot]) -> f64 {
    let swings = pivot_swings(pivots);
    if swings.is_empty() {
        return 0.6;
    }

    let max = swings
        .iter()
        .copied()
        .fold(f64::MIN, |acc, value| acc.max(value));
    let min = swings
        .iter()
        .copied()
        .fold(f64::MAX, |acc, value| acc.min(value));
    let uniformity = if max <= f64::EPSILON {
        0.0
    } else {
        1.0 - clamp01((max - min) / max)
    };

    let first = pivots.first().map(|pivot| pivot.price).unwrap_or(0.0);
    let last = pivots.last().map(|pivot| pivot.price).unwrap_or(0.0);
    let base = first.abs().max(1.0);
    let magnitude = clamp01((last - first).abs() / base / 0.05);

    let confidence = 0.55 + 0.2 * uniformity + 0.2 * magnitude;
    confidence.clamp(0.55, 0.9)
}

fn pivot_swings(pivots: &[Pivot]) -> Vec<f64> {
    pivots
        .windows(2)
        .map(|window| (window[1].price - window[0].price).abs())
        .collect()
}

fn fractal_confidence(
    candles: &[Candle],
    center_idx: usize,
    direction: TrendDirection,
) -> f64 {
    if center_idx < 2 || center_idx + 2 >= candles.len() {
        return 0.6;
    }

    let center = &candles[center_idx];
    let neighbors = [
        &candles[center_idx - 2],
        &candles[center_idx - 1],
        &candles[center_idx + 1],
        &candles[center_idx + 2],
    ];

    let margin = match direction {
        TrendDirection::Up => {
            let neighbor_high = neighbors
                .iter()
                .map(|c| c.high)
                .fold(f64::MIN, |acc, value| acc.max(value));
            if center.high <= f64::EPSILON {
                0.0
            } else {
                (center.high - neighbor_high) / center.high
            }
        }
        TrendDirection::Down => {
            let neighbor_low = neighbors
                .iter()
                .map(|c| c.low)
                .fold(f64::MAX, |acc, value| acc.min(value));
            if center.low.abs() <= f64::EPSILON {
                0.0
            } else {
                (neighbor_low - center.low) / center.low.abs()
            }
        }
    };

    let margin_score = clamp01(margin / 0.01);
    let avg_range = neighbors
        .iter()
        .fold(range(center), |acc, candle| acc + range(candle))
        / 5.0;
    let range_score = if avg_range > f64::EPSILON {
        clamp01(range(center) / avg_range / 1.5)
    } else {
        0.0
    };

    let confidence = 0.55 + 0.25 * margin_score + 0.15 * range_score;
    confidence.clamp(0.55, 0.9)
}

fn range(candle: &Candle) -> f64 {
    (candle.high - candle.low).abs()
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

    #[test]
    fn fib_confidence_rewards_proximity() {
        let near = fib_confidence(0.001, 120.0, 1000.0);
        let far = fib_confidence(0.004, 120.0, 1000.0);
        assert!(near > far);
    }

    #[test]
    fn wave_confidence_rewards_uniform_swings() {
        let uniform = vec![
            pivot(0, 100.0, PivotKind::Low),
            pivot(1, 110.0, PivotKind::High),
            pivot(2, 100.0, PivotKind::Low),
            pivot(3, 110.0, PivotKind::High),
            pivot(4, 100.0, PivotKind::Low),
            pivot(5, 110.0, PivotKind::High),
        ];
        let irregular = vec![
            pivot(0, 100.0, PivotKind::Low),
            pivot(1, 140.0, PivotKind::High),
            pivot(2, 90.0, PivotKind::Low),
            pivot(3, 160.0, PivotKind::High),
            pivot(4, 80.0, PivotKind::Low),
            pivot(5, 170.0, PivotKind::High),
        ];

        let uniform_score = wave_confidence(&uniform);
        let irregular_score = wave_confidence(&irregular);
        assert!(uniform_score > irregular_score);
    }

    #[test]
    fn fractal_confidence_stays_in_bounds() {
        let candles = vec![
            candle(10.0, 11.0),
            candle(10.5, 11.5),
            candle(9.0, 13.0),
            candle(10.5, 11.5),
            candle(10.0, 11.0),
        ];

        let confidence = fractal_confidence(&candles, 2, TrendDirection::Up);
        assert!(confidence >= 0.55);
        assert!(confidence <= 0.9);
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

    fn pivot(index: usize, price: f64, kind: PivotKind) -> Pivot {
        Pivot {
            index,
            time: index as u64,
            price,
            kind,
        }
    }
}
