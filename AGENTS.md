# Agent Guidelines

## Specification Files

All spec files must be placed in the `spec/` folder in this directory.

No file should exceed 600 lines of code.

Guidelines:
- Backend: `guidelines/backend_guidelines.md`
- Frontend: `guidelines/frontend_guidelines.md`

## Language Guidance

### Rust

- Do NOT use unwraps or anything that can panic in Rust code, handle errors. Obviously in tests unwraps and panics are fine!
- In Rust code I prefer using `crate::` to `super::`; please don't use `super::`. If you see a lingering `super::` from someone else clean it up.
- Avoid `pub use` on imports unless you are re-exposing a dependency so downstream consumers do not have to depend on it directly.
- Skip global state via `lazy_static!`, `Once`, or similar; prefer passing explicit context structs for any shared state.

#### Rust Workflow Checklist

1. Run `cargo fmt`.
1. Run `cargo clippy --all --benches --tests --examples --all-features` and address warnings.
1. Execute the relevant `cargo test` or `just` targets to cover unit and end-to-end paths.

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

## Development

- Prefer `bacon run` for auto rebuild/restart during dev; refresh Swagger UI after restarts to pick up OpenAPI changes.
