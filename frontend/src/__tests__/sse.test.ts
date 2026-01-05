import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { startSseStream } from "../services/sse";

class MockEventSource {
  static OPEN = 1;
  static CONNECTING = 0;
  static CLOSED = 2;

  readyState = MockEventSource.OPEN;
  url: string;

  constructor(url: string) {
    this.url = url;
  }

  addEventListener = vi.fn();
  close = vi.fn();
}

describe("startSseStream", () => {
  const originalEventSource = globalThis.EventSource;

  beforeEach(() => {
    globalThis.EventSource = MockEventSource as unknown as typeof EventSource;
  });

  afterEach(() => {
    globalThis.EventSource = originalEventSource;
  });

  it("marks the stream open when the EventSource is already open", () => {
    const statuses: string[] = [];

    const stop = startSseStream("http://example.test/stream", {
      onStatus: (status) => statuses.push(status),
      onSnapshot: vi.fn()
    });

    stop();

    expect(statuses).toEqual(["connecting", "open"]);
  });
});
