use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::business_logic::features::{compute_features, FeatureConfig, FeatureSnapshot};
use crate::models::candle::Candle;
use crate::services::candle_store::CandleKey;

#[derive(Debug)]
pub struct FeatureStoreInner {
    config: FeatureConfig,
    features: RwLock<HashMap<CandleKey, FeatureSnapshot>>,
}

pub type SharedFeatureStore = Arc<FeatureStoreInner>;

impl FeatureStoreInner {
    pub fn new(config: FeatureConfig) -> Self {
        Self {
            config,
            features: RwLock::new(HashMap::new()),
        }
    }

    pub async fn recompute(&self, key: CandleKey, candles: &[Candle]) -> FeatureSnapshot {
        let snapshot = compute_features(candles, &self.config);
        let mut guard = self.features.write().await;
        guard.insert(key, snapshot.clone());
        snapshot
    }

    pub async fn get(&self, key: &CandleKey) -> Option<FeatureSnapshot> {
        let guard = self.features.read().await;
        guard.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::interval::CandleInterval;

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
    async fn recompute_stores_snapshot() {
        let store = FeatureStoreInner::new(FeatureConfig::default());
        let key = CandleKey::new("BTC", CandleInterval::OneMinute);
        let candles = vec![candle(1, 1.0), candle(2, 2.0)];

        store.recompute(key.clone(), &candles).await;
        let stored = store.get(&key).await.expect("feature snapshot");

        assert_eq!(stored.as_of_ms, 2);
    }
}
