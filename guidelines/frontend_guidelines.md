# Frontend Guidelines (React + TypeScript)

## Scope

- Frontend-only work; backend rules live in `spec/backend_guidelines.md`.
- Frontend code lives in `frontend/` at the repo root.

---

## Tech Stack

- **Framework:** React (TypeScript, functional components)
- **Build tool:** use existing tooling; if none, prefer Vite
- **Testing:** React Testing Library + Vitest (or existing test stack if already present)

---

## Architecture

- Prefer feature modules to keep concerns isolated and testable.
- Suggested structure:
  - `src/features/<feature>/` — feature-scoped components, hooks, services, types, tests
  - `src/components/` — shared UI components
  - `src/hooks/` — shared hooks
  - `src/services/` — API/SSE clients, data access helpers
  - `src/types/` — shared TypeScript types
  - `src/utils/` — pure utilities
  - `src/styles/` — global styles, theme tokens

---

## Data Access

- Keep network I/O in `services/` with typed request/response models.
- For SSE, wrap `EventSource` in a small client with reconnect/backoff handling.
- Provide a non-SSE fallback (HTTP) for snapshot data when needed.

---

## State and Composition

- Use local state for component-level concerns.
- For cross-feature state, use Context or a small store only when necessary.
- Keep components small and focused; separate container logic from presentational UI.

---

## Type Safety and Validation

- Avoid `any`; prefer `unknown` with narrowing.
- Prefer reusable, well-named types in `src/types/`.
- Validate user input at the boundary; use existing validation tooling if present.

---

## Code Quality and Testing (Required)

- Always add tests for new or changed behavior.
- Test the behavior, not implementation details.
- Minimum coverage:
  - Components: user-visible behavior
  - Hooks: state transitions
  - Services: request/response shape handling
  - SSE: mock `EventSource`, verify connect/disconnect/retry flow
- Keep code modularized: isolate features, keep layers clean, avoid large mixed-responsibility files.
