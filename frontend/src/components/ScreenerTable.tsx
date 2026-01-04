import { StatusPill, StatusTone } from "./StatusPill";
import { DoubleTopState } from "../types/doubleTop";
import { VwapTimeframe } from "../types/vwap";
import { PatternState, VwapCell, VwapTokenState } from "../types/ui";
import { formatDistancePct, formatPosition } from "../utils/format";
import { formatAge } from "../utils/time";

type ScreenerTableProps = {
  tokens: string[];
  patternsByToken: Record<string, PatternState>;
  vwapByToken: Record<string, VwapTokenState>;
  timeframes: VwapTimeframe[];
  nowMs: number;
};

const patternTone = (state: DoubleTopState): StatusTone => {
  switch (state) {
    case "CONFIRMED":
      return "negative";
    case "FORMING":
      return "warning";
    case "INVALIDATED":
      return "neutral";
    default:
      return "neutral";
  }
};

const VwapCellView = ({ cell }: { cell?: VwapCell }) => {
  if (!cell || !cell.hasData) {
    return <span className="text-xs text-slate-400">--</span>;
  }

  const toneClass = cell.position === "above" ? "text-emerald-700" : "text-rose-700";

  return (
    <div className="flex flex-col">
      <span className={`text-sm font-semibold ${toneClass}`}>{formatPosition(cell.position)}</span>
      <span className="text-xs text-slate-500">{formatDistancePct(cell.distancePct)}</span>
    </div>
  );
};

export const ScreenerTable = ({
  tokens,
  patternsByToken,
  vwapByToken,
  timeframes,
  nowMs
}: ScreenerTableProps) => {
  const emptyPattern: PatternState = {
    stateKey: "WATCHING",
    stateLabel: "Watching",
    lastUpdatedMs: 0,
    hasData: false
  };

  return (
    <section className="glass-panel rounded-3xl border border-white/70 p-6 shadow-[0_20px_60px_rgba(15,23,42,0.12)]">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-slate-900">Live Screener</h2>
          <p className="text-sm text-slate-500">Double Top + VWAP snapshot per token</p>
        </div>
        <span className="rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-semibold text-slate-500">
          {tokens.length} tokens
        </span>
      </div>

      <div className="mt-4 rounded-2xl border border-slate-200 bg-white/70 p-4 text-sm text-slate-600">
        <span className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
          Double Top States
        </span>
        <div className="mt-3 grid gap-2 md:grid-cols-2">
          <div>
            <span className="font-semibold text-slate-700">Watching</span> — waiting for a first peak.
          </div>
          <div>
            <span className="font-semibold text-slate-700">Peak Found</span> — first peak confirmed.
          </div>
          <div>
            <span className="font-semibold text-slate-700">Trough Found</span> — pullback/neckline formed.
          </div>
          <div>
            <span className="font-semibold text-slate-700">Forming</span> — price approaching the first peak.
          </div>
          <div>
            <span className="font-semibold text-slate-700">Confirmed</span> — breakdown below neckline.
          </div>
          <div>
            <span className="font-semibold text-slate-700">Invalidated</span> — pattern failed (broke above peak or timed out).
          </div>
        </div>
      </div>

      <div className="mt-6 overflow-x-auto">
        <table className="w-full min-w-[720px] border-separate border-spacing-y-3 text-left">
          <thead>
            <tr className="text-xs uppercase tracking-[0.2em] text-slate-400">
              <th className="px-4">Token</th>
              <th className="px-4">Double Top State</th>
              <th className="px-4">Timeframe</th>
              <th className="px-4">Age (last update)</th>
              {timeframes.map((timeframe) => (
                <th key={timeframe} className="px-4">
                  VWAP {timeframe}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {tokens.map((token) => {
              const pattern = patternsByToken[token] ?? emptyPattern;
              const vwap = vwapByToken[token];
              const ageLabel = pattern.hasData ? formatAge(pattern.lastUpdatedMs, nowMs) : "--";

              return (
                <tr
                  key={token}
                  className="rounded-2xl bg-white/80 shadow-[0_16px_40px_rgba(15,23,42,0.08)]"
                >
                  <td className="px-4 py-4 text-sm font-semibold text-slate-900">{token}</td>
                  <td className="px-4 py-4">
                    {pattern.hasData ? (
                      <StatusPill
                        label={pattern.stateLabel}
                        tone={patternTone(pattern.stateKey)}
                        className=""
                      />
                    ) : (
                      <span className="text-xs text-slate-400">No signal</span>
                    )}
                  </td>
                  <td className="px-4 py-4 text-sm text-slate-600">1m</td>
                  <td className="px-4 py-4 text-sm text-slate-600">{ageLabel}</td>
                  {timeframes.map((timeframe) => {
                    const cell = vwap?.byTimeframe[timeframe];
                    return (
                      <td key={`${token}-${timeframe}`} className="px-4 py-4">
                        <VwapCellView cell={cell} />
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
};
