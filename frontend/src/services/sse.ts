import { StreamStatus } from "../types/stream";

export type SseHandlers = {
  onStatus: (status: StreamStatus) => void;
  onSnapshot: (data: string) => void;
};

const BASE_DELAY_MS = 1000;
const MAX_DELAY_MS = 30000;

export const startSseStream = (url: string, handlers: SseHandlers): (() => void) => {
  let active = true;
  let retryDelay = BASE_DELAY_MS;
  let retryTimerId = 0;
  let retryScheduled = false;
  let source = new EventSource(url);

  const clearRetry = () => {
    if (retryScheduled) {
      window.clearTimeout(retryTimerId);
      retryScheduled = false;
    }
  };

  const scheduleReconnect = () => {
    clearRetry();
    retryScheduled = true;
    retryTimerId = window.setTimeout(() => {
      if (!active) {
        return;
      }
      source = new EventSource(url);
      connect(source);
    }, retryDelay);
    retryDelay = Math.min(retryDelay * 2, MAX_DELAY_MS);
  };

  const connect = (currentSource: EventSource) => {
    clearRetry();
    handlers.onStatus("connecting");

    const handleSnapshot = (event: Event) => {
      if (currentSource !== source) {
        return;
      }
      const message = event as MessageEvent<string>;
      retryDelay = BASE_DELAY_MS;
      handlers.onSnapshot(message.data);
    };

    const handleOpen = () => {
      if (currentSource !== source) {
        return;
      }
      retryDelay = BASE_DELAY_MS;
      handlers.onStatus("open");
    };

    const handleError = () => {
      if (currentSource !== source) {
        return;
      }
      handlers.onStatus("reconnecting");
      currentSource.close();
      scheduleReconnect();
    };

    currentSource.addEventListener("open", handleOpen);
    currentSource.addEventListener("snapshot", handleSnapshot);
    currentSource.addEventListener("error", handleError);
  };

  connect(source);

  return () => {
    active = false;
    clearRetry();
    source.close();
  };
};
