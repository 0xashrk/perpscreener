import { AdvancedPatternDetection } from "../../types/patterns";
import { StreamStatus } from "../../types/stream";

type AdvancedPatternListProps = {
  detections: AdvancedPatternDetection[];
  status: StreamStatus;
  error: string;
};

const formatConfidence = (value: number) => `${(value * 100).toFixed(1)}%`;

export const AdvancedPatternList = ({ detections, status, error }: AdvancedPatternListProps) => {
  if (status === "connecting" || status === "reconnecting") {
    return (
      <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <p className="text-sm text-slate-600">Loading advanced detections…</p>
      </div>
    );
  }

  if (status === "error") {
    return (
      <div className="rounded-3xl border border-rose-200 bg-rose-50/80 p-6 text-sm text-rose-700 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        {error || "Unable to load advanced patterns."}
      </div>
    );
  }

  if (detections.length === 0) {
    return (
      <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <p className="text-sm text-slate-600">No advanced patterns detected yet.</p>
      </div>
    );
  }

  return (
    <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
      <div className="flex items-center justify-between gap-4">
        <div>
          <p className="text-xs uppercase tracking-[0.3em] text-slate-500">Advanced Patterns</p>
          <p className="text-sm text-slate-600">Fibonacci, Elliott wave, and fractal signals.</p>
        </div>
        <span className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-400">
          {detections.length} signals
        </span>
      </div>
      <div className="mt-4 overflow-hidden rounded-2xl border border-slate-200">
        <div className="grid grid-cols-[1.2fr_0.5fr_0.6fr_0.7fr_0.6fr] gap-2 bg-slate-100/70 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.2em] text-slate-500">
          <span>Pattern</span>
          <span>Coin</span>
          <span>Interval</span>
          <span>Method</span>
          <span>Confidence</span>
        </div>
        <div className="max-h-[420px] divide-y divide-slate-100 overflow-y-auto text-sm text-slate-700">
          {detections.map((detection) => {
            const assumptionLabel = detection.assumptions.join(", ");
            const confidenceTitle = [detection.basis, assumptionLabel]
              .filter((value) => value.length > 0)
              .join(" · ");
            return (
              <div
                key={`${detection.coin}-${detection.interval}-${detection.pattern}-${detection.method}`}
                className="grid grid-cols-[1.2fr_0.5fr_0.6fr_0.7fr_0.6fr] gap-2 px-4 py-3"
              >
                <div className="flex flex-col">
                  <span className="font-semibold text-slate-900">{detection.pattern}</span>
                  <span className="text-xs text-slate-400">{detection.basis}</span>
                </div>
                <span className="font-semibold">{detection.coin}</span>
                <span>{detection.interval}</span>
                <span className="text-xs uppercase tracking-[0.2em] text-slate-500">
                  {detection.method.replaceAll("_", " ")}
                </span>
                <span className="font-semibold" title={confidenceTitle || undefined}>
                  {formatConfidence(detection.confidence)}
                </span>
              </div>
            );
          })}
        </div>
      </div>
      <div className="mt-4 flex flex-wrap gap-2 text-xs text-slate-500">
        {detections.slice(0, 6).flatMap((detection, index) =>
          detection.assumptions.map((assumption, idx) => (
            <span
              key={`${index}-${idx}`}
              className="rounded-full bg-slate-100 px-3 py-1 text-slate-500"
            >
              {assumption}
            </span>
          ))
        )}
      </div>
    </div>
  );
};
