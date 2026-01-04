import { useEffect, useMemo, useState } from "react";
import { parseChartSnapshot, buildChartStreamUrl } from "../services/chart";
import { ChartSnapshot } from "../types/chart";
import { StreamStatus } from "../types/stream";
import { startSseStream } from "../services/sse";

type ChartStreamState = {
  status: StreamStatus;
  snapshot: ChartSnapshot;
  error: string;
};

const EMPTY_SNAPSHOT: ChartSnapshot = {
  asOfMs: 0,
  coin: "",
  interval: "",
  candles: []
};

export const useChartStream = (coin: string, interval: string, limit: number): ChartStreamState => {
  const [state, setState] = useState<ChartStreamState>({
    status: "connecting",
    snapshot: EMPTY_SNAPSHOT,
    error: ""
  });

  const streamUrl = useMemo(() => {
    if (!coin || !interval) {
      return "";
    }
    return buildChartStreamUrl({ coin, interval, limit });
  }, [coin, interval, limit]);

  useEffect(() => {
    if (!streamUrl) {
      setState((prev) => ({ ...prev, status: "error", error: "Missing chart query." }));
      return () => {};
    }

    const stop = startSseStream(streamUrl, {
      onStatus: (status) => {
        setState((prev) => ({ ...prev, status }));
      },
      onSnapshot: (data) => {
        const parsed = parseChartSnapshot(data);
        if (!parsed.ok) {
          setState((prev) => ({ ...prev, status: "error", error: parsed.reason }));
          return;
        }
        setState({ status: "open", snapshot: parsed.value, error: "" });
      }
    });

    return () => {
      stop();
    };
  }, [streamUrl]);

  return state;
};
