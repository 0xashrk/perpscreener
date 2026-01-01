use crate::models::candle::Candle;
use crate::models::patterns::PatternClassification;

use super::candlesticks::{
    approx_eq, avg_high_low_diff, body, build_pattern, candle, max_high, max_open, range,
};
use super::DetectedPattern;

pub(super) fn detect(candles: &[Candle]) -> Vec<DetectedPattern> {
    let mut results = Vec::new();

    if abandoned_baby(candles) {
        results.push(build_pattern(
            "Abandoned Baby",
            PatternClassification::Bearish,
            3,
        ));
    }
    if belt_hold(candles) {
        results.push(build_pattern(
            "Belt Hold",
            PatternClassification::Bearish,
            4,
        ));
    }
    if dark_cloud_cover(candles) {
        results.push(build_pattern(
            "Dark Cloud Cover",
            PatternClassification::Bearish,
            2,
        ));
    }
    if doji_gravestone(candles) {
        results.push(build_pattern(
            "Doji (Gravestone)",
            PatternClassification::Bearish,
            1,
        ));
    }
    if engulfing(candles) {
        results.push(build_pattern(
            "Engulfing",
            PatternClassification::Bearish,
            2,
        ));
    }
    if evening_star(candles) {
        results.push(build_pattern(
            "Evening Star",
            PatternClassification::Bearish,
            3,
        ));
    }
    if evening_doji_star(candles) {
        results.push(build_pattern(
            "Evening Doji Star",
            PatternClassification::Bearish,
            3,
        ));
    }
    if hanging_man(candles) {
        results.push(build_pattern(
            "Hanging Man",
            PatternClassification::Bearish,
            1,
        ));
    }
    if harami(candles) {
        results.push(build_pattern("Harami", PatternClassification::Bearish, 2));
    }
    if shooting_star(candles) {
        results.push(build_pattern(
            "Shooting Star",
            PatternClassification::Bearish,
            1,
        ));
    }
    if three_black_crows(candles) {
        results.push(build_pattern(
            "Three Black Crows",
            PatternClassification::Bearish,
            3,
        ));
    }
    if tweezer_top(candles) {
        results.push(build_pattern(
            "Tweezer Top",
            PatternClassification::Bearish,
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

    body(c2) > 0.5 * range(c2)
        && c2.close > c2.open
        && body(c1) <= 0.05 * range(c1)
        && ((c1.close + c1.open) / 2.0 - c1.low) >= 0.4 * range(c1)
        && ((c1.close + c1.open) / 2.0 - c1.low) <= 0.6 * range(c1)
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

    let Some(max_open_10) = max_open(candles, 10, 0) else {
        return false;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };

    approx_eq(c.open, max_open_10)
        && c.open > c1.high
        && c.open - c.close >= 0.7 * range(c)
        && range(c) >= 1.2 * avg_range_10
        && (c.high - c.open) <= 0.01 * range(c)
        && c.close >= c1.high - 0.5 * (c1.high - c1.low)
        && c1.high > c1.low
        && c.high > c.low
        && c1.close > c2.close
        && c2.close < c3.close
}

fn dark_cloud_cover(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return false;
    };

    (c1.close - c1.open) >= 0.7 * range(c1)
        && range(c1) >= avg_range_10_prev
        && c.open > c1.close
        && c.close < c1.close - 0.5 * (c1.close - c1.open)
        && c.close > c1.open
}

fn doji_gravestone(candles: &[Candle]) -> bool {
    let Some(c) = candle(candles, 0) else {
        return false;
    };

    let Some(max_high_10) = max_high(candles, 10, 0) else {
        return false;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };

    body(c) <= 0.01 * range(c)
        && (c.high - c.close) >= 0.95 * range(c)
        && c.high > c.low
        && approx_eq(c.high, max_high_10)
        && range(c) >= avg_range_10
}

fn engulfing(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };

    c1.close > c1.open
        && c.open - c.close >= 0.7 * range(c)
        && c.close < c1.open
        && c.open > c1.close
        && range(c) >= 1.2 * avg_range_10
}

fn evening_star(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return false;
    };

    let Some(avg_range_10_prev2) = avg_high_low_diff(candles, 10, 2) else {
        return false;
    };
    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };

    (c2.close - c2.open) >= 0.7 * range(c2)
        && range(c2) >= avg_range_10_prev2
        && c1.close > c2.close
        && c1.open > c2.close
        && range(c) >= avg_range_10
        && c.open - c.close >= 0.7 * range(c)
        && c.open < c1.open
        && c.open < c1.close
}

fn evening_doji_star(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return false;
    };

    let Some(avg_range_21_prev) = avg_high_low_diff(candles, 21, 1) else {
        return false;
    };

    body(c2) > 0.5 * range(c)
        && c2.close > c2.open
        && body(c1) < 0.05 * range(c1)
        && range(c1) < 0.2 * avg_range_21_prev
        && c1.open > c2.close
        && c.close < c.open
}

fn hanging_man(candles: &[Candle]) -> bool {
    let Some(c) = candle(candles, 0) else {
        return false;
    };

    let min_body = if c.close >= c.open { c.open } else { c.close };
    let body_size = body(c);
    let midpoint = (c.close + c.open) / 2.0;

    (min_body - c.low) >= 2.0 * body_size
        && (midpoint - c.low) > 2.0 * (c.high - midpoint)
        && body_size > 0.01
}

fn harami(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_10_prev) = avg_high_low_diff(candles, 10, 1) else {
        return false;
    };

    (c1.close - c1.open) >= 0.7 * range(c1)
        && range(c1) >= avg_range_10_prev
        && c.close < c.open
        && c.open < c1.close
        && c.close > c1.open
        && c.open - c.close <= 0.6 * (c1.close - c1.open)
}

fn shooting_star(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_10) = avg_high_low_diff(candles, 10, 0) else {
        return false;
    };
    let Some(max_high_5) = max_high(candles, 5, 0) else {
        return false;
    };

    let body_size = body(c);
    let range_size = range(c);
    let upper_shadow = c.high - c.open.max(c.close);
    let lower_shadow = c.open.min(c.close) - c.low;

    body_size <= 0.2 * range_size
        && body_size >= 0.1 * range_size
        && upper_shadow >= 0.5 * range_size
        && (lower_shadow <= 0.05 * range_size)
        && range_size >= 0.8 * avg_range_10
        && c.open >= (c1.low + 0.5 * (c1.high - c1.low))
        && c.close >= (c1.low + 0.5 * (c1.high - c1.low))
        && approx_eq(c.high, max_high_5)
        && c.high > c.low
}

fn three_black_crows(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1), Some(c2)) =
        (candle(candles, 0), candle(candles, 1), candle(candles, 2))
    else {
        return false;
    };

    c1.open < c2.open
        && c1.open > c2.close
        && c.open < c1.open
        && c.open > c1.close
        && c1.close < c2.low
        && c.close < c1.low
        && c2.close < 1.05 * c2.low
        && c1.close < 1.05 * c1.low
        && c.close < 1.05 * c.low
}

fn tweezer_top(candles: &[Candle]) -> bool {
    let (Some(c), Some(c1)) = (candle(candles, 0), candle(candles, 1)) else {
        return false;
    };

    let Some(avg_range_20) = avg_high_low_diff(candles, 20, 0) else {
        return false;
    };

    approx_eq(c.high, c1.high)
        && body(c) < 0.2 * body(c1)
        && body(c1) >= 0.9 * range(c1)
        && range(c1) >= 1.3 * avg_range_20
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
