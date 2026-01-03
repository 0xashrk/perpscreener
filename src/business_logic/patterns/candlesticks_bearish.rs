use crate::models::candle::Candle;
use crate::models::patterns::PatternClassification;

use super::candlesticks::{
    approx_eq, avg_high_low_diff, body, body_ratio, build_pattern, candle, clamp01,
    lower_wick_ratio, max_high, max_open, pattern_confidence, proximity_score, range,
    range_score, scaled_score, trend_score, upper_wick_ratio,
};
use super::DetectedPattern;

pub(super) fn detect(candles: &[Candle]) -> Vec<DetectedPattern> {
    let mut results = Vec::new();

    if let Some(confidence) = abandoned_baby(candles) {
        results.push(build_pattern(
            "Abandoned Baby",
            PatternClassification::Bearish,
            3,
            confidence,
        ));
    }
    if let Some(confidence) = belt_hold(candles) {
        results.push(build_pattern(
            "Belt Hold",
            PatternClassification::Bearish,
            4,
            confidence,
        ));
    }
    if let Some(confidence) = dark_cloud_cover(candles) {
        results.push(build_pattern(
            "Dark Cloud Cover",
            PatternClassification::Bearish,
            2,
            confidence,
        ));
    }
    if let Some(confidence) = doji_gravestone(candles) {
        results.push(build_pattern(
            "Doji (Gravestone)",
            PatternClassification::Bearish,
            1,
            confidence,
        ));
    }
    if let Some(confidence) = engulfing(candles) {
        results.push(build_pattern(
            "Engulfing",
            PatternClassification::Bearish,
            2,
            confidence,
        ));
    }
    if let Some(confidence) = evening_star(candles) {
        results.push(build_pattern(
            "Evening Star",
            PatternClassification::Bearish,
            3,
            confidence,
        ));
    }
    if let Some(confidence) = evening_doji_star(candles) {
        results.push(build_pattern(
            "Evening Doji Star",
            PatternClassification::Bearish,
            3,
            confidence,
        ));
    }
    if let Some(confidence) = hanging_man(candles) {
        results.push(build_pattern(
            "Hanging Man",
            PatternClassification::Bearish,
            1,
            confidence,
        ));
    }
    if let Some(confidence) = harami(candles) {
        results.push(build_pattern("Harami", PatternClassification::Bearish, 2, confidence));
    }
    if let Some(confidence) = shooting_star(candles) {
        results.push(build_pattern(
            "Shooting Star",
            PatternClassification::Bearish,
            1,
            confidence,
        ));
    }
    if let Some(confidence) = three_black_crows(candles) {
        results.push(build_pattern(
            "Three Black Crows",
            PatternClassification::Bearish,
            3,
            confidence,
        ));
    }
    if let Some(confidence) = tweezer_top(candles) {
        results.push(build_pattern(
            "Tweezer Top",
            PatternClassification::Bearish,
            2,
            confidence,
        ));
    }

    results
}

fn abandoned_baby(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return None;
    };

    if body(c2) > 0.5 * range(c2)
        && c2.close > c2.open
        && body(c1) <= 0.05 * range(c1)
        && ((c1.close + c1.open) / 2.0 - c1.low) >= 0.4 * range(c1)
        && ((c1.close + c1.open) / 2.0 - c1.low) <= 0.6 * range(c1)
        && c1.low > c2.high
        && c.close < c.open
        && c.high < c1.low
        && c.open > c2.close
        && (c.low > c2.open || c.close < c2.low)
    {
        let body_score = body_ratio(c2);
        let doji_score = 1.0 - body_ratio(c1);
        let gap1 = scaled_score(c1.low - c2.high, range(c2).abs());
        let gap2 = scaled_score(c1.low - c.high, range(c).abs().max(range(c1).abs()));
        let confidence = pattern_confidence(0.55, &[body_score, doji_score, gap1, gap2]);
        return Some(confidence);
    }

    None
}

fn belt_hold(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2), Some(c3)) = (
        candle(candles, 0),
        candle(candles, 1),
        candle(candles, 2),
        candle(candles, 3),
    ) else {
        return None;
    };

    let Some(max_open_10) = max_open(candles, 10, 0) else {
        return None;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };

    if approx_eq(c.open, max_open_10)
        && c.open > c1.high
        && c.open - c.close >= 0.7 * range(c)
        && range(c) >= 1.2 * avg_range_10
        && (c.high - c.open) <= 0.01 * range(c)
        && c.close >= c1.high - 0.5 * (c1.high - c1.low)
        && c1.high > c1.low
        && c.high > c.low
        && c1.close > c2.close
        && c2.close < c3.close
    {
        let open_score = proximity_score(c.open - max_open_10, avg_range_10);
        let body_score = body_ratio(c);
        let range_strength = range_score(range(c), avg_range_10);
        let wick_score = 1.0 - upper_wick_ratio(c);
        let trend_strength = trend_score(c3.close, c1.close);
        let confidence =
            pattern_confidence(0.55, &[open_score, body_score, range_strength, wick_score, trend_strength]);
        return Some(confidence);
    }

    None
}

fn dark_cloud_cover(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return None;
    };

    if (c1.close - c1.open) >= 0.7 * range(c1)
        && range(c1) >= avg_range_10_prev
        && c.open > c1.close
        && c.close < c1.close - 0.5 * (c1.close - c1.open)
        && c.close > c1.open
    {
        let prev_body = body_ratio(c1);
        let body_score = body_ratio(c);
        let penetration = scaled_score(c1.close - c.close, body(c1).abs());
        let range_strength = range_score(range(c1), avg_range_10_prev);
        let confidence =
            pattern_confidence(0.55, &[prev_body, body_score, penetration, range_strength]);
        return Some(confidence);
    }

    None
}

fn doji_gravestone(candles: &[Candle]) -> Option<f64> {
    let Some(c) = candle(candles, 0) else {
        return None;
    };

    let Some(max_high_10) = max_high(candles, 10, 0) else {
        return None;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };

    if body(c) <= 0.01 * range(c)
        && (c.high - c.close) >= 0.95 * range(c)
        && c.high > c.low
        && approx_eq(c.high, max_high_10)
        && range(c) >= avg_range_10
    {
        let doji_score = 1.0 - body_ratio(c);
        let wick_score = upper_wick_ratio(c);
        let range_strength = range_score(range(c), avg_range_10);
        let high_score = proximity_score(c.high - max_high_10, avg_range_10);
        let confidence =
            pattern_confidence(0.55, &[doji_score, wick_score, range_strength, high_score]);
        return Some(confidence);
    }

    None
}

fn engulfing(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };

    if c1.close > c1.open
        && c.open - c.close >= 0.7 * range(c)
        && c.close < c1.open
        && c.open > c1.close
        && range(c) >= 1.2 * avg_range_10
    {
        let body_score = body_ratio(c);
        let range_strength = range_score(range(c), avg_range_10);
        let engulf_score = scaled_score(c.open - c1.close, range(c).abs());
        let prev_body = body_ratio(c1);
        let confidence =
            pattern_confidence(0.55, &[body_score, range_strength, engulf_score, prev_body]);
        return Some(confidence);
    }

    None
}

fn evening_star(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return None;
    };

    let Some(avg_range_10_prev2) = avg_high_low_diff(candles, 10, 2) else {
        return None;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };

    if (c2.close - c2.open) >= 0.7 * range(c2)
        && range(c2) >= avg_range_10_prev2
        && c1.close > c2.close
        && c1.open > c2.close
        && range(c) >= avg_range_10
        && c.open - c.close >= 0.7 * range(c)
        && c.open < c1.open
        && c.open < c1.close
    {
        let first_body = body_ratio(c2);
        let mid_small = 1.0 - body_ratio(c1);
        let last_body = body_ratio(c);
        let range_strength = range_score(range(c2), avg_range_10_prev2);
        let confidence =
            pattern_confidence(0.55, &[first_body, mid_small, last_body, range_strength]);
        return Some(confidence);
    }

    None
}

fn evening_doji_star(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return None;
    };

    let Some(avg_range_21_prev) = avg_high_low_diff(candles, 21, 1) else {
        return None;
    };

    if body(c2) > 0.5 * range(c)
        && c2.close > c2.open
        && body(c1) < 0.05 * range(c1)
        && range(c1) < 0.2 * avg_range_21_prev
        && c1.open > c2.close
        && c.close < c.open
    {
        let first_body = body_ratio(c2);
        let doji_score = 1.0 - body_ratio(c1);
        let last_body = body_ratio(c);
        let range_strength = range_score(range(c1), avg_range_21_prev);
        let confidence =
            pattern_confidence(0.55, &[first_body, doji_score, last_body, range_strength]);
        return Some(confidence);
    }

    None
}

fn hanging_man(candles: &[Candle]) -> Option<f64> {
    let Some(c) = candle(candles, 0) else {
        return None;
    };

    let min_body = if c.close >= c.open { c.open } else { c.close };
    let body_size = body(c);
    let midpoint = (c.close + c.open) / 2.0;

    if (min_body - c.low) >= 2.0 * body_size
        && (midpoint - c.low) > 2.0 * (c.high - midpoint)
        && body_size > 0.01
    {
        let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
            return None;
        };
        let wick_score = lower_wick_ratio(c);
        let body_score = centered_score(body_ratio(c), 0.2, 0.2);
        let range_strength = range_score(range(c), avg_range_10);
        let upper_wick = 1.0 - upper_wick_ratio(c);
        let confidence =
            pattern_confidence(0.55, &[wick_score, body_score, range_strength, upper_wick]);
        return Some(confidence);
    }

    None
}

fn harami(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return None;
    };

    if (c1.close - c1.open) >= 0.7 * range(c1)
        && range(c1) >= avg_range_10_prev
        && c.close < c.open
        && c.open < c1.close
        && c.close > c1.open
        && c.open - c.close <= 0.6 * (c1.close - c1.open)
    {
        let prev_body = body_ratio(c1);
        let body_small = 1.0 - body_ratio(c);
        let range_strength = range_score(range(c1), avg_range_10_prev);
        let size_ratio = scaled_score(body(c1) - body(c), body(c1).abs());
        let confidence =
            pattern_confidence(0.55, &[prev_body, body_small, range_strength, size_ratio]);
        return Some(confidence);
    }

    None
}

fn shooting_star(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };
    let Some(max_high_5) = max_high(candles, 5, 0) else {
        return None;
    };

    let body_size = body(c);
    let range_size = range(c);
    let upper_shadow = c.high - c.open.max(c.close);
    let lower_shadow = c.open.min(c.close) - c.low;

    if body_size <= 0.2 * range_size
        && body_size >= 0.1 * range_size
        && upper_shadow >= 0.5 * range_size
        && (lower_shadow <= 0.05 * range_size)
        && range_size >= 0.8 * avg_range_10
        && c.open >= (c1.low + 0.5 * (c1.high - c1.low))
        && c.close >= (c1.low + 0.5 * (c1.high - c1.low))
        && approx_eq(c.high, max_high_5)
        && c.high > c.low
    {
        let wick_score = upper_wick_ratio(c);
        let body_score = centered_score(body_ratio(c), 0.15, 0.15);
        let range_strength = range_score(range(c), avg_range_10);
        let high_score = proximity_score(c.high - max_high_5, avg_range_10);
        let confidence = pattern_confidence(
            0.55,
            &[wick_score, body_score, range_strength, high_score],
        );
        return Some(confidence);
    }

    None
}

fn three_black_crows(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return None;
    };

    if c1.open < c2.open
        && c1.open > c2.close
        && c.open < c1.open
        && c.open > c1.close
        && c1.close < c2.low
        && c.close < c1.low
        && c2.close < 1.05 * c2.low
        && c1.close < 1.05 * c1.low
        && c.close < 1.05 * c.low
    {
        let body_avg = (body_ratio(c) + body_ratio(c1) + body_ratio(c2)) / 3.0;
        let progression = trend_score(c2.open, c.close);
        let confidence = pattern_confidence(0.6, &[body_avg, progression]);
        return Some(confidence);
    }

    None
}

fn tweezer_top(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(avg_range_20) = avg_high_low_diff(candles, 20, 0) else {
        return None;
    };

    if approx_eq(c.high, c1.high)
        && body(c) < 0.2 * body(c1)
        && body(c1) >= 0.9 * range(c1)
        && range(c1) >= 1.3 * avg_range_20
    {
        let high_score = proximity_score(c.high - c1.high, avg_range_20);
        let body_score = body_ratio(c1);
        let range_strength = range_score(range(c1), avg_range_20);
        let confidence = pattern_confidence(0.55, &[high_score, body_score, range_strength]);
        return Some(confidence);
    }

    None
}

fn centered_score(value: f64, target: f64, tolerance: f64) -> f64 {
    if tolerance.abs() <= f64::EPSILON {
        return 0.0;
    }
    clamp01(1.0 - ((value - target).abs() / tolerance))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn detects_bearish_engulfing() {
        let mut candles = Vec::new();
        for _ in 0..8 {
            candles.push(candle(10.0, 11.0, 11.0, 8.0));
        }
        candles.push(candle(9.0, 11.0, 11.5, 8.5));
        candles.push(candle(12.0, 8.0, 12.5, 7.5));

        let detections = detect(&candles);
        assert!(detections
            .iter()
            .any(|pattern| pattern.pattern == "Engulfing"
                && pattern.classification == PatternClassification::Bearish));
    }
}
