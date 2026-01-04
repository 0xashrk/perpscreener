import { useEffect, useMemo, useState } from "react";
import { startSseStream } from "../services/sse";
import { parsePatternLifecycleSnapshot } from "../services/patterns";
import { PatternLifecycleSnapshot } from "../types/patterns";
import { StreamStatus } from "../types/stream";
import { buildApiUrl } from "../services/url";

type PatternLifecycleStreamState = {
  status: StreamStatus;
  snapshot: PatternLifecycleSnapshot;
  error: string;
};

const EMPTY_SNAPSHOT: PatternLifecycleSnapshot = { asOfMs: 0, entries: [] };

export const usePatternLifecycleStream = (): PatternLifecycleStreamState => {
  const [state, setState] = useState<PatternLifecycleStreamState>({
    status: "connecting",
    snapshot: EMPTY_SNAPSHOT,
    error: ""
  });

  const streamUrl = useMemo(() => buildApiUrl("/patterns/lifecycle/stream", {}), []);

  useEffect(() => {
    const stop = startSseStream(streamUrl, {
      onStatus: (status) => {
        setState((prev) => ({ ...prev, status }));
      },
      onSnapshot: (data) => {
        const parsed = parsePatternLifecycleSnapshot(data);
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
