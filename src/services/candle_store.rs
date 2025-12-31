use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::models::candle::Candle;
use crate::models::interval::CandleInterval;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CandleKey {
    pub coin: String,
    pub interval: CandleInterval,
}

impl CandleKey {
    pub fn new(coin: impl Into<String>, interval: CandleInterval) -> Self {
        Self {
            coin: coin.into(),
            interval,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CandleUpdateSummary {
    pub added: usize,
    pub trimmed: usize,
    pub total: usize,
    pub newest_close_time: Option<u64>,
    pub oldest_close_time: Option<u64>,
}

#[derive(Debug)]
pub struct CandleStoreInner {
    max_candles: usize,
    candles: RwLock<HashMap<CandleKey, Vec<Candle>>>,
}

pub type SharedCandleStore = Arc<CandleStoreInner>;

impl CandleStoreInner {
    pub fn new(max_candles: usize) -> Self {
        Self {
            max_candles,
            candles: RwLock::new(HashMap::new()),
        }
    }

    pub async fn upsert(&self, key: CandleKey, new_candles: Vec<Candle>) -> CandleUpdateSummary {
        let mut guard = self.candles.write().await;
        let entry = guard.entry(key).or_insert_with(Vec::new);
        let before = entry.len();
        *entry = merge_candles(entry, new_candles, self.max_candles);
        let after = entry.len();

        CandleUpdateSummary {
            added: after.saturating_sub(before),
            trimmed: before.saturating_add(new_candles.len()).saturating_sub(after),
            total: after,
            newest_close_time: entry.last().map(|c| c.close_time),
            oldest_close_time: entry.first().map(|c| c.close_time),
        }
    }

    pub async fn get(&self, key: &CandleKey) -> Option<Vec<Candle>> {
        let guard = self.candles.read().await;
        guard.get(key).cloned()
    }
}

fn merge_candles(existing: &[Candle], new_candles: Vec<Candle>, max_candles: usize) -> Vec<Candle> {
    let mut merged = BTreeMap::new();

    for candle in existing {
        merged.insert(candle.close_time, candle.clone());
    }

    for candle in new_candles {
        merged.insert(candle.close_time, candle);
    }

    let mut values: Vec<Candle> = merged.into_values().collect();
    if values.len() > max_candles {
        let trimmed = values.len() - max_candles;
        values.drain(0..trimmed);
    }

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(close_time: u64, close: f64) -> Candle {
        Candle {
            open_time: close_time.saturating_sub(1),
            close_time,
            open: close,
            high: close,
            low: close,
            close,
            volume: 0.0,
            num_trades: 0,
            interval: None,
            symbol: None,
        }
    }

    #[tokio::test]
    async fn upsert_merges_and_trims() {
        let store = CandleStoreInner::new(3);
        let key = CandleKey::new("BTC", CandleInterval::OneMinute);

        store
            .upsert(
                key.clone(),
                vec![candle(1, 1.0), candle(2, 2.0), candle(3, 3.0)],
            )
            .await;

        store
            .upsert(key.clone(), vec![candle(3, 3.3), candle(4, 4.0)])
            .await;

        let stored = store.get(&key).await.expect("candles");
        let times: Vec<u64> = stored.iter().map(|c| c.close_time).collect();

        assert_eq!(times, vec![2, 3, 4]);
        assert!((stored[1].close - 3.3).abs() < 0.001);
    }
}
