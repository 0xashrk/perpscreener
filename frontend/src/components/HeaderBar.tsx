import { StatusPill, StatusTone } from "./StatusPill";
import { StreamStatus } from "../types/stream";
import { VwapTimeframe } from "../types/vwap";
import { formatStreamStatus } from "../utils/format";

type HeaderBarProps = {
  tokens: string[];
  activeTokens: string[];
  onToggleToken: (token: string) => void;
  timeframes: VwapTimeframe[];
  activeTimeframes: VwapTimeframe[];
  onToggleTimeframe: (timeframe: VwapTimeframe) => void;
  streamStatus: StreamStatus;
};

const statusTone = (status: StreamStatus): StatusTone => {
  switch (status) {
    case "open":
      return "positive";
    case "reconnecting":
      return "warning";
    case "error":
      return "negative";
    default:
      return "neutral";
  }
};

export const HeaderBar = ({
  tokens,
  activeTokens,
  onToggleToken,
  timeframes,
  activeTimeframes,
  onToggleTimeframe,
  streamStatus
}: HeaderBarProps) => {
  return (
    <header className="glass-panel bg-grid rounded-3xl border border-white/70 p-6 shadow-[0_20px_60px_rgba(15,23,42,0.12)]">
      <div className="flex flex-col gap-5 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500">
            Perp Screener
          </p>
          <h1 className="mt-2 text-3xl font-semibold text-slate-900">
            Double Top + VWAP Dashboard
          </h1>
        </div>
        <StatusPill
          label={formatStreamStatus(streamStatus)}
          tone={statusTone(streamStatus)}
          className="self-start lg:self-auto"
        />
      </div>
      <div className="mt-6 flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-sm font-semibold text-slate-600">Tokens</span>
          {tokens.map((token) => {
            const isActive = activeTokens.includes(token);
            return (
              <button
                key={token}
                type="button"
                onClick={() => onToggleToken(token)}
                className={`rounded-full border px-4 py-1.5 text-sm font-semibold transition ${
                  isActive
                    ? "border-slate-900 bg-slate-900 text-white"
                    : "border-slate-200 bg-white text-slate-700 hover:border-slate-300"
                }`}
              >
                {token}
              </button>
            );
          })}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-sm font-semibold text-slate-600">VWAP Columns</span>
          {timeframes.map((timeframe) => {
            const isActive = activeTimeframes.includes(timeframe);
            return (
              <button
                key={timeframe}
                type="button"
                onClick={() => onToggleTimeframe(timeframe)}
                className={`rounded-full border px-4 py-1.5 text-sm font-semibold uppercase transition ${
                  isActive
                    ? "border-emerald-600 bg-emerald-600 text-white"
                    : "border-slate-200 bg-white text-slate-700 hover:border-slate-300"
                }`}
              >
                {timeframe}
              </button>
            );
          })}
        </div>
      </div>
    </header>
  );
};
