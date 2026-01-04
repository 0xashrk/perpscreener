import { useEffect, useMemo, useState } from "react";
import { startSseStream } from "../services/sse";
import { parsePatternSnapshot } from "../services/patterns";
import { PatternSnapshot } from "../types/patterns";
import { StreamStatus } from "../types/stream";
import { buildApiUrl } from "../services/url";

type PatternStreamState = {
  status: StreamStatus;
  snapshot: PatternSnapshot;
  error: string;
};

const EMPTY_SNAPSHOT: PatternSnapshot = { asOfMs: 0, detections: [], summaries: [] };

export const usePatternStream = (): PatternStreamState => {
  const [state, setState] = useState<PatternStreamState>({
    status: "connecting",
    snapshot: EMPTY_SNAPSHOT,
    error: ""
  });

  const streamUrl = useMemo(() => buildApiUrl("/patterns/stream", {}), []);

  useEffect(() => {
    const stop = startSseStream(streamUrl, {
      onStatus: (status) => {
        setState((prev) => ({ ...prev, status }));
      },
      onSnapshot: (data) => {
        const parsed = parsePatternSnapshot(data);
        if (!parsed.ok) {
          setState((prev) => ({ ...prev, status: "error", error: parsed.reason }));
          return;
        }
        setState((prev) => ({
          ...prev,
          status: "open",
          snapshot: parsed.value,
          error: ""
        }));
      }
    });

    return () => {
      stop();
    };
  }, [streamUrl]);

  return state;
};
