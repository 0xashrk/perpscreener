# services

Orchestration layer that calls external APIs and business logic.

Files:
- `chart.rs` - chart snapshot service backed by Hyperliquid candles.
- `hyperliquid.rs` - HTTP client for the Hyperliquid info API.
- `monitor.rs` - double top monitoring loop and state updates.
- `pattern_state.rs` - shared pattern state for SSE updates.
- `vwap.rs` - VWAP service: anchoring, coverage checks, snapshot assembly.
- `mod.rs` - module exports.
