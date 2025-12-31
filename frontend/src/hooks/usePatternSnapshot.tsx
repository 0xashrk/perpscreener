import { useEffect, useMemo, useState } from "react";
import { PatternSnapshot } from "../types/patterns";
import { fetchPatternSnapshot } from "../services/patterns";

type PatternSnapshotState = {
  status: "idle" | "loading" | "ready" | "error";
  data: PatternSnapshot;
  error: string;
};

type PatternSnapshotQuery = {
  coins: string[];
  intervals: string[];
  limit: number;
  sinceMs: number;
};

const EMPTY_SNAPSHOT: PatternSnapshot = { asOfMs: 0, detections: [], summaries: [] };

export const usePatternSnapshot = (query: PatternSnapshotQuery): PatternSnapshotState => {
  const [state, setState] = useState<PatternSnapshotState>({
    status: "idle",
    data: EMPTY_SNAPSHOT,
    error: ""
  });

  const queryKey = useMemo(() => JSON.stringify(query), [query]);

  useEffect(() => {
    let active = true;
    setState((prev) => ({ ...prev, status: "loading", error: "" }));

    fetchPatternSnapshot(query)
      .then((data) => {
        if (!active) {
          return;
        }
        setState({ status: "ready", data, error: "" });
      })
      .catch((error: Error) => {
        if (!active) {
          return;
        }
        setState({
          status: "error",
          data: EMPTY_SNAPSHOT,
          error: error.message || "Failed to load patterns"
        });
      });

    return () => {
      active = false;
    };
  }, [queryKey]);

  return state;
};
