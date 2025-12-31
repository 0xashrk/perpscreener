use std::sync::Arc;

use crate::services::candle_store::SharedCandleStore;
use crate::services::core_pattern_state::SharedCorePatternState;
use crate::services::feature_store::SharedFeatureStore;
use crate::services::hyperliquid::HyperliquidClient;
use crate::services::pattern_state::SharedPatternState;

/// Shared application state for handlers and services.
#[derive(Clone)]
pub struct AppState {
    /// In-memory pattern status store and broadcaster.
    pub pattern_state: SharedPatternState,
    /// Core pattern detections for screening endpoints.
    pub core_pattern_state: SharedCorePatternState,
    /// Shared candle cache.
    pub candle_store: SharedCandleStore,
    /// Shared feature snapshots for detectors.
    pub feature_store: SharedFeatureStore,
    /// Shared Hyperliquid API client.
    pub hyperliquid: Arc<HyperliquidClient>,
}
