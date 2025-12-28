import { useMemo, useState } from "react";
import { HeaderBar } from "./components/HeaderBar";
import { ScreenerTable } from "./components/ScreenerTable";
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

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#F6F2EB] via-[#F2F6F9] to-[#E7EFF6] px-6 py-10 text-slate-900">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-8">
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
      </div>
    </div>
  );
};

export default App;
