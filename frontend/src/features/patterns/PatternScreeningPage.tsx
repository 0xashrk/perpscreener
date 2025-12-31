import { useMemo, useState } from "react";
import { PATTERN_INTERVALS, TOKENS } from "../../config";
import { usePatternSnapshot } from "../../hooks/usePatternSnapshot";
import { PatternFilters } from "./PatternFilters";
import { PatternList } from "./PatternList";
import { PatternScreeningStub } from "./PatternScreeningStub";

const toggleItem = <T,>(items: T[], item: T): T[] => {
  if (items.includes(item)) {
    return items.filter((entry) => entry !== item);
  }
  return [...items, item];
};

const clampSelection = <T,>(items: T[], item: T, fallback: T[]): T[] => {
  const next = toggleItem(items, item);
  if (next.length === 0) {
    return fallback;
  }
  return next;
};

export const PatternScreeningPage = () => {
  const [activeTokens, setActiveTokens] = useState<string[]>([...TOKENS]);
  const [activeIntervals, setActiveIntervals] = useState<string[]>([...PATTERN_INTERVALS]);

  const tokensInScope = useMemo(() => [...activeTokens], [activeTokens]);
  const intervalsInScope = useMemo(() => [...activeIntervals], [activeIntervals]);

  const snapshot = usePatternSnapshot({
    coins: tokensInScope,
    intervals: intervalsInScope,
    limit: 25,
    sinceMs: 0
  });

  const sortedDetections = useMemo(() => {
    return [...snapshot.data.detections].sort((a, b) => b.detectedAtMs - a.detectedAtMs);
  }, [snapshot.data.detections]);

  const handleToggleToken = (token: string) => {
    setActiveTokens((prev) => clampSelection(prev, token, TOKENS));
  };

  const handleToggleInterval = (interval: string) => {
    setActiveIntervals((prev) => clampSelection(prev, interval, PATTERN_INTERVALS));
  };

  return (
    <section className="flex flex-col gap-6">
      <header className="flex flex-col gap-2">
        <p className="text-xs uppercase tracking-[0.3em] text-slate-500">Pattern Screening</p>
        <h1 className="text-3xl font-semibold text-slate-900">
          Multi-timeframe pattern visualization
        </h1>
        <p className="max-w-2xl text-sm text-slate-600">
          Data ingestion is running and feature precompute is warming up. This space will surface
          pattern detections, confidence, and visual overlays as the backend phases roll out.
        </p>
      </header>
      <div className="grid gap-6 xl:grid-cols-[1.5fr_0.9fr]">
        <PatternList detections={sortedDetections} status={snapshot.status} error={snapshot.error} />
        <div className="flex flex-col gap-4">
          <PatternFilters
            tokens={TOKENS}
            activeTokens={tokensInScope}
            onToggleToken={handleToggleToken}
            intervals={PATTERN_INTERVALS}
            activeIntervals={intervalsInScope}
            onToggleInterval={handleToggleInterval}
          />
          <PatternScreeningStub />
        </div>
      </div>
    </section>
  );
};
