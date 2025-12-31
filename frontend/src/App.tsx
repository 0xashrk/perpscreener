import { useMemo, useState } from "react";
import { HeaderBar } from "./components/HeaderBar";
import { ScreenerTable } from "./components/ScreenerTable";
import { PatternScreeningPage } from "./features/patterns/PatternScreeningPage";
import { useHashRoute } from "./hooks/useHashRoute";
import { useDoubleTopStream } from "./hooks/useDoubleTopStream";
import { useNow } from "./hooks/useNow";
import { useVwapStreams } from "./hooks/useVwapStreams";
import { DEFAULT_TIMEFRAMES, TOKENS, VWAP_INTERVAL } from "./config";
import { StreamStatus } from "./types/stream";
import { VwapTimeframe } from "./types/vwap";

const getAggregateStatus = (statuses: StreamStatus[]): StreamStatus => {
  if (statuses.includes("error")) {
    return "error";
  }
  if (statuses.includes("reconnecting")) {
    return "reconnecting";
  }
  if (statuses.every((status) => status === "open")) {
    return "open";
  }
  return "connecting";
};

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

const App = () => {
  const [activeTokens, setActiveTokens] = useState<string[]>([...TOKENS]);
  const [activeTimeframes, setActiveTimeframes] = useState<VwapTimeframe[]>([...DEFAULT_TIMEFRAMES]);
  const route = useHashRoute();

  const tokensInScope = useMemo(() => [...activeTokens], [activeTokens]);
  const timeframesInScope = useMemo(() => [...activeTimeframes], [activeTimeframes]);

  const nowMs = useNow(1_000);

  const { status: doubleTopStatus, patternsByToken } = useDoubleTopStream(tokensInScope);
  const { statusByToken, vwapByToken } = useVwapStreams(
    tokensInScope,
    timeframesInScope,
    VWAP_INTERVAL
  );

  const aggregateStatus = useMemo(() => {
    const vwapStatuses = Object.values(statusByToken);
    return getAggregateStatus([doubleTopStatus, ...vwapStatuses]);
  }, [doubleTopStatus, statusByToken]);

  const handleToggleToken = (token: string) => {
    setActiveTokens((prev) => clampSelection(prev, token, TOKENS));
  };

  const handleToggleTimeframe = (timeframe: VwapTimeframe) => {
    setActiveTimeframes((prev) => clampSelection(prev, timeframe, DEFAULT_TIMEFRAMES));
  };

  const isPatternsRoute = route === "/patterns";

  const linkStyles = (active: boolean) =>
    [
      "rounded-full px-4 py-2 text-sm font-semibold transition",
      active
        ? "bg-slate-900 text-white shadow-lg shadow-slate-900/20"
        : "text-slate-600 hover:bg-white/70 hover:text-slate-900",
    ].join(" ");

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#F6F2EB] via-[#F2F6F9] to-[#E7EFF6] px-6 py-10 text-slate-900">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-8">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-[0.3em] text-slate-500">Perp Screener</p>
            <p className="text-2xl font-semibold text-slate-900">Market Signals</p>
          </div>
          <nav className="flex gap-2 rounded-full bg-white/70 p-1 shadow-[0_12px_30px_rgba(15,23,42,0.12)]">
            <a href="#/" className={linkStyles(!isPatternsRoute)}>
              Screener
            </a>
            <a href="#/patterns" className={linkStyles(isPatternsRoute)}>
              Pattern Screening
            </a>
          </nav>
        </div>
        {isPatternsRoute ? (
          <PatternScreeningPage />
        ) : (
          <>
            <HeaderBar
              tokens={TOKENS}
              activeTokens={tokensInScope}
              onToggleToken={handleToggleToken}
              timeframes={DEFAULT_TIMEFRAMES}
              activeTimeframes={timeframesInScope}
              onToggleTimeframe={handleToggleTimeframe}
              streamStatus={aggregateStatus}
            />
            <ScreenerTable
              tokens={tokensInScope}
              patternsByToken={patternsByToken}
              vwapByToken={vwapByToken}
              timeframes={timeframesInScope}
              nowMs={nowMs}
            />
          </>
        )}
      </div>
    </div>
  );
};

export default App;
