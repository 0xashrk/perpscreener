import { StatusPill, StatusTone } from "../../components/StatusPill";
import { PatternLifecycleEntry } from "../../types/patterns";
import { StreamStatus } from "../../types/stream";
import { formatAge } from "../../utils/time";

const STATE_LABELS: Record<PatternLifecycleEntry["state"], string> = {
  warming: "Warming",
  watching: "Watching",
  forming: "Forming",
  confirmed: "Confirmed",
  invalidated: "Invalidated",
  expired: "Expired"
};

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

const formatConfidence = (value: number) => `${(value * 100).toFixed(1)}%`;

type PatternLifecycleBoardProps = {
  entries: PatternLifecycleEntry[];
  status: StreamStatus;
  error: string;
  nowMs: number;
};

export const PatternLifecycleBoard = ({
  entries,
  status,
  error,
  nowMs
}: PatternLifecycleBoardProps) => {
  const activeEntries = entries.filter(
    (entry) => entry.state !== "watching" && entry.state !== "warming"
  );
  const visibleEntries = [...activeEntries]
    .sort((a, b) => b.lastUpdatedMs - a.lastUpdatedMs)
    .slice(0, 24);

  const statusLabel = status === "open" ? "Live" : status.replace(/_/g, " ");

  return (
    <section className="glass-panel rounded-3xl border border-white/70 p-6 shadow-[0_20px_60px_rgba(15,23,42,0.12)]">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
            Pattern Lifecycle
          </p>
          <h2 className="mt-2 text-xl font-semibold text-slate-900">Live pattern board</h2>
          <p className="text-sm text-slate-500">
            State machine snapshots across all detected patterns.
          </p>
        </div>
        <div className="flex flex-col items-end gap-2">
          <span className="rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-semibold text-slate-500">
            {activeEntries.length} active
          </span>
          <span className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
            {statusLabel}
          </span>
        </div>
      </div>

      <div className="mt-4 rounded-2xl border border-slate-200 bg-white/70 p-4 text-xs text-slate-500">
        <div className="grid gap-2 md:grid-cols-3">
          {Object.entries(STATE_LABELS).map(([key, label]) => (
            <div key={key}>
              <span className="font-semibold text-slate-700">{label}</span> —{" "}
              {key === "warming" && "collecting enough candles."}
              {key === "watching" && "no setup detected yet."}
              {key === "forming" && "setup detected, awaiting confirmation."}
              {key === "confirmed" && "pattern confirmed on last close."}
              {key === "invalidated" && "setup failed before confirmation."}
              {key === "expired" && "pattern aged out without follow-through."}
            </div>
          ))}
        </div>
      </div>

      {status === "error" ? (
        <div className="mt-4 rounded-2xl border border-rose-200 bg-rose-50/70 p-4 text-xs text-rose-600">
          {error || "Lifecycle stream error."}
        </div>
      ) : null}

      <div className="mt-6 overflow-x-auto">
        <table className="w-full min-w-[760px] border-separate border-spacing-y-3 text-left">
          <thead>
            <tr className="text-xs uppercase tracking-[0.2em] text-slate-400">
              <th className="px-4">Token</th>
              <th className="px-4">Interval</th>
              <th className="px-4">Pattern</th>
              <th className="px-4">State</th>
              <th className="px-4">Confidence</th>
              <th className="px-4">Age</th>
            </tr>
          </thead>
          <tbody>
            {visibleEntries.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-4 py-6 text-center text-xs text-slate-400">
                  No active pattern lifecycles yet.
                </td>
              </tr>
            ) : (
              visibleEntries.map((entry) => (
                <tr
                  key={`${entry.coin}-${entry.interval}-${entry.pattern}-${entry.classification}`}
                  className="rounded-2xl bg-white/80 shadow-[0_16px_40px_rgba(15,23,42,0.08)]"
                >
                  <td className="px-4 py-4 text-sm font-semibold text-slate-900">
                    {entry.coin}
                  </td>
                  <td className="px-4 py-4 text-sm text-slate-600">{entry.interval}</td>
                  <td className="px-4 py-4">
                    <div className="text-sm font-semibold text-slate-900">{entry.pattern}</div>
                    <div className="text-xs text-slate-500">{entry.classification}</div>
                    {entry.notes ? (
                      <div className="text-[11px] text-slate-400">{entry.notes}</div>
                    ) : null}
                  </td>
                  <td className="px-4 py-4">
                    <StatusPill
                      label={STATE_LABELS[entry.state]}
                      tone={stateTone(entry)}
                      className=""
                    />
                  </td>
                  <td className="px-4 py-4 text-sm text-slate-600">
                    {formatConfidence(entry.confidence)}
                  </td>
                  <td className="px-4 py-4 text-sm text-slate-600">
                    {formatAge(entry.lastUpdatedMs, nowMs)}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
};
