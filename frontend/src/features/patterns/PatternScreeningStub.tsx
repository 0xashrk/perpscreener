export const PatternScreeningStub = () => {
  return (
    <div className="grid gap-4 lg:grid-cols-[1.1fr_0.9fr]">
      <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <div className="flex flex-col gap-3">
          <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
            Live Canvas
          </p>
          <div className="h-56 rounded-2xl border border-dashed border-slate-200 bg-gradient-to-br from-slate-50 via-white to-slate-100" />
          <p className="text-sm text-slate-600">
            Chart overlays and detected patterns will render here once detectors stream results.
          </p>
        </div>
      </div>
      <div className="flex flex-col gap-4">
        <div className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
          <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
            Filters
          </p>
          <div className="mt-4 grid gap-3 text-sm text-slate-600">
            <div className="rounded-2xl border border-dashed border-slate-200 px-4 py-3">
              Coin selection (coming soon)
            </div>
            <div className="rounded-2xl border border-dashed border-slate-200 px-4 py-3">
              Timeframe weighting (coming soon)
            </div>
            <div className="rounded-2xl border border-dashed border-slate-200 px-4 py-3">
              Pattern categories (coming soon)
            </div>
          </div>
        </div>
        <div className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
          <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
            Stream Status
          </p>
          <p className="mt-3 text-sm text-slate-600">
            Ready to connect. The SSE feed will populate this panel with snapshot and update
            timings.
          </p>
        </div>
      </div>
    </div>
  );
};
