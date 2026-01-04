import { useMemo, useState } from "react";
import { PATTERN_INTERVALS, TOKENS } from "../../config";
import { useAdvancedPatternStream } from "../../hooks/useAdvancedPatternStream";
import { usePatternStream } from "../../hooks/usePatternStream";
import { usePatternLifecycleStream } from "../../hooks/usePatternLifecycleStream";
import { PatternSignalType } from "../../types/patterns";
import { createDefaultWeightConfig, summarizeDetections } from "../../utils/patternAggregation";
import { AdvancedPatternList } from "./AdvancedPatternList";
import { PatternFilters } from "./PatternFilters";
import { PatternList } from "./PatternList";
import { PatternLiveCanvas } from "./PatternLiveCanvas";
import { PatternSummaryPanel } from "./PatternSummaryPanel";
import { PatternLifecycleBoard } from "./PatternLifecycleBoard";
import { PatternWeightControls } from "./PatternWeightControls";

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
  const [activePanel, setActivePanel] = useState<"core" | "advanced">("core");
  const [activeTokens, setActiveTokens] = useState<string[]>([...TOKENS]);
  const [activeIntervals, setActiveIntervals] = useState<string[]>([...PATTERN_INTERVALS]);
  const [weightConfig, setWeightConfig] = useState(() => createDefaultWeightConfig());

  const tokensInScope = useMemo(() => [...activeTokens], [activeTokens]);
  const intervalsInScope = useMemo(() => [...activeIntervals], [activeIntervals]);

  const stream = usePatternStream();
  const advancedStream = useAdvancedPatternStream();
  const lifecycleStream = usePatternLifecycleStream();

  const scopedDetections = useMemo(() => {
    return stream.snapshot.detections
      .filter((detection) => tokensInScope.includes(detection.coin))
      .filter((detection) => intervalsInScope.includes(detection.interval));
  }, [intervalsInScope, stream.snapshot.detections, tokensInScope]);

  const sortedDetections = useMemo(() => {
    return [...scopedDetections].sort((a, b) => b.detectedAtMs - a.detectedAtMs).slice(0, 100);
  }, [scopedDetections]);

  const summaries = useMemo(() => {
    return summarizeDetections(scopedDetections, weightConfig);
  }, [scopedDetections, weightConfig]);

  const scopedLifecycleEntries = useMemo(() => {
    const scoped = lifecycleStream.snapshot.entries
      .filter((entry) => tokensInScope.includes(entry.coin))
      .filter((entry) => intervalsInScope.includes(entry.interval));
    if (activePanel === "advanced") {
      return scoped.filter((entry) =>
        ["fibonacci_retracement", "elliott_wave", "williams_fractal"].includes(entry.category)
      );
    }
    return scoped.filter(
      (entry) => !["fibonacci_retracement", "elliott_wave", "williams_fractal"].includes(entry.category)
    );
  }, [activePanel, intervalsInScope, lifecycleStream.snapshot.entries, tokensInScope]);

  const sortedAdvanced = useMemo(() => {
    return advancedStream.snapshot.detections
      .filter((detection) => tokensInScope.includes(detection.coin))
      .filter((detection) => intervalsInScope.includes(detection.interval))
      .sort((a, b) => b.detectedAtMs - a.detectedAtMs)
      .slice(0, 100);
  }, [advancedStream.snapshot.detections, intervalsInScope, tokensInScope]);

  const handleToggleToken = (token: string) => {
    setActiveTokens((prev) => clampSelection(prev, token, TOKENS));
  };

  const handleToggleInterval = (interval: string) => {
    setActiveIntervals((prev) => clampSelection(prev, interval, PATTERN_INTERVALS));
  };

  const handleTimeframeWeightChange = (interval: string, value: number) => {
    setWeightConfig((prev) => ({
      ...prev,
      timeframe: {
        ...prev.timeframe,
        [interval]: value
      }
    }));
  };

  const handleSignalWeightChange = (signalType: PatternSignalType, value: number) => {
    setWeightConfig((prev) => ({
      ...prev,
      signalType: {
        ...prev.signalType,
        [signalType]: value
      }
    }));
  };

  const handleResetWeights = () => {
    setWeightConfig(createDefaultWeightConfig());
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
        <div className="flex flex-col gap-4">
          <div className="flex gap-2 rounded-full bg-white/70 p-1 shadow-[0_12px_30px_rgba(15,23,42,0.12)]">
            <button
              className={`rounded-full px-4 py-2 text-sm font-semibold transition ${
                activePanel === "core"
                  ? "bg-slate-900 text-white shadow-lg shadow-slate-900/20"
                  : "text-slate-600 hover:bg-white/70 hover:text-slate-900"
              }`}
              onClick={() => setActivePanel("core")}
              type="button"
            >
              Core
            </button>
            <button
              className={`rounded-full px-4 py-2 text-sm font-semibold transition ${
                activePanel === "advanced"
                  ? "bg-slate-900 text-white shadow-lg shadow-slate-900/20"
                  : "text-slate-600 hover:bg-white/70 hover:text-slate-900"
              }`}
              onClick={() => setActivePanel("advanced")}
              type="button"
            >
              Advanced
            </button>
          </div>
          {activePanel === "core" ? (
            <PatternList detections={sortedDetections} status={stream.status} error={stream.error} />
          ) : (
            <AdvancedPatternList
              detections={sortedAdvanced}
              status={advancedStream.status}
              error={advancedStream.error}
            />
          )}
        </div>
        <div className="flex flex-col gap-4">
          {activePanel === "core" ? (
            <>
              <PatternLifecycleBoard
                entries={scopedLifecycleEntries}
                status={lifecycleStream.status}
                error={lifecycleStream.error}
                nowMs={Date.now()}
              />
              <PatternSummaryPanel summaries={summaries} />
              <PatternWeightControls
                intervals={PATTERN_INTERVALS}
                timeframeWeights={weightConfig.timeframe}
                signalWeights={weightConfig.signalType}
                onTimeframeChange={handleTimeframeWeightChange}
                onSignalChange={handleSignalWeightChange}
                onReset={handleResetWeights}
              />
            </>
          ) : (
            <PatternLifecycleBoard
              entries={scopedLifecycleEntries}
              status={lifecycleStream.status}
              error={lifecycleStream.error}
              nowMs={Date.now()}
            />
          )}
          <PatternFilters
            tokens={TOKENS}
            activeTokens={tokensInScope}
            onToggleToken={handleToggleToken}
            intervals={PATTERN_INTERVALS}
            activeIntervals={intervalsInScope}
            onToggleInterval={handleToggleInterval}
          />
          <PatternLiveCanvas
            signals={(activePanel === "core" ? sortedDetections : sortedAdvanced).map(
              (signal) => ({
                pattern: signal.pattern,
                coin: signal.coin,
                interval: signal.interval,
                detectedAtMs: signal.detectedAtMs,
                windowEndMs: signal.windowEndMs
              })
            )}
            status={activePanel === "core" ? stream.status : advancedStream.status}
            lastUpdatedMs={
              activePanel === "core" ? stream.snapshot.asOfMs : advancedStream.snapshot.asOfMs
            }
          />
        </div>
      </div>
    </section>
  );
};
