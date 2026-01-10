# Scalper Paper Trader

Paper-trades the SCALPER recipe and logs signals/trades/equity to SQLite.

```bash
# From repo root
cargo run -p scalper -- --coin ETH --capital 100 --duration 24h
```

Flags:
- `--coin` (required): asset symbol (e.g., BTC, ETH)
- `--capital` (optional, default 100): starting USD capital
- `--duration` (optional): run time (e.g., 6h, 2d); omit for unlimited
- `--backend` (optional, default http://localhost:30001): backend base URL
- `--db` (optional, default data/scalper.db): SQLite path
- `--interval` (optional, default 60): poll interval seconds
