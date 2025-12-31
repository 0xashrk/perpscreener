import { PatternDetection } from "../../types/patterns";

type PatternListProps = {
  detections: PatternDetection[];
  status: "idle" | "loading" | "ready" | "error";
  error: string;
};

const classificationStyle = (classification: PatternDetection["classification"]) => {
  switch (classification) {
    case "bullish":
      return "bg-emerald-100 text-emerald-700";
    case "bearish":
      return "bg-rose-100 text-rose-700";
    default:
      return "bg-slate-100 text-slate-600";
  }
};

const formatTimestamp = (value: number) => {
  if (value <= 0) {
    return "—";
  }
  return new Date(value).toLocaleString();
};

export const PatternList = ({ detections, status, error }: PatternListProps) => {
  if (status === "loading") {
    return (
      <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <p className="text-sm text-slate-600">Loading pattern detections…</p>
      </div>
    );
  }

  if (status === "error") {
    return (
      <div className="rounded-3xl border border-rose-200 bg-rose-50/80 p-6 text-sm text-rose-700 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        {error || "Unable to load patterns."}
      </div>
    );
  }

  if (detections.length === 0) {
    return (
      <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <p className="text-sm text-slate-600">No patterns detected yet.</p>
      </div>
    );
  }

  return (
    <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
      <div className="flex items-center justify-between gap-4">
        <div>
          <p className="text-xs uppercase tracking-[0.3em] text-slate-500">Core Patterns</p>
          <p className="text-sm text-slate-600">Latest candlestick and gap detections.</p>
        </div>
        <span className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
          {detections.length} signals
        </span>
      </div>
      <div className="mt-4 overflow-hidden rounded-2xl border border-slate-200">
        <div className="grid grid-cols-[1.3fr_0.6fr_0.6fr_0.6fr_0.7fr] gap-2 bg-slate-100/70 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.2em] text-slate-500">
          <span>Pattern</span>
          <span>Coin</span>
          <span>Interval</span>
          <span>Bias</span>
          <span>Confidence</span>
        </div>
        <div className="max-h-[420px] divide-y divide-slate-100 overflow-y-auto text-sm text-slate-700">
          {detections.map((detection, index) => (
            <div
              key={`${detection.coin}-${detection.interval}-${detection.pattern}-${index}`}
              className="grid grid-cols-[1.3fr_0.6fr_0.6fr_0.6fr_0.7fr] gap-2 px-4 py-3"
            >
              <div className="flex flex-col">
                <span className="font-semibold text-slate-900">{detection.pattern}</span>
                <span className="text-xs text-slate-400">{formatTimestamp(detection.detectedAtMs)}</span>
              </div>
              <span className="font-semibold">{detection.coin}</span>
              <span>{detection.interval}</span>
              <span
                className={`inline-flex w-fit items-center rounded-full px-2 py-0.5 text-xs font-semibold ${classificationStyle(
                  detection.classification
                )}`}
              >
                {detection.classification}
              </span>
              <span className="font-semibold">
                {Math.round(detection.confidence * 100)}%
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
