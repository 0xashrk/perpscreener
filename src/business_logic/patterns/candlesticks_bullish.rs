use crate::models::candle::Candle;
use crate::models::patterns::PatternClassification;

use super::candlesticks::{
    approx_eq, avg_high_low_diff, body, body_ratio, build_pattern, candle, clamp01,
    lower_wick_ratio, min_low, min_open, pattern_confidence, proximity_score, range, range_score,
    scaled_score, stochastic, trend_score, upper_wick_ratio,
};
use super::DetectedPattern;

const STOCH_WINDOW: usize = 14;

pub(super) fn detect(candles: &[Candle]) -> Vec<DetectedPattern> {
    let mut results = Vec::new();

    if let Some(confidence) = abandoned_baby(candles) {
        results.push(build_pattern(
            "Abandoned Baby",
            PatternClassification::Bullish,
            3,
            confidence,
        ));
    }
    if let Some(confidence) = belt_hold(candles) {
        results.push(build_pattern(
            "Belt Hold",
            PatternClassification::Bullish,
            4,
            confidence,
        ));
    }
    if let Some(confidence) = breakaway(candles) {
        results.push(build_pattern(
            "Breakaway",
            PatternClassification::Bullish,
            5,
            confidence,
        ));
    }
    if let Some(confidence) = doji_dragonfly(candles) {
        results.push(build_pattern(
            "Doji (Dragonfly)",
            PatternClassification::Bullish,
            1,
            confidence,
        ));
    }
    if let Some(confidence) = doji_star(candles) {
        results.push(build_pattern(
            "Doji Star",
            PatternClassification::Bullish,
            2,
            confidence,
        ));
    }
    if let Some(confidence) = engulfing(candles) {
        results.push(build_pattern(
            "Engulfing",
            PatternClassification::Bullish,
            2,
            confidence,
        ));
    }
    if let Some(confidence) = hammer(candles) {
        results.push(build_pattern(
            "Hammer",
            PatternClassification::Bullish,
            1,
            confidence,
        ));
    }
    if let Some(confidence) = harami(candles) {
        results.push(build_pattern(
            "Harami",
            PatternClassification::Bullish,
            2,
            confidence,
        ));
    }
    if let Some(confidence) = inverted_hammer(candles) {
        results.push(build_pattern(
            "Inverted Hammer",
            PatternClassification::Bullish,
            2,
            confidence,
        ));
    }
    if let Some(confidence) = morning_star(candles) {
        results.push(build_pattern(
            "Morning Star",
            PatternClassification::Bullish,
            3,
            confidence,
        ));
    }
    if let Some(confidence) = morning_doji_star(candles) {
        results.push(build_pattern(
            "Morning Doji Star",
            PatternClassification::Bullish,
            3,
            confidence,
        ));
    }
    if let Some(confidence) = piercing_line(candles) {
        results.push(build_pattern(
            "Piercing Line",
            PatternClassification::Bullish,
            2,
            confidence,
        ));
    }
    if let Some(confidence) = three_white_soldiers(candles) {
        results.push(build_pattern(
            "Three White Soldiers",
            PatternClassification::Bullish,
            4,
            confidence,
        ));
    }
    if let Some(confidence) = tweezer_bottom(candles) {
        results.push(build_pattern(
            "Tweezer Bottom",
            PatternClassification::Bullish,
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

    if 2.0 * body(c2) > range(c2)
        && c2.close > c2.open
        && 20.0 * body(c1) <= range(c1)
        && 5.0 * (((c1.close + c1.open) / 2.0) - c1.low) >= 2.0 * range(c1)
        && 5.0 * (((c1.close + c1.open) / 2.0) - c1.low) <= 3.0 * range(c1)
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

    let Some(min_open_10) = min_open(candles, 10, 0) else {
        return None;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };

    if approx_eq(c.open, min_open_10)
        && c.open < c1.low
        && 10.0 * (c.close - c.open) >= 7.0 * range(c)
        && 5.0 * range(c) >= 6.0 * avg_range_10
        && 100.0 * (c.open - c.low) <= range(c)
        && 2.0 * c.close <= c1.high - c1.low
        && c1.high > c1.low
        && c.high > c.low
        && c1.close < c2.close
        && c2.close < c3.close
    {
        let open_score = proximity_score(c.open - min_open_10, avg_range_10);
        let body_score = body_ratio(c);
        let range_strength = range_score(range(c), avg_range_10);
        let wick_score = 1.0 - lower_wick_ratio(c);
        let trend_strength = trend_score(c3.close, c1.close);
        let confidence = pattern_confidence(
            0.55,
            &[
                open_score,
                body_score,
                range_strength,
                wick_score,
                trend_strength,
            ],
        );
        return Some(confidence);
    }

    None
}

fn breakaway(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2), Some(c3), Some(c4)) = (
        candle(candles, 0),
        candle(candles, 1),
        candle(candles, 2),
        candle(candles, 3),
        candle(candles, 4),
    ) else {
        return None;
    };

    if c4.close < c4.open
        && 2.0 * body(c4) > range(c4)
        && c3.close < c3.open
        && c3.high < c4.low
        && c2.close < c3.close
        && c1.close < c2.close
        && 5.0 * body(c) > 3.0 * range(c)
        && c.close > c.open
        && c.close > c3.high
    {
        let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
            return None;
        };
        let body_score = body_ratio(c);
        let range_strength = range_score(range(c), avg_range_10);
        let breakout_score = scaled_score(c.close - c3.high, range(c).abs());
        let trend_strength = trend_score(c4.close, c.close);
        let confidence = pattern_confidence(
            0.55,
            &[body_score, range_strength, breakout_score, trend_strength],
        );
        return Some(confidence);
    }

    None
}

fn doji_dragonfly(candles: &[Candle]) -> Option<f64> {
    let Some(c) = candle(candles, 0) else {
        return None;
    };

    let Some(stoch) = stochastic(candles, STOCH_WINDOW, 0) else {
        return None;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };
    let Some(min_low_10) = min_low(candles, 10, 0) else {
        return None;
    };

    if 50.0 * body(c) <= range(c)
        && stoch >= 70.0
        && range(c) >= avg_range_10
        && approx_eq(c.low, min_low_10)
    {
        let doji_score = 1.0 - body_ratio(c);
        let wick_score = lower_wick_ratio(c);
        let stoch_score = scaled_score(stoch, 100.0);
        let range_strength = range_score(range(c), avg_range_10);
        let low_score = proximity_score(c.low - min_low_10, avg_range_10);
        let confidence = pattern_confidence(
            0.55,
            &[
                doji_score,
                wick_score,
                stoch_score,
                range_strength,
                low_score,
            ],
        );
        return Some(confidence);
    }

    None
}

fn doji_star(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return None;
    };
    let Some(min_low_10) = min_low(candles, 10, 0) else {
        return None;
    };

    if 10.0 * (c1.open - c1.close) >= 7.0 * range(c1)
        && range(c1) >= avg_range_10_prev
        && c.close < c1.close
        && c.open < c1.close
        && 20.0 * body(c) <= range(c)
        && approx_eq(c.low, min_low_10)
        && c1.high > c1.low
        && c.high > c.low
    {
        let doji_score = 1.0 - body_ratio(c);
        let prev_body = body_ratio(c1);
        let range_strength = range_score(range(c1), avg_range_10_prev);
        let low_score = proximity_score(c.low - min_low_10, avg_range_10_prev);
        let confidence =
            pattern_confidence(0.55, &[doji_score, prev_body, range_strength, low_score]);
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

    if c1.open > c1.close
        && 10.0 * (c.close - c.open) >= 7.0 * range(c)
        && c.close > c1.open
        && c1.close > c.open
        && 10.0 * range(c) >= 12.0 * avg_range_10
    {
        let body_score = body_ratio(c);
        let range_strength = range_score(range(c), avg_range_10);
        let engulf_score = scaled_score(c.close - c1.open, range(c).abs());
        let prev_body = body_ratio(c1);
        let confidence =
            pattern_confidence(0.55, &[body_score, range_strength, engulf_score, prev_body]);
        return Some(confidence);
    }

    None
}

fn hammer(candles: &[Candle]) -> Option<f64> {
    let Some(c) = candle(candles, 0) else {
        return None;
    };

    let Some(stoch) = stochastic(candles, STOCH_WINDOW, 0) else {
        return None;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };
    let Some(min_low_5) = min_low(candles, 5, 0) else {
        return None;
    };

    if 5.0 * body(c) <= range(c)
        && 10.0 * body(c) >= range(c)
        && 2.0 * c.open >= c.high + c.low
        && stoch >= 50.0
        && (20.0 * c.open >= 19.0 * c.high + c.low || stoch >= 95.0)
        && 10.0 * range(c) >= 8.0 * avg_range_10
        && approx_eq(c.low, min_low_5)
        && c.high > c.low
    {
        let wick_score = lower_wick_ratio(c);
        let body_score = centered_score(body_ratio(c), 0.2, 0.2);
        let stoch_score = scaled_score(stoch, 100.0);
        let range_strength = range_score(range(c), avg_range_10);
        let low_score = proximity_score(c.low - min_low_5, avg_range_10);
        let confidence = pattern_confidence(
            0.55,
            &[
                wick_score,
                body_score,
                stoch_score,
                range_strength,
                low_score,
            ],
        );
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

    if 10.0 * (c1.open - c1.close) >= 7.0 * range(c1)
        && range(c1) >= avg_range_10_prev
        && c.close > c.open
        && c.open > c1.close
        && c1.open > c.close
        && 6.0 * (c1.open - c1.close) >= 10.0 * (c.close - c.open)
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

fn inverted_hammer(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(stoch) = stochastic(candles, STOCH_WINDOW, 0) else {
        return None;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return None;
    };
    let Some(min_low_5) = min_low(candles, 5, 0) else {
        return None;
    };

    if 5.0 * body(c) <= range(c)
        && 10.0 * body(c) >= range(c)
        && 2.0 * (c.high - c.open) >= range(c)
        && 2.0 * (c.high - c.close) >= range(c)
        && (2.0 * (c.open - c.low) <= range(c) || 20.0 * (c.close - c.low) <= range(c))
        && 5.0 * range(c) >= 4.0 * avg_range_10
        && 2.0 * c.open <= c1.high + c1.low
        && stoch <= 50.0
        && approx_eq(c.low, min_low_5)
        && c.high > c.low
    {
        let wick_score = upper_wick_ratio(c);
        let body_score = centered_score(body_ratio(c), 0.2, 0.2);
        let stoch_score = 1.0 - scaled_score(stoch, 100.0);
        let range_strength = range_score(range(c), avg_range_10);
        let low_score = proximity_score(c.low - min_low_5, avg_range_10);
        let confidence = pattern_confidence(
            0.55,
            &[
                wick_score,
                body_score,
                stoch_score,
                range_strength,
                low_score,
            ],
        );
        return Some(confidence);
    }

    None
}

fn morning_star(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return None;
    };

    if c2.open > c2.close
        && 5.0 * (c2.open - c2.close) > 3.0 * range(c2)
        && c2.close > c1.open
        && 2.0 * (c1.open - c1.close).abs() < (c2.open - c2.close).abs()
        && range(c1) > 3.0 * (c1.close - c1.open)
        && c.close > c.open
        && c.open > c1.open
        && c.open > c1.close
    {
        let first_body = body_ratio(c2);
        let mid_small = 1.0 - body_ratio(c1);
        let last_body = body_ratio(c);
        let recovery = scaled_score(c.close - c1.open, range(c2).abs());
        let confidence = pattern_confidence(0.55, &[first_body, mid_small, last_body, recovery]);
        return Some(confidence);
    }

    None
}

fn morning_doji_star(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return None;
    };

    let Some(avg_range_10_prev2) = avg_high_low_diff(candles, 10, 2) else {
        return None;
    };

    if 10.0 * (c2.open - c2.close) >= 7.0 * range(c2)
        && range(c2) >= avg_range_10_prev2
        && 10.0 * (c.close - c.open) >= 7.0 * range(c)
        && c.open > c1.close
        && c.open > c1.open
    {
        let first_body = body_ratio(c2);
        let doji_score = 1.0 - body_ratio(c1);
        let last_body = body_ratio(c);
        let range_strength = range_score(range(c2), avg_range_10_prev2);
        let confidence =
            pattern_confidence(0.55, &[first_body, doji_score, last_body, range_strength]);
        return Some(confidence);
    }

    None
}

fn piercing_line(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return None;
    };

    if c1.open > c1.close
        && range(c1) >= avg_range_10_prev
        && c.open < c1.close
        && 2.0 * c.close > c1.close + c1.open
        && c.close < c1.open
    {
        let prev_body = body_ratio(c1);
        let body_score = body_ratio(c);
        let penetration = scaled_score(c.close - (c1.close + c1.open) / 2.0, body(c1).abs());
        let range_strength = range_score(range(c1), avg_range_10_prev);
        let confidence =
            pattern_confidence(0.55, &[prev_body, body_score, penetration, range_strength]);
        return Some(confidence);
    }

    None
}

fn three_white_soldiers(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1), Some(c2), Some(c3)) = (
        candle(candles, 0),
        candle(candles, 1),
        candle(candles, 2),
        candle(candles, 3),
    ) else {
        return None;
    };

    let Some(avg_range_21) = avg_high_low_diff(candles, 21, 0) else {
        return None;
    };

    if c.close > c1.close
        && c1.close > c2.close
        && c.close > c.open
        && c1.close > c1.open
        && c2.close > c2.open
        && 2.0 * body(c2) > range(c2)
        && 2.0 * body(c1) > range(c1)
        && range(c) > avg_range_21
        && c.open > c1.open
        && c.open < c1.close
        && c1.open > c2.open
        && c1.open < c2.close
        && c2.open > c3.open
        && c2.open < c3.close
        && 20.0 * c.close > 17.0 * c.high
        && 20.0 * c1.close > 17.0 * c1.high
        && 20.0 * c2.close > 17.0 * c2.high
    {
        let body_avg = (body_ratio(c) + body_ratio(c1) + body_ratio(c2)) / 3.0;
        let range_strength = range_score(range(c), avg_range_21);
        let progression = trend_score(c3.close, c.close);
        let confidence = pattern_confidence(0.6, &[body_avg, range_strength, progression]);
        return Some(confidence);
    }

    None
}

fn tweezer_bottom(candles: &[Candle]) -> Option<f64> {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return None;
    };

    let Some(avg_range_20) = avg_high_low_diff(candles, 20, 0) else {
        return None;
    };

    if approx_eq(c.low, c1.low)
        && 5.0 * body(c) < body(c1)
        && 10.0 * body(c1) >= 9.0 * range(c1)
        && 10.0 * range(c1) >= 13.0 * avg_range_20
    {
        let low_score = proximity_score(c.low - c1.low, avg_range_20);
        let body_score = body_ratio(c1);
        let range_strength = range_score(range(c1), avg_range_20);
        let confidence = pattern_confidence(0.55, &[low_score, body_score, range_strength]);
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
    fn detects_bullish_engulfing() {
        let mut candles = Vec::new();
        for _ in 0..8 {
            candles.push(candle(10.0, 9.0, 11.0, 8.0));
        }
        candles.push(candle(11.0, 9.0, 11.5, 8.5));
        candles.push(candle(8.5, 12.5, 13.0, 8.0));

        let detections = detect(&candles);
        assert!(detections
            .iter()
            .any(|pattern| pattern.pattern == "Engulfing"
                && pattern.classification == PatternClassification::Bullish));
    }
}
