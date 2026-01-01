import { PatternSummary } from "../../types/patterns";

type PatternSummaryPanelProps = {
  summaries: PatternSummary[];
};

const formatPercent = (value: number) => `${Math.round(value * 100)}%`;

const dominantTone = (summary: PatternSummary) => {
  if (summary.bullishScore >= summary.bearishScore && summary.bullishScore >= summary.neutralScore) {
    return { label: "Bullish", tone: "text-emerald-600", value: summary.bullishScore };
  }
  if (summary.bearishScore >= summary.neutralScore) {
    return { label: "Bearish", tone: "text-rose-600", value: summary.bearishScore };
  }
  return { label: "Neutral", tone: "text-slate-500", value: summary.neutralScore };
};

export const PatternSummaryPanel = ({ summaries }: PatternSummaryPanelProps) => {
  const sortedSummaries = [...summaries].sort((a, b) => {
    const aTop = Math.max(a.bullishScore, a.bearishScore, a.neutralScore);
    const bTop = Math.max(b.bullishScore, b.bearishScore, b.neutralScore);
    return bTop - aTop;
  });

  const visibleSummaries = sortedSummaries.slice(0, 6);

  return (
    <div className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
            Aggregation
          </p>
          <h3 className="mt-2 text-lg font-semibold text-slate-900">Signal balance</h3>
          <p className="text-xs text-slate-500">
            Weighted summary across detected core patterns.
          </p>
        </div>
        <span className="rounded-full bg-slate-900/5 px-3 py-1 text-xs font-semibold text-slate-600">
          {summaries.length} tiles
        </span>
      </div>
      <div className="mt-4 grid gap-3">
        {visibleSummaries.length === 0 ? (
          <div className="rounded-2xl border border-dashed border-slate-200 bg-white/60 p-6 text-sm text-slate-400">
            Summaries will populate once detectors stream signals.
          </div>
        ) : (
          visibleSummaries.map((summary) => {
            const dominant = dominantTone(summary);
            return (
              <div
                key={`${summary.coin}-${summary.interval}`}
                className="rounded-2xl border border-slate-100 bg-white/60 p-4"
              >
                <div className="flex items-center justify-between">
                  <div className="text-sm font-semibold text-slate-900">{summary.coin}</div>
                  <div className="text-[11px] font-semibold uppercase tracking-[0.2em] text-slate-500">
                    {summary.interval}
                  </div>
                </div>
                <div className="mt-2 flex items-center justify-between text-xs text-slate-500">
                  <span className={`font-semibold ${dominant.tone}`}>{dominant.label}</span>
                  <span>{formatPercent(dominant.value)}</span>
                </div>
                <div className="mt-2 flex h-2 overflow-hidden rounded-full bg-slate-100">
                  <div
                    className="h-full bg-emerald-400"
                    style={{ width: `${summary.bullishScore * 100}%` }}
                  />
                  <div
                    className="h-full bg-rose-400"
                    style={{ width: `${summary.bearishScore * 100}%` }}
                  />
                  <div
                    className="h-full bg-slate-400"
                    style={{ width: `${summary.neutralScore * 100}%` }}
                  />
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {summary.topSignals.length > 0 ? (
                    summary.topSignals.map((signal, index) => (
                      <span
                        key={`${signal.pattern}-${index}`}
                        className="rounded-full bg-slate-900/5 px-2.5 py-1 text-[11px] font-semibold text-slate-600"
                      >
                        {signal.pattern}
                      </span>
                    ))
                  ) : (
                    <span className="text-xs text-slate-400">No top signals yet.</span>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
