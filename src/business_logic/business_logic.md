# business_logic

Pure, synchronous business rules and calculations. No I/O or async work.

Files:
- `config.rs` - `DoubleTopConfig` tuning parameters and defaults.
- `double_top.rs` - double top detector state machine and alerts.
- `indicators.rs` - ATR calculator and swing high/low detection.
- `vwap.rs` - VWAP computation helper for candle windows.
- `mod.rs` - module exports.
