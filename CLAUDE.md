# Agent Guidelines

## Specification Files

All spec files must be placed in the `spec/` folder in this directory.

No file should exceed 600 lines of code.

Guidelines:
- Backend: `guidelines/backend_guidelines.md`
- Frontend: `guidelines/frontend_guidelines.md`
- OpenAPI: `guidelines/openapi_guidelines.md`
- Scripts: `guidelines/scripts_guidelines.md`

Frontend code lives in this repo under `frontend/`.

Backend endpoint details live in `openapi/`.
Use `openapi/backend_openapi.outline.json` for general details and `openapi/backend_openapi.json` for full details.

---

## Project Hierarchy

```
src/
  main.rs          # app bootstrap, router setup
  state.rs         # shared application state
  handlers/        # HTTP handlers and SSE endpoints
  services/        # orchestration layer and external API access
  business_logic/  # pure domain logic and calculations
  models/          # request/response DTOs and schema helpers
  errors/          # AppError and error payloads
frontend/          # React + TypeScript frontend (separate app)
script/            # standalone Rust CLI tools
  signal/          # V2 trade signal scanner (VWAP, regime, daily structure)
  momentum/        # intrahour momentum scanner
  monitor/         # position monitor (exit signal watcher)
  strattest/       # strategy backtester
  obdata/          # historical orderbook data pipeline (Tardis.dev)
  backtest/        # legacy backtest data feeder
  scalper/         # scalper paper trading CLI
```

Folder docs:
- `src/business_logic/business_logic.md`
- `src/errors/errors.md`
- `src/handlers/handlers.md`
- `src/models/models.md`
- `src/services/services.md`

---

## Global Rules

- Always add tests for new or changed behavior.
- Keep code modularized: isolate features, keep layers clean, avoid large mixed-responsibility files.
- **Zero Rust warnings.** Fix all compiler warnings before considering a task done. Use `#[allow(dead_code)]` on structs with fields kept for completeness, but never leave unused imports, unused variables, or unused mut warnings.

## Development

- Prefer `bacon run` for auto rebuild/restart during dev; refresh Swagger UI after restarts to pick up OpenAPI changes.

## OpenAPI Snapshot

After adding or modifying backend endpoints, update the OpenAPI JSON files. See `guidelines/openapi_guidelines.md`.

## Scripts

Standalone CLI tools live in `script/`. See `guidelines/scripts_guidelines.md`.

## Backtesting

See `guidelines/backtest_guidelines.md` for data sources, fee model, and how to run backtests.
