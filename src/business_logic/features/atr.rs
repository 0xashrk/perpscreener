use crate::business_logic::indicators::AtrCalculator;
use crate::models::candle::Candle;

use super::types::{AtrPoint, AtrSnapshot};

pub fn compute_atr(candles: &[Candle], period: usize) -> Option<AtrSnapshot> {
    if period == 0 || candles.len() < period {
        return None;
    }

    let mut calculator = AtrCalculator::new(period);
    let mut values = Vec::new();

    for candle in candles {
        if let Some(value) = calculator.update(candle) {
            values.push(AtrPoint {
                close_time: candle.close_time,
                value,
            });
        }
    }

    if values.is_empty() {
        None
    } else {
        Some(AtrSnapshot { period, values })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(high: f64, low: f64, close: f64, time: u64) -> Candle {
        Candle {
            open_time: time,
            close_time: time,
            open: close,
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
    fn returns_atr_snapshot() {
        let candles = vec![
            candle(10.0, 8.0, 9.0, 0),
            candle(11.0, 8.5, 10.0, 1),
            candle(12.0, 9.0, 11.0, 2),
            candle(12.5, 9.5, 12.0, 3),
        ];

        let snapshot = compute_atr(&candles, 3).expect("atr snapshot");
        assert_eq!(snapshot.period, 3);
        assert!(!snapshot.values.is_empty());
    }
}
