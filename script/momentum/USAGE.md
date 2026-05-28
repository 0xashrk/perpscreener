# Momentum

Intrahour momentum scanner. Computes BTC (or any asset) momentum context from 1m candles within the current hour.

## Quick Start

```bash
# Single asset via Hyperliquid
cargo run -p momentum -- --coin BTC --use-hl

# Top 50 assets by volume
cargo run -p momentum -- --top 50

# Top 10
cargo run -p momentum -- --top 10

# Via backend instead of direct HL
cargo run -p momentum -- --coin BTC --backend http://localhost:30001
```

## Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required (unless --top) | Asset symbol |
| `--top N` | - | Scan top N assets by 24h volume |
| `--use-hl` | `false` | Fetch directly from Hyperliquid (implied by --top) |
| `--backend` | `http://localhost:30001` | Backend base URL |
| `--limit` | `180` | Number of 1m candles to pull |

## Output

Single coin: detailed table with direction vs open, micro/meso trends, streaks, volatility, support/resistance, agreement signal.

Multi-coin: compact summary table sorted by volume ranking.

## Spec

Full specification: `spec/scripts/momentum.md`
