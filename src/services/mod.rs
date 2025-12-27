/// Candle normalization helpers shared across services.
pub mod candles;
/// Chart snapshot and streaming orchestration.
pub mod chart;
/// Hyperliquid API client.
pub mod hyperliquid;
/// Background monitoring service for pattern updates.
pub mod monitor;
/// Shared in-memory pattern state.
pub mod pattern_state;
/// VWAP snapshot orchestration and validation.
pub mod vwap;
