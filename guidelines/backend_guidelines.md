# Backend Guidelines (Axum)

## Tech Stack

- **Framework:** Axum
- **OpenAPI:** utoipa + utoipa-swagger-ui
- **Async Runtime:** Tokio
- **Validation:** validator crate
- **Error Handling:** thiserror + anyhow

---

## Architecture

Follow a layered architecture:
- `handlers/` — thin HTTP layer, extracts request data, calls services, returns responses
- `services/` — orchestration layer, coordinates between business logic and repositories
- `business_logic/` — pure business logic and domain rules, no I/O or external dependencies
- `repositories/` — database access, no business logic (if added)
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

## Code Style and Quality

- Run `cargo fmt` before committing
- Run `cargo clippy` with no warnings
- Keep functions under 50 lines
- Prefer `Arc<T>` for shared state
- Use descriptive error messages
- Keep code modularized: isolate features, keep layers clean, avoid large mixed-responsibility files

---

## Testing (Required)

- Always add tests for new or changed behavior.
- Favor behavior-focused tests over implementation details.
