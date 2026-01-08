# Scripts

Standalone Rust CLI tools in `script/`. Each subfolder is a workspace member.

## Available Scripts

| Script | Path | Description |
|--------|------|-------------|
| backtest | `script/backtest/` | Backtest trading recipes against historical data |

## Usage

```bash
# Run a script from repo root
cargo run -p <name> -- [args]

# Example
cargo run -p backtest -- --coin BTC --hours 12
```

## Adding a New Script

1. Create folder: `script/<name>/`
2. Add `Cargo.toml` and `src/<name>.rs`
3. Add to workspace in root `Cargo.toml`: `members = [".", "script/<name>"]`
4. Add spec: `spec/scripts/<name>.md`
5. Update this README
