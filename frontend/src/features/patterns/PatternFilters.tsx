type PatternFiltersProps = {
  tokens: string[];
  activeTokens: string[];
  onToggleToken: (token: string) => void;
  intervals: string[];
  activeIntervals: string[];
  onToggleInterval: (interval: string) => void;
};

const toggleClass = (active: boolean) =>
  [
    "rounded-full border px-3 py-1 text-xs font-semibold uppercase tracking-wide transition",
    active
      ? "border-slate-900 bg-slate-900 text-white"
      : "border-slate-200 bg-white/70 text-slate-600 hover:border-slate-400"
  ].join(" ");

export const PatternFilters = ({
  tokens,
  activeTokens,
  onToggleToken,
  intervals,
  activeIntervals,
  onToggleInterval
}: PatternFiltersProps) => {
  return (
    <div className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
      <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">Filters</p>
      <div className="mt-4 flex flex-col gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">Coins</p>
          <div className="mt-2 flex flex-wrap gap-2">
            {tokens.map((token) => (
              <button
                key={token}
                className={toggleClass(activeTokens.includes(token))}
                onClick={() => onToggleToken(token)}
                type="button"
              >
                {token}
              </button>
            ))}
          </div>
        </div>
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
            Intervals
          </p>
          <div className="mt-2 flex flex-wrap gap-2">
            {intervals.map((interval) => (
              <button
                key={interval}
                className={toggleClass(activeIntervals.includes(interval))}
                onClick={() => onToggleInterval(interval)}
                type="button"
              >
                {interval}
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
