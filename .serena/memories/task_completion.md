# Task completion checklist
- Run relevant cargo commands: `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test` (or targeted subsets) before finalizing changes.
- Ensure server still starts via `cargo run` when applicable; verify http://localhost:3000 and Swagger UI at /swagger-ui.
- Update docs/README if behavior or endpoints change.
- Keep tracing logs meaningful and consistent.
- Confirm new/updated patterns or services are covered by unit tests in corresponding modules.