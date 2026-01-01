import { PatternSignalType } from "../../types/patterns";

const SIGNAL_LABELS: Record<PatternSignalType, string> = {
  reversal: "Reversal",
  continuation: "Continuation",
  trend: "Trend",
  range: "Range",
  key_level: "Key level",
  impulse: "Impulse",
  correction: "Correction"
};

type PatternWeightControlsProps = {
  intervals: string[];
  timeframeWeights: Record<string, number>;
  signalWeights: Record<PatternSignalType, number>;
  onTimeframeChange: (interval: string, value: number) => void;
  onSignalChange: (signalType: PatternSignalType, value: number) => void;
  onReset: () => void;
};

const clampWeight = (value: number) => Math.min(3, Math.max(0, value));

const parseWeight = (value: string, fallback: number) => {
  const parsed = Number.parseFloat(value);
  if (Number.isNaN(parsed)) {
    return fallback;
  }
  return clampWeight(parsed);
};

export const PatternWeightControls = ({
  intervals,
  timeframeWeights,
  signalWeights,
  onTimeframeChange,
  onSignalChange,
  onReset
}: PatternWeightControlsProps) => {
  return (
    <div className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
      <div className="flex items-center justify-between">
        <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
          Weighting
        </p>
        <button
          className="rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-semibold text-slate-600 transition hover:border-slate-300"
          onClick={onReset}
          type="button"
        >
          Reset
        </button>
      </div>
      <div className="mt-4 grid gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
            Timeframes
          </p>
          <div className="mt-2 grid grid-cols-2 gap-2 text-xs">
            {intervals.map((interval) => {
              const value = timeframeWeights[interval] ?? 1;
              return (
                <label
                  key={interval}
                  className="flex items-center justify-between rounded-2xl border border-slate-100 bg-white/80 px-3 py-2"
                >
                  <span className="font-semibold text-slate-600">{interval}</span>
                  <input
                    className="w-16 rounded-lg border border-slate-200 bg-white px-2 py-1 text-right text-xs font-semibold text-slate-700"
                    max={3}
                    min={0}
                    onChange={(event) =>
                      onTimeframeChange(interval, parseWeight(event.target.value, value))
                    }
                    step={0.1}
                    type="number"
                    value={value}
                  />
                </label>
              );
            })}
          </div>
        </div>
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
            Signal types
          </p>
          <div className="mt-2 grid grid-cols-2 gap-2 text-xs">
            {(Object.keys(SIGNAL_LABELS) as PatternSignalType[]).map((signalType) => {
              const value = signalWeights[signalType] ?? 1;
              return (
                <label
                  key={signalType}
                  className="flex items-center justify-between rounded-2xl border border-slate-100 bg-white/80 px-3 py-2"
                >
                  <span className="font-semibold text-slate-600">
                    {SIGNAL_LABELS[signalType]}
                  </span>
                  <input
                    className="w-16 rounded-lg border border-slate-200 bg-white px-2 py-1 text-right text-xs font-semibold text-slate-700"
                    max={3}
                    min={0}
                    onChange={(event) =>
                      onSignalChange(signalType, parseWeight(event.target.value, value))
                    }
                    step={0.1}
                    type="number"
                    value={value}
                  />
                </label>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
