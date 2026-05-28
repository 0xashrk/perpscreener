# Monitor

Watches an open position for exit signals. Polls momentum every N minutes and alerts when regime/agreement flips or TP/SL is hit.

## Quick Start

```bash
# Monitor a HYPE long
cargo run -p monitor -- \
  --coin HYPE \
  --entry 57.584 \
  --dir long \
  --tp 60.34 \
  --sl 56.206 \
  --interval 3 \
  --max-minutes 60
```

## Options

| Arg | Default | Description |
|-----|---------|-------------|
| `--coin` | required | Asset symbol |
| `--entry` | required | Entry price |
| `--dir` | required | Position direction: `long` or `short` |
| `--tp` | required | Take profit price |
| `--sl` | required | Stop loss price |
| `--interval` | `3` | Poll interval in minutes |
| `--max-minutes` | `60` | Maximum monitoring duration |

## Exit Triggers

1. **TP hit** — price reaches take profit level
2. **SL hit** — price reaches stop loss level
3. **Agreement flip** — momentum signal turns against position (e.g., CONTINUATION → PULLBACK)
4. **Regime flip** — trend regime changes from TRENDING to CHOPPY
5. **Hour reset** — hour boundary crossed (momentum resets)
6. **Timeout** — max monitoring duration reached

## Output

Prints tick-by-tick updates with price, agreement, regime, strength, and P&L. Prints exit signal with reason when triggered.
