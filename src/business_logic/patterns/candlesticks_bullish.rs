use crate::models::candle::Candle;
use crate::models::patterns::PatternClassification;

use super::candlesticks::{
    approx_eq, avg_high_low_diff, body, build_pattern, candle, min_low, min_open, range, stochastic,
};
use super::DetectedPattern;

const STOCH_WINDOW: usize = 14;

pub(super) fn detect(candles: &[Candle]) -> Vec<DetectedPattern> {
    let mut results = Vec::new();

    if abandoned_baby(candles) {
        results.push(build_pattern(
            "Abandoned Baby",
            PatternClassification::Bullish,
            3,
        ));
    }
    if belt_hold(candles) {
        results.push(build_pattern(
            "Belt Hold",
            PatternClassification::Bullish,
            4,
        ));
    }
    if breakaway(candles) {
        results.push(build_pattern(
            "Breakaway",
            PatternClassification::Bullish,
            5,
        ));
    }
    if doji_dragonfly(candles) {
        results.push(build_pattern(
            "Doji (Dragonfly)",
            PatternClassification::Bullish,
            1,
        ));
    }
    if doji_star(candles) {
        results.push(build_pattern(
            "Doji Star",
            PatternClassification::Bullish,
            2,
        ));
    }
    if engulfing(candles) {
        results.push(build_pattern(
            "Engulfing",
            PatternClassification::Bullish,
            2,
        ));
    }
    if hammer(candles) {
        results.push(build_pattern("Hammer", PatternClassification::Bullish, 1));
    }
    if harami(candles) {
        results.push(build_pattern("Harami", PatternClassification::Bullish, 2));
    }
    if inverted_hammer(candles) {
        results.push(build_pattern(
            "Inverted Hammer",
            PatternClassification::Bullish,
            2,
        ));
    }
    if morning_star(candles) {
        results.push(build_pattern(
            "Morning Star",
            PatternClassification::Bullish,
            3,
        ));
    }
    if morning_doji_star(candles) {
        results.push(build_pattern(
            "Morning Doji Star",
            PatternClassification::Bullish,
            3,
        ));
    }
    if piercing_line(candles) {
        results.push(build_pattern(
            "Piercing Line",
            PatternClassification::Bullish,
            2,
        ));
    }
    if three_white_soldiers(candles) {
        results.push(build_pattern(
            "Three White Soldiers",
            PatternClassification::Bullish,
            4,
        ));
    }
    if tweezer_bottom(candles) {
        results.push(build_pattern(
            "Tweezer Bottom",
            PatternClassification::Bullish,
            2,
        ));
    }

    results
}

fn abandoned_baby(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return false;
    };

    2.0 * body(c2) > range(c2)
        && c2.close > c2.open
        && 20.0 * body(c1) <= range(c1)
        && 5.0 * (((c1.close + c1.open) / 2.0) - c1.low) >= 2.0 * range(c1)
        && 5.0 * (((c1.close + c1.open) / 2.0) - c1.low) <= 3.0 * range(c1)
        && c1.low > c2.high
        && c.close < c.open
        && c.high < c1.low
        && c.open > c2.close
        && (c.low > c2.open || c.close < c2.low)
}

fn belt_hold(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2), Some(c3)) = (
        candle(candles, 0),
        candle(candles, 1),
        candle(candles, 2),
        candle(candles, 3),
    ) else {
        return false;
    };

    let Some(min_open_10) = min_open(candles, 10, 0) else {
        return false;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };

    approx_eq(c.open, min_open_10)
        && c.open < c1.low
        && 10.0 * (c.close - c.open) >= 7.0 * range(c)
        && 5.0 * range(c) >= 6.0 * avg_range_10
        && 100.0 * (c.open - c.low) <= range(c)
        && 2.0 * c.close <= c1.high - c1.low
        && c1.high > c1.low
        && c.high > c.low
        && c1.close < c2.close
        && c2.close < c3.close
}

fn breakaway(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2), Some(c3), Some(c4)) = (
        candle(candles, 0),
        candle(candles, 1),
        candle(candles, 2),
        candle(candles, 3),
        candle(candles, 4),
    ) else {
        return false;
    };

    c4.close < c4.open
        && 2.0 * body(c4) > range(c4)
        && c3.close < c3.open
        && c3.high < c4.low
        && c2.close < c3.close
        && c1.close < c2.close
        && 5.0 * body(c) > 3.0 * range(c)
        && c.close > c.open
        && c.close > c3.high
}

fn doji_dragonfly(candles: &[Candle]) -> bool {
    let Some(c) = candle(candles, 0) else {
        return false;
    };

    let Some(stoch) = stochastic(candles, STOCH_WINDOW, 0) else {
        return false;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };
    let Some(min_low_10) = min_low(candles, 10, 0) else {
        return false;
    };

    50.0 * body(c) <= range(c)
        && stoch >= 70.0
        && range(c) >= avg_range_10
        && approx_eq(c.low, min_low_10)
}

fn doji_star(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return false;
    };
    let Some(min_low_10) = min_low(candles, 10, 0) else {
        return false;
    };

    10.0 * (c1.open - c1.close) >= 7.0 * range(c1)
        && range(c1) >= avg_range_10_prev
        && c.close < c1.close
        && c.open < c1.close
        && 20.0 * body(c) <= range(c)
        && approx_eq(c.low, min_low_10)
        && c1.high > c1.low
        && c.high > c.low
}

fn engulfing(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };

    c1.open > c1.close
        && 10.0 * (c.close - c.open) >= 7.0 * range(c)
        && c.close > c1.open
        && c1.close > c.open
        && 10.0 * range(c) >= 12.0 * avg_range_10
}

fn hammer(candles: &[Candle]) -> bool {
    let Some(c) = candle(candles, 0) else {
        return false;
    };

    let Some(stoch) = stochastic(candles, STOCH_WINDOW, 0) else {
        return false;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };
    let Some(min_low_5) = min_low(candles, 5, 0) else {
        return false;
    };

    5.0 * body(c) <= range(c)
        && 10.0 * body(c) >= range(c)
        && 2.0 * c.open >= c.high + c.low
        && stoch >= 50.0
        && (20.0 * c.open >= 19.0 * c.high + c.low || stoch >= 95.0)
        && 10.0 * range(c) >= 8.0 * avg_range_10
        && approx_eq(c.low, min_low_5)
        && c.high > c.low
}

fn harami(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return false;
    };

    10.0 * (c1.open - c1.close) >= 7.0 * range(c1)
        && range(c1) >= avg_range_10_prev
        && c.close > c.open
        && c.open > c1.close
        && c1.open > c.close
        && 6.0 * (c1.open - c1.close) >= 10.0 * (c.close - c.open)
}

fn inverted_hammer(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(stoch) = stochastic(candles, STOCH_WINDOW, 0) else {
        return false;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };
    let Some(min_low_5) = min_low(candles, 5, 0) else {
        return false;
    };

    5.0 * body(c) <= range(c)
        && 10.0 * body(c) >= range(c)
        && 2.0 * (c.high - c.open) >= range(c)
        && 2.0 * (c.high - c.close) >= range(c)
        && (2.0 * (c.open - c.low) <= range(c) || 20.0 * (c.close - c.low) <= range(c))
        && 5.0 * range(c) >= 4.0 * avg_range_10
        && 2.0 * c.open <= c1.high + c1.low
        && stoch <= 50.0
        && approx_eq(c.low, min_low_5)
        && c.high > c.low
}

fn morning_star(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return false;
    };

    c2.open > c2.close
        && 5.0 * (c2.open - c2.close) > 3.0 * range(c2)
        && c2.close > c1.open
        && 2.0 * (c1.open - c1.close).abs() < (c2.open - c2.close).abs()
        && range(c1) > 3.0 * (c1.close - c1.open)
        && c.close > c.open
        && c.open > c1.open
        && c.open > c1.close
}

fn morning_doji_star(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return false;
    };

    let Some(avg_range_10_prev2) = avg_high_low_diff(candles, 10, 2) else {
        return false;
    };

    10.0 * (c2.open - c2.close) >= 7.0 * range(c2)
        && range(c2) >= avg_range_10_prev2
        && 10.0 * (c.close - c.open) >= 7.0 * range(c)
        && c.open > c1.close
        && c.open > c1.open
}

fn piercing_line(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return false;
    };

    c1.open > c1.close
        && range(c1) >= avg_range_10_prev
        && c.open < c1.close
        && 2.0 * c.close > c1.close + c1.open
        && c.close < c1.open
}

fn three_white_soldiers(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2), Some(c3)) = (
        candle(candles, 0),
        candle(candles, 1),
        candle(candles, 2),
        candle(candles, 3),
    ) else {
        return false;
    };

    let Some(avg_range_21) = avg_high_low_diff(candles, 21, 0) else {
        return false;
    };

    c.close > c1.close
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
}

fn tweezer_bottom(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_20) = avg_high_low_diff(candles, 20, 0) else {
        return false;
    };

    approx_eq(c.low, c1.low)
        && 5.0 * body(c) < body(c1)
        && 10.0 * body(c1) >= 9.0 * range(c1)
        && 10.0 * range(c1) >= 13.0 * avg_range_20
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
