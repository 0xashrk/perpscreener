use crate::models::candle::Candle;
use crate::models::interval::CandleInterval;
use crate::models::patterns::{PatternClassification, PatternSignalType};

use crate::business_logic::features::{FeatureSnapshot, Pivot, PivotKind, Trendline, TrendlineKind};
use super::DetectedPattern;

const PIVOT_TOLERANCE_PCT: f64 = 0.02;
const SHOULDER_TOLERANCE_PCT: f64 = 0.025;
const WEDGE_SLOPE_DELTA: f64 = 0.0005;
const CHANNEL_SLOPE_TOLERANCE: f64 = 0.0005;
const FLAT_SLOPE_PCT: f64 = 0.001;
const FLAG_TREND_THRESHOLD: f64 = 0.03;
const FLAG_RANGE_THRESHOLD: f64 = 0.015;

pub fn detect_chart_patterns(
    candles: &[Candle],
    features: Option<&FeatureSnapshot>,
    interval: CandleInterval,
) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    let Some(features) = features else {
        return results;
    };
    let Some(last_close) = candles.last().map(|c| c.close) else {
        return results;
    };

    results.extend(detect_triangles(features, interval, last_close));
    results.extend(detect_channels(features, interval, last_close));
    results.extend(detect_wedges(features, interval, last_close));
    results.extend(detect_head_shoulders(features));
    results.extend(detect_double_triple(features));
    results.extend(detect_flags_pennants(candles, features, interval, last_close));
    results.extend(detect_three_methods(candles));
    results.extend(detect_cup_handle(features));

    results
}

fn detect_triangles(
    features: &FeatureSnapshot,
    interval: CandleInterval,
    price_ref: f64,
) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    let (support, resistance) = trendline_pair(features);
    let (Some(support), Some(resistance)) = (support, resistance) else {
        return results;
    };

    let support_slope = slope_per_bar(&support, interval);
    let resistance_slope = slope_per_bar(&resistance, interval);

    let support_flat = is_flat(support_slope, price_ref);
    let resistance_flat = is_flat(resistance_slope, price_ref);

    if resistance_flat && support_slope > 0.0 {
        results.push(chart_pattern(
            "Ascending Triangle",
            PatternClassification::Bullish,
            PatternSignalType::Continuation,
            "chart_continuation",
            10,
        ));
    } else if support_flat && resistance_slope < 0.0 {
        results.push(chart_pattern(
            "Descending Triangle",
            PatternClassification::Bearish,
            PatternSignalType::Continuation,
            "chart_continuation",
            10,
        ));
    } else if resistance_slope < 0.0 && support_slope > 0.0 {
        results.push(chart_pattern(
            "Symmetrical Triangle",
            PatternClassification::Neutral,
            PatternSignalType::Continuation,
            "chart_continuation",
            10,
        ));
    }

    results
}

fn detect_channels(
    features: &FeatureSnapshot,
    interval: CandleInterval,
    price_ref: f64,
) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    let (support, resistance) = trendline_pair(features);
    let (Some(support), Some(resistance)) = (support, resistance) else {
        return results;
    };

    let support_slope = slope_per_bar(&support, interval);
    let resistance_slope = slope_per_bar(&resistance, interval);
    let slope_delta = (support_slope - resistance_slope).abs();

    if slope_delta > price_ref * CHANNEL_SLOPE_TOLERANCE {
        return results;
    }

    let avg_slope = (support_slope + resistance_slope) / 2.0;
    if is_flat(avg_slope, price_ref) {
        results.push(chart_pattern(
            "Horizontal Channel",
            PatternClassification::Neutral,
            PatternSignalType::Range,
            "channel",
            10,
        ));
    } else if avg_slope > 0.0 {
        results.push(chart_pattern(
            "Ascending Channel",
            PatternClassification::Bullish,
            PatternSignalType::Trend,
            "channel",
            10,
        ));
    } else {
        results.push(chart_pattern(
            "Descending Channel",
            PatternClassification::Bearish,
            PatternSignalType::Trend,
            "channel",
            10,
        ));
    }

    results
}

fn detect_wedges(
    features: &FeatureSnapshot,
    interval: CandleInterval,
    price_ref: f64,
) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    let (support, resistance) = trendline_pair(features);
    let (Some(support), Some(resistance)) = (support, resistance) else {
        return results;
    };

    let support_slope = slope_per_bar(&support, interval);
    let resistance_slope = slope_per_bar(&resistance, interval);

    if support_slope > 0.0 && resistance_slope > 0.0 {
        if support_slope > resistance_slope + price_ref * WEDGE_SLOPE_DELTA {
            results.push(chart_pattern(
                "Rising Wedge",
                PatternClassification::Bearish,
                PatternSignalType::Reversal,
                "chart_reversal",
                10,
            ));
        }
    } else if support_slope < 0.0 && resistance_slope < 0.0 {
        if resistance_slope < support_slope - price_ref * WEDGE_SLOPE_DELTA {
            results.push(chart_pattern(
                "Falling Wedge",
                PatternClassification::Bullish,
                PatternSignalType::Reversal,
                "chart_reversal",
                10,
            ));
        }
    }

    results
}

fn detect_head_shoulders(features: &FeatureSnapshot) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    let highs = last_pivots(&features.pivots, PivotKind::High, 3);
    if highs.len() == 3 {
        let (left, head, right) = (highs[0], highs[1], highs[2]);
        if head.price > left.price && head.price > right.price {
            let shoulders_close = within_pct(left.price, right.price, SHOULDER_TOLERANCE_PCT);
            let head_above = head.price >= left.price * (1.0 + PIVOT_TOLERANCE_PCT);
            if shoulders_close && head_above {
                results.push(chart_pattern(
                    "Head and Shoulders",
                    PatternClassification::Bearish,
                    PatternSignalType::Reversal,
                    "chart_reversal",
                    20,
                ));
            }
        }
    }

    let lows = last_pivots(&features.pivots, PivotKind::Low, 3);
    if lows.len() == 3 {
        let (left, head, right) = (lows[0], lows[1], lows[2]);
        if head.price < left.price && head.price < right.price {
            let shoulders_close = within_pct(left.price, right.price, SHOULDER_TOLERANCE_PCT);
            let head_below = head.price <= left.price * (1.0 - PIVOT_TOLERANCE_PCT);
            if shoulders_close && head_below {
                results.push(chart_pattern(
                    "Inverse Head and Shoulders",
                    PatternClassification::Bullish,
                    PatternSignalType::Reversal,
                    "chart_reversal",
                    20,
                ));
            }
        }
    }

    results
}

fn detect_double_triple(features: &FeatureSnapshot) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    let highs = last_pivots(&features.pivots, PivotKind::High, 3);
    if highs.len() >= 2 {
        let last_two = &highs[highs.len() - 2..];
        if within_pct(last_two[0].price, last_two[1].price, PIVOT_TOLERANCE_PCT) {
            results.push(chart_pattern(
                "Double Top",
                PatternClassification::Bearish,
                PatternSignalType::Reversal,
                "chart_reversal",
                15,
            ));
        }
    }
    if highs.len() == 3 {
        let avg = (highs[0].price + highs[1].price + highs[2].price) / 3.0;
        if within_pct(highs[0].price, avg, PIVOT_TOLERANCE_PCT)
            && within_pct(highs[1].price, avg, PIVOT_TOLERANCE_PCT)
            && within_pct(highs[2].price, avg, PIVOT_TOLERANCE_PCT)
        {
            results.push(chart_pattern(
                "Triple Top",
                PatternClassification::Bearish,
                PatternSignalType::Reversal,
                "chart_reversal",
                20,
            ));
        }
    }

    let lows = last_pivots(&features.pivots, PivotKind::Low, 3);
    if lows.len() >= 2 {
        let last_two = &lows[lows.len() - 2..];
        if within_pct(last_two[0].price, last_two[1].price, PIVOT_TOLERANCE_PCT) {
            results.push(chart_pattern(
                "Double Bottom",
                PatternClassification::Bullish,
                PatternSignalType::Reversal,
                "chart_reversal",
                15,
            ));
        }
    }
    if lows.len() == 3 {
        let avg = (lows[0].price + lows[1].price + lows[2].price) / 3.0;
        if within_pct(lows[0].price, avg, PIVOT_TOLERANCE_PCT)
            && within_pct(lows[1].price, avg, PIVOT_TOLERANCE_PCT)
            && within_pct(lows[2].price, avg, PIVOT_TOLERANCE_PCT)
        {
            results.push(chart_pattern(
                "Triple Bottom",
                PatternClassification::Bullish,
                PatternSignalType::Reversal,
                "chart_reversal",
                20,
            ));
        }
    }

    results
}

fn detect_flags_pennants(
    candles: &[Candle],
    features: &FeatureSnapshot,
    interval: CandleInterval,
    price_ref: f64,
) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    if candles.len() < 15 {
        return results;
    }

    let trend_window = 10;
    let flag_window = 5;
    let trend_start = candles.len() - flag_window - trend_window;
    let trend_end = candles.len() - flag_window;
    let trend_start_close = candles.get(trend_start).map(|c| c.close).unwrap_or(0.0);
    let trend_end_close = candles.get(trend_end).map(|c| c.close).unwrap_or(0.0);
    if trend_start_close.abs() <= f64::EPSILON {
        return results;
    }

    let trend_pct = (trend_end_close - trend_start_close) / trend_start_close;
    if trend_pct.abs() < FLAG_TREND_THRESHOLD {
        return results;
    }

    let flag_slice = &candles[candles.len() - flag_window..];
    let flag_high = flag_slice
        .iter()
        .map(|c| c.high)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(price_ref);
    let flag_low = flag_slice
        .iter()
        .map(|c| c.low)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(price_ref);
    let flag_range_pct = (flag_high - flag_low) / trend_end_close.abs().max(1.0);
    if flag_range_pct > FLAG_RANGE_THRESHOLD {
        return results;
    }

    let (support, resistance) = trendline_pair(features);
    let (Some(support), Some(resistance)) = (support, resistance) else {
        return results;
    };
    let support_slope = slope_per_bar(&support, interval);
    let resistance_slope = slope_per_bar(&resistance, interval);

    if trend_pct > 0.0 {
        if support_slope < 0.0 && resistance_slope < 0.0 {
            results.push(chart_pattern(
                "Bull Flag",
                PatternClassification::Bullish,
                PatternSignalType::Continuation,
                "chart_continuation",
                10,
            ));
        } else if support_slope > 0.0 && resistance_slope < 0.0 {
            results.push(chart_pattern(
                "Bull Pennant",
                PatternClassification::Bullish,
                PatternSignalType::Continuation,
                "chart_continuation",
                10,
            ));
        }
    } else {
        if support_slope > 0.0 && resistance_slope > 0.0 {
            results.push(chart_pattern(
                "Bear Flag",
                PatternClassification::Bearish,
                PatternSignalType::Continuation,
                "chart_continuation",
                10,
            ));
        } else if support_slope > 0.0 && resistance_slope < 0.0 {
            results.push(chart_pattern(
                "Bear Pennant",
                PatternClassification::Bearish,
                PatternSignalType::Continuation,
                "chart_continuation",
                10,
            ));
        }
    }

    results
}

fn detect_three_methods(candles: &[Candle]) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    if rising_three_methods(candles) {
        results.push(chart_pattern(
            "Rising Three Methods",
            PatternClassification::Bullish,
            PatternSignalType::Continuation,
            "chart_continuation",
            5,
        ));
    }
    if falling_three_methods(candles) {
        results.push(chart_pattern(
            "Falling Three Methods",
            PatternClassification::Bearish,
            PatternSignalType::Continuation,
            "chart_continuation",
            5,
        ));
    }
    results
}

fn rising_three_methods(candles: &[Candle]) -> bool {
    if candles.len() < 5 {
        return false;
    }
    let c = candle_at(candles, 0);
    let c1 = candle_at(candles, 1);
    let c2 = candle_at(candles, 2);
    let c3 = candle_at(candles, 3);
    let c4 = candle_at(candles, 4);

    let Some(c) = c else { return false; };
    let Some(c1) = c1 else { return false; };
    let Some(c2) = c2 else { return false; };
    let Some(c3) = c3 else { return false; };
    let Some(c4) = c4 else { return false; };

    let Some(avg_range_20) = avg_range(candles, 20, 4) else {
        return false;
    };
    let Some(max_high_10) = max_high(candles, 10, 4) else {
        return false;
    };

    10.0 * (c4.close - c4.open) >= 7.0 * range(c4)
        && range(c4) >= avg_range_20
        && approx_eq(c4.high, max_high_10)
        && 2.0 * c3.close == 2.0 * c4.open + c4.high - c4.low
        && c2.open > c4.open
        && c.open > c4.open
        && 5.0 * c.open <= 3.0 * c4.high + 2.0 * c4.low
        && c.close > c4.close
}

fn falling_three_methods(candles: &[Candle]) -> bool {
    if candles.len() < 5 {
        return false;
    }
    let c = candle_at(candles, 0);
    let c1 = candle_at(candles, 1);
    let c2 = candle_at(candles, 2);
    let c3 = candle_at(candles, 3);
    let c4 = candle_at(candles, 4);

    let Some(c) = c else { return false; };
    let Some(c1) = c1 else { return false; };
    let Some(c2) = c2 else { return false; };
    let Some(c3) = c3 else { return false; };
    let Some(c4) = c4 else { return false; };

    body(c4) > 0.5 * range(c4)
        && c4.close < c4.open
        && body(c3) < body(c4)
        && body(c2) < body(c4)
        && body(c1) < body(c4)
        && c3.low >= c4.low
        && c3.high <= c4.high
        && c2.low >= c4.low
        && c2.high <= c4.high
        && c1.low >= c4.low
        && c1.high <= c4.high
        && c2.high > c3.high
        && c1.high > c2.high
        && c.close < c.open
        && c.close < c4.close
}

fn detect_cup_handle(features: &FeatureSnapshot) -> Vec<DetectedPattern> {
    let mut results = Vec::new();
    let highs = last_pivots(&features.pivots, PivotKind::High, 2);
    let lows = last_pivots(&features.pivots, PivotKind::Low, 2);

    if highs.len() == 2 && lows.len() >= 1 {
        let left = highs[0];
        let right = highs[1];
        let cup_low = lows[0];
        if within_pct(left.price, right.price, PIVOT_TOLERANCE_PCT)
            && cup_low.price < left.price * (1.0 - 0.05)
        {
            results.push(chart_pattern(
                "Cup and Handle",
                PatternClassification::Bullish,
                PatternSignalType::Continuation,
                "chart_continuation",
                25,
            ));
        }
    }

    results
}

fn trendline_pair(features: &FeatureSnapshot) -> (Option<Trendline>, Option<Trendline>) {
    let mut support = None;
    let mut resistance = None;
    for line in &features.trendlines {
        match line.kind {
            TrendlineKind::Support => support = Some(line.clone()),
            TrendlineKind::Resistance => resistance = Some(line.clone()),
        }
    }
    (support, resistance)
}

fn slope_per_bar(line: &Trendline, interval: CandleInterval) -> f64 {
    line.slope * interval.ms() as f64
}

fn is_flat(slope_per_bar: f64, price_ref: f64) -> bool {
    slope_per_bar.abs() <= price_ref * FLAT_SLOPE_PCT
}

fn within_pct(a: f64, b: f64, pct: f64) -> bool {
    if b.abs() <= f64::EPSILON {
        return false;
    }
    ((a - b) / b).abs() <= pct
}

fn chart_pattern(
    name: &'static str,
    classification: PatternClassification,
    signal_type: PatternSignalType,
    category: &'static str,
    window: usize,
) -> DetectedPattern {
    DetectedPattern {
        pattern: name,
        category,
        classification,
        signal_type,
        confidence: 0.65,
        window,
        notes: None,
    }
}

fn last_pivots(pivots: &[Pivot], kind: PivotKind, count: usize) -> Vec<&Pivot> {
    let mut filtered: Vec<&Pivot> = pivots.iter().filter(|pivot| pivot.kind == kind).collect();
    filtered.sort_by_key(|pivot| pivot.index);
    if filtered.len() < count {
        return Vec::new();
    }
    filtered.split_off(filtered.len() - count)
}

fn candle_at(candles: &[Candle], offset: usize) -> Option<&Candle> {
    if candles.len() <= offset {
        return None;
    }
    let idx = candles.len() - 1 - offset;
    candles.get(idx)
}

fn range(candle: &Candle) -> f64 {
    candle.high - candle.low
}

fn body(candle: &Candle) -> f64 {
    (candle.close - candle.open).abs()
}

fn avg_range(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    let sum = slice.iter().map(range).sum::<f64>();
    Some(sum / slice.len() as f64)
}

fn max_high(candles: &[Candle], window: usize, offset: usize) -> Option<f64> {
    let slice = window_slice(candles, window, offset)?;
    slice.iter().map(|c| c.high).max_by(|a, b| a.partial_cmp(b).unwrap())
}

fn window_slice(candles: &[Candle], window: usize, offset: usize) -> Option<&[Candle]> {
    if window == 0 || candles.len() <= offset {
        return None;
    }
    let end = candles.len() - 1 - offset;
    if end + 1 < window {
        return None;
    }
    let start = end + 1 - window;
    candles.get(start..=end)
}

fn approx_eq(a: f64, b: f64) -> bool {
    let tol = 1e-6_f64.max(1e-6 * b.abs());
    (a - b).abs() <= tol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_double_top_from_pivots() {
        let pivots = vec![
            Pivot {
                index: 1,
                time: 0,
                price: 100.0,
                kind: PivotKind::High,
            },
            Pivot {
                index: 2,
                time: 1,
                price: 102.0,
                kind: PivotKind::High,
            },
        ];
        let features = FeatureSnapshot {
            as_of_ms: 0,
            body_ratios: Vec::new(),
            gaps: Vec::new(),
            pivots,
            trendlines: Vec::new(),
            ranges: Vec::new(),
            atr: None,
            volatility: None,
        };

        let results = detect_double_triple(&features);
        assert!(results
            .iter()
            .any(|pattern| pattern.pattern == "Double Top"));
    }
}
