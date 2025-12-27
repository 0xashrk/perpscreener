# Style & conventions
- Rust 2021 with async/await and tokio; axum handlers async and return `impl IntoResponse`/JSON types.
- Uses tracing for logging; prefer structured logs (`tracing::info!`, `tracing::error!`).
- OpenAPI via `utoipa` macros; keep route structs/handlers annotated for schema exposure.
- Modules split by domain: `routes`, `services`, `business_logic`; keep logic/test helpers near detectors.
- Tests colocated in modules (see `business_logic/double_top.rs`); use helper builders for test data.
- Configuration structs derive `Default` where sensible (e.g., `DoubleTopConfig`).
- Prefer not to read full files unless necessary; use symbolic tools for navigation/editing.