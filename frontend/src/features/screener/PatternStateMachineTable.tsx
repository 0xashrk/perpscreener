import { StatusPill, StatusTone } from "../../components/StatusPill";
import { PatternLifecycleEntry, PatternLifecycleState } from "../../types/patterns";
import { formatAge } from "../../utils/time";

const STATE_LABELS: Record<PatternLifecycleState, string> = {
  warming: "Warming",
  watching: "Watching",
  forming: "Forming",
  confirmed: "Confirmed",
  invalidated: "Invalidated",
  expired: "Expired"
};

const STATE_DETAILS: Array<{ state: PatternLifecycleState; description: string }> = [
  { state: "warming", description: "collecting enough candles." },
  { state: "watching", description: "no setup detected yet." },
  { state: "forming", description: "setup detected, awaiting confirmation." },
  { state: "confirmed", description: "pattern confirmed on last close." },
  { state: "invalidated", description: "setup failed before confirmation." },
  { state: "expired", description: "pattern aged out without follow-through." }
];

const stateTone = (entry: PatternLifecycleEntry): StatusTone => {
  if (entry.state === "confirmed") {
    if (entry.classification === "bullish") {
      return "positive";
    }
    if (entry.classification === "bearish") {
      return "negative";
    }
    return "neutral";
  }
  if (entry.state === "forming") {
    return "warning";
  }
  return "neutral";
};

type PatternStateMachineTableProps = {
  pattern: string;
  tokens: string[];
  entriesByToken: Record<string, PatternLifecycleEntry | undefined>;
  nowMs: number;
};

export const PatternStateMachineTable = ({
  pattern,
  tokens,
  entriesByToken,
  nowMs
}: PatternStateMachineTableProps) => {
  return (
    <section className="glass-panel rounded-3xl border border-white/70 p-6 shadow-[0_20px_60px_rgba(15,23,42,0.12)]">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-slate-900">Live Screener</h2>
          <p className="text-sm text-slate-500">{pattern} state snapshot per token</p>
        </div>
        <span className="rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-semibold text-slate-500">
          {tokens.length} tokens
        </span>
      </div>

      <div className="mt-4 rounded-2xl border border-slate-200 bg-white/70 p-4 text-sm text-slate-600">
        <span className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
          Pattern States
        </span>
        <div className="mt-3 grid gap-2 md:grid-cols-2">
          {STATE_DETAILS.map((detail) => (
            <div key={detail.state}>
              <span className="font-semibold text-slate-700">{STATE_LABELS[detail.state]}</span> —{" "}
              {detail.description}
            </div>
          ))}
        </div>
      </div>

      <div className="mt-6 overflow-x-auto">
        <table className="w-full min-w-[640px] border-separate border-spacing-y-3 text-left">
          <thead>
            <tr className="text-xs uppercase tracking-[0.2em] text-slate-400">
              <th className="px-4">Token</th>
              <th className="px-4">{pattern} State</th>
              <th className="px-4">Timeframe</th>
              <th className="px-4">Age (last update)</th>
            </tr>
          </thead>
          <tbody>
            {tokens.map((token) => {
              const entry = entriesByToken[token];
              const ageLabel = entry ? formatAge(entry.lastUpdatedMs, nowMs) : "--";
              const intervalLabel = entry ? entry.interval : "--";

              return (
                <tr
                  key={`${pattern}-${token}`}
                  className="rounded-2xl bg-white/80 shadow-[0_16px_40px_rgba(15,23,42,0.08)]"
                >
                  <td className="px-4 py-4 text-sm font-semibold text-slate-900">{token}</td>
                  <td className="px-4 py-4">
                    {entry ? (
                      <StatusPill
                        label={STATE_LABELS[entry.state]}
                        tone={stateTone(entry)}
                        className=""
                      />
                    ) : (
                      <span className="text-xs text-slate-400">No signal</span>
                    )}
                  </td>
                  <td className="px-4 py-4 text-sm text-slate-600">{intervalLabel}</td>
                  <td className="px-4 py-4 text-sm text-slate-600">{ageLabel}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
};
