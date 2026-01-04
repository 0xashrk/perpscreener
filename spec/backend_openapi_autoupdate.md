# Backend OpenAPI Auto-Update Options (Minified JSON)

This doc describes how automatic OpenAPI exports would look for a minified JSON spec focused on endpoint inputs and outputs.

## Output Artifact

- Suggested path: `spec/backend_openapi.min.json`
- Format: single-line minified JSON
- Scope:
  - Full OpenAPI document, or
  - Reduced document containing only `paths` and `components` (request/response schemas)

### Example Output (Full, Minified)

```json
{"openapi":"3.0.3","info":{"title":"Perp Screener","version":"0.1.0"},"paths":{"/double-top/stream":{"get":{"responses":{"200":{"description":"SSE stream","content":{"text/event-stream":{"schema":{"$ref":"#/components/schemas/DoubleTopStreamEvent"}}}}}}}},"components":{"schemas":{"DoubleTopStreamEvent":{"oneOf":[{"$ref":"#/components/schemas/DoubleTopSnapshotEvent"},{"$ref":"#/components/schemas/HeartbeatEvent"}]},"DoubleTopSnapshotEvent":{"type":"object","properties":{"event":{"type":"string","enum":["snapshot"]},"data":{"$ref":"#/components/schemas/DoubleTopSnapshot"}},"required":["event","data"]},"HeartbeatEvent":{"type":"object","properties":{"event":{"type":"string","enum":["heartbeat"]},"data":{"$ref":"#/components/schemas/Heartbeat"}},"required":["event","data"]},"DoubleTopSnapshot":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"},"patterns":{"type":"array","items":{"$ref":"#/components/schemas/DoubleTopPattern"}}},"required":["as_of_ms","patterns"]},"DoubleTopPattern":{"type":"object","properties":{"coin":{"type":"string"},"state":{"type":"string","enum":["WATCHING","PEAK_FOUND","TROUGH_FOUND","FORMING","CONFIRMED","INVALIDATED"]},"peak1_price":{"type":"number","nullable":true},"neckline_price":{"type":"number","nullable":true},"peak2_price":{"type":"number","nullable":true},"is_warmed_up":{"type":"boolean"},"summary":{"type":"string"}},"required":["coin","state","is_warmed_up","summary"]},"Heartbeat":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"}},"required":["as_of_ms"]}}}}
```

### Example Output (Reduced, Minified)

```json
{"paths":{"/double-top/stream":{"get":{"responses":{"200":{"description":"SSE stream","content":{"text/event-stream":{"schema":{"$ref":"#/components/schemas/DoubleTopStreamEvent"}}}}}}}},"components":{"schemas":{"DoubleTopStreamEvent":{"oneOf":[{"$ref":"#/components/schemas/DoubleTopSnapshotEvent"},{"$ref":"#/components/schemas/HeartbeatEvent"}]},"DoubleTopSnapshotEvent":{"type":"object","properties":{"event":{"type":"string","enum":["snapshot"]},"data":{"$ref":"#/components/schemas/DoubleTopSnapshot"}},"required":["event","data"]},"HeartbeatEvent":{"type":"object","properties":{"event":{"type":"string","enum":["heartbeat"]},"data":{"$ref":"#/components/schemas/Heartbeat"}},"required":["event","data"]},"DoubleTopSnapshot":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"},"patterns":{"type":"array","items":{"$ref":"#/components/schemas/DoubleTopPattern"}}},"required":["as_of_ms","patterns"]},"DoubleTopPattern":{"type":"object","properties":{"coin":{"type":"string"},"state":{"type":"string","enum":["WATCHING","PEAK_FOUND","TROUGH_FOUND","FORMING","CONFIRMED","INVALIDATED"]},"peak1_price":{"type":"number","nullable":true},"neckline_price":{"type":"number","nullable":true},"peak2_price":{"type":"number","nullable":true},"is_warmed_up":{"type":"boolean"},"summary":{"type":"string"}},"required":["coin","state","is_warmed_up","summary"]},"Heartbeat":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"}},"required":["as_of_ms"]}}}}
```

## Option A: Watcher (local dev)

**What it looks like**
- A local watch command runs during development.
- On code changes, it rebuilds the server and exports `spec/backend_openapi.min.json`.

**Implementation steps**
1) Add an exporter binary (e.g., `src/bin/export_openapi.rs`) that:
   - Calls `ApiDoc::openapi()`.
   - Serializes to minified JSON (`serde_json::to_string`).
   - Optionally reduces the payload to `{ paths, components }`.
   - Writes to `spec/backend_openapi.min.json`.
2) Wire the exporter into a watcher:
   - `bacon` job runs `cargo run` and then `cargo run --bin export_openapi`, or
   - `cargo watch -x run -x "run --bin export_openapi"` if you prefer a single command.
3) Commit `spec/backend_openapi.min.json` so the repo always has the latest spec.

**Typical flow**
1) Developer runs a watcher (e.g., `bacon run` or `cargo watch`).
2) Export step runs after rebuild.
3) File stays updated in `spec/` while you iterate.

**Sample output (on change)**
```json
{"openapi":"3.0.3","info":{"title":"Perp Screener","version":"0.1.0"},"paths":{"/double-top/stream":{"get":{"responses":{"200":{"description":"SSE stream","content":{"text/event-stream":{"schema":{"$ref":"#/components/schemas/DoubleTopStreamEvent"}}}}}}}},"components":{"schemas":{"DoubleTopStreamEvent":{"oneOf":[{"$ref":"#/components/schemas/DoubleTopSnapshotEvent"},{"$ref":"#/components/schemas/HeartbeatEvent"}]},"DoubleTopSnapshotEvent":{"type":"object","properties":{"event":{"type":"string","enum":["snapshot"]},"data":{"$ref":"#/components/schemas/DoubleTopSnapshot"}},"required":["event","data"]},"HeartbeatEvent":{"type":"object","properties":{"event":{"type":"string","enum":["heartbeat"]},"data":{"$ref":"#/components/schemas/Heartbeat"}},"required":["event","data"]},"DoubleTopSnapshot":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"},"patterns":{"type":"array","items":{"$ref":"#/components/schemas/DoubleTopPattern"}}},"required":["as_of_ms","patterns"]},"DoubleTopPattern":{"type":"object","properties":{"coin":{"type":"string"},"state":{"type":"string","enum":["WATCHING","PEAK_FOUND","TROUGH_FOUND","FORMING","CONFIRMED","INVALIDATED"]},"peak1_price":{"type":"number","nullable":true},"neckline_price":{"type":"number","nullable":true},"peak2_price":{"type":"number","nullable":true},"is_warmed_up":{"type":"boolean"},"summary":{"type":"string"}},"required":["coin","state","is_warmed_up","summary"]},"Heartbeat":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"}},"required":["as_of_ms"]}}}}
```

**Pros**
- Always current during dev.
- No CI reliance.

**Cons**
- Depends on developer running the watcher.

## Option B: CI Enforcement

**What it looks like**
- CI runs the export step and checks `spec/backend_openapi.min.json`.
- If the file is out of date, CI fails with a diff.

**Typical flow**
1) CI job runs the export command.
2) CI compares generated output to the repo file.
3) Build fails if there is a mismatch.

**Sample output (in repo)**
```json
{"paths":{"/double-top/stream":{"get":{"responses":{"200":{"description":"SSE stream","content":{"text/event-stream":{"schema":{"$ref":"#/components/schemas/DoubleTopStreamEvent"}}}}}}}},"components":{"schemas":{"DoubleTopStreamEvent":{"oneOf":[{"$ref":"#/components/schemas/DoubleTopSnapshotEvent"},{"$ref":"#/components/schemas/HeartbeatEvent"}]},"DoubleTopSnapshotEvent":{"type":"object","properties":{"event":{"type":"string","enum":["snapshot"]},"data":{"$ref":"#/components/schemas/DoubleTopSnapshot"}},"required":["event","data"]},"HeartbeatEvent":{"type":"object","properties":{"event":{"type":"string","enum":["heartbeat"]},"data":{"$ref":"#/components/schemas/Heartbeat"}},"required":["event","data"]},"DoubleTopSnapshot":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"},"patterns":{"type":"array","items":{"$ref":"#/components/schemas/DoubleTopPattern"}}},"required":["as_of_ms","patterns"]},"DoubleTopPattern":{"type":"object","properties":{"coin":{"type":"string"},"state":{"type":"string","enum":["WATCHING","PEAK_FOUND","TROUGH_FOUND","FORMING","CONFIRMED","INVALIDATED"]},"peak1_price":{"type":"number","nullable":true},"neckline_price":{"type":"number","nullable":true},"peak2_price":{"type":"number","nullable":true},"is_warmed_up":{"type":"boolean"},"summary":{"type":"string"}},"required":["coin","state","is_warmed_up","summary"]},"Heartbeat":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"}},"required":["as_of_ms"]}}}}
```

**Pros**
- Guarantees main branch stays in sync.
- No reliance on local dev habits.

**Cons**
- Requires CI configuration.

## Option C: App Startup (dev-only flag)

**What it looks like**
- Server writes the file on startup if an env flag is set.

**Typical flow**
1) Developer sets `EXPORT_OPENAPI=1`.
2) Server starts and writes `spec/backend_openapi.min.json`.
3) File updates whenever the server restarts.

**Sample output (written on boot)**
```json
{"openapi":"3.0.3","info":{"title":"Perp Screener","version":"0.1.0"},"paths":{"/double-top/stream":{"get":{"responses":{"200":{"description":"SSE stream","content":{"text/event-stream":{"schema":{"$ref":"#/components/schemas/DoubleTopStreamEvent"}}}}}}}},"components":{"schemas":{"DoubleTopStreamEvent":{"oneOf":[{"$ref":"#/components/schemas/DoubleTopSnapshotEvent"},{"$ref":"#/components/schemas/HeartbeatEvent"}]},"DoubleTopSnapshotEvent":{"type":"object","properties":{"event":{"type":"string","enum":["snapshot"]},"data":{"$ref":"#/components/schemas/DoubleTopSnapshot"}},"required":["event","data"]},"HeartbeatEvent":{"type":"object","properties":{"event":{"type":"string","enum":["heartbeat"]},"data":{"$ref":"#/components/schemas/Heartbeat"}},"required":["event","data"]},"DoubleTopSnapshot":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"},"patterns":{"type":"array","items":{"$ref":"#/components/schemas/DoubleTopPattern"}}},"required":["as_of_ms","patterns"]},"DoubleTopPattern":{"type":"object","properties":{"coin":{"type":"string"},"state":{"type":"string","enum":["WATCHING","PEAK_FOUND","TROUGH_FOUND","FORMING","CONFIRMED","INVALIDATED"]},"peak1_price":{"type":"number","nullable":true},"neckline_price":{"type":"number","nullable":true},"peak2_price":{"type":"number","nullable":true},"is_warmed_up":{"type":"boolean"},"summary":{"type":"string"}},"required":["coin","state","is_warmed_up","summary"]},"Heartbeat":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"}},"required":["as_of_ms"]}}}}
```

**Pros**
- Minimal extra tooling.
- Works with existing dev flow.

**Cons**
- Tied to server restarts.
- Needs guardrails to avoid writing in production.

## Option D: Manual Export Script

**What it looks like**
- A command is run by hand to regenerate the file.

**Typical flow**
1) Developer runs an export command.
2) File updates in `spec/`.

**Sample output (after manual run)**
```json
{"paths":{"/double-top/stream":{"get":{"responses":{"200":{"description":"SSE stream","content":{"text/event-stream":{"schema":{"$ref":"#/components/schemas/DoubleTopStreamEvent"}}}}}}}},"components":{"schemas":{"DoubleTopStreamEvent":{"oneOf":[{"$ref":"#/components/schemas/DoubleTopSnapshotEvent"},{"$ref":"#/components/schemas/HeartbeatEvent"}]},"DoubleTopSnapshotEvent":{"type":"object","properties":{"event":{"type":"string","enum":["snapshot"]},"data":{"$ref":"#/components/schemas/DoubleTopSnapshot"}},"required":["event","data"]},"HeartbeatEvent":{"type":"object","properties":{"event":{"type":"string","enum":["heartbeat"]},"data":{"$ref":"#/components/schemas/Heartbeat"}},"required":["event","data"]},"DoubleTopSnapshot":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"},"patterns":{"type":"array","items":{"$ref":"#/components/schemas/DoubleTopPattern"}}},"required":["as_of_ms","patterns"]},"DoubleTopPattern":{"type":"object","properties":{"coin":{"type":"string"},"state":{"type":"string","enum":["WATCHING","PEAK_FOUND","TROUGH_FOUND","FORMING","CONFIRMED","INVALIDATED"]},"peak1_price":{"type":"number","nullable":true},"neckline_price":{"type":"number","nullable":true},"peak2_price":{"type":"number","nullable":true},"is_warmed_up":{"type":"boolean"},"summary":{"type":"string"}},"required":["coin","state","is_warmed_up","summary"]},"Heartbeat":{"type":"object","properties":{"as_of_ms":{"type":"integer","format":"int64"}},"required":["as_of_ms"]}}}}
```

**Pros**
- Simple to implement.

**Cons**
- Easy to forget, no guarantees.

## Recommendation

- Best default: Option A (Watcher) for local iteration + Option B (CI) to enforce sync.
- Option C is a good fallback when the team prefers fewer tools.
