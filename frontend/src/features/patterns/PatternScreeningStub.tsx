import { StreamStatus } from "../../types/stream";

type PatternScreeningStubProps = {
  signals: Array<{ pattern: string }>;
  status: StreamStatus;
  lastUpdatedMs: number;
};

const statusTone = (status: StreamStatus) => {
  switch (status) {
    case "open":
      return "bg-emerald-500";
    case "reconnecting":
      return "bg-amber-500";
    case "error":
      return "bg-rose-500";
    default:
      return "bg-slate-400";
  }
};

const formatTimestamp = (value: number) => {
  if (value <= 0) {
    return "—";
  }
  return new Date(value).toLocaleTimeString();
};

export const PatternScreeningStub = ({
  signals,
  status,
  lastUpdatedMs
}: PatternScreeningStubProps) => {
  return (
    <div className="flex flex-col gap-4">
      <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <div className="flex flex-col gap-3">
          <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
            Live Canvas
          </p>
          <div className="relative h-56 overflow-hidden rounded-2xl border border-dashed border-slate-200 bg-gradient-to-br from-slate-50 via-white to-slate-100">
            {signals.map((signal, index) => (
              <span
                key={`${signal.pattern}-${index}`}
                className="absolute rounded-full bg-slate-900/80 px-3 py-1 text-xs font-semibold text-white shadow-lg"
                style={{
                  top: `${18 + index * 24}%`,
                  left: `${12 + index * 18}%`
                }}
              >
                {signal.pattern}
              </span>
            ))}
            {signals.length === 0 ? (
              <div className="flex h-full items-center justify-center text-sm text-slate-400">
                Overlay markers will appear here.
              </div>
            ) : null}
          </div>
          <p className="text-sm text-slate-600">
            Chart overlays and detected patterns will render here once detectors stream results.
          </p>
        </div>
      </div>
      <div className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
          Stream Status
        </p>
        <div className="mt-3 flex items-center gap-3 text-sm text-slate-600">
          <span className={`h-2.5 w-2.5 rounded-full ${statusTone(status)}`} />
          <span className="capitalize">{status}</span>
          <span className="text-xs text-slate-400">Last update: {formatTimestamp(lastUpdatedMs)}</span>
        </div>
      </div>
    </div>
  );
};
