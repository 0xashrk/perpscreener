# Backtest

Backtest trading recipes against historical Hyperliquid data.

## Quick Start

```bash
# From repo root
cargo run -p backtest -- --coin BTC --hours 12
```

## Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required | Asset symbol (BTC, ETH) |
| `--hours` | 12 | Lookback period |
| `--scan-interval` | 1m | Candle interval for scanning |
| `--sma-periods` | 20,50 | SMA periods (on 4h) |
| `--donchian-len` | 20 | Donchian length (on 1h) |
| `--atr-period` | 14 | ATR period (on 1h) |
| `--include-scans` | false | Include per-candle data |

## Output

JSON with indicators, orderbook data, and breakout summary.

## Spec

Full specification: `spec/scripts/backtest.md`
