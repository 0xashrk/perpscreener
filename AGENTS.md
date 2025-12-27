# Agent Guidelines

## Specification Files

All spec files must be placed in the `spec/` folder in this directory.

No file should exceed 600 lines of code.

Guidelines:
- Backend: `guidelines/backend_guidelines.md`
- Frontend: `guidelines/frontend_guidelines.md`

Frontend code lives in this repo under `frontend/`.

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
