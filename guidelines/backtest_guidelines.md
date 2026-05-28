# Backtesting Guidelines

## Strategy Under Test

Signal V2 — VWAP + regime routing + daily structure + volume confirmation. Full spec at `spec/scripts/signal_v2.md`.

## Data Pipeline

### Step 1: Download Historical Data

**0xArchive (recommended — full 6-month coverage):**

```bash
node script/obdata/oxarchive/download.mjs --coins BTC,ETH,HYPE --months 6
```

- Generates a burner EVM wallet, signs SIWE challenge for free 14-day trial
- Downloads 15m candles + hourly L2 orderbook snapshots
- Outputs: `data/candles/{coin}_15m.csv` and `data/ob/{coin}_ob.csv`
- 15m candles available from Nov 2025+; OB from Apr 2026+
- Token expires after 24h; re-run for fresh token

**Tardis.dev (legacy — 1st-of-month samples only):**

```bash
cargo run --release -p obdata -- --coin BTC --start 2025-01-01 --end 2025-05-01
```

### Step 2: Run Backtest

```bash
# Single asset, $1000 starting capital, compounding
cargo run --release -p strattest -- --coin HYPE --months 6 --av 1000 --micro-interval 15m

# Export daily/weekly/monthly PnL CSVs
cargo run --release -p strattest -- --coin BTC --months 6 --av 1000 --micro-interval 15m --csv-dir data/pnl
```

The backtester automatically loads:
- `data/candles/{coin}_15m.csv` — uses local CSV instead of fetching from HL when `--micro-interval 15m`
- `data/ob/{coin}_ob.csv` — applies real OB imbalance + spread gates
- 4h and 1h candles are still fetched live from HL (they have 6+ months retention)

### Step 3: Interpret Results

Results at `script/strattest/BACKTEST_RESULTS.md`. PnL exports at `data/pnl/`.

## Fee Model

All fees are simulated in `script/strattest/src/engine.rs`:

| Fee | Rate | When Applied |
|-----|------|-------------|
| **Maker** | 0.035% | Entry (always limit order) + TP exit (limit) |
| **Taker** | 0.10% | SL, trailing stop, signal exits (market order) |
| **Funding** | 0.01%/hr | Charged when position crosses hourly settlement boundary. Longs pay, shorts receive. Conservative fixed estimate; actual rate varies per asset. |

### Fee Constants

```rust
const MAKER_FEE: f64 = 0.00035;   // 0.035%
const TAKER_FEE: f64 = 0.001;     // 0.10%
const FUNDING_RATE: f64 = 0.0001;  // 0.01% per hour
```

### Fee Impact (6-month backtest, $1000/asset)

| Component | BTC | ETH | HYPE |
|-----------|-----|-----|------|
| Gross P&L | +$534 | +$724 | +$1,574 |
| Trading fees | -$352 | -$322 | -$377 |
| Funding | -$3 | ~$0 | -$9 |
| **Net P&L** | **+$179** | **+$402** | **+$1,188** |

Trading fees account for ~95% of fee drag. Funding is negligible at 17 min avg hold.

## CLI Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required | Asset symbol |
| `--months` | `6` | Lookback period |
| `--av` | required | Starting account value (USD) |
| `--micro-interval` | `5m` | Candle interval: `1m`, `5m`, `15m`. Use `15m` for 6-month backtests. |
| `--check-interval` | `15` | Signal check interval in minutes |
| `--cooldown` | `60` | Minutes after exit before re-entry |
| `--csv-dir` | none | Export daily/weekly/monthly PnL CSVs to this directory |

## Data Retention (HL direct)

If local CSV data is not available, the backtester fetches from Hyperliquid directly. Retention limits:

| Interval | HL Retention |
|----------|-------------|
| 1m | ~3.5 days |
| 5m | ~17 days |
| 15m | ~2 months |
| 1h, 4h | 6+ months |

Always prefer local 0xArchive CSVs for backtests longer than 2 weeks.

## Caveats

- **Compounding is on by default.** Trades are sized based on current equity, not initial AV.
- **No slippage simulation.** Limit entries assumed at VWAP. Real fills may differ.
- **Funding rate is fixed.** Uses 0.01%/hr conservative estimate. Actual rates vary by asset and market conditions.
- **OB data gaps.** 0xArchive OB history starts ~April 2026. Earlier dates have no OB data — backtester defaults to no OB gates.
- **Survivorship bias.** Only assets that existed for the full period are testable.
- **Bull-period bias.** Nov 2025 – May 2026 was predominantly bullish. Bear market results would differ.

## Adding Fee Model Changes

If HL changes fee tiers (e.g., volume-based discounts), update the constants in `script/strattest/src/engine.rs`:

```rust
const MAKER_FEE: f64 = 0.00035;   // update here
const TAKER_FEE: f64 = 0.001;     // update here
const FUNDING_RATE: f64 = 0.0001;  // update here
```

The `compute_pnl` function applies:
- `MAKER_FEE` on entry notional (always)
- `MAKER_FEE` on exit notional for TP exits (limit order)
- `TAKER_FEE` on exit notional for all other exits (market order)
- `funding_paid` accumulated during position hold via `charge_funding()`
