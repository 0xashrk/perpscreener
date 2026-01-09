# Signal

Evaluate HL_ALPHA trading signals against live backend data.

## Quick Start

```bash
# From repo root (backend must be running)
cargo run -p signal -- --coin BTC
```

## Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required | Asset symbol (BTC, ETH) |
| `--backend` | `http://localhost:30001` | Backend base URL |
| `--profile` | `auto` | Profile: auto, aggressive, balanced, conservative |

## Output

JSON with indicators, signals, and recommendation (LONG/SHORT/NONE).

## Spec

Full specification: `spec/scripts/signal.md`
