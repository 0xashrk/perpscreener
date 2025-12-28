import { ReactNode } from "react";

export type StatusTone = "positive" | "negative" | "neutral" | "warning";

type StatusPillProps = {
  label: ReactNode;
  tone: StatusTone;
  className: string;
};

const toneClasses: Record<StatusTone, string> = {
  positive: "bg-emerald-100 text-emerald-800 border-emerald-200",
  negative: "bg-rose-100 text-rose-800 border-rose-200",
  neutral: "bg-slate-100 text-slate-700 border-slate-200",
  warning: "bg-amber-100 text-amber-800 border-amber-200"
};

export const StatusPill = ({ label, tone, className }: StatusPillProps) => {
  return (
    <span
      className={`inline-flex items-center rounded-full border px-3 py-1 text-xs font-semibold uppercase tracking-wide ${toneClasses[tone]} ${className}`}
    >
      {label}
    </span>
  );
};
