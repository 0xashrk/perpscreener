# Backtest Script Spec

## Overview

A standalone Rust CLI tool for backtesting trading recipes against historical candle data. Designed for Claude to invoke when asked to backtest a recipe like `HL_ALPHA_RISKON_V5`.

## Location

```
script/
  backtest/
    Cargo.toml
    src/
      main.rs
```

## Goals

- Fetch candles at multiple intervals (1m, 1h, 4h) from Hyperliquid API
- Calculate indicators: SMA, Donchian channel, ATR
- Fetch current orderbook for spread/OB imbalance
- Output structured JSON for Claude to evaluate recipe signal logic
- Highly configurable via CLI args

## Non-Goals

- Execute trades
- Evaluate recipe-specific signal logic (Claude does this)
- Store historical data long-term

---

## CLI Interface

```bash
# Basic usage
cargo run --manifest-path script/backtest/Cargo.toml -- \
  --coin BTC \
  --hours 12

# Full options
cargo run --manifest-path script/backtest/Cargo.toml -- \
  --coin BTC \
  --hours 12 \
  --scan-interval 1m \
  --sma-periods 20,50 \
  --donchian-len 20 \
  --atr-period 14 \
  --include-scans
```

### Arguments

| Arg | Type | Default | Description |
|-----|------|---------|-------------|
| `--coin` | string | required | Asset symbol (BTC, ETH) |
| `--hours` | u32 | 12 | Lookback period |
| `--scan-interval` | string | "1m" | Candle interval for scanning (1m, 5m, 15m) |
| `--sma-periods` | string | "20,50" | Comma-separated SMA periods (calculated on 4h) |
| `--donchian-len` | u8 | 20 | Donchian channel length (calculated on 1h) |
| `--atr-period` | u8 | 14 | ATR period (calculated on 1h) |
| `--include-scans` | flag | false | Include per-candle scan data in output |

---

## Output Schema

```json
{
  "coin": "BTC",
  "generated_at": "2026-01-08T12:00:00Z",
  "params": {
    "hours": 12,
    "scan_interval": "1m",
    "sma_periods": [20, 50],
    "donchian_len": 20,
    "atr_period": 14
  },
  "data": {
    "candles_1m": 720,
    "candles_1h": 100,
    "candles_4h": 100
  },
  "orderbook": {
    "time": 1767910954677,
    "bid": 91335.0,
    "ask": 91336.0,
    "mid": 91335.5,
    "spread": 0.000011,
    "spread_pct": 0.0011,
    "ob_imbalance": 0.1041
  },
  "indicators": {
    "sma20_4h": 92382.5,
    "sma50_4h": 90844.4,
    "don_hi_1h": 91490.0,
    "don_lo_1h": 89300.0,
    "atr14_1h": 566.71
  },
  "derived": {
    "bull": true,
    "trend_strength": 0.0169,
    "atr_pct": 0.0062,
    "current_vs_don_hi": -0.0017,
    "current_vs_don_lo": 0.0228
  },
  "price_range": {
    "low": 89300.0,
    "high": 91490.0,
    "current": 91338.0
  },
  "summary": {
    "long_breakouts": 0,
    "short_breakouts": 12,
    "first_long_breakout_ts": null,
    "first_short_breakout_ts": 1767869160000
  },
  "scans": [
    {
      "ts": 1767867720000,
      "o": 90057.0,
      "h": 90090.0,
      "l": 90056.0,
      "c": 90089.0,
      "don_hi": 91200.0,
      "don_lo": 89400.0,
      "breakout_long": false,
      "breakout_short": false
    }
  ]
}
```

### Field Descriptions

**orderbook**: Current orderbook snapshot
- `ob_imbalance`: sum(bid_sz[0..9]) / sum(ask_sz[0..9])
- `spread_pct`: (ask - bid) / mid * 100

**indicators**: Calculated from closed candles only
- `sma20_4h`, `sma50_4h`: Simple moving averages on 4h closes
- `don_hi_1h`, `don_lo_1h`: Donchian high/low on last N 1h candles
- `atr14_1h`: Average True Range on 1h candles

**derived**: Values Claude needs for recipe evaluation
- `bull`: sma20 > sma50
- `trend_strength`: abs(sma20 - sma50) / mid
- `atr_pct`: atr / mid
- `current_vs_don_hi`: (current - don_hi) / don_hi (negative = below)
- `current_vs_don_lo`: (current - don_lo) / don_lo (positive = above)

**summary**: Aggregated breakout stats
- `long_breakouts`: Count of candles where mid > don_hi
- `short_breakouts`: Count of candles where mid < don_lo
- Timestamps of first breakouts (null if none)

**scans**: Per-candle data (only if `--include-scans`)
- Donchian values at that point in time (from 1h candles closed before)
- Breakout flags

---

## Implementation Details

### Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
```

### API Calls

1. **Candles** - POST to `https://api.hyperliquid.xyz/info`
   ```json
   {"type": "candleSnapshot", "req": {"coin": "BTC", "interval": "1m", "startTime": ..., "endTime": ...}}
   ```

2. **Orderbook** - POST to `https://api.hyperliquid.xyz/info`
   ```json
   {"type": "l2Book", "coin": "BTC"}
   ```

### Calculation Rules (from HL_ALPHA recipe)

1. **Closed candles only**: Drop most recent candle from each interval
2. **SMA**: Simple average of close prices over period
3. **Donchian**:
   - `don_hi` = max(high) over last N closed 1h candles
   - `don_lo` = min(low) over last N closed 1h candles
4. **ATR**: Wilder's smoothed average of True Range
   - TR = max(H-L, |H-prev_close|, |L-prev_close|)
5. **OB Imbalance**: sum(bid sizes 0-9) / sum(ask sizes 0-9)

### Scan Logic

For each scan-interval candle:
1. Find 1h candles closed before this candle's timestamp
2. Calculate Donchian from those 1h candles
3. `breakout_long` = candle_mid > don_hi_at_time
4. `breakout_short` = candle_mid < don_lo_at_time

---

## Usage Example

```bash
# Claude runs this when asked to backtest HL_ALPHA on BTC for 12 hours
cargo run --manifest-path script/backtest/Cargo.toml -- --coin BTC --hours 12

# Output is JSON that Claude parses to evaluate:
# - strongL = bull && mid > DonHi && OBimb >= obL && spread <= sprMax
# - strongS = bear && mid < DonLo && OBimb <= obS && spread <= sprMax
```

---

## Error Handling

- API failures: Return error JSON with message
- Insufficient candles: Return partial data with warning
- Invalid args: Exit with usage message

```json
{
  "error": "Failed to fetch 4h candles: rate limited",
  "partial_data": { ... }
}
```
