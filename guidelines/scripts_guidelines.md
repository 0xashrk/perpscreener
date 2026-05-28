# Scripts

Standalone Rust CLI tools in `script/`. Each subfolder is a workspace member.

## Available Scripts

| Script | Path | Description |
|--------|------|-------------|
| signal | `script/signal/` | V2 trade signal scanner — VWAP, regime routing, daily structure, volume confirmation |
| momentum | `script/momentum/` | Intrahour momentum scanner — top N assets by volume |
| monitor | `script/monitor/` | Position monitor — watches for exit signals (TP/SL, regime flip, VWAP cross) |
| strattest | `script/strattest/` | Strategy backtester — simulates V2 signals over historical data with trailing stops |
| obdata | `script/obdata/` | OB data pipeline — downloads L2 orderbook history from Tardis.dev |
| backtest | `script/backtest/` | Legacy backtest data feeder (outputs JSON for recipe evaluation) |
| scalper | `script/scalper/` | Scalper paper trading CLI |

## Usage

```bash
# Scan for trades (default $100 AV)
cargo run -p signal -- --top 10
cargo run -p signal -- --coin HYPE --av 1000

# Intrahour momentum
cargo run -p momentum -- --top 50

# Monitor a position
cargo run -p monitor -- --coin HYPE --entry 59.68 --dir long --tp 62.0 --sl 58.36

# Backtest strategy
cargo run --release -p strattest -- --coin HYPE --months 6 --av 1000 --micro-interval 15m

# Download OB data for backtesting
cargo run --release -p obdata -- --coin BTC --start 2025-01-01 --end 2025-05-01
```

## Adding a New Script

1. Create folder: `script/<name>/`
2. Add `Cargo.toml` with `[[bin]]` and `src/main.rs`
3. Add to workspace in root `Cargo.toml`
4. Add spec: `spec/scripts/<name>.md`
5. Add `USAGE.md` in the script folder
6. Update this README

## Rust Quality Rules

- **Zero warnings.** All scripts must compile with no warnings.
- Use `#[allow(dead_code)]` on structs with fields kept for API completeness.
- Never leave unused imports, unused variables, or unused mut.
- No file should exceed 600 lines.
- Use `--release` for backtesting and data-heavy scripts.
