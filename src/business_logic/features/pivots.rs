use crate::models::candle::Candle;

use super::types::{Pivot, PivotKind};

pub fn detect_pivots(candles: &[Candle], left: usize, right: usize) -> Vec<Pivot> {
    let len = candles.len();
    if len == 0 || left + right + 1 > len {
        return Vec::new();
    }

    let mut pivots = Vec::new();
    let end = len.saturating_sub(right);

    for idx in left..end {
        if idx + right >= len {
            break;
        }

        if is_pivot_high(candles, idx, left, right) {
            pivots.push(Pivot {
                index: idx,
                time: candles[idx].close_time,
                price: candles[idx].high,
                kind: PivotKind::High,
            });
        }

        if is_pivot_low(candles, idx, left, right) {
            pivots.push(Pivot {
                index: idx,
                time: candles[idx].close_time,
                price: candles[idx].low,
                kind: PivotKind::Low,
            });
        }
    }

    pivots
}

fn is_pivot_high(candles: &[Candle], index: usize, left: usize, right: usize) -> bool {
    let high = candles[index].high;
    let left_bound = index.saturating_sub(left);
    let right_bound = (index + right).min(candles.len().saturating_sub(1));

    candles[left_bound..=right_bound]
        .iter()
        .enumerate()
        .all(|(offset, candle)| {
            let idx = left_bound + offset;
            idx == index || candle.high < high
        })
}

fn is_pivot_low(candles: &[Candle], index: usize, left: usize, right: usize) -> bool {
    let low = candles[index].low;
    let left_bound = index.saturating_sub(left);
    let right_bound = (index + right).min(candles.len().saturating_sub(1));

    candles[left_bound..=right_bound]
        .iter()
        .enumerate()
        .all(|(offset, candle)| {
            let idx = left_bound + offset;
            idx == index || candle.low > low
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(high: f64, low: f64, time: u64) -> Candle {
        Candle {
            open_time: time,
            close_time: time,
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
    fn finds_pivot_high_and_low() {
        let candles = vec![
            candle(1.0, 0.8, 0),
            candle(3.0, 2.0, 1),
            candle(5.0, 4.0, 2),
            candle(4.0, 3.5, 3),
            candle(2.0, 1.0, 4),
            candle(4.0, 2.0, 5),
            candle(1.5, 0.5, 6),
        ];

        let pivots = detect_pivots(&candles, 1, 1);

        assert!(pivots
            .iter()
            .any(|p| p.kind == PivotKind::High && p.index == 2));
        assert!(pivots
            .iter()
            .any(|p| p.kind == PivotKind::Low && p.index == 4));
    }
}
