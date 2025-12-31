use crate::models::candle::Candle;

use super::types::CandleBodyRatio;

pub fn compute_body_ratios(candles: &[Candle]) -> Vec<CandleBodyRatio> {
    candles
        .iter()
        .map(|candle| {
            let body = (candle.close - candle.open).abs();
            let range = (candle.high - candle.low).abs();
            let ratio = if range > 0.0 { body / range } else { 0.0 };

            CandleBodyRatio {
                open_time: candle.open_time,
                close_time: candle.close_time,
                body,
                range,
                ratio,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(open: f64, close: f64, high: f64, low: f64) -> Candle {
        Candle {
            open_time: 0,
            close_time: 1,
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
    fn computes_body_ratio() {
        let candles = vec![candle(100.0, 110.0, 115.0, 95.0)];
        let ratios = compute_body_ratios(&candles);

        assert_eq!(ratios.len(), 1);
        assert!((ratios[0].body - 10.0).abs() < 0.001);
        assert!((ratios[0].range - 20.0).abs() < 0.001);
        assert!((ratios[0].ratio - 0.5).abs() < 0.001);
    }
}
