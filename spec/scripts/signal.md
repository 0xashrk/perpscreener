# Signal Script Spec

CLI tool to evaluate trading signals from the HL_ALPHA recipe against live backend data.

## Purpose

Read-only signal scanner. Fetches data from backend endpoints, calculates indicators, and outputs whether entry conditions are met. No trading execution.

## CLI Interface

```bash
cargo run -p signal -- --coin BTC [options]
```

### Arguments

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required | Asset symbol (BTC, ETH) |
| `--backend` | `http://localhost:30001` | Backend base URL |
| `--profile` | `auto` | Profile override: auto, aggressive, balanced, conservative |

## Data Sources

All data fetched from backend endpoints:

| Endpoint | Data | Used For |
|----------|------|----------|
| `/chart?coin={}&interval=4h&limit=60` | 4h candles | SMA20, SMA50 |
| `/chart?coin={}&interval=1h&limit=25` | 1h candles | Donchian, ATR |
| `/orderbook?coin={}&depth=10` | L2 book | Spread, OB imbalance |

## Indicator Calculations

From recipe params:
- `donLen = 20`
- `atrLen = 14`

### Trend (4h candles, closed only)
- `SMA20`: 20-period simple moving average of close
- `SMA50`: 50-period simple moving average of close
- `bull = SMA20 > SMA50`
- `bear = SMA20 < SMA50`

### Breakout (1h candles, closed only)
- `DonHi`: highest high of last 20 candles
- `DonLo`: lowest low of last 20 candles

### Volatility (1h candles, closed only)
- `ATR14`: 14-period average true range
- `atrPct = ATR / mid`

### Orderbook
- `mid = (bestBid + bestAsk) / 2`
- `spread = (bestAsk - bestBid) / mid`
- `OBimb = sum(bidSizes[0..9]) / sum(askSizes[0..9])`

## Profile Selection

If `--profile auto`:
```
if atrPct >= 0.06 OR spread >= 0.0014 => CON
else if trend >= 0.003 AND 0.015 <= atrPct <= 0.05 AND spread <= 0.0011 => AGG
else => BAL
```

Where `trend = abs(SMA20 - SMA50) / mid`.

### Profile Params

| Param | AGG | BAL | CON |
|-------|-----|-----|-----|
| obL | 1.05 | 1.10 | 1.15 |
| obS | 0.95 | 0.90 | 0.87 |
| sprMax | 0.0012 | 0.0008 | 0.0006 |

## Signal Logic

```
strongL = bull AND mid > DonHi AND OBimb >= obL AND spread <= sprMax
strongS = bear AND mid < DonLo AND OBimb <= obS AND spread <= sprMax
```

## Output

JSON to stdout:

```json
{
  "coin": "BTC",
  "timestamp": "2025-01-09T12:00:00Z",
  "profile": "balanced",
  "indicators": {
    "mid": 94500.0,
    "sma20": 94200.0,
    "sma50": 93800.0,
    "trend": "bull",
    "donchian_hi": 94400.0,
    "donchian_lo": 93200.0,
    "atr": 450.0,
    "atr_pct": 0.0048,
    "spread": 0.0003,
    "ob_imbalance": 1.12
  },
  "signals": {
    "strong_long": true,
    "strong_short": false
  },
  "recommendation": "LONG"
}
```

Recommendation values: `LONG`, `SHORT`, `NONE`.

## Error Handling

- Backend unreachable: exit with error message
- Insufficient candles: exit with error (need 55 4h, 40 1h closed)
- Invalid coin: exit with error

## Implementation Notes

- Drop most recent candle from each interval (use closed candles only)
- Reuse indicator logic from backtest script where possible
