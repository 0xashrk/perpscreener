# Frontend Guidelines (React + TypeScript)

## Scope

- Frontend-only work; backend rules live in `guidelines/backend_guidelines.md`.
- Frontend code lives in `frontend/` at the repo root.

---

## Tech Stack

- **Framework:** React (TypeScript, functional components)
- **Build tool:** use existing tooling; if none, prefer Vite
- **Testing:** React Testing Library + Vitest (or existing test stack if already present)
- **Styling:** Tailwind CSS (utility-first, no custom CSS unless necessary)

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
  - `src/styles/` — minimal global styles only when Tailwind cannot cover the need

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

- Do not use `any`, `unknown`, or `undefined` in app types. Convert external input to typed shapes at the boundary.
- Use optional props (`?`) sparingly with defaults; do not annotate values as `| undefined`.
- Avoid `null` in app types; normalize incoming data so the app layer never handles `null`.
- Prefer reusable, well-named types in `src/types/`.
- Validate user input at the boundary; use existing validation tooling if present.

## Modernization

- For every feature change, clean up legacy code in the touched area and remove outdated patterns.
- No backward-compatibility obligations: prefer forward-looking APIs and delete legacy shims as you go.

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
