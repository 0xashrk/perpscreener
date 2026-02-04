# Scripts

Standalone Rust CLI tools in `script/`. Each subfolder is a workspace member.

## Available Scripts

| Script | Path | Description |
|--------|------|-------------|
| backtest | `script/backtest/` | Backtest trading recipes against historical data |
| signal | `script/signal/` | Evaluate HL_ALPHA signals against live backend data |
| scalper | `script/scalper/` | Long-running SCALPER paper trading CLI |
| momentum | `script/momentum/` | Intrahour momentum context report (MOMENTUM recipe) |

## Usage

```bash
# Run a script from repo root
cargo run -p <name> -- [args]

# Examples
cargo run -p backtest -- --coin BTC --hours 12
cargo run -p signal -- --coin BTC
cargo run -p momentum -- --coin BTC --limit 120
```

## Adding a New Script

1. Create folder: `script/<name>/`
2. Add `Cargo.toml` and `src/<name>.rs`
3. Add to workspace in root `Cargo.toml`: `members = [".", "script/<name>"]`
4. Add spec: `spec/scripts/<name>.md`
5. Update this README
