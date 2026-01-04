import { useEffect, useState } from "react";
import { fetchPatternRegistry } from "../services/patterns";
import { PatternRegistryEntry } from "../types/patterns";

type PatternRegistryState = {
  status: "loading" | "ready" | "error";
  entries: PatternRegistryEntry[];
  error: string;
};

export const usePatternRegistry = (): PatternRegistryState => {
  const [state, setState] = useState<PatternRegistryState>({
    status: "loading",
    entries: [],
    error: ""
  });

  useEffect(() => {
    let active = true;

    fetchPatternRegistry()
      .then((entries) => {
        if (!active) {
          return;
        }
        setState({ status: "ready", entries, error: "" });
      })
      .catch((error: Error) => {
        if (!active) {
          return;
        }
        setState({
          status: "error",
          entries: [],
          error: error.message || "Failed to load registry"
        });
      });

    return () => {
      active = false;
    };
  }, []);

  return state;
};
