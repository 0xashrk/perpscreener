# Perp Screener

## Setup

```bash
cargo build
```

## Run

```bash
cargo run
```

Server: http://localhost:3000
Swagger UI: http://localhost:3000/swagger-ui

## Endpoints

- `GET /health` - Health check

## Dependencies

- `axum` 0.8.8
- `utoipa` 5.4.0
- `utoipa-swagger-ui` 9.0.2

## Project Structure

```
src/
├── main.rs              # Entry point, router setup
├── routes/              # HTTP handlers
├── services/            # External API calls, data fetching
└── business_logic/      # Core algorithms, pattern detection
```
