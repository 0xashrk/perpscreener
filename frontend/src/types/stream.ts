export type StreamStatus = "connecting" | "open" | "reconnecting" | "error";

export type ParseResult<T> =
  | { ok: true; value: T }
  | { ok: false; reason: string };
