# OB Data Pipeline

Downloads historical L2 orderbook data and 15m candles for backtesting. Two data sources: 0xArchive (recommended, full history) and Tardis.dev (free samples only).

## 0xArchive (Recommended)

Full 6-month history of 15m candles AND L2 orderbook snapshots. Free 14-day trial, no credit card — uses a burner EVM wallet for signup.

```bash
# Download 6 months of BTC, ETH, HYPE data
node script/obdata/oxarchive/download.mjs --coins BTC,ETH,HYPE --months 6

# Custom output directory
node script/obdata/oxarchive/download.mjs --coins BTC --months 3 --out-dir data
```

### Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coins` | `BTC,ETH,HYPE` | Comma-separated asset symbols |
| `--months` | `6` | Lookback in months |
| `--out-dir` | `data` | Output directory |

### Output

- `data/candles/{coin}_15m.csv` — 15m candles (timestamp_ms, o, h, l, c, v)
- `data/ob/{coin}_ob.csv` — OB snapshots (timestamp_ms, ob_imbalance, spread_pct, best_bid, best_ask, bid_depth, ask_depth)

### How It Works

1. Generates a burner EVM wallet (no funds needed)
2. Signs a SIWE challenge to get a free 0xArchive build-trial (14 days)
3. Downloads 15m candles and hourly L2 book snapshots via REST API
4. Computes OB imbalance + spread from raw book data
5. Writes CSV files that `strattest` automatically picks up

### Data Coverage

- **15m candles**: Nov 2025 onwards for BTC, ETH, HYPE
- **L2 orderbook**: April 2026 onwards (100 snapshots/hour, 20 levels)
- **Auth**: Token expires after 24h. Re-run the script to get a fresh token.

## Tardis.dev (Legacy)

Free tier: L2 data for the 1st of each month only. Use for sparse samples or cross-validation.

```bash
cargo run --release -p obdata -- --coin BTC --start 2025-01-01 --end 2025-05-01
```

### Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required | Asset symbol |
| `--start` | required | Start date YYYY-MM-DD |
| `--end` | required | End date YYYY-MM-DD |
| `--interval` | `15` | Snapshot interval in minutes |
| `--out-dir` | `data/ob` | Output directory |

## Backtester Integration

The `strattest` backtester automatically loads:
- `data/candles/{coin}_15m.csv` — used instead of fetching from HL when `--micro-interval 15m`
- `data/ob/{coin}_ob.csv` — used for OB imbalance and spread gates

Just download the data, then run:
```bash
cargo run --release -p strattest -- --coin HYPE --months 6 --av 1000 --micro-interval 15m
```
