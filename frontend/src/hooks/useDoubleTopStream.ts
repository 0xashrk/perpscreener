import { useEffect, useMemo, useState } from "react";
import { buildApiUrl } from "../services/url";
import { startSseStream } from "../services/sse";
import { parseDoubleTopSnapshot } from "../services/parsers";
import { DoubleTopState } from "../types/doubleTop";
import { StreamStatus } from "../types/stream";
import { PatternState } from "../types/ui";
import { formatDoubleTopState } from "../utils/format";

const DEFAULT_STATE: PatternState = {
  stateKey: "WATCHING",
  stateLabel: formatDoubleTopState("WATCHING"),
  lastUpdatedMs: 0,
  hasData: false
};

const buildDefaults = (tokens: string[]): Record<string, PatternState> => {
  return tokens.reduce<Record<string, PatternState>>((acc, token) => {
    acc[token] = { ...DEFAULT_STATE };
    return acc;
  }, {});
};

export const useDoubleTopStream = (tokens: string[]) => {
  const [status, setStatus] = useState<StreamStatus>("connecting");
  const [patternsByToken, setPatternsByToken] = useState<Record<string, PatternState>>(() =>
    buildDefaults(tokens)
  );

  const tokenKey = useMemo(() => tokens.join(","), [tokens]);
  const tokenSet = useMemo(() => new Set(tokens), [tokenKey]);

  useEffect(() => {
    setPatternsByToken((prev) => {
      const next: Record<string, PatternState> = {};
      tokens.forEach((token) => {
        next[token] = prev[token] ?? { ...DEFAULT_STATE };
      });
      return next;
    });
  }, [tokenKey]);

  useEffect(() => {
    const url = buildApiUrl("/double-top/stream", {});

    const stop = startSseStream(url, {
      onStatus: setStatus,
      onSnapshot: (data) => {
        const parsed = parseDoubleTopSnapshot(data);
        if (!parsed.ok) {
          return;
        }

        const receivedAtMs = Date.now();
        setPatternsByToken((prev) => {
          const next = { ...prev };
          parsed.value.patterns.forEach((pattern) => {
            if (!tokenSet.has(pattern.coin)) {
              return;
            }
            next[pattern.coin] = {
              stateKey: pattern.state,
              stateLabel: formatDoubleTopState(pattern.state),
              lastUpdatedMs: receivedAtMs,
              hasData: true
            };
          });
          return next;
        });
      }
    });

    return () => {
      stop();
    };
  }, [tokenKey, tokenSet]);

  return { status, patternsByToken };
};
