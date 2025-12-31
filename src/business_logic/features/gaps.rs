use crate::models::candle::Candle;

use super::types::{Gap, GapDirection};

pub fn detect_gaps(candles: &[Candle], min_gap_pct: f64) -> Vec<Gap> {
    if candles.len() < 2 {
        return Vec::new();
    }

    let mut gaps = Vec::new();

    for window in candles.windows(2) {
        let prev = &window[0];
        let current = &window[1];
        let delta = current.open - prev.close;
        let percent = if prev.close.abs() > f64::EPSILON {
            delta / prev.close
        } else {
            0.0
        };

        if percent.abs() < min_gap_pct {
            continue;
        }

        let direction = if delta >= 0.0 {
            GapDirection::Up
        } else {
            GapDirection::Down
        };

        gaps.push(Gap {
            open_time: current.open_time,
            close_time: current.close_time,
            previous_close: prev.close,
            gap_open: current.open,
            size: delta.abs(),
            percent: percent.abs(),
            direction,
        });
    }

    gaps
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(open: f64, close: f64, time: u64) -> Candle {
        Candle {
            open_time: time,
            close_time: time + 1,
            open,
            high: open.max(close),
            low: open.min(close),
            close,
            volume: 0.0,
            num_trades: 0,
            interval: None,
            symbol: None,
        }
    }

    #[test]
    fn detects_gap_up() {
        let candles = vec![candle(100.0, 100.0, 0), candle(103.0, 104.0, 2)];
        let gaps = detect_gaps(&candles, 0.02);

        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].direction, GapDirection::Up);
        assert!((gaps[0].percent - 0.03).abs() < 0.0001);
    }

    #[test]
    fn ignores_small_gaps() {
        let candles = vec![candle(100.0, 100.0, 0), candle(101.0, 102.0, 2)];
        let gaps = detect_gaps(&candles, 0.02);

        assert!(gaps.is_empty());
    }
}
