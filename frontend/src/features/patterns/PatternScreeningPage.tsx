import { PatternScreeningStub } from "./PatternScreeningStub";

export const PatternScreeningPage = () => {
  return (
    <section className="flex flex-col gap-6">
      <header className="flex flex-col gap-2">
        <p className="text-xs uppercase tracking-[0.3em] text-slate-500">Pattern Screening</p>
        <h1 className="text-3xl font-semibold text-slate-900">
          Multi-timeframe pattern visualization
        </h1>
        <p className="max-w-2xl text-sm text-slate-600">
          Data ingestion is running and feature precompute is warming up. This space will surface
          pattern detections, confidence, and visual overlays as the backend phases roll out.
        </p>
      </header>
      <PatternScreeningStub />
    </section>
  );
};
