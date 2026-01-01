# Linting and Unused Code Enforcement

## Goal

Provide repeatable linting for Rust and React/TypeScript to eliminate unused variables/imports,
keep formatting consistent, and catch common correctness issues.

## Scope

- Rust backend (`src/`)
- React + TypeScript frontend (`frontend/`)

## Rust (Backend)

Rust does not use ESLint. Use Clippy + rustfmt.

### Install

- `rustup component add clippy`
- `rustup component add rustfmt`

### Commands

- Format check:
  - `cargo fmt --all -- --check`
- Lint with warnings as errors:
  - `cargo clippy --all-targets --all-features -- -D warnings`

### Notes

- `-D warnings` will fail CI for unused imports/variables and other warnings.
- If needed, allow targeted exceptions with `#[allow(unused_imports)]` or `#[allow(dead_code)]`,
  but only for deliberate cases (document in-code).

## React + TypeScript (Frontend)

Use ESLint with TypeScript + React rules and an unused imports plugin.

### Install

From `frontend/`:

- `bun add -d eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin`
- `bun add -d eslint-plugin-react eslint-plugin-react-hooks`
- `bun add -d eslint-plugin-unused-imports eslint-plugin-import`
- `bun add -d eslint-import-resolver-typescript`
- `bun add -d eslint-config-prettier`

### Config (example)

Create `frontend/.eslintrc.cjs`:

```js
module.exports = {
  root: true,
  parser: "@typescript-eslint/parser",
  plugins: ["@typescript-eslint", "react", "react-hooks", "unused-imports", "import"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:react/recommended",
    "plugin:react-hooks/recommended",
    "plugin:import/recommended",
    "plugin:import/typescript",
    "prettier"
  ],
  settings: {
    react: { version: "detect" },
    "import/resolver": { typescript: { project: ["./tsconfig.json"] } }
  },
  rules: {
    "unused-imports/no-unused-imports": "error",
    "unused-imports/no-unused-vars": [
      "warn",
      { vars: "all", varsIgnorePattern: "^_", args: "after-used", argsIgnorePattern: "^_" }
    ],
    "@typescript-eslint/no-unused-vars": "off",
    "react/prop-types": "off"
  }
};
```

### Scripts

Add to `frontend/package.json`:

- `"lint": "eslint \"src/**/*.{ts,tsx}\""`
- `"lint:fix": "eslint \"src/**/*.{ts,tsx}\" --fix"`

### Notes

- `unused-imports` actively removes unused imports on `lint:fix`.
- Use underscore-prefixed args/vars to intentionally ignore.

## CI / Local Workflow

- Backend:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Frontend:
  - `bun run lint`

## Implementation Checklist

- [ ] Add ESLint dependencies in `frontend/package.json`.
- [ ] Create `frontend/.eslintrc.cjs` with the rules above (or equivalent).
- [ ] Add lint scripts to `frontend/package.json`.
- [ ] Add CI steps or local docs for Rust/Frontend lint commands.
