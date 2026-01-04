import { useEffect, useMemo, useState } from "react";
import { startSseStream } from "../services/sse";
import { parseAdvancedPatternSnapshot } from "../services/patterns";
import { AdvancedPatternSnapshot } from "../types/patterns";
import { StreamStatus } from "../types/stream";
import { buildApiUrl } from "../services/url";

type AdvancedPatternStreamState = {
  status: StreamStatus;
  snapshot: AdvancedPatternSnapshot;
  error: string;
};

const EMPTY_SNAPSHOT: AdvancedPatternSnapshot = { asOfMs: 0, detections: [] };

export const useAdvancedPatternStream = (): AdvancedPatternStreamState => {
  const [state, setState] = useState<AdvancedPatternStreamState>({
    status: "connecting",
    snapshot: EMPTY_SNAPSHOT,
    error: ""
  });

  const streamUrl = useMemo(() => buildApiUrl("/patterns/advanced/stream", {}), []);

  useEffect(() => {
    const stop = startSseStream(streamUrl, {
      onStatus: (status) => {
        setState((prev) => ({ ...prev, status }));
      },
      onSnapshot: (data) => {
        const parsed = parseAdvancedPatternSnapshot(data);
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
