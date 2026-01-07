/// Advanced pattern monitoring loop.
pub mod advanced_pattern_monitor;
/// Advanced pattern detection state store.
pub mod advanced_pattern_state;
/// Candle ingestion and cache updater.
pub mod candle_ingestion;
/// In-memory candle cache.
pub mod candle_store;
/// Candle normalization helpers shared across services.
pub mod candles;
/// Chart snapshot and streaming orchestration.
pub mod chart;
/// Core pattern monitoring loop.
pub mod core_pattern_monitor;
/// Core pattern detection state store.
pub mod core_pattern_state;
/// Shared feature precompute store.
pub mod feature_store;
/// Hyperliquid API client.
pub mod hyperliquid;
/// Background monitoring service for pattern updates.
pub mod monitor;
/// Pattern lifecycle monitoring loop.
pub mod pattern_lifecycle_monitor;
/// Pattern lifecycle snapshot state store.
pub mod pattern_lifecycle_state;
/// Shared in-memory pattern state.
pub mod pattern_state;
/// SQLite-backed token persistence.
pub mod token_store;
/// VWAP snapshot orchestration and validation.
pub mod vwap;
