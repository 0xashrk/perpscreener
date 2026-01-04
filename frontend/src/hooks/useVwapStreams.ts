import { useEffect, useMemo, useState } from "react";
import { buildApiUrl } from "../services/url";
import { startSseStream } from "../services/sse";
import { parseVwapSnapshot } from "../services/parsers";
import { StreamStatus } from "../types/stream";
import { VwapTimeframe } from "../types/vwap";
import { VwapCell, VwapTokenState } from "../types/ui";

const createEmptyCell = (): VwapCell => ({
  position: "above",
  distancePct: 0,
  hasData: false
});

const buildDefaults = (tokens: string[], timeframes: VwapTimeframe[]) => {
  const byTimeframe = timeframes.reduce<Record<VwapTimeframe, VwapCell>>((acc, timeframe) => {
    acc[timeframe] = createEmptyCell();
    return acc;
  }, {} as Record<VwapTimeframe, VwapCell>);

  return tokens.reduce<Record<string, VwapTokenState>>((acc, token) => {
    acc[token] = {
      lastUpdatedMs: 0,
      byTimeframe: { ...byTimeframe }
    };
    return acc;
  }, {});
};

export const useVwapStreams = (tokens: string[], timeframes: VwapTimeframe[], interval: string) => {
  const [statusByToken, setStatusByToken] = useState<Record<string, StreamStatus>>(() =>
    tokens.reduce<Record<string, StreamStatus>>((acc, token) => {
      acc[token] = "connecting";
      return acc;
    }, {})
  );
  const [vwapByToken, setVwapByToken] = useState<Record<string, VwapTokenState>>(() =>
    buildDefaults(tokens, timeframes)
  );

  const tokenKey = useMemo(() => tokens.join(","), [tokens]);
  const timeframeKey = useMemo(() => timeframes.join(","), [timeframes]);

  useEffect(() => {
    setStatusByToken((prev) => {
      const next: Record<string, StreamStatus> = {};
      tokens.forEach((token) => {
        next[token] = prev[token] ?? "connecting";
      });
      return next;
    });
    setVwapByToken((prev) => {
      const next: Record<string, VwapTokenState> = {};
      tokens.forEach((token) => {
        const existing = prev[token];
        const nextByTimeframe = timeframes.reduce<Record<VwapTimeframe, VwapCell>>(
          (acc, timeframe) => {
            acc[timeframe] = existing?.byTimeframe[timeframe] ?? createEmptyCell();
            return acc;
          },
          {} as Record<VwapTimeframe, VwapCell>
        );
        next[token] = {
          lastUpdatedMs: existing?.lastUpdatedMs ?? 0,
          byTimeframe: nextByTimeframe
        };
      });
      return next;
    });
  }, [tokenKey, timeframeKey, tokens, timeframes]);

  useEffect(() => {
    const stops = tokens.map((token) => {
      const url = buildApiUrl("/vwap/stream", {
        coin: token,
        timeframes: timeframes.join(","),
        interval
      });

      return startSseStream(url, {
        onStatus: (status) => {
          setStatusByToken((prev) => ({
            ...prev,
            [token]: status
          }));
        },
        onSnapshot: (data) => {
          const parsed = parseVwapSnapshot(data);
          if (!parsed.ok) {
            return;
          }

          const receivedAtMs = Date.now();
          setVwapByToken((prev) => {
            const current = prev[token] ?? {
              lastUpdatedMs: 0,
              byTimeframe: timeframes.reduce<Record<VwapTimeframe, VwapCell>>((acc, tf) => {
                acc[tf] = createEmptyCell();
                return acc;
              }, {} as Record<VwapTimeframe, VwapCell>)
            };

            const nextByTimeframe = { ...current.byTimeframe };
            parsed.value.vwaps.forEach((entry) => {
              if (!(entry.timeframe in nextByTimeframe)) {
                return;
              }
              nextByTimeframe[entry.timeframe] = {
                position: entry.position,
                distancePct: entry.distancePct,
                hasData: true
              };
            });

            return {
              ...prev,
              [token]: {
                lastUpdatedMs: receivedAtMs,
                byTimeframe: nextByTimeframe
              }
            };
          });
        }
      });
    });

    return () => {
      stops.forEach((stop) => stop());
    };
  }, [tokenKey, timeframeKey, interval, tokens, timeframes]);

  return { statusByToken, vwapByToken };
};
