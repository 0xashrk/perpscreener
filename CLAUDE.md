# Agent Guidelines

## Specification Files

All spec files must be placed in the `spec/` folder in this directory.

No file should exceed 600 lines of code.

---

## Tech Stack

### Backend (Rust)
- **Framework:** Axum
- **OpenAPI:** utoipa + utoipa-swagger-ui
- **Async Runtime:** Tokio
- **Validation:** validator crate
- **Error Handling:** thiserror + anyhow

### Frontend (TypeScript)
- **Runtime/Package Manager:** Bun
- **Framework:** React 18+ with TypeScript
- **Build Tool:** Vite
- **Styling:** Tailwind CSS
- **State Management:** React Query (TanStack Query) for server state
- **Location:** `frontend/` directory

---

## Architecture

Follow a layered architecture:
- `handlers/` — thin HTTP layer, extracts request data, calls services, returns responses
- `services/` — orchestration layer, coordinates between business logic and repositories
- `business/` — pure business logic and domain rules, no I/O or external dependencies
- `repositories/` — database access, no business logic
- `models/` — request/response DTOs with `Serialize`, `Deserialize`, `ToSchema`
- `errors/` — custom `AppError` enum implementing `IntoResponse`

### Service vs Business Logic

- **Services** handle orchestration: call repos, call business logic, handle transactions
- **Business logic** is pure: validations, calculations, domain rules — no async, no DB, no HTTP

---

## Conventions

- All handlers must have `#[utoipa::path(...)]` annotations
- All request/response structs must derive `ToSchema`
- Use `State` extractor for dependency injection
- Use `Result<T, AppError>` return types and propagate errors with `?`
- Validate requests with `validator` crate at handler level
- Group routes with `Router::nest()` by resource
- Serve Swagger UI at `/swagger-ui`

---

## Code Style

### Backend (Rust)
- Run `cargo fmt` before committing
- Run `cargo clippy` with no warnings
- Keep functions under 50 lines
- Prefer `Arc<T>` for shared state
- Use descriptive error messages

### Frontend (TypeScript)
- Use `bun` for all package management and scripts
- Strict TypeScript (`strict: true` in tsconfig)
- Functional components with hooks only (no class components)
- Use named exports over default exports
- Keep components under 100 lines; extract logic to custom hooks
- Run `bun run lint` before committing
