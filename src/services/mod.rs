/// Candle normalization helpers shared across services.
pub mod candles;
/// Candle ingestion and cache updater.
pub mod candle_ingestion;
/// In-memory candle cache.
pub mod candle_store;
/// Core pattern detection state store.
pub mod core_pattern_state;
/// Advanced pattern detection state store.
pub mod advanced_pattern_state;
/// Advanced pattern monitoring loop.
pub mod advanced_pattern_monitor;
/// Core pattern monitoring loop.
pub mod core_pattern_monitor;
/// Chart snapshot and streaming orchestration.
pub mod chart;
/// Shared feature precompute store.
pub mod feature_store;
/// Hyperliquid API client.
pub mod hyperliquid;
/// Background monitoring service for pattern updates.
pub mod monitor;
/// Shared in-memory pattern state.
pub mod pattern_state;
/// VWAP snapshot orchestration and validation.
pub mod vwap;
