# OpenAPI Guidelines

## Updating OpenAPI Snapshots

After adding or modifying backend endpoints, update the OpenAPI JSON files using [openapi-snapshot](https://github.com/0xashrk/openapi-snapshot).

### Prerequisites

```bash
# Install (one-time)
cargo install openapi-snapshot
```

### Usage

The server must be running before executing openapi-snapshot.

```bash
# Start server first
bacon run  # or: cargo run

# Generate snapshots
openapi-snapshot --url http://localhost:30001/api-docs/openapi.json \
  --out openapi/backend_openapi.json \
  --outline-out openapi/backend_openapi.outline.json
```

### Watch Mode

For continuous updates during development:

```bash
openapi-snapshot watch --url http://localhost:30001/api-docs/openapi.json \
  --out openapi/backend_openapi.json \
  --outline-out openapi/backend_openapi.outline.json
```

### Output Files

- `openapi/backend_openapi.json` - Full OpenAPI spec
- `openapi/backend_openapi.outline.json` - Condensed outline for quick reference
