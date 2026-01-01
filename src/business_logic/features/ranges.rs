use crate::models::candle::Candle;

use super::types::PriceRange;

pub fn compute_ranges(candles: &[Candle], window: usize) -> Vec<PriceRange> {
    if window == 0 || candles.len() < window {
        return Vec::new();
    }

    let mut ranges = Vec::new();

    for slice in candles.windows(window) {
        let (high, low) = slice.iter().fold((f64::MIN, f64::MAX), |acc, candle| {
            (acc.0.max(candle.high), acc.1.min(candle.low))
        });
        let start = slice.first().expect("window start");
        let end = slice.last().expect("window end");

        ranges.push(PriceRange {
            start_time: start.open_time,
            end_time: end.close_time,
            high,
            low,
            midpoint: (high + low) / 2.0,
        });
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(high: f64, low: f64, time: u64) -> Candle {
        Candle {
            open_time: time,
            close_time: time + 1,
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

    #[test]
    fn computes_ranges_per_window() {
        let candles = vec![
            candle(5.0, 1.0, 0),
            candle(6.0, 2.0, 1),
            candle(4.0, 0.5, 2),
        ];
        let ranges = compute_ranges(&candles, 2);

        assert_eq!(ranges.len(), 2);
        assert!((ranges[0].high - 6.0).abs() < 0.001);
        assert!((ranges[0].low - 1.0).abs() < 0.001);
    }
}
