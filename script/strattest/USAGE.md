# Strattest

Backtester for the V2 signal strategy. Simulates trades over historical data with VWAP, regime routing, trailing stops, and optional OB data.

## Quick Start

```bash
# Backtest HYPE over available history, $100 AV, no compounding
cargo run --release -p strattest -- --coin HYPE --months 6 --av 100

# With 15m candles (recommended — 2 months HL history available)
cargo run --release -p strattest -- --coin HYPE --months 6 --av 100 --micro-interval 15m

# Compounding enabled (default — sizes based on current equity)
cargo run --release -p strattest -- --coin BTC --months 6 --av 1000 --micro-interval 15m
```

## Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required | Asset symbol |
| `--months` | `6` | Lookback period in months |
| `--av` | required | Starting account value in USD |
| `--micro-interval` | `5m` | Micro candle interval: `1m`, `5m`, or `15m` |
| `--check-interval` | `15` | Signal check interval in minutes |
| `--cooldown` | `60` | Minutes after exit before re-entry |

## OB Data

If `data/ob/{coin}_ob.csv` exists, the backtester uses real orderbook data for OB imbalance and spread gates. Generate it with the `obdata` script.

## Data Limitations

Hyperliquid candle data retention:
- 1m: ~3.5 days
- 5m: ~17 days
- 15m: ~2 months
- 1h/4h: 6+ months

Use `--micro-interval 15m` for backtests longer than 2 weeks.

## Output

Performance report with: total trades, win rate, P&L, profit factor, max drawdown, avg hold time. Breakdowns by direction (long/short), strategy (trend-follow/mean-revert), conviction tier, exit reason, and month.

## Note on Compounding

The backtester sizes trades based on current equity (compounding). To simulate flat sizing, modify `engine.rs` to use `initial_av` instead of `equity` in the `decide` call.
