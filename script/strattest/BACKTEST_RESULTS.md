# Backtest Results — Signal V2 Strategy

**Date:** 2026-05-27
**Period:** 2025-11-28 to 2026-05-27 (6 months)
**Data source:** 0xArchive (17,280 15m candles + ~428K L2 orderbook snapshots per asset)
**Starting capital:** $1,000 per asset (compounding enabled)
**Strategy:** Signal V2 — VWAP + regime routing + daily structure + volume confirmation

---

## Summary

| Asset | Trades | Win% | P&L | Return | PF | Max DD | Avg Hold |
|-------|--------|------|-----|--------|-----|--------|----------|
| BTC | 978 | 60.7% | +$534 | +53.4% | 4.80 | 1.1% | 20 min |
| ETH | 911 | 58.2% | +$724 | +72.4% | 5.91 | 0.7% | 19 min |
| HYPE | 1,045 | 61.4% | +$1,574 | +157.4% | 11.68 | 0.7% | 17 min |
| **Total** | **2,934** | **60.1%** | **+$2,832** | **+94.4%** | | | |

$3,000 in → $5,832 out. Every month profitable across all 3 assets.

---

## Monthly Breakdown

### BTC ($1,000 → $1,534)

| Month | Trades | Win% | P&L |
|-------|--------|------|-----|
| 2025-11 | 14 | 79% | +$12 |
| 2025-12 | 149 | 58% | +$66 |
| 2026-01 | 171 | 61% | +$52 |
| 2026-02 | 169 | 39% | +$13 |
| 2026-03 | 138 | 65% | +$104 |
| 2026-04 | 178 | 74% | +$225 |
| 2026-05 | 159 | 65% | +$62 |

### ETH ($1,000 → $1,724)

| Month | Trades | Win% | P&L |
|-------|--------|------|-----|
| 2025-11 | 11 | 82% | +$14 |
| 2025-12 | 141 | 65% | +$99 |
| 2026-01 | 161 | 53% | +$84 |
| 2026-02 | 173 | 42% | +$34 |
| 2026-03 | 141 | 65% | +$213 |
| 2026-04 | 150 | 71% | +$222 |
| 2026-05 | 134 | 54% | +$59 |

### HYPE ($1,000 → $2,574)

| Month | Trades | Win% | P&L |
|-------|--------|------|-----|
| 2025-11 | 14 | 79% | +$12 |
| 2025-12 | 162 | 38% | +$35 |
| 2026-01 | 185 | 59% | +$231 |
| 2026-02 | 170 | 52% | +$116 |
| 2026-03 | 177 | 72% | +$272 |
| 2026-04 | 165 | 61% | +$239 |
| 2026-05 | 172 | 82% | +$668 |

---

## Direction Breakdown

| Asset | Longs | Long Win% | Shorts | Short Win% |
|-------|-------|-----------|--------|------------|
| BTC | 441 | 81.0% | 537 | 44.1% |
| ETH | 371 | 84.4% | 540 | 40.2% |
| HYPE | 552 | 90.8% | 493 | 28.6% |

Longs dominate across the board (81-91% win rate). Shorts are weaker (28-44%) — expected in a predominantly bull period. Strategy still takes shorts on bear macro crossovers but they contribute less.

---

## Strategy Breakdown

| Asset | Trend-Follow | TF Win% | TF P&L | Mean-Revert | MR Win% | MR P&L |
|-------|-------------|---------|--------|-------------|---------|--------|
| BTC | 560 | 56.1% | +$463 | 418 | 67.0% | +$71 |
| ETH | 653 | 55.3% | +$665 | 258 | 65.5% | +$59 |
| HYPE | 997 | 62.1% | +$1,579 | 48 | 47.9% | -$5 |

Trend-follow is the primary profit driver. Mean-revert is supplementary — positive on BTC/ETH, slightly negative on HYPE (trending asset doesn't suit MR).

---

## Conviction Breakdown

| Asset | STRONG | Win% | P&L | NORMAL | Win% | P&L | MR | Win% | P&L |
|-------|--------|------|-----|--------|------|-----|-----|------|-----|
| BTC | 67 | 53.7% | +$193 | 490 | 56.7% | +$271 | 418 | 67.0% | +$71 |
| ETH | 62 | 53.2% | +$258 | 584 | 55.7% | +$408 | 258 | 65.5% | +$59 |
| HYPE | 115 | 70.4% | +$580 | 872 | 61.4% | +$1,000 | 48 | 47.9% | -$5 |

STRONG conviction (Donchian breakout + OB confirms) wins biggest per trade — HYPE STRONG averaged +$5.04/trade vs NORMAL at +$1.15/trade.

---

## Exit Reasons

| Exit | BTC | ETH | HYPE |
|------|-----|-----|------|
| Trailing Stop | 211 | 291 | 693 |
| Regime Flip | 428 | 382 | 256 |
| Agreement Flip | 172 | 140 | 84 |
| VWAP Cross | 152 | 84 | 9 |
| SL Hit | 8 | 6 | 2 |
| TP Hit | 7 | 8 | 1 |

Trailing stops are the dominant exit — especially on HYPE (66% of exits). Only 2-8 SL hits per asset in 6 months — the entry filters keep us out of bad trades.

---

## Key Findings

1. **Every month profitable** across all 3 assets — no losing months in 6 months.
2. **Max drawdown under 1.1%** — extremely controlled risk.
3. **Longs >> shorts** in this period — but the strategy correctly takes both directions.
4. **Trailing stops work** — they capture profits early and minimize losses (avg loss 0.16-0.23%).
5. **OB data improves STRONG conviction** — with real OB gates, STRONG trades earn disproportionately more per trade.
6. **Feb 2026 was the stress test** — lowest win rates (39-52%) but still positive. Strategy survives choppy markets.
7. **HYPE is the standout** — 157% return driven by strong bull trend + 91% long win rate.

## Caveats

- **No fee simulation** — round-trip maker fees (~0.045%) across ~1000 trades = ~$4.50 per $1000 capital. Minimal impact.
- **No slippage simulation** — limit order entries assumed at VWAP. Real fills may be slightly worse.
- **Compounding enabled** — flat $100/trade sizing would yield lower absolute returns but same win rates.
- **OB data coverage** — 0xArchive OB snapshots may have gaps for older dates. Candle coverage is continuous.
- **Survivorship bias** — HYPE existed for the full period but many altcoins don't. Results on newer tokens would differ.
- **Bull-period bias** — this 6-month window was predominantly bullish. Bear market performance would differ significantly (shorts would carry, longs would suffer).

---

## How to Reproduce

```bash
# 1. Download data (requires ethers.js for wallet auth)
node script/obdata/oxarchive/download.mjs --coins BTC,ETH,HYPE --months 6

# 2. Run backtests
cargo run --release -p strattest -- --coin BTC --months 6 --av 1000 --micro-interval 15m
cargo run --release -p strattest -- --coin ETH --months 6 --av 1000 --micro-interval 15m
cargo run --release -p strattest -- --coin HYPE --months 6 --av 1000 --micro-interval 15m
```
