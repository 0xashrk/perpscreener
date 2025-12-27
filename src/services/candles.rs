use crate::models::candle::Candle;

/// Fill missing symbol/interval fields in candle payloads.
pub fn normalize_candles(candles: &mut [Candle], coin: &str, interval: &str) {
    for candle in candles {
        if candle.interval.is_none() {
            candle.interval = Some(interval.to_string());
        }
        if candle.symbol.is_none() {
            candle.symbol = Some(coin.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_candles_sets_missing_fields() {
        let mut candles = vec![Candle {
            open_time: 1,
            close_time: 2,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10.0,
            num_trades: 5,
            interval: None,
            symbol: None,
        }];

        normalize_candles(&mut candles, "BTC", "1m");

        assert_eq!(candles[0].interval.as_deref(), Some("1m"));
        assert_eq!(candles[0].symbol.as_deref(), Some("BTC"));
    }
}
