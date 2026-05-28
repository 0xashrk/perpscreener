# Signal V2

Trade signal scanner with VWAP, regime routing, daily structure levels, and volume confirmation. Fetches directly from Hyperliquid — no backend needed.

## Quick Start

```bash
# Single coin (defaults to $100 AV)
cargo run -p signal -- --coin HYPE

# With custom account value
cargo run -p signal -- --coin BTC --av 1000

# Scan top 10 assets by volume
cargo run -p signal -- --top 10

# Top 50 with custom AV
cargo run -p signal -- --top 50 --av 5000
```

## Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required (unless --top) | Asset symbol (BTC, ETH, HYPE, etc.) |
| `--top N` | - | Scan top N assets by 24h volume |
| `--av` | `100` | Account value in USD |
| `--vwap-slope-threshold` | `0.0003` | Min VWAP slope for TRENDING regime |
| `--bb-tight-threshold` | `0.015` | Max BB width for RANGING regime |

## What It Does

1. **Macro** (4h): SMA20/50 trend direction, Donchian levels, ATR volatility
2. **VWAP** (15m): Daily VWAP, slope, bands — intraday anchor
3. **Micro** (15m): Momentum, trend strength, VWAP-based agreement signal
4. **Daily structure** (1d): 20-day high/low resistance/support zones
5. **Volume** (15m): Volume ratio vs 20-period average — declining/confirming
6. **Regime router**: TRENDING → trend-follow, RANGING → mean-revert (BB+RSI), CHOPPY → sit out
7. **Decision**: Signal + conviction + limit price + SL + leverage recommendation

## Output

Single coin shows: signal, conviction, limit entry price, size, leverage, SL, trailing stop params, and full context (macro, VWAP, micro, OB, daily structure, volume).

Multi-coin shows: compact table sorted by volume ranking.

## Key Filters

- **Near 20-day high + declining volume** → blocks longs (resistance zone)
- **Near 20-day low + declining volume** → blocks shorts (support zone)
- **Near resistance + NORMAL conviction** → blocked unless volume confirms
- **Wide spread (>0.1%)** → blocks all entries
- **Choppy regime** → no trades

## Spec

Full specification: `spec/scripts/signal_v2.md`
